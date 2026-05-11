use enigo::Direction;
use enigo::{Axis, Button, Coordinate, Enigo, Key, Keyboard, Mouse, Settings};

use crate::error::{MacroCenterError, Result};
use crate::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};

/// Backend abstraction for emitting keyboard and mouse actions.
pub trait EmitterBackend {
    fn type_text(&mut self, text: &str) -> Result<()>;
    fn key_action(&mut self, key: &str, mode: ActionMode) -> Result<()>;
    fn mouse_click(&mut self, button: MouseButton, mode: ActionMode) -> Result<()>;
    fn mouse_move(&mut self, x: i32, y: i32, mode: CoordinateMode) -> Result<()>;
    fn scroll(&mut self, axis: ScrollAxis, amount: i32) -> Result<()>;
}

/// Input emitter wrapping enigo for keyboard and mouse simulation.
pub struct EnigoEmitter {
    enigo: Enigo,
}

/// Backwards-compatible name for the default input simulator.
pub type InputSimulator = EnigoEmitter;

impl EnigoEmitter {
    /// Create a new input emitter with default settings.
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| MacroCenterError::InputError(e.to_string()))?;
        Ok(Self { enigo })
    }

    /// Type a string of text.
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        EmitterBackend::type_text(self, text)
    }

    /// Perform a key action (click, press, or release).
    ///
    /// The `key` parameter is a human-readable key name like "enter", "tab", "a", "F5", etc.
    pub fn key_action(&mut self, key: &str, mode: ActionMode) -> Result<()> {
        EmitterBackend::key_action(self, key, mode)
    }

    /// Perform a mouse button action (click, press, or release).
    pub fn mouse_click(&mut self, button: MouseButton, mode: ActionMode) -> Result<()> {
        EmitterBackend::mouse_click(self, button, mode)
    }

    /// Move the mouse cursor.
    pub fn mouse_move(&mut self, x: i32, y: i32, mode: CoordinateMode) -> Result<()> {
        EmitterBackend::mouse_move(self, x, y, mode)
    }

    /// Scroll the mouse wheel.
    pub fn scroll(&mut self, axis: ScrollAxis, amount: i32) -> Result<()> {
        EmitterBackend::scroll(self, axis, amount)
    }
}

