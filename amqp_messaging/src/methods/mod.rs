mod consume;
mod queue;

use amqp_core::amqp_todo;
use amqp_core::error::ProtocolError;
use amqp_core::message::Message;
use amqp_core::methods::Method;
use amqp_core::ChannelHandle;
use tracing::info;

pub async fn handle_basic_publish(_channel_handle: ChannelHandle, message: Message) {
    info!(
        ?message,
        "Someone has summoned the almighty Basic.Publish handler"
    );
}

pub async fn handle_method(
    channel_handle: ChannelHandle,
    method: Method,
) -> Result<(), ProtocolError> {
    info!(?method, "Handling method");

    match method {
        Method::ExchangeDeclare { .. } => amqp_todo!(),
        Method::ExchangeDeclareOk => amqp_todo!(),
        Method::ExchangeDelete { .. } => amqp_todo!(),
        Method::ExchangeDeleteOk => amqp_todo!(),
        Method::QueueDeclare {
            queue,
            passive,
            durable,
            exclusive,
            auto_delete,
            no_wait,
            arguments,
            ..
        } => {
            queue::declare(
                channel_handle,
                queue,
                passive,
                durable,
                exclusive,
                auto_delete,
                no_wait,
                arguments,
            )
            .await
        }
        Method::QueueDeclareOk { .. } => amqp_todo!(),
        Method::QueueBind {
            queue,
            exchange,
            routing_key,
            no_wait,
            arguments,
            ..
        } => {
            queue::bind(
                channel_handle,
                queue,
                exchange,
                routing_key,
                no_wait,
                arguments,
            )
            .await
        }
        Method::QueueBindOk => amqp_todo!(),
        Method::QueueUnbind { .. } => amqp_todo!(),
        Method::QueueUnbindOk => amqp_todo!(),
        Method::QueuePurge { .. } => amqp_todo!(),
        Method::QueuePurgeOk { .. } => amqp_todo!(),
        Method::QueueDelete { .. } => amqp_todo!(),
        Method::QueueDeleteOk { .. } => amqp_todo!(),
        Method::BasicQos { .. } => amqp_todo!(),
        Method::BasicQosOk => amqp_todo!(),
        Method::BasicConsume {
            queue,
            consumer_tag,
            no_local,
            no_ack,
            exclusive,
            no_wait,
            arguments,
            ..
        } => {
            consume::consume(
                channel_handle,
                queue,
                consumer_tag,
                no_local,
                no_ack,
                exclusive,
                no_wait,
                arguments,
            )
            .await
        }
        Method::BasicConsumeOk { .. } => amqp_todo!(),
        Method::BasicCancel { .. } => amqp_todo!(),
        Method::BasicCancelOk { .. } => amqp_todo!(),
        Method::BasicReturn { .. } => amqp_todo!(),
        Method::BasicDeliver { .. } => amqp_todo!(),
        Method::BasicGet { .. } => amqp_todo!(),
        Method::BasicGetOk { .. } => amqp_todo!(),
        Method::BasicGetEmpty { .. } => amqp_todo!(),
        Method::BasicAck { .. } => amqp_todo!(),
        Method::BasicReject { .. } => amqp_todo!(),
        Method::BasicRecoverAsync { .. } => amqp_todo!(),
        Method::BasicRecover { .. } => amqp_todo!(),
        Method::BasicRecoverOk => amqp_todo!(),
        Method::TxSelect
        | Method::TxSelectOk
        | Method::TxCommit
        | Method::TxCommitOk
        | Method::TxRollback
        | Method::TxRollbackOk => amqp_todo!(),
        Method::BasicPublish { .. } => {
            unreachable!("Basic.Publish is handled somewhere else because it has a body")
        }
        _ => unreachable!("Method handled by transport layer"),
    }
}
