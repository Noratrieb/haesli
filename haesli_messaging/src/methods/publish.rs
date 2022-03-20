use std::sync::Arc;

use haesli_core::{
    amqp_todo,
    connection::Channel,
    error::{ChannelException, ConException},
    message::Message,
    queue::QueueEvent,
};
use tracing::{debug, error};

use crate::{routing, Result};

pub fn publish(channel_handle: Channel, message: Message) -> Result<()> {
    debug!(?message, "Publishing message");

    let global_data = channel_handle.global_data.clone();

    let routing = &message.routing;

    if !routing.exchange.is_empty() {
        amqp_todo!();
    }

    let global_data = global_data.lock();

    let exchange = &message.routing.exchange;

    let exchange = global_data
        .exchanges
        .get(exchange.as_str())
        .ok_or(ChannelException::NotFound)?;

    let queues = routing::route_message(exchange, &message.routing.routing_key)
        .ok_or(ChannelException::NotFound)?; // todo this isn't really correct but the tests pass ✔️

    for queue in queues {
        queue
            .event_send
            .try_send(QueueEvent::PublishMessage(Arc::clone(&message)))
            .map_err(|err| {
                error!(?err, "Failed to send message to queue event queue");
                ConException::InternalError
            })?;
    }
    Ok(())
}
