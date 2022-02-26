use amqp_core::error::{ConException, ProtocolError};
use amqp_core::message::Message;
use amqp_core::methods::Method;
use amqp_core::ChannelHandle;
use tracing::{debug, info};

pub async fn handle_basic_publish(_channel_handle: ChannelHandle, message: Message) {
    info!(
        ?message,
        "Someone has summoned the almighty Basic.Publish handler"
    );
}

pub async fn handle_method(
    _channel_handle: ChannelHandle,
    method: Method,
) -> Result<(), ProtocolError> {
    match method {
        Method::ExchangeDeclare { .. } => Err(ConException::NotImplemented.into()),
        Method::ExchangeDeclareOk => Err(ConException::NotImplemented.into()),
        Method::ExchangeDelete { .. } => Err(ConException::NotImplemented.into()),
        Method::ExchangeDeleteOk => Err(ConException::NotImplemented.into()),
        Method::QueueDeclare { .. } => Err(ConException::NotImplemented.into()),
        Method::QueueDeclareOk { .. } => Err(ConException::NotImplemented.into()),
        Method::QueueBind { .. } => Err(ConException::NotImplemented.into()),
        Method::QueueBindOk => Err(ConException::NotImplemented.into()),
        Method::QueueUnbind { .. } => Err(ConException::NotImplemented.into()),
        Method::QueueUnbindOk => Err(ConException::NotImplemented.into()),
        Method::QueuePurge { .. } => Err(ConException::NotImplemented.into()),
        Method::QueuePurgeOk { .. } => Err(ConException::NotImplemented.into()),
        Method::QueueDelete { .. } => Err(ConException::NotImplemented.into()),
        Method::QueueDeleteOk { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicQos { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicQosOk => Err(ConException::NotImplemented.into()),
        Method::BasicConsume { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicConsumeOk { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicCancel { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicCancelOk { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicReturn { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicDeliver { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicGet { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicGetOk { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicGetEmpty { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicAck { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicReject { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicRecoverAsync { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicRecover { .. } => Err(ConException::NotImplemented.into()),
        Method::BasicRecoverOk => Err(ConException::NotImplemented.into()),
        Method::TxSelect
        | Method::TxSelectOk
        | Method::TxCommit
        | Method::TxCommitOk
        | Method::TxRollback
        | Method::TxRollbackOk => Err(ConException::NotImplemented.into()),
        Method::BasicPublish { .. } => {
            unreachable!("Basic.Publish is handled somewhere else because it has a body")
        }
        _ => unreachable!("Method handled by transport layer"),
    }
}
