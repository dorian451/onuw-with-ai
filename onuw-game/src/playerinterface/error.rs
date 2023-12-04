use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum PlayerInterfaceError {
    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}

impl<T> From<SendError<T>> for PlayerInterfaceError {
    fn from(value: SendError<T>) -> Self {
        Self::CommunicationError(value.to_string())
    }
}
