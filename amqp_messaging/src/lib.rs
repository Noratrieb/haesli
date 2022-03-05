#![warn(rust_2018_idioms)]

use amqp_core::error::ProtocolError;

pub mod methods;
mod queue;

type Result<T> = std::result::Result<T, ProtocolError>;
