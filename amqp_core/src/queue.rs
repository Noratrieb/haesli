use crate::{
    consumer::{Consumer, ConsumerId},
    message::Message,
    newtype, newtype_id, ChannelId,
};
use parking_lot::Mutex;
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    sync::{atomic::AtomicUsize, Arc},
};
use tokio::sync::mpsc;

pub type Queue = Arc<QueueInner>;

#[derive(Debug)]
pub enum QueueEvent {}

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
        std::borrow::Borrow::borrow(&self.0)
    }
}

impl Display for QueueName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub struct QueueInner {
    pub id: QueueId,
    pub name: QueueName,
    pub messages: Mutex<Vec<Message>>, // use a concurrent linked list???
    pub durable: bool,
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
