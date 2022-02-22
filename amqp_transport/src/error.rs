#![allow(dead_code)]

use std::io::Error;

pub type StdResult<T, E> = std::result::Result<T, E>;

pub type Result<T> = StdResult<T, TransError>;

#[derive(Debug, thiserror::Error)]
pub enum TransError {
    #[error("{0}")]
    Invalid(#[from] ProtocolError),
    #[error("connection error: `{0}`")]
    Other(#[from] anyhow::Error),
}

impl From<std::io::Error> for TransError {
    fn from(err: Error) -> Self {
        Self::Other(err.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("fatal error")]
    Fatal,
    #[error("{0}")]
    ConException(#[from] ConException),
    #[error("{0}")]
    ChannelException(#[from] ChannelException),
    #[error("Connection must be closed")]
    CloseNow,
    #[error("Graceful connection closing requested")]
    GracefulClose,
}

#[derive(Debug, thiserror::Error)]
pub enum ConException {
    #[error("501 Frame error")]
    FrameError,
    #[error("503 Command invalid")]
    CommandInvalid,
    #[error("503 Syntax error | {0:?}")]
    /// A method was received but there was a syntax error. The string stores where it occurred.
    SyntaxError(Vec<String>),
    #[error("504 Channel error")]
    ChannelError,
    #[error("505 Unexpected Frame")]
    UnexpectedFrame,
    #[error("xxx Not decided yet")]
    Todo,
}

impl ConException {
    pub fn into_trans(self) -> TransError {
        TransError::Invalid(ProtocolError::ConException(self))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelException {}
