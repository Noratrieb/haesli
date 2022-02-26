use amqp_core::connection::ChannelHandle;
use amqp_core::error::{ConException, ProtocolError};
use amqp_core::methods::{Bit, ExchangeName, NoWait, QueueName, Shortstr, Table};
use amqp_core::queue::{QueueDeletion, QueueId, RawQueue};
use amqp_core::{amqp_todo, GlobalData};
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

#[allow(clippy::too_many_arguments)]
pub async fn declare(
    channel_handle: ChannelHandle,
    queue_name: QueueName,
    passive: Bit,
    durable: Bit,
    exclusive: Bit,
    auto_delete: Bit,
    no_wait: NoWait,
    arguments: Table,
) -> Result<(), ProtocolError> {
    if !arguments.is_empty() {
        return Err(ConException::Todo.into());
    }

    let (global_data, id) = {
        let channel = channel_handle.lock();

        if passive || no_wait {
            amqp_todo!();
        }

        let id = QueueId::random();
        let queue = Arc::new(RawQueue {
            id,
            name: queue_name.clone(),
            messages: Mutex::default(),
            durable,
            exclusive: exclusive.then(|| channel.id),
            deletion: if auto_delete {
                QueueDeletion::Auto(AtomicUsize::default())
            } else {
                QueueDeletion::Manual
            },
        });

        let global_data = channel.global_data.clone();

        {
            let mut global_data_lock = global_data.lock();
            global_data_lock.queues.insert(id, queue);
        }

        (global_data, id)
    };

    bind_queue(global_data, id, (), queue_name).await
}

pub async fn bind(
    _channel_handle: ChannelHandle,
    _queue: QueueName,
    _exchange: ExchangeName,
    _routing_key: Shortstr,
    _no_wait: NoWait,
    _arguments: Table,
) -> Result<(), ProtocolError> {
    amqp_todo!();
}

async fn bind_queue(
    _global_data: GlobalData,
    _queue: QueueId,
    _exchange: (),
    _routing_key: String,
) -> Result<(), ProtocolError> {
    amqp_todo!();
}
