use amqp_core::message::Message;
use amqp_core::methods::Method;
use amqp_core::ChannelHandle;
use std::time::Duration;
use tokio::time;
use tracing::{debug, info};

pub async fn handle_basic_publish(_channel_handle: ChannelHandle, message: Message) {
    info!(
        ?message,
        "Someone has summoned the almighty Basic.Publish handler"
    );
}

pub async fn handle_method(_channel_handle: ChannelHandle, _method: Method) {
    debug!("handling method or something in that cool new future");
    time::sleep(Duration::from_secs(10)).await;
}
