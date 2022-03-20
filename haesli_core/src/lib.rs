#![warn(rust_2018_idioms)]

pub mod connection;
pub mod consumer;
pub mod error;
pub mod exchange;
mod macros;
pub mod message;
pub mod methods;
pub mod queue;

use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::Arc,
};

use connection::{ChannelId, ConnectionId};
use parking_lot::Mutex;
use uuid::Uuid;

use crate::{
    connection::{Channel, Connection},
    exchange::{Exchange, ExchangeName},
    queue::{Queue, QueueName},
};

pub type SingleVec<T> = smallvec::SmallVec<[T; 1]>;

#[derive(Clone)]
// todo: what if this was downstream?
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
                exchanges: exchange::default_exchanges(),
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
    pub exchanges: HashMap<ExchangeName, Exchange>,
    /// Todo: This is just for testing and will be removed later!
    pub default_exchange: HashMap<String, Queue>,
}

pub fn random_uuid() -> Uuid {
    Uuid::from_bytes(rand::random())
}
