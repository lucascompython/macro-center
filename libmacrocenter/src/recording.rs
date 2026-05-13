use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use rdev::{Button, Event, EventType, Key, listen};
use serde::{Deserialize, Serialize};

use crate::error::{MacroCenterError, Result};
use crate::input::EmitterBackend;
use crate::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};

const ALL_MOVES_MIN_INTERVAL: Duration = Duration::from_millis(50);
const ALL_MOVES_MIN_DISTANCE_PX: i32 = 8;

type MousePositionCallback = Arc<dyn Fn(MousePositionEvent) + Send + Sync + 'static>;

/// Mouse movement policy used while recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingMouseMode {
    /// Keep only the last mouse position before a mouse button event.
    MovesBeforeClicks,
    /// Keep every distinct mouse position event.
    AllMoves,
}

impl FromStr for RecordingMouseMode {
    type Err = MacroCenterError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "movesbeforeclicks" | "clicks" | "clicksonly" | "beforeclicks" => {
                Ok(Self::MovesBeforeClicks)
            }
            "allmoves" | "all" | "mousemove" | "mousemoves" => Ok(Self::AllMoves),
            _ => Err(MacroCenterError::ParseError(format!(
                "Unknown recording mouse mode: '{s}'. Expected 'movesBeforeClicks' or 'allMoves'"
            ))),
        }
    }
}

/// A serializable macro captured from global input events.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedMacro {
    pub actions: Vec<RecordedAction>,
}

