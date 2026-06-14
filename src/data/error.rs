use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DataError {
    #[error("io error: {0}")]
    Io(String),

    #[error("json error: {0}")]
    Json(String),

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

    #[error("folder not found: {0}")]
    FolderNotFound(String),

    #[error("invalid node name: {0}")]
    InvalidNodeName(String),

    #[error("node already exists: {0}")]
    NodeAlreadyExists(String),

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("config parse error: {0}")]
    ConfigParse(String),

    #[error("config serialize error: {0}")]
    ConfigSerialize(String),
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for DataError {
    fn from(e: serde_json::Error) -> Self {
        DataError::Json(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DataError>;
