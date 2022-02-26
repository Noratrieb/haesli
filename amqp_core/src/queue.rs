use crate::message::Message;
use crate::{newtype_id, ChannelId};
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub type Queue = Arc<RawQueue>;

newtype_id!(pub QueueId);

#[derive(Debug)]
pub struct RawQueue {
    pub id: QueueId,
    pub name: String,
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