impl RecordedMacro {
    pub fn new(actions: Vec<RecordedAction>) -> Self {
        Self { actions }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| MacroCenterError::SerializationError(e.to_string()))
    }

    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| MacroCenterError::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| MacroCenterError::SerializationError(e.to_string()))
    }

    /// Playback the recorded macro at normal speed.
    pub fn playback<B: EmitterBackend>(&self, emitter: &mut B) -> Result<()> {
        self.playback_at_speed(emitter, 1.0)
    }

    /// Playback the recorded macro with a speed multiplier.
    ///
    /// `2.0` plays twice as fast, `0.5` plays at half speed. Invalid speed
    /// values fall back to normal speed.
    pub fn playback_at_speed<B: EmitterBackend>(&self, emitter: &mut B, speed: f64) -> Result<()> {
        let speed = if speed.is_finite() && speed > 0.0 {
            speed
        } else {
            1.0
        };

        for action in &self.actions {
            let delay_ms = (action.delay_ms as f64 / speed).round();
            if delay_ms > 0.0 {
                thread::sleep(Duration::from_millis(delay_ms.min(u64::MAX as f64) as u64));
            }
            action.kind.emit(emitter)?;
        }

        Ok(())
    }

    /// Remove the trailing click used to press an in-app Stop button.
    ///
    /// This is intentionally separate from `stop_recording` because library
    /// consumers may stop recording via another channel and want the raw event
    /// stream.
    pub fn trim_trailing_mouse_click(&mut self) {
        let mut remove_end = self.actions.len();
        while remove_end > 0 && self.actions[remove_end - 1].kind.is_mouse_move() {
            remove_end -= 1;
        }

        let Some((stop_button, stop_mode)) = self
            .actions
            .get(remove_end.saturating_sub(1))
            .and_then(|action| action.kind.mouse_button())
        else {
            return;
        };

        let mut remove_start = remove_end - 1;

        if stop_mode == ActionMode::Release {
            let mut candidate = remove_start;
            while candidate > 0 && self.actions[candidate - 1].kind.is_mouse_move() {
                candidate -= 1;
            }

            if candidate > 0
                && self.actions[candidate - 1].kind.mouse_button()
                    == Some((stop_button, ActionMode::Press))
            {
                remove_start = candidate - 1;
            }
        }

        let stop_move_start = preceding_mouse_move_start(&self.actions, remove_start);
        if self.actions[..stop_move_start]
            .iter()
            .any(|action| !action.kind.is_mouse_move())
        {
            remove_start = stop_move_start;
        }

        self.actions.drain(remove_start..self.actions.len());
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedAction {
    pub delay_ms: u64,
    pub kind: RecordedActionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MousePositionEvent {
    pub x: i32,
    pub y: i32,
    pub clicked: bool,
    pub button: Option<MouseButton>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RecordedActionKind {
    Key {
        key: String,
        mode: ActionMode,
    },
    MouseButton {
        button: MouseButton,
        mode: ActionMode,
    },
    MouseMove {
        x: i32,
        y: i32,
        coordinate_mode: CoordinateMode,
    },
    Scroll {
        axis: ScrollAxis,
        amount: i32,
    },
}

impl RecordedActionKind {
    fn is_mouse_move(&self) -> bool {
        matches!(self, Self::MouseMove { .. })
    }

    fn mouse_button(&self) -> Option<(MouseButton, ActionMode)> {
        match self {
            Self::MouseButton { button, mode } => Some((*button, *mode)),
            _ => None,
        }
    }

    fn emit<B: EmitterBackend>(&self, emitter: &mut B) -> Result<()> {
        match self {
            Self::Key { key, mode } => emitter.key_action(key, *mode),
            Self::MouseButton { button, mode } => emitter.mouse_click(*button, *mode),
            Self::MouseMove {
                x,
                y,
                coordinate_mode,
            } => emitter.mouse_move(*x, *y, *coordinate_mode),
            Self::Scroll { axis, amount } => emitter.scroll(*axis, *amount),
        }
    }
}

/// Backend abstraction for recording global keyboard and mouse actions.
pub trait RecorderBackend {
    fn start_recording(&self, mode: RecordingMouseMode) -> Result<()>;
    fn stop_recording(&self) -> Result<RecordedMacro>;
    fn is_recording(&self) -> bool;
    fn start_mouse_listening<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(MousePositionEvent) + Send + Sync + 'static;
    fn stop_mouse_listening(&self);
    fn is_mouse_listening(&self) -> bool;
}

#[derive(Clone)]
pub struct RdevRecorder {
    state: Arc<Mutex<RecorderState>>,
    listener_started: Arc<AtomicBool>,
}

impl RdevRecorder {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecorderState::default())),
            listener_started: Arc::new(AtomicBool::new(false)),
        }
    }

    fn ensure_listener_started(&self) -> Result<()> {
        if self
            .listener_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(());
        }

        let state = Arc::clone(&self.state);
        let listener_started = Arc::clone(&self.listener_started);
        thread::Builder::new()
            .name("macro-center-rdev-listener".to_string())
            .spawn(move || {
                let callback_state = Arc::clone(&state);
                if let Err(error) = listen(move |event| record_event(&callback_state, event)) {
                    if let Ok(mut state) = state.lock() {
                        state.active = false;
                        state.listener_error = Some(format!("{error:?}"));
                    }
                    listener_started.store(false, Ordering::SeqCst);
                }
            })
            .map_err(|e| {
                self.listener_started.store(false, Ordering::SeqCst);
                MacroCenterError::RecorderError(e.to_string())
            })?;

        Ok(())
    }
}

