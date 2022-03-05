use crate::Result;
use amqp_core::{
    amqp_todo,
    connection::{Channel, ConnectionEvent},
    error::ChannelException,
    message::Message,
    methods::{BasicDeliver, Method},
};
use tracing::debug;

pub async fn publish(channel_handle: Channel, message: Message) -> Result<()> {
    debug!(?message, "Publishing message");

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
            let method = Box::new(Method::BasicDeliver(BasicDeliver {
                consumer_tag: consumer.tag.clone(),
                delivery_tag: 0,
                redelivered: false,
                exchange: routing.exchange.clone(),
                routing_key: routing.routing_key.clone(),
            }));

            consumer
                .channel
                .event_sender
                .try_send(ConnectionEvent::MethodContent(
                    consumer.channel.num,
                    method,
                    message.header.clone(),
                    message.content.clone(),
                ))
                .unwrap();
        }
    }

    Ok(())
}
