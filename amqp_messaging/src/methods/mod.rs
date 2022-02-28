mod consume;
mod queue;

use amqp_core::amqp_todo;
use amqp_core::connection::ChannelHandle;
use amqp_core::error::ProtocolError;
use amqp_core::message::Message;
use amqp_core::methods::Method;
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
) -> Result<Method, ProtocolError> {
    info!(?method, "Handling method");

    let response = match method {
        Method::ExchangeDeclare(_) => amqp_todo!(),
        Method::ExchangeDeclareOk(_) => amqp_todo!(),
        Method::ExchangeDelete(_) => amqp_todo!(),
        Method::ExchangeDeleteOk(_) => amqp_todo!(),
        Method::QueueDeclare(queue_declare) => {
            queue::declare(channel_handle, queue_declare).await?
        }
        Method::QueueDeclareOk { .. } => amqp_todo!(),
        Method::QueueBind(queue_bind) => queue::bind(channel_handle, queue_bind).await?,
        Method::QueueBindOk(_) => amqp_todo!(),
        Method::QueueUnbind { .. } => amqp_todo!(),
        Method::QueueUnbindOk(_) => amqp_todo!(),
        Method::QueuePurge { .. } => amqp_todo!(),
        Method::QueuePurgeOk { .. } => amqp_todo!(),
        Method::QueueDelete { .. } => amqp_todo!(),
        Method::QueueDeleteOk { .. } => amqp_todo!(),
        Method::BasicQos { .. } => amqp_todo!(),
        Method::BasicQosOk(_) => amqp_todo!(),
        Method::BasicConsume(consume) => consume::consume(channel_handle, consume).await?,
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
        Method::BasicRecoverOk(_) => amqp_todo!(),
        Method::TxSelect(_)
        | Method::TxSelectOk(_)
        | Method::TxCommit(_)
        | Method::TxCommitOk(_)
        | Method::TxRollback(_)
        | Method::TxRollbackOk(_) => amqp_todo!(),
        Method::BasicPublish { .. } => {
            unreachable!("Basic.Publish is handled somewhere else because it has a body")
        }
        _ => unreachable!("Method handled by transport layer"),
    };

    Ok(response)
}
