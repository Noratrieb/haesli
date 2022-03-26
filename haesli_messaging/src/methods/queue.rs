use std::{
    ops::Not,
    sync::{atomic::AtomicUsize, Arc},
};

use haesli_core::{
    amqp_todo,
    connection::Channel,
    error::ChannelException,
    methods::{Method, QueueBind, QueueBindOk, QueueDeclare, QueueDeclareOk},
    queue::{Queue, QueueDeletion, QueueId, QueueInner, QueueName},
    GlobalData,
};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{methods::MethodResponse, queue_worker::QueueTask, routing, Result};

pub fn declare(channel: Channel, queue_declare: QueueDeclare) -> MethodResponse {
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

    if passive {
        amqp_todo!();
    }

    let global_data = channel.global_data.clone();

    let queue = {
        let global_data_lock = global_data.lock();
        global_data_lock.queues.get(&queue_name).cloned()
    };

    let queue = if let Some(queue) = queue {
        debug!(%queue_name, "Declaring queue that already exists");
        queue
    } else {
        info!(%queue_name, "Creating queue");

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

        bind_queue(
            global_data.clone(),
            queue.clone(),
            "",
            queue_name.to_string(),
        )?;

        {
            let mut global_data_lock = global_data.lock();
            global_data_lock
                .queues
                .insert(queue_name.clone(), queue.clone());
        }

        let queue_task = QueueTask::new(global_data, event_recv, queue.clone());

        tokio::spawn(async move { queue_task.start().await });

        queue
    };

    Ok(no_wait.not().then(|| {
        Method::QueueDeclareOk(QueueDeclareOk {
            queue: queue_name.to_string(),
            message_count: u32::try_from(queue.messages.len()).unwrap(),
            consumer_count: u32::try_from(queue.consumers.lock().len()).unwrap(),
        })
    }))
}

pub fn bind(channel_handle: Channel, queue_bind: QueueBind) -> MethodResponse {
    let QueueBind {
        queue,
        exchange,
        routing_key,
        no_wait,
        arguments,
        ..
    } = queue_bind;

    if !arguments.is_empty() {
        amqp_todo!();
    }

    let queue = {
        let global_data = channel_handle.global_data.lock();
        global_data
            .queues
            .get(queue.as_str())
            .ok_or(ChannelException::NotFound)?
            .clone()
    };

    bind_queue(
        channel_handle.global_data.clone(),
        queue,
        &exchange,
        routing_key,
    )?;

    Ok(no_wait.not().then(|| Method::QueueBindOk(QueueBindOk)))
}

fn bind_queue(
    global_data: GlobalData,
    queue: Queue,
    exchange: &str,
    routing_key: String,
) -> Result<()> {
    let mut global_data = global_data.lock();

    let exchange = global_data
        .exchanges
        .get_mut(exchange)
        .ok_or(ChannelException::NotFound)?;

    routing::bind(exchange, routing_key, queue);

    Ok(())
}
