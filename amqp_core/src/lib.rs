#![warn(rust_2018_idioms)]

pub mod connection;
pub mod error;
mod macros;
pub mod message;
pub mod methods;
pub mod queue;

use crate::connection::{ChannelHandle, ConnectionHandle};
use crate::queue::{Queue, QueueName};
use connection::{ChannelId, ConnectionId};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

type Handle<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone)]
pub struct GlobalData {
    inner: Arc<Mutex<GlobalDataInner>>,
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
    pub connections: HashMap<ConnectionId, ConnectionHandle>,
    pub channels: HashMap<ChannelId, ChannelHandle>,
    pub queues: HashMap<QueueName, Queue>,
    /// Todo: This is just for testing and will be removed later!
    pub default_exchange: HashMap<String, Queue>,
}
