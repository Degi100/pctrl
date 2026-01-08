//! Error types

/// Application error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Coolify error: {0}")]
    Coolify(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
