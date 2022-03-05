#![warn(rust_2018_idioms)]

pub mod connection;
pub mod consumer;
pub mod error;
mod macros;
pub mod message;
pub mod methods;
pub mod queue;

use crate::{
    connection::{Channel, Connection},
    queue::{Queue, QueueName},
};
use connection::{ChannelId, ConnectionId};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::Arc,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct GlobalData {
    inner: Arc<Mutex<GlobalDataInner>>,
}

impl Debug for GlobalData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[global data]")
    }
}

impl Default for GlobalData {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(GlobalDataInner {
                connections: HashMap::new(),
                channels: HashMap::new(),
                queues: HashMap::new(),
                default_exchange: HashMap::new(),
            })),
        }
    }
}

impl GlobalData {
    pub fn lock(&self) -> parking_lot::MutexGuard<'_, GlobalDataInner> {
        self.inner.lock()
    }
}

#[derive(Debug)]
pub struct GlobalDataInner {
    pub connections: HashMap<ConnectionId, Connection>,
    pub channels: HashMap<ChannelId, Channel>,
    pub queues: HashMap<QueueName, Queue>,
    /// Todo: This is just for testing and will be removed later!
    pub default_exchange: HashMap<String, Queue>,
}

pub fn random_uuid() -> Uuid {
    Uuid::from_bytes(rand::random())
}
