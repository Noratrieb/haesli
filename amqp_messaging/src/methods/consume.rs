use amqp_core::connection::ChannelHandle;
use amqp_core::error::ProtocolError;
use amqp_core::methods::{Bit, ConsumerTag, NoAck, NoLocal, NoWait, QueueName, Table};

#[allow(clippy::too_many_arguments)]
pub async fn consume(
    _channel_handle: ChannelHandle,
    _queue: QueueName,
    _consumer_tag: ConsumerTag,
    _no_local: NoLocal,
    _no_ack: NoAck,
    _exclusive: Bit,
    _no_wait: NoWait,
    _arguments: Table,
) -> Result<(), ProtocolError> {
    Ok(())
}
