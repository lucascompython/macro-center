use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

use crate::error::{MacroCenterError, Result};
use crate::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};

/// Input simulator wrapping enigo for keyboard and mouse simulation.
pub struct InputSimulator {
    enigo: Enigo,
}

impl InputSimulator {
    /// Create a new input simulator with default settings.
    pub fn new() -> Result<Self> {
        let enigo =
            Enigo::new(&Settings::default()).map_err(|e| MacroCenterError::InputError(e.to_string()))?;
        Ok(Self { enigo })
    }

    /// Type a string of text.
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo
            .text(text)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    /// Perform a key action (click, press, or release).
    ///
    /// The `key` parameter is a human-readable key name like "enter", "tab", "a", "F5", etc.
    pub fn key_action(&mut self, key: &str, mode: ActionMode) -> Result<()> {
        let key_enum = parse_key(key)?;
        let direction = action_mode_to_direction(mode);

        self.enigo
            .key(key_enum, direction)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    /// Perform a mouse button action (click, press, or release).
    pub fn mouse_click(&mut self, button: MouseButton, mode: ActionMode) -> Result<()> {
        let button_enum = mouse_button_to_enigo(button);
        let direction = action_mode_to_direction(mode);

        self.enigo
            .button(button_enum, direction)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    /// Move the mouse cursor.
    pub fn mouse_move(&mut self, x: i32, y: i32, mode: CoordinateMode) -> Result<()> {
        let coordinate = match mode {
            CoordinateMode::Absolute => Coordinate::Abs,
            CoordinateMode::Relative => Coordinate::Rel,
        };

        self.enigo
            .move_mouse(x, y, coordinate)
            .map_err(|e| MacroCenterError::InputError(e.to_string()))
    }

    /// Scroll the mouse wheel.
    pub fn scroll(&mut self, axis: ScrollAxis, amount: i32) -> Result<()> {
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
        "capslock" => Ok(Key::CapsLock),
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
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),
        _ => Err(MacroCenterError::ParseError(format!("Unknown key: '{key}'"))),
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
    }
}
