use crate::Result;
use amqp_core::{
    amqp_todo,
    connection::Channel,
    consumer::{Consumer, ConsumerId},
    error::ChannelException,
    methods::{BasicConsume, BasicConsumeOk, Method},
};
use std::sync::Arc;
use tracing::info;

pub fn consume(channel: Channel, basic_consume: BasicConsume) -> Result<Method> {
    let BasicConsume {
        queue: queue_name,
        consumer_tag,
        no_local,
        no_ack,
        exclusive,
        no_wait,
        ..
    } = basic_consume;

    if no_wait || no_local || exclusive || no_ack {
        amqp_todo!();
    }

    let global_data = channel.global_data.clone();

    let consumer_tag = if consumer_tag.is_empty() {
        amqp_core::random_uuid().to_string()
    } else {
        consumer_tag
    };

    let mut global_data = global_data.lock();

    let queue = global_data
        .queues
        .get_mut(queue_name.as_str())
        .ok_or(ChannelException::NotFound)?;

    let consumer = Consumer {
        id: ConsumerId::random(),
        tag: consumer_tag.clone(),
        channel: Arc::clone(&channel),
        queue: Arc::clone(queue),
    };

    queue.consumers.lock().insert(consumer.id, consumer.clone());

    channel.connection.consuming.lock().push(consumer);

    info!(%queue_name, %consumer_tag, "Consumer started consuming");

    let method = Method::BasicConsumeOk(BasicConsumeOk { consumer_tag });

    Ok(method)
}
