use amqp_core::methods::Method;
use amqp_core::ChannelHandle;
use std::time::Duration;
use tokio::time;
use tracing::debug;

pub async fn handle_method(_channel_handle: ChannelHandle, _method: Method) {
    debug!("handling method or something in that cool new future");
    time::sleep(Duration::from_secs(10)).await;
}
