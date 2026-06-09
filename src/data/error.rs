use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("card not found: {0}")]
    CardNotFound(String),

    #[error("timer not found: {0}")]
    TimerNotFound(String),

    #[error("preset not found: {0}")]
    PresetNotFound(String),

    #[error("todo not found: {0}")]
    TodoNotFound(String),

    #[error("schedule not found: {0}")]
    ScheduleNotFound(String),

    #[error("invalid data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, DataError>;
