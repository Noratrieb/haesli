mod consume;
mod exchange;
mod publish;
mod queue;

use haesli_core::{amqp_todo, connection::Channel, error::ConException, methods::Method};
pub use publish::publish;
use tracing::{info, warn};

use crate::Result;

type MethodResponse = Result<Option<Method>>;

/// This is the entrypoint of methods not handled by the connection itself.
/// Note that Basic.Publish is *not* sent here, but to [`handle_basic_publish`](crate::handle_basic_publish)
pub fn handle_method(channel: Channel, method: Method) -> Result<Option<Method>> {
    use Method::*;

    info!(?method, "Handling method");

    let response = match method {
        ExchangeDeclare(exchange_declare) => exchange::declare(channel, exchange_declare)?,
        ExchangeDelete(_) => amqp_todo!(),
        QueueDeclare(queue_declare) => queue::declare(channel, queue_declare)?,
        QueueBind(queue_bind) => queue::bind(channel, queue_bind)?,
        QueueUnbind(_) => amqp_todo!(),
        QueuePurge(_) => amqp_todo!(),
        QueueDelete(_) => amqp_todo!(),
        BasicQos(_) => amqp_todo!(),
        BasicConsume(consume) => consume::consume(channel, consume)?,
        BasicCancel(_) => amqp_todo!(),
        BasicGet(_) => amqp_todo!(),
        BasicAck(_) => amqp_todo!(),
        BasicReject(_) => amqp_todo!(),
        BasicRecoverAsync(_) => amqp_todo!(),
        BasicRecover(_) => amqp_todo!(),
        TxSelect(_) => amqp_todo!(),
        TxSelectOk(_) => amqp_todo!(),
        TxCommit(_) => amqp_todo!(),
        TxRollback(_) => amqp_todo!(),
        BasicPublish(_) => {
            unreachable!("Basic.Publish is handled somewhere else because it has a body")
        }
        ConnectionStartOk(_)
        | ConnectionSecureOk(_)
        | ConnectionTuneOk(_)
        | ConnectionOpenOk(_)
        | ConnectionCloseOk(_)
        | ChannelOpenOk(_)
        | ChannelFlowOk(_)
        | ChannelCloseOk(_)
        | ExchangeDeclareOk(_)
        | ExchangeDeleteOk(_)
        | QueueDeclareOk(_)
        | QueueBindOk(_)
        | QueueUnbindOk(_)
        | QueuePurgeOk(_)
        | QueueDeleteOk(_)
        | BasicQosOk(_)
        | BasicCancelOk(_)
        | BasicConsumeOk(_)
        | BasicReturn(_)
        | BasicDeliver(_)
        | BasicGetOk(_)
        | BasicGetEmpty(_)
        | BasicRecoverOk(_)
        | TxCommitOk(_)
        | TxRollbackOk(_) => return Err(ConException::NotAllowed.into()), // only sent by server
        ConnectionStart(_) | ConnectionSecure(_) | ConnectionTune(_) | ConnectionOpen(_)
        | ConnectionClose(_) | ChannelOpen(_) | ChannelFlow(_) | ChannelClose(_) => {
            warn!("method should be processed by transport layer");
            return Err(ConException::NotAllowed.into());
        }
    };

    Ok(response)
}
