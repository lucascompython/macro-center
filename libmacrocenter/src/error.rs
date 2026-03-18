use thiserror::Error;

#[derive(Debug, Error)]
pub enum MacroCenterError {
    #[error("Input simulation error: {0}")]
    InputError(String),

    #[error("Hotkey error: {0}")]
    HotkeyError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, MacroCenterError>;
