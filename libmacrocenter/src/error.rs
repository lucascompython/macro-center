use thiserror::Error;

#[derive(Debug, Error)]
pub enum MacroCenterError {
    #[error("Input simulation error: {0}")]
    InputError(String),

    #[error("Hotkey error: {0}")]
    HotkeyError(String),

    #[error("Recorder error: {0}")]
    RecorderError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, MacroCenterError>;
