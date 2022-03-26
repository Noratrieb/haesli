use std::{ops::Not, sync::Arc};

use haesli_core::{
    amqp_todo,
    connection::Channel,
    consumer::{Consumer, ConsumerId},
    error::ChannelException,
    methods::{BasicConsume, BasicConsumeOk, Method},
};
use tracing::info;

use crate::methods::MethodResponse;

pub fn consume(channel: Channel, basic_consume: BasicConsume) -> MethodResponse {
    let BasicConsume {
        queue: queue_name,
        consumer_tag,
        no_local,
        no_ack,
        exclusive,
        no_wait,
        ..
    } = basic_consume;

    if no_local || exclusive || no_ack {
        amqp_todo!();
    }

    let global_data = channel.global_data.clone();

    let consumer_tag = if consumer_tag.is_empty() {
        haesli_core::random_uuid().to_string()
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

    Ok(no_wait
        .not()
        .then(|| Method::BasicConsumeOk(BasicConsumeOk { consumer_tag })))
}
