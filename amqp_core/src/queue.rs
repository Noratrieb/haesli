use crate::message::Message;
use crate::{newtype, newtype_id, ChannelId};
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub type Queue = Arc<RawQueue>;

newtype_id!(pub QueueId);

newtype!(
    /// The name of a queue. A newtype wrapper around `Arc<str>`, which guarantees cheap clones.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub QueueName: Arc<str>
);

#[derive(Debug)]
pub struct RawQueue {
    pub id: QueueId,
    pub name: QueueName,
    pub messages: Mutex<Vec<Message>>, // use a concurrent linked list???
    pub durable: bool,
    pub exclusive: Option<ChannelId>,
    /// Whether the queue will automatically be deleted when no consumers uses it anymore.
    /// The queue can always be manually deleted.
    /// If auto-delete is enabled, it keeps track of the consumer count.
    pub deletion: QueueDeletion,
}

#[derive(Debug)]
pub enum QueueDeletion {
    Auto(AtomicUsize),
    Manual,
}
