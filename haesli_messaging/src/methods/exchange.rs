use std::{collections::HashMap, ops::Not};

use haesli_core::{
    amqp_todo,
    connection::Channel,
    error::ConException,
    exchange::{Exchange, ExchangeName, ExchangeType},
    methods::{ExchangeDeclare, ExchangeDeclareOk, Method},
};
use tracing::info;

use crate::methods::MethodResponse;

fn parse_exchange_type(str: &str) -> Option<ExchangeType> {
    match str {
        "direct" => Some(ExchangeType::Direct {
            bindings: HashMap::new(),
        }),
        "fanout" => Some(ExchangeType::Fanout {
            bindings: Vec::new(),
        }),
        "topic" => Some(ExchangeType::Topic {
            bindings: Vec::new(),
        }),
        _ => None,
    }
}

pub fn declare(channel: Channel, exchange_declare: ExchangeDeclare) -> MethodResponse {
    let ExchangeDeclare {
        exchange: name,
        r#type: kind,
        passive,
        durable,
        no_wait,
        arguments,
        ..
    } = exchange_declare;

    if !arguments.is_empty() {
        amqp_todo!();
    }

    // todo: implement durable

    if passive {
        amqp_todo!();
    }

    let name = ExchangeName::new(name.into());

    let kind = parse_exchange_type(&kind).ok_or(ConException::CommandInvalid)?;

    info!(%name, "Creating exchange");

    let exchange = Exchange {
        name: name.clone(),
        durable,
        kind,
    };

    {
        let mut global_data = channel.global_data.lock();
        global_data.exchanges.entry(name).or_insert(exchange);
    }

    Ok(no_wait
        .not()
        .then(|| Method::ExchangeDeclareOk(ExchangeDeclareOk)))
}
