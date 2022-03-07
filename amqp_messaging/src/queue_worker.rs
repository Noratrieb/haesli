use amqp_core::{
    connection::ConnectionEvent,
    consumer::Consumer,
    message::Message,
    methods::{BasicDeliver, Method},
    queue::{Queue, QueueEvent, QueueEventReceiver},
    GlobalData,
};
use std::borrow::Borrow;
use tracing::info;

#[derive(Debug)]
#[allow(dead_code)]
pub struct QueueTask {
    global_data: GlobalData,
    event_recv: QueueEventReceiver,
    queue: Queue,
}

impl QueueTask {
    fn show_name(&self) -> &str {
        self.queue.name.borrow()
    }

    pub fn new(global_data: GlobalData, event_recv: QueueEventReceiver, queue: Queue) -> Self {
        Self {
            global_data,
            event_recv,
            queue,
        }
    }

    #[tracing::instrument(skip(self), fields(name = self.show_name()))]
    pub async fn start(mut self) {
        info!("Started queue worker task");

        loop {
            let next_event = self.event_recv.recv().await;

            match next_event {
                Some(QueueEvent::PublishMessage(message)) => {
                    self.handle_publish_message(message).await
                }
                Some(QueueEvent::Shutdown) | None => {
                    self.cleanup().await;
                    return;
                }
            }
        }
    }

    #[tracing::instrument(skip(self), fields(name = self.show_name()), level = "debug")]
    async fn handle_publish_message(&mut self, message: Message) {
        // todo: we just send it to the consumer directly and ignore it if the consumer doesn't exist
        // consuming is hard, but this should work *for now*

        let could_deliver = {
            let consumers = self.queue.consumers.lock();
            if let Some(consumer) = consumers.values().next() {
                Self::try_deliver(&message, consumer)
            } else {
                Err(())
            }
        };

        if let Err(()) = could_deliver {
            self.queue_message(message).await;
        }
    }

    #[tracing::instrument(skip(consumer), level = "trace")]
    fn try_deliver(message: &Message, consumer: &Consumer) -> Result<(), ()> {
        let routing = &message.routing;

        let method = Box::new(Method::BasicDeliver(BasicDeliver {
            consumer_tag: consumer.tag.clone(),
            delivery_tag: 0,
            redelivered: false,
            exchange: routing.exchange.clone(),
            routing_key: routing.routing_key.clone(),
        }));

        let result = consumer
            .channel
            .event_sender
            .try_send(ConnectionEvent::MethodContent(
                consumer.channel.num,
                method,
                message.header.clone(),
                message.content.clone(),
            ));

        result.map_err(drop)
    }

    #[tracing::instrument(skip(self), fields(name = self.show_name()), level = "trace")]
    async fn queue_message(&mut self, message: Message) {
        self.queue.messages.append(message);
    }

    async fn cleanup(&mut self) {
        // do stuff or something like that id whatever
    }
}
