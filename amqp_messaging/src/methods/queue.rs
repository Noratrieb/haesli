use crate::{queue::QueueTask, Result};
use amqp_core::{
    amqp_todo,
    connection::Channel,
    methods::{Method, QueueBind, QueueDeclare, QueueDeclareOk},
    queue::{QueueDeletion, QueueId, QueueInner, QueueName},
    GlobalData,
};
use parking_lot::Mutex;
use std::sync::{atomic::AtomicUsize, Arc};
use tokio::sync::mpsc;
use tracing::{info_span, Instrument};

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

    let queue_name = QueueName::new(queue_name.into());

    if !arguments.is_empty() {
        amqp_todo!();
    }

    // todo: durable is technically spec-compliant, the spec doesn't really require it, but it's a todo
    // not checked here because it's the default for amqplib which is annoying
    if passive || no_wait {
        amqp_todo!();
    }

    let global_data = channel.global_data.clone();

    let (event_send, event_recv) = mpsc::channel(10);

    let id = QueueId::random();
    let queue = Arc::new(QueueInner {
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
        consumers: Mutex::default(),
        event_send,
    });

    {
        let mut global_data_lock = global_data.lock();

        global_data_lock
            .queues
            .entry(queue_name.clone())
            .or_insert_with(|| queue.clone());
    }

    bind_queue(global_data.clone(), (), queue_name.clone().into_inner())?;

    let queue_task = QueueTask::new(global_data, event_recv, queue);

    let queue_worker_span = info_span!(parent: None, "queue-worker", %queue_name);
    tokio::spawn(queue_task.start().instrument(queue_worker_span));

    Ok(Method::QueueDeclareOk(QueueDeclareOk {
        queue: queue_name.to_string(),
        message_count: 0,
        consumer_count: 0,
    }))
}

pub async fn bind(_channel_handle: Channel, _queue_bind: QueueBind) -> Result<Method> {
    amqp_todo!();
}

fn bind_queue(global_data: GlobalData, _exchange: (), routing_key: Arc<str>) -> Result<()> {
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
