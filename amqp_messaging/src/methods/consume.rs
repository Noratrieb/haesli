use amqp_core::amqp_todo;
use amqp_core::connection::ChannelHandle;
use amqp_core::error::ProtocolError;
use amqp_core::methods::{BasicConsume, Method};

pub async fn consume(
    channel_handle: ChannelHandle,
    _basic_consume: BasicConsume,
) -> Result<Method, ProtocolError> {
    let _channel = channel_handle.lock();

    amqp_todo!()
}
