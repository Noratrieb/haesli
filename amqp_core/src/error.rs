use crate::methods::{ReplyCode, ReplyText};

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("fatal error")]
    Fatal,
    #[error("{0}")]
    ConException(#[from] ConException),
    #[error("{0}")]
    ChannelException(#[from] ChannelException),
    #[error("Protocol negotiation failed")]
    ProtocolNegotiationFailed,
    #[error("Graceful connection closing requested")]
    GracefullyClosed,
}

#[derive(Debug, thiserror::Error)]
pub enum ConException {
    #[error("320 Connection forced")]
    ConnectionForced,
    #[error("402 Invalid path")]
    InvalidPath,
    #[error("501 Frame error")]
    FrameError,
    #[error("502 Syntax error | {0:?}")]
    /// A method was received but there was a syntax error. The string stores where it occurred.
    SyntaxError(Vec<String>),
    #[error("503 Command invalid")]
    CommandInvalid,
    #[error("504 Channel error")]
    ChannelError,
    #[error("505 Unexpected Frame")]
    UnexpectedFrame,
    #[error("506 Resource Error")]
    ResourceError,
    #[error("530 Not allowed")]
    NotAllowed,
    #[error("540 Not implemented. '{0}'")]
    NotImplemented(&'static str),
    #[error("541 Internal error")]
    InternalError,
    #[error("xxx Todo")]
    Todo,
}

impl ConException {
    pub fn reply_code(&self) -> ReplyCode {
        match self {
            ConException::ConnectionForced => 320,
            ConException::InvalidPath => 402,
            ConException::FrameError => 501,
            ConException::CommandInvalid => 503,
            ConException::SyntaxError(_) => 503,
            ConException::ChannelError => 504,
            ConException::UnexpectedFrame => 505,
            ConException::ResourceError => 506,
            ConException::NotAllowed => 530,
            ConException::InternalError => 541,
            ConException::NotImplemented(_) => 540,
            ConException::Todo => 0,
        }
    }
    pub fn reply_text(&self) -> ReplyText {
        match self {
            ConException::ConnectionForced => "connection-forced",
            ConException::InvalidPath => "invalid-path",
            ConException::FrameError => "frame-error",
            ConException::SyntaxError(_) => "syntax-error",
            ConException::CommandInvalid => "command-invalid",
            ConException::ChannelError => "channel-error",
            ConException::UnexpectedFrame => "unexpected-frame",
            ConException::ResourceError => "resource-error",
            ConException::NotAllowed => "not-allowed",
            ConException::NotImplemented(_) => "not-implemented",
            ConException::InternalError => "internal-error",
            ConException::Todo => "todo",
        }
        .to_owned()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelException {
    #[error("311 Content too large")]
    ContentTooLarge,
    #[error("313 No consumers")]
    NoConsumers,
    #[error("403 Access refused")]
    AccessRefused,
    #[error("404 Not found")]
    NotFound,
    #[error("405 Resource locked")]
    ResourceLocked,
    #[error("406 Precondition failed")]
    PreconditionFailed,
}

impl ChannelException {
    pub fn reply_code(&self) -> ReplyCode {
        match self {
            ChannelException::ContentTooLarge => 311,
            ChannelException::NoConsumers => 313,
            ChannelException::AccessRefused => 403,
            ChannelException::NotFound => 404,
            ChannelException::ResourceLocked => 405,
            ChannelException::PreconditionFailed => 406,
        }
    }
    pub fn reply_text(&self) -> ReplyText {
        match self {
            ChannelException::ContentTooLarge => "content-too-large",
            ChannelException::NoConsumers => "no-consumers",
            ChannelException::AccessRefused => "access-refused",
            ChannelException::NotFound => "not-found",
            ChannelException::ResourceLocked => "resource-locked",
            ChannelException::PreconditionFailed => "precondition-failed",
        }
        .to_owned()
    }
}
