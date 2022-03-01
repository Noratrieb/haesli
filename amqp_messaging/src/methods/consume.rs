use crate::Result;
use amqp_core::amqp_todo;
use amqp_core::connection::ChannelHandle;
use amqp_core::consumer::{Consumer, ConsumerId};
use amqp_core::error::{ChannelException};
use amqp_core::methods::{BasicConsume, BasicConsumeOk, Method};
use std::sync::Arc;
use tracing::info;

pub fn consume(channel_handle: ChannelHandle, basic_consume: BasicConsume) -> Result<Method> {
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

    let global_data = {
        let channel = channel_handle.lock();
        channel.global_data.clone()
    };

    let consumer_tag = if consumer_tag.is_empty() {
        amqp_core::random_uuid().to_string()
    } else {
        consumer_tag
    };

    let mut global_data = global_data.lock();

    let consumer = Consumer {
        id: ConsumerId::random(),
        tag: consumer_tag.clone(),
        channel: Arc::clone(&channel_handle),
    };

    let queue = global_data
        .queues
        .get_mut(queue_name.as_str())
        .ok_or(ChannelException::NotFound)?;

    queue.consumers.lock().push(consumer);

    info!(%queue_name, %consumer_tag, "Consumer started consuming");

    let method = Method::BasicConsumeOk(BasicConsumeOk { consumer_tag });

    Ok(method)
}
