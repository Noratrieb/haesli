use crate::Result;
use amqp_core::{
    amqp_todo,
    connection::{Channel, QueuedMethod},
    error::ChannelException,
    message::Message,
    methods::{BasicPublish, Method},
};
use tracing::info;

pub async fn publish(channel_handle: Channel, message: Message) -> Result<()> {
    info!(?message, "Publishing message");

    let global_data = channel_handle.global_data.clone();

    let routing = &message.routing;

    if !routing.exchange.is_empty() {
        amqp_todo!();
    }

    let mut global_data = global_data.lock();

    let queue = global_data
        .queues
        .get_mut(routing.routing_key.as_str())
        .ok_or(ChannelException::NotFound)?;

    {
        // todo: we just send it to the consumer directly and ignore it if the consumer doesn't exist
        // consuming is hard, but this should work *for now*
        let consumers = queue.consumers.lock();
        if let Some(consumer) = consumers.first() {
            let method = Method::BasicPublish(BasicPublish {
                reserved_1: 0,
                exchange: routing.exchange.clone(),
                routing_key: routing.routing_key.clone(),
                mandatory: false,
                immediate: false,
            });

            consumer.channel.queue_method(QueuedMethod::WithContent(
                method,
                message.header.clone(),
                message.content.clone(),
            ));
        }
    }

    Ok(())
}
