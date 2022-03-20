use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::Debug,
    sync::{atomic::AtomicUsize, Arc},
};

use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::{
    consumer::{Consumer, ConsumerId},
    message::Message,
    newtype, newtype_id, ChannelId,
};

pub type Queue = Arc<QueueInner>;

#[derive(Debug)]
pub enum QueueEvent {
    PublishMessage(Message),
    Shutdown,
}

pub type QueueEventSender = mpsc::Sender<QueueEvent>;
pub type QueueEventReceiver = mpsc::Receiver<QueueEvent>;

newtype_id!(pub QueueId);

newtype!(
    /// The name of a queue. A newtype wrapper around `Arc<str>`, which guarantees cheap clones.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub QueueName: Arc<str>
);

impl Borrow<str> for QueueName {
    fn borrow(&self) -> &str {
        Borrow::borrow(&self.0)
    }
}

#[derive(Debug)]
pub struct QueueInner {
    /// Internal ID, might actually be unused
    pub id: QueueId,
    /// The visible name of the queue
    pub name: QueueName,
    pub messages: haesli_datastructure::MessageQueue<Message>,
    /// Whether the queue should be kept when the server restarts
    pub durable: bool,
    /// To which connection the queue belongs to it will be deleted when the connection closes
    // todo: connection or channel?
    pub exclusive: Option<ChannelId>,
    /// Whether the queue will automatically be deleted when no consumers uses it anymore.
    /// The queue can always be manually deleted.
    /// If auto-delete is enabled, it keeps track of the consumer count.
    pub deletion: QueueDeletion,
    pub consumers: Mutex<HashMap<ConsumerId, Consumer>>,
    pub event_send: QueueEventSender,
}

#[derive(Debug)]
pub enum QueueDeletion {
    Auto(AtomicUsize),
    Manual,
}
