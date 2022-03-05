use amqp_core::{
    queue::{Queue, QueueEventReceiver},
    GlobalData,
};
use tracing::{debug, info};

#[derive(Debug)]
#[allow(dead_code)]
pub struct QueueTask {
    global_data: GlobalData,
    event_recv: QueueEventReceiver,
    queue: Queue,
}

impl QueueTask {
    pub fn new(global_data: GlobalData, event_recv: QueueEventReceiver, queue: Queue) -> Self {
        Self {
            global_data,
            event_recv,
            queue,
        }
    }

    pub async fn start(mut self) {
        info!("Started queue worker task");

        loop {
            let next_event = self.event_recv.recv().await;

            match next_event {
                Some(event) => debug!(?event, "Received event"),
                None => {
                    self.cleanup().await;
                    return;
                }
            }
        }
    }

    async fn cleanup(&mut self) {
        // do stuff or something like that id whatever
    }
}
