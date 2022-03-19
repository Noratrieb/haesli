mod consume;
mod publish;
mod queue;

use haesli_core::{connection::Channel, haesli_todo, message::Message, methods::Method};
use tracing::info;

use crate::Result;

pub fn handle_basic_publish(channel_handle: Channel, message: Message) -> Result<()> {
    publish::publish(channel_handle, message)
}

pub async fn handle_method(channel_handle: Channel, method: Method) -> Result<Method> {
    info!(?method, "Handling method");

    let response = match method {
        Method::ExchangeDeclare(_) => haesli_todo!(),
        Method::ExchangeDeclareOk(_) => haesli_todo!(),
        Method::ExchangeDelete(_) => haesli_todo!(),
        Method::ExchangeDeleteOk(_) => haesli_todo!(),
        Method::QueueDeclare(queue_declare) => queue::declare(channel_handle, queue_declare)?,
        Method::QueueDeclareOk { .. } => haesli_todo!(),
        Method::QueueBind(queue_bind) => queue::bind(channel_handle, queue_bind).await?,
        Method::QueueBindOk(_) => haesli_todo!(),
        Method::QueueUnbind { .. } => haesli_todo!(),
        Method::QueueUnbindOk(_) => haesli_todo!(),
        Method::QueuePurge { .. } => haesli_todo!(),
        Method::QueuePurgeOk { .. } => haesli_todo!(),
        Method::QueueDelete { .. } => haesli_todo!(),
        Method::QueueDeleteOk { .. } => haesli_todo!(),
        Method::BasicQos { .. } => haesli_todo!(),
        Method::BasicQosOk(_) => haesli_todo!(),
        Method::BasicConsume(consume) => consume::consume(channel_handle, consume)?,
        Method::BasicConsumeOk { .. } => haesli_todo!(),
        Method::BasicCancel { .. } => haesli_todo!(),
        Method::BasicCancelOk { .. } => haesli_todo!(),
        Method::BasicReturn { .. } => haesli_todo!(),
        Method::BasicDeliver { .. } => haesli_todo!(),
        Method::BasicGet { .. } => haesli_todo!(),
        Method::BasicGetOk { .. } => haesli_todo!(),
        Method::BasicGetEmpty { .. } => haesli_todo!(),
        Method::BasicAck { .. } => haesli_todo!(),
        Method::BasicReject { .. } => haesli_todo!(),
        Method::BasicRecoverAsync { .. } => haesli_todo!(),
        Method::BasicRecover { .. } => haesli_todo!(),
        Method::BasicRecoverOk(_) => haesli_todo!(),
        Method::TxSelect(_)
        | Method::TxSelectOk(_)
        | Method::TxCommit(_)
        | Method::TxCommitOk(_)
        | Method::TxRollback(_)
        | Method::TxRollbackOk(_) => haesli_todo!(),
        Method::BasicPublish { .. } => {
            unreachable!("Basic.Publish is handled somewhere else because it has a body")
        }
        _ => unreachable!("Method handled by transport layer"),
    };

    Ok(response)
}