impl EmitterBackend for EnigoEmitter {
    fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo
            .text(text)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    fn key_action(&mut self, key: &str, mode: ActionMode) -> Result<()> {
        let key_enum = parse_key(key)?;
        let direction = action_mode_to_direction(mode);

        self.enigo
            .key(key_enum, direction)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    fn mouse_click(&mut self, button: MouseButton, mode: ActionMode) -> Result<()> {
        let button_enum = mouse_button_to_enigo(button);
        let direction = action_mode_to_direction(mode);

        self.enigo
            .button(button_enum, direction)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    fn mouse_move(&mut self, x: i32, y: i32, mode: CoordinateMode) -> Result<()> {
        let coordinate = match mode {
            CoordinateMode::Absolute => Coordinate::Abs,
            CoordinateMode::Relative => Coordinate::Rel,
        };

        self.enigo
            .move_mouse(x, y, coordinate)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    fn scroll(&mut self, axis: ScrollAxis, amount: i32) -> Result<()> {
        let direction = match axis {
            ScrollAxis::Vertical => Axis::Vertical,
            ScrollAxis::Horizontal => Axis::Horizontal,
        };

        self.enigo
            .scroll(amount, direction)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }
}

/// Parse a human-readable key string into an enigo `Key`.
pub fn parse_key(key: &str) -> Result<Key> {
    match key.to_lowercase().as_str() {
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "escape" | "esc" => Ok(Key::Escape),
        "control" | "ctrl" => Ok(Key::Control),
        "shift" => Ok(Key::Shift),
        "alt" => Ok(Key::Alt),
        "meta" | "super" | "win" | "cmd" => Ok(Key::Meta),
        "arrowup" | "up" => Ok(Key::UpArrow),
        "arrowdown" | "down" => Ok(Key::DownArrow),
        "arrowleft" | "left" => Ok(Key::LeftArrow),
        "arrowright" | "right" => Ok(Key::RightArrow),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "delete" | "del" => Ok(Key::Delete),
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "insert" => Ok(Key::Insert),
        "capslock" => Ok(Key::CapsLock),
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "printscreen" | "printscr" => Ok(Key::PrintScr),
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "numlock" => Ok(Key::Numlock),
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "pause" => Ok(Key::Pause),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "f13" => Ok(Key::F13),
        "f14" => Ok(Key::F14),
        "f15" => Ok(Key::F15),
        "f16" => Ok(Key::F16),
        "f17" => Ok(Key::F17),
        "f18" => Ok(Key::F18),
        "f19" => Ok(Key::F19),
        "f20" => Ok(Key::F20),
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),
        _ => Err(MacroCenterError::ParseError(format!(
            "Unknown key: '{key}'"
        ))),
    }
}

/// Convert an `ActionMode` to an enigo `Direction`.
fn action_mode_to_direction(mode: ActionMode) -> Direction {
    match mode {
        ActionMode::Click => Direction::Click,
        ActionMode::Press => Direction::Press,
        ActionMode::Release => Direction::Release,
    }
}

/// Convert a `MouseButton` to an enigo `Button`.
fn mouse_button_to_enigo(button: MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Right => Button::Right,
        MouseButton::Middle => Button::Middle,
        MouseButton::Mouse4 => Button::Back,
        MouseButton::Mouse5 => Button::Forward,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_keys() {
        assert!(matches!(parse_key("enter").unwrap(), Key::Return));
        assert!(matches!(parse_key("Return").unwrap(), Key::Return));
        assert!(matches!(parse_key("TAB").unwrap(), Key::Tab));
        assert!(matches!(parse_key("space").unwrap(), Key::Space));
        assert!(matches!(parse_key("Escape").unwrap(), Key::Escape));
        assert!(matches!(parse_key("esc").unwrap(), Key::Escape));
        assert!(matches!(parse_key("ctrl").unwrap(), Key::Control));
        assert!(matches!(parse_key("Control").unwrap(), Key::Control));
        assert!(matches!(parse_key("shift").unwrap(), Key::Shift));
        assert!(matches!(parse_key("alt").unwrap(), Key::Alt));
        assert!(matches!(parse_key("meta").unwrap(), Key::Meta));
        assert!(matches!(parse_key("super").unwrap(), Key::Meta));
        assert!(matches!(parse_key("win").unwrap(), Key::Meta));
        assert!(matches!(parse_key("cmd").unwrap(), Key::Meta));
    }

    #[test]
    fn parse_arrow_keys() {
        assert!(matches!(parse_key("up").unwrap(), Key::UpArrow));
        assert!(matches!(parse_key("ArrowUp").unwrap(), Key::UpArrow));
        assert!(matches!(parse_key("down").unwrap(), Key::DownArrow));
        assert!(matches!(parse_key("left").unwrap(), Key::LeftArrow));
        assert!(matches!(parse_key("right").unwrap(), Key::RightArrow));
    }

    #[test]
    fn parse_function_keys() {
        assert!(matches!(parse_key("f1").unwrap(), Key::F1));
        assert!(matches!(parse_key("F5").unwrap(), Key::F5));
        assert!(matches!(parse_key("f12").unwrap(), Key::F12));
    }

    #[test]
    fn parse_unicode_char() {
        assert!(matches!(parse_key("a").unwrap(), Key::Unicode('a')));
        assert!(matches!(parse_key("Z").unwrap(), Key::Unicode('z'))); // lowercase
        assert!(matches!(parse_key("1").unwrap(), Key::Unicode('1')));
    }

    #[test]
    fn parse_unknown_key() {
        assert!(parse_key("foobar").is_err());
        assert!(parse_key("").is_err());
    }

    #[test]
    fn action_mode_to_direction_mapping() {
        assert!(matches!(
            action_mode_to_direction(ActionMode::Click),
            Direction::Click
        ));
        assert!(matches!(
            action_mode_to_direction(ActionMode::Press),
            Direction::Press
        ));
        assert!(matches!(
            action_mode_to_direction(ActionMode::Release),
            Direction::Release
        ));
    }

    #[test]
    fn mouse_button_to_enigo_mapping() {
        assert!(matches!(
            mouse_button_to_enigo(MouseButton::Left),
            Button::Left
        ));
        assert!(matches!(
            mouse_button_to_enigo(MouseButton::Right),
            Button::Right
        ));
        assert!(matches!(
            mouse_button_to_enigo(MouseButton::Middle),
            Button::Middle
        ));
        assert!(matches!(
            mouse_button_to_enigo(MouseButton::Mouse4),
            Button::Back
        ));
        assert!(matches!(
            mouse_button_to_enigo(MouseButton::Mouse5),
            Button::Forward
        ));
    }
}
