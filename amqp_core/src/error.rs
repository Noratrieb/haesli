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
    #[error("540 Not implemented. '{0}'")]
    NotImplemented(&'static str),
    #[error("xxx Not decided yet")]
    Todo,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelException {}
