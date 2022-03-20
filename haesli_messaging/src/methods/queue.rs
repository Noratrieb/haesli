use std::sync::{atomic::AtomicUsize, Arc};

use haesli_core::{
    amqp_todo,
    connection::Channel,
    methods::{Method, QueueBind, QueueDeclare, QueueDeclareOk},
    queue::{QueueDeletion, QueueId, QueueInner, QueueName},
    GlobalData,
};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tracing::debug;

use crate::{queue_worker::QueueTask, routing, Result};

pub fn declare(channel: Channel, queue_declare: QueueDeclare) -> Result<Method> {
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

    // 2.1.4.1 - If no queue name is given, chose a name
    let queue_name = if !queue_name.is_empty() {
        queue_name
    } else {
        format!("q_{}", haesli_core::random_uuid())
    };

    let queue_name = QueueName::new(queue_name.into());

    if !arguments.is_empty() {
        amqp_todo!();
    }

    // todo: implement durable, not checked here because it's the amqplib default

    if passive || no_wait {
        amqp_todo!();
    }

    let global_data = channel.global_data.clone();

    let (event_send, event_recv) = mpsc::channel(10);

    let id = QueueId::random();
    let queue = Arc::new(QueueInner {
        id,
        name: queue_name.clone(),
        messages: haesli_datastructure::MessageQueue::new(),
        durable,
        exclusive: exclusive.then(|| channel.id),
        deletion: if auto_delete {
            QueueDeletion::Auto(AtomicUsize::default())
        } else {
            QueueDeletion::Manual
        },
        consumers: Mutex::default(),
        event_send,
    });

    debug!(%queue_name, "Creating queue");

    {
        let mut global_data_lock = global_data.lock();

        global_data_lock
            .queues
            .entry(queue_name.clone())
            .or_insert_with(|| queue.clone());
    }

    bind_queue(global_data.clone(), (), queue_name.to_string())?;

    let queue_task = QueueTask::new(global_data, event_recv, queue);

    tokio::spawn(async move { queue_task.start().await });

    Ok(Method::QueueDeclareOk(QueueDeclareOk {
        queue: queue_name.to_string(),
        message_count: 0,
        consumer_count: 0,
    }))
}

pub fn bind(_channel_handle: Channel, _queue_bind: QueueBind) -> Result<Method> {
    amqp_todo!();
}

fn bind_queue(global_data: GlobalData, _exchange: (), routing_key: String) -> Result<()> {
    let mut global_data = global_data.lock();

    // todo: don't
    let queue = global_data
        .queues
        .get(&QueueName::new(routing_key.clone().into()))
        .unwrap()
        .clone();

    let exchange = global_data
        .exchanges
        .get_mut("")
        .expect("default empty exchange");

    routing::bind(exchange, routing_key, queue);

    Ok(())
}