impl Default for RdevRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl RecorderBackend for RdevRecorder {
    fn start_recording(&self, mode: RecordingMouseMode) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| MacroCenterError::RecorderError(e.to_string()))?;
            if state.active {
                return Err(MacroCenterError::RecorderError(
                    "A recording is already active".to_string(),
                ));
            }
            state.listener_error = None;
        }

        self.ensure_listener_started()?;

        let mut state = self
            .state
            .lock()
            .map_err(|e| MacroCenterError::RecorderError(e.to_string()))?;
        state.active = true;
        state.mode = mode;
        state.started_at = Some(SystemTime::now());
        state.last_action_time = None;
        state.pending_mouse_move = None;
        state.last_mouse_position = None;
        state.last_mouse_move_time = None;
        state.actions.clear();
        Ok(())
    }

    fn stop_recording(&self) -> Result<RecordedMacro> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| MacroCenterError::RecorderError(e.to_string()))?;

        if let Some(error) = state.listener_error.take() {
            return Err(MacroCenterError::RecorderError(error));
        }

        state.active = false;
        state.pending_mouse_move = None;
        Ok(RecordedMacro::new(std::mem::take(&mut state.actions)))
    }

    fn is_recording(&self) -> bool {
        self.state.lock().map(|state| state.active).unwrap_or(false)
    }

    fn start_mouse_listening<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(MousePositionEvent) + Send + Sync + 'static,
    {
        self.ensure_listener_started()?;

        let mut state = self
            .state
            .lock()
            .map_err(|e| MacroCenterError::RecorderError(e.to_string()))?;
        state.mouse_listener_active = true;
        state.mouse_listener_callback = Some(Arc::new(callback));
        state.mouse_listener_last_position = None;
        Ok(())
    }

    fn stop_mouse_listening(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.mouse_listener_active = false;
            state.mouse_listener_callback = None;
            state.mouse_listener_last_position = None;
        }
    }

    fn is_mouse_listening(&self) -> bool {
        self.state
            .lock()
            .map(|state| state.mouse_listener_active)
            .unwrap_or(false)
    }
}

struct RecorderState {
    active: bool,
    mode: RecordingMouseMode,
    started_at: Option<SystemTime>,
    last_action_time: Option<SystemTime>,
    actions: Vec<RecordedAction>,
    pending_mouse_move: Option<PendingMouseMove>,
    last_mouse_position: Option<(i32, i32)>,
    last_mouse_move_time: Option<SystemTime>,
    listener_error: Option<String>,
    mouse_listener_active: bool,
    mouse_listener_callback: Option<MousePositionCallback>,
    mouse_listener_last_position: Option<(i32, i32)>,
}

