#![warn(rust_2018_idioms)]
#![deny(clippy::future_not_send)]

use amqp_core::error::ProtocolError;

pub mod methods;
mod queue_worker;

type Result<T> = std::result::Result<T, ProtocolError>;
