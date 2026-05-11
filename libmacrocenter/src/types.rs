use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::MacroCenterError;

/// Action mode for key and mouse button actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionMode {
    Click,
    Press,
    Release,
}

impl FromStr for ActionMode {
    type Err = MacroCenterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "click" => Ok(Self::Click),
            "press" => Ok(Self::Press),
            "release" => Ok(Self::Release),
            _ => Err(MacroCenterError::ParseError(format!(
                "Unknown action mode: '{s}'. Expected 'click', 'press', or 'release'"
            ))),
        }
    }
}

impl fmt::Display for ActionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Click => write!(f, "click"),
            Self::Press => write!(f, "press"),
            Self::Release => write!(f, "release"),
        }
    }
}

/// Coordinate mode for mouse movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoordinateMode {
    Absolute,
    Relative,
}

impl FromStr for CoordinateMode {
    type Err = MacroCenterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "absolute" | "abs" => Ok(Self::Absolute),
            "relative" | "rel" => Ok(Self::Relative),
            _ => Err(MacroCenterError::ParseError(format!(
                "Unknown coordinate mode: '{s}'. Expected 'absolute' or 'relative'"
            ))),
        }
    }
}

impl fmt::Display for CoordinateMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute => write!(f, "absolute"),
            Self::Relative => write!(f, "relative"),
        }
    }
}

/// Scroll axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}

impl FromStr for ScrollAxis {
    type Err = MacroCenterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vertical" | "v" => Ok(Self::Vertical),
            "horizontal" | "h" => Ok(Self::Horizontal),
            _ => Err(MacroCenterError::ParseError(format!(
                "Unknown scroll axis: '{s}'. Expected 'vertical' or 'horizontal'"
            ))),
        }
    }
}

impl fmt::Display for ScrollAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vertical => write!(f, "vertical"),
            Self::Horizontal => write!(f, "horizontal"),
        }
    }
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Mouse4,
    Mouse5,
}

impl FromStr for MouseButton {
    type Err = MacroCenterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "middle" => Ok(Self::Middle),
            "mouse4" | "back" => Ok(Self::Mouse4),
            "mouse5" | "forward" => Ok(Self::Mouse5),
            _ => Err(MacroCenterError::ParseError(format!(
                "Unknown mouse button: '{s}'. Expected 'left', 'right', 'middle', 'mouse4', or 'mouse5'"
            ))),
        }
    }
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Right => write!(f, "right"),
            Self::Middle => write!(f, "middle"),
            Self::Mouse4 => write!(f, "mouse4"),
            Self::Mouse5 => write!(f, "mouse5"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_mode() {
        assert_eq!("click".parse::<ActionMode>().unwrap(), ActionMode::Click);
        assert_eq!("Press".parse::<ActionMode>().unwrap(), ActionMode::Press);
        assert_eq!(
            "RELEASE".parse::<ActionMode>().unwrap(),
            ActionMode::Release
        );
        assert!("invalid".parse::<ActionMode>().is_err());
    }

    #[test]
    fn parse_coordinate_mode() {
        assert_eq!(
            "absolute".parse::<CoordinateMode>().unwrap(),
            CoordinateMode::Absolute
        );
        assert_eq!(
            "rel".parse::<CoordinateMode>().unwrap(),
            CoordinateMode::Relative
        );
        assert!("invalid".parse::<CoordinateMode>().is_err());
    }

    #[test]
    fn parse_scroll_axis() {
        assert_eq!(
            "vertical".parse::<ScrollAxis>().unwrap(),
            ScrollAxis::Vertical
        );
        assert_eq!("h".parse::<ScrollAxis>().unwrap(), ScrollAxis::Horizontal);
        assert!("invalid".parse::<ScrollAxis>().is_err());
    }

    #[test]
    fn parse_mouse_button() {
        assert_eq!("left".parse::<MouseButton>().unwrap(), MouseButton::Left);
        assert_eq!("Right".parse::<MouseButton>().unwrap(), MouseButton::Right);
        assert_eq!(
            "MIDDLE".parse::<MouseButton>().unwrap(),
            MouseButton::Middle
        );
        assert_eq!(
            "mouse4".parse::<MouseButton>().unwrap(),
            MouseButton::Mouse4
        );
        assert_eq!(
            "forward".parse::<MouseButton>().unwrap(),
            MouseButton::Mouse5
        );
        assert!("invalid".parse::<MouseButton>().is_err());
    }
}
