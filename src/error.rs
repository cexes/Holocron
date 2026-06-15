use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("PTY error: {0}")]
    Pty(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
