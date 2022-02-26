use amqp_core::connection::ChannelHandle;
use amqp_core::error::ProtocolError;
use amqp_core::methods::BasicConsume;

pub async fn consume(
    _channel_handle: ChannelHandle,
    _basic_consume: BasicConsume,
) -> Result<(), ProtocolError> {
    Ok(())
}
