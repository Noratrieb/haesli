use amqp_core::connection::ChannelHandle;
use amqp_core::error::{ConException, ProtocolError};
use amqp_core::methods::{Method, QueueBind, QueueDeclare, QueueDeclareOk};
use amqp_core::queue::{QueueDeletion, QueueId, QueueName, RawQueue};
use amqp_core::{amqp_todo, GlobalData};
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub async fn declare(
    channel_handle: ChannelHandle,
    queue_declare: QueueDeclare,
) -> Result<Method, ProtocolError> {
    let QueueDeclare {
        queue: queue_name,
        passive,
        durable,
        exclusive,
        auto_delete,
        no_wait,
        arguments,
        ..
    } = queue_declare;

    let queue_name = QueueName::new(queue_name.into());

    if !arguments.is_empty() {
        amqp_todo!();
    }

    if passive || no_wait || durable {
        amqp_todo!();
    }

    let global_data = {
        let channel = channel_handle.lock();
        let global_data = channel.global_data.clone();

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

        {
            let mut global_data_lock = global_data.lock();
            global_data_lock.queues.insert(queue_name.clone(), queue);
        }

        global_data
    };

    bind_queue(global_data, (), queue_name.clone().into_inner()).await?;

    Ok(Method::QueueDeclareOk(QueueDeclareOk {
        queue: queue_name.to_string(),
        message_count: 0,
        consumer_count: 0,
    }))
}

pub async fn bind(
    _channel_handle: ChannelHandle,
    _queue_bind: QueueBind,
) -> Result<Method, ProtocolError> {
    amqp_todo!();
}

async fn bind_queue(
    global_data: GlobalData,
    _exchange: (),
    routing_key: Arc<str>,
) -> Result<(), ProtocolError> {
    let mut global_data = global_data.lock();

    // todo: don't
    let queue = global_data
        .queues
        .get(&QueueName::new(routing_key.clone()))
        .unwrap()
        .clone();
    global_data
        .default_exchange
        .insert(routing_key.to_string(), queue);

    Ok(())
}
