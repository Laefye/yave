pub mod client;
pub mod types;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("QMP connection error: {0}")]
    IO(#[from] std::io::Error),
    #[error("QMP serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("QMP protocol error: {0}")]
    Protocol(String),
    #[error("QMP channel closed")]
    ChannelClosed,
    #[error("QMP handshake missing greeting")]
    HandshakeMissing,
}


pub type Result<T> = std::result::Result<T, Error>;
