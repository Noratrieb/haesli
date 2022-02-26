#![allow(dead_code)]

use std::io::Error;

pub use amqp_core::error::{ConException, ProtocolError};

pub type StdResult<T, E> = std::result::Result<T, E>;

pub type Result<T> = StdResult<T, TransError>;

#[derive(Debug, thiserror::Error)]
pub enum TransError {
    #[error("{0}")]
    Protocol(#[from] ProtocolError),
    #[error("connection error: `{0}`")]
    Other(#[from] anyhow::Error),
}

impl From<std::io::Error> for TransError {
    fn from(err: Error) -> Self {
        Self::Other(err.into())
    }
}

impl From<amqp_core::error::ConException> for TransError {
    fn from(err: ConException) -> Self {
        Self::Protocol(ProtocolError::ConException(err))
    }
}
