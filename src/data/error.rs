use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("record not found: {table} id={id}")]
    NotFound { table: String, id: i64 },

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("export error: {0}")]
    Export(String),
}

pub type Result<T> = std::result::Result<T, DataError>;