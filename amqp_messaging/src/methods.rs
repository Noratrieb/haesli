use amqp_core::methods::Method;
use amqp_core::ChannelHandle;

pub async fn handle_method(_channel_handle: ChannelHandle, _method: Method) {}
