use crate::methods::{ReplyCode, ReplyText};

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
    GracefullyClosed,
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
    #[error("540 Not implemented. '{0}'")]
    NotImplemented(&'static str),
    #[error("xxx Not decided yet")]
    Todo,
}

impl ConException {
    pub fn reply_code(&self) -> ReplyCode {
        match self {
            ConException::FrameError => 501,
            ConException::CommandInvalid => 503,
            ConException::SyntaxError(_) => 503,
            ConException::ChannelError => 504,
            ConException::UnexpectedFrame => 505,
            ConException::NotImplemented(_) => 540,
            ConException::Todo => 0,
        }
    }
    pub fn reply_text(&self) -> ReplyText {
        "cant be bothered yet".to_string() // todo
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelException {}

impl ChannelException {
    pub fn reply_code(&self) -> ReplyCode {
        todo!()
    }
    pub fn reply_text(&self) -> ReplyText {
        todo!()
    }
}
