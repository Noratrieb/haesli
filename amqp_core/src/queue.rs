use crate::message::Message;
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use uuid::Uuid;

pub type Queue = Arc<RawQueue>;

#[derive(Debug)]
pub struct RawQueue {
    pub id: Uuid,
    pub name: String,
    pub messages: Mutex<Vec<Message>>, // use a concurrent linked list???
    pub durable: bool,
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