impl Default for RecorderState {
    fn default() -> Self {
        Self {
            active: false,
            mode: RecordingMouseMode::MovesBeforeClicks,
            started_at: None,
            last_action_time: None,
            actions: Vec::new(),
            pending_mouse_move: None,
            last_mouse_position: None,
            last_mouse_move_time: None,
            listener_error: None,
            mouse_listener_active: false,
            mouse_listener_callback: None,
            mouse_listener_last_position: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PendingMouseMove {
    time: SystemTime,
    x: i32,
    y: i32,
}

fn record_event(state: &Arc<Mutex<RecorderState>>, event: Event) {
    let mouse_listener_event = {
        let Ok(mut state) = state.lock() else {
            return;
        };

        match event.event_type {
            EventType::KeyPress(key) => {
                if state.active
                    && let Some(key) = rdev_key_to_macro_key(key)
                {
                    commit_action(
                        &mut state,
                        event.time,
                        RecordedActionKind::Key {
                            key,
                            mode: ActionMode::Press,
                        },
                    );
                }
                None
            }
            EventType::KeyRelease(key) => {
                if state.active
                    && let Some(key) = rdev_key_to_macro_key(key)
                {
                    commit_action(
                        &mut state,
                        event.time,
                        RecordedActionKind::Key {
                            key,
                            mode: ActionMode::Release,
                        },
                    );
                }
                None
            }
            EventType::ButtonPress(button) => {
                let button = rdev_button_to_macro_button(button);
                let mouse_event = if state.mouse_listener_active {
                    state
                        .mouse_listener_last_position
                        .zip(state.mouse_listener_callback.as_ref())
                        .map(|((x, y), callback)| {
                            (
                                callback.clone(),
                                MousePositionEvent {
                                    x,
                                    y,
                                    clicked: true,
                                    button,
                                },
                            )
                        })
                } else {
                    None
                };

                if state.active
                    && let Some(button) = button
                {
                    commit_pending_mouse_move(&mut state);
                    commit_action(
                        &mut state,
                        event.time,
                        RecordedActionKind::MouseButton {
                            button,
                            mode: ActionMode::Press,
                        },
                    );
                }

                mouse_event
            }
            EventType::ButtonRelease(button) => {
                if state.active
                    && let Some(button) = rdev_button_to_macro_button(button)
                {
                    commit_action(
                        &mut state,
                        event.time,
                        RecordedActionKind::MouseButton {
                            button,
                            mode: ActionMode::Release,
                        },
                    );
                }
                None
            }
            EventType::MouseMove { x, y } => {
                let x = saturating_f64_to_i32(x);
                let y = saturating_f64_to_i32(y);
                let mouse_event = if state.mouse_listener_active {
                    state.mouse_listener_last_position = Some((x, y));
                    state.mouse_listener_callback.as_ref().map(|callback| {
                        (
                            callback.clone(),
                            MousePositionEvent {
                                x,
                                y,
                                clicked: false,
                                button: None,
                            },
                        )
                    })
                } else {
                    None
                };

                if state.active {
                    match state.mode {
                        RecordingMouseMode::MovesBeforeClicks => {
                            state.pending_mouse_move = Some(PendingMouseMove {
                                time: event.time,
                                x,
                                y,
                            });
                        }
                        RecordingMouseMode::AllMoves => {
                            if should_commit_sampled_mouse_move(&state, event.time, x, y) {
                                state.pending_mouse_move = None;
                                commit_mouse_move(&mut state, event.time, x, y);
                            } else {
                                state.pending_mouse_move = Some(PendingMouseMove {
                                    time: event.time,
                                    x,
                                    y,
                                });
                            }
                        }
                    }
                }

                mouse_event
            }
            EventType::Wheel { delta_x, delta_y } => {
                if state.active {
                    if delta_y != 0 {
                        commit_action(
                            &mut state,
                            event.time,
                            RecordedActionKind::Scroll {
                                axis: ScrollAxis::Vertical,
                                // rdev reports positive vertical wheel movement as up;
                                // enigo uses positive values for down.
                                amount: saturating_i64_to_i32(-delta_y),
                            },
                        );
                    }
                    if delta_x != 0 {
                        commit_action(
                            &mut state,
                            event.time,
                            RecordedActionKind::Scroll {
                                axis: ScrollAxis::Horizontal,
                                amount: saturating_i64_to_i32(delta_x),
                            },
                        );
                    }
                }
                None
            }
        }
    };

    if let Some((callback, event)) = mouse_listener_event {
        callback(event);
    }
}

fn commit_pending_mouse_move(state: &mut RecorderState) {
    if let Some(pending) = state.pending_mouse_move.take() {
        commit_mouse_move(state, pending.time, pending.x, pending.y);
    }
}

fn commit_mouse_move(state: &mut RecorderState, time: SystemTime, x: i32, y: i32) {
    if state.last_mouse_position == Some((x, y)) {
        return;
    }

    state.last_mouse_position = Some((x, y));
    state.last_mouse_move_time = Some(time);
    commit_action(
        state,
        time,
        RecordedActionKind::MouseMove {
            x,
            y,
            coordinate_mode: CoordinateMode::Absolute,
        },
    );
}

fn should_commit_sampled_mouse_move(
    state: &RecorderState,
    time: SystemTime,
    x: i32,
    y: i32,
) -> bool {
    let Some((last_x, last_y)) = state.last_mouse_position else {
        return true;
    };

    if (last_x, last_y) == (x, y) {
        return false;
    }

    let elapsed_ok = state
        .last_mouse_move_time
        .and_then(|last_time| time.duration_since(last_time).ok())
        .map(|elapsed| elapsed >= ALL_MOVES_MIN_INTERVAL)
        .unwrap_or(true);
    let dx = x.saturating_sub(last_x);
    let dy = y.saturating_sub(last_y);
    let distance_squared = dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy));
    let min_distance_squared = ALL_MOVES_MIN_DISTANCE_PX * ALL_MOVES_MIN_DISTANCE_PX;

    elapsed_ok && distance_squared >= min_distance_squared
}

fn commit_action(state: &mut RecorderState, time: SystemTime, kind: RecordedActionKind) {
    let delay_ms = state
        .last_action_time
        .or(state.started_at)
        .and_then(|last_time| time.duration_since(last_time).ok())
        .map(duration_millis)
        .unwrap_or(0);

    if should_advance_time(state.last_action_time, time) {
        state.last_action_time = Some(time);
    }

    state.actions.push(RecordedAction { delay_ms, kind });
}

fn should_advance_time(current: Option<SystemTime>, next: SystemTime) -> bool {
    current
        .map(|current| next.duration_since(current).is_ok())
        .unwrap_or(true)
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}

fn saturating_f64_to_i32(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    value.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

fn saturating_i64_to_i32(value: i64) -> i32 {
    value.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

fn preceding_mouse_move_start(actions: &[RecordedAction], index: usize) -> usize {
    let mut start = index;
    while start > 0 && actions[start - 1].kind.is_mouse_move() {
        start -= 1;
    }
    start
}

fn rdev_button_to_macro_button(button: Button) -> Option<MouseButton> {
    match button {
        Button::Left => Some(MouseButton::Left),
        Button::Right => Some(MouseButton::Right),
        Button::Middle => Some(MouseButton::Middle),
        Button::Unknown(4) => Some(MouseButton::Mouse4),
        Button::Unknown(5) => Some(MouseButton::Mouse5),
        Button::Unknown(_) => None,
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

fn rdev_key_to_macro_key(key: Key) -> Option<String> {
    let key = match key {
        Key::Alt | Key::AltGr => "Alt",
        Key::Backspace => "Backspace",
        Key::CapsLock => "CapsLock",
        Key::ControlLeft | Key::ControlRight => "Control",
        Key::Delete => "Delete",
        Key::DownArrow => "ArrowDown",
        Key::End => "End",
        Key::Escape => "Escape",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::Home => "Home",
        Key::Insert => "Insert",
        Key::LeftArrow => "ArrowLeft",
        Key::MetaLeft | Key::MetaRight => "Meta",
        Key::PageDown => "PageDown",
        Key::PageUp => "PageUp",
        Key::Return | Key::KpReturn => "Enter",
        Key::RightArrow => "ArrowRight",
        Key::ShiftLeft | Key::ShiftRight => "Shift",
        Key::Space => "Space",
        Key::Tab => "Tab",
        Key::UpArrow => "ArrowUp",
        Key::PrintScreen => "PrintScreen",
        Key::ScrollLock => "ScrollLock",
        Key::Pause => "Pause",
        Key::NumLock => "NumLock",
        Key::BackQuote => "`",
        Key::Num1 | Key::Kp1 => "1",
        Key::Num2 | Key::Kp2 => "2",
        Key::Num3 | Key::Kp3 => "3",
        Key::Num4 | Key::Kp4 => "4",
        Key::Num5 | Key::Kp5 => "5",
        Key::Num6 | Key::Kp6 => "6",
        Key::Num7 | Key::Kp7 => "7",
        Key::Num8 | Key::Kp8 => "8",
        Key::Num9 | Key::Kp9 => "9",
        Key::Num0 | Key::Kp0 => "0",
        Key::Minus | Key::KpMinus => "-",
        Key::Equal => "=",
        Key::KeyQ => "q",
        Key::KeyW => "w",
        Key::KeyE => "e",
        Key::KeyR => "r",
        Key::KeyT => "t",
        Key::KeyY => "y",
        Key::KeyU => "u",
        Key::KeyI => "i",
        Key::KeyO => "o",
        Key::KeyP => "p",
        Key::LeftBracket => "[",
        Key::RightBracket => "]",
        Key::KeyA => "a",
        Key::KeyS => "s",
        Key::KeyD => "d",
        Key::KeyF => "f",
        Key::KeyG => "g",
        Key::KeyH => "h",
        Key::KeyJ => "j",
        Key::KeyK => "k",
        Key::KeyL => "l",
        Key::SemiColon => ";",
        Key::Quote => "'",
        Key::BackSlash | Key::IntlBackslash => "\\",
        Key::KeyZ => "z",
        Key::KeyX => "x",
        Key::KeyC => "c",
        Key::KeyV => "v",
        Key::KeyB => "b",
        Key::KeyN => "n",
        Key::KeyM => "m",
        Key::Comma => ",",
        Key::Dot | Key::KpDelete => ".",
        Key::Slash | Key::KpDivide => "/",
        Key::KpPlus => "+",
        Key::KpMultiply => "*",
        Key::Function | Key::Unknown(_) => return None,
        #[allow(unreachable_patterns)]
        _ => return None,
    };

    Some(key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_trailing_stop_click_removes_button_pair_and_moves() {
        let mut recorded = RecordedMacro::new(vec![
            RecordedAction {
                delay_ms: 0,
                kind: RecordedActionKind::Key {
                    key: "a".to_string(),
                    mode: ActionMode::Press,
                },
            },
            RecordedAction {
                delay_ms: 10,
                kind: RecordedActionKind::MouseMove {
                    x: 10,
                    y: 10,
                    coordinate_mode: CoordinateMode::Absolute,
                },
            },
            RecordedAction {
                delay_ms: 0,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Press,
                },
            },
            RecordedAction {
                delay_ms: 1,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Release,
                },
            },
        ]);

        recorded.trim_trailing_mouse_click();

        assert_eq!(recorded.actions.len(), 1);
    }

    #[test]
    fn trim_trailing_stop_click_keeps_prior_mouse_click() {
        let mut recorded = RecordedMacro::new(vec![
            RecordedAction {
                delay_ms: 0,
                kind: RecordedActionKind::MouseMove {
                    x: 100,
                    y: 100,
                    coordinate_mode: CoordinateMode::Absolute,
                },
            },
            RecordedAction {
                delay_ms: 5,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Press,
                },
            },
            RecordedAction {
                delay_ms: 5,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Release,
                },
            },
            RecordedAction {
                delay_ms: 100,
                kind: RecordedActionKind::MouseMove {
                    x: 20,
                    y: 20,
                    coordinate_mode: CoordinateMode::Absolute,
                },
            },
            RecordedAction {
                delay_ms: 5,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Press,
                },
            },
            RecordedAction {
                delay_ms: 5,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Release,
                },
            },
        ]);

        recorded.trim_trailing_mouse_click();

        assert_eq!(recorded.actions.len(), 3);
        assert!(matches!(
            recorded.actions.last().map(|action| &action.kind),
            Some(RecordedActionKind::MouseButton {
                mode: ActionMode::Release,
                ..
            })
        ));
    }

