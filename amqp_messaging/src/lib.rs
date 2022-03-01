#![warn(rust_2018_idioms)]

use amqp_core::error::ProtocolError;

pub mod methods;

type Result<T> = std::result::Result<T, ProtocolError>;
