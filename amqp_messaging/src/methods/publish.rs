use amqp_core::{
    amqp_todo,
    connection::Channel,
    error::{ChannelException, ConException},
    message::Message,
    queue::QueueEvent,
};
use tracing::{debug, error};

use crate::Result;

pub fn publish(channel_handle: Channel, message: Message) -> Result<()> {
    debug!(?message, "Publishing message");

    let global_data = channel_handle.global_data.clone();

    let routing = &message.routing;

    if !routing.exchange.is_empty() {
        amqp_todo!();
    }

    let global_data = global_data.lock();

    let queue = global_data
        .queues
        .get(routing.routing_key.as_str())
        .ok_or(ChannelException::NotFound)?;

    queue
        .event_send
        .try_send(QueueEvent::PublishMessage(message))
        .map_err(|err| {
            error!(?err, "Failed to send message to queue event queue");
            ConException::InternalError
        })?;

    Ok(())
}