    #[test]
    fn trim_trailing_stop_click_keeps_mouse_move_only_macro() {
        let mut recorded = RecordedMacro::new(vec![
            RecordedAction {
                delay_ms: 0,
                kind: RecordedActionKind::MouseMove {
                    x: 100,
                    y: 100,
                    coordinate_mode: CoordinateMode::Absolute,
                },
            },
            RecordedAction {
                delay_ms: 25,
                kind: RecordedActionKind::MouseMove {
                    x: 140,
                    y: 130,
                    coordinate_mode: CoordinateMode::Absolute,
                },
            },
            RecordedAction {
                delay_ms: 5,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Press,
                },
            },
            RecordedAction {
                delay_ms: 5,
                kind: RecordedActionKind::MouseButton {
                    button: MouseButton::Left,
                    mode: ActionMode::Release,
                },
            },
        ]);

        recorded.trim_trailing_mouse_click();

        assert_eq!(recorded.actions.len(), 2);
        assert!(
            recorded
                .actions
                .iter()
                .all(|action| action.kind.is_mouse_move())
        );
    }

    #[test]
    fn json_round_trip_keeps_actions() {
        let recorded = RecordedMacro::new(vec![RecordedAction {
            delay_ms: 25,
            kind: RecordedActionKind::Scroll {
                axis: ScrollAxis::Vertical,
                amount: -1,
            },
        }]);

        let json = recorded.to_json().unwrap();
        let restored = RecordedMacro::from_json(&json).unwrap();

        assert_eq!(recorded, restored);
    }
}
