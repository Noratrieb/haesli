#![warn(rust_2018_idioms)]

pub mod error;
pub mod message;
pub mod methods;
pub mod queue;

use crate::queue::Queue;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

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
    pub connections: HashMap<Uuid, ConnectionHandle>,
    pub channels: HashMap<Uuid, ChannelHandle>,
    pub queues: HashMap<Uuid, Queue>,
    /// Todo: This is just for testing and will be removed later!
    pub default_exchange: HashMap<String, Queue>,
}

pub type ConnectionHandle = Handle<Connection>;

#[derive(Debug)]
pub struct Connection {
    pub id: Uuid,
    pub peer_addr: SocketAddr,
    pub global_data: GlobalData,
    pub channels: HashMap<u16, ChannelHandle>,
    pub exclusive_queues: Vec<Queue>,
}

impl Connection {
    pub fn new_handle(
        id: Uuid,
        peer_addr: SocketAddr,
        global_data: GlobalData,
    ) -> ConnectionHandle {
        Arc::new(Mutex::new(Self {
            id,
            peer_addr,
            global_data,
            channels: HashMap::new(),
            exclusive_queues: vec![],
        }))
    }

    pub fn close(&self) {
        let mut global_data = self.global_data.lock();
        global_data.connections.remove(&self.id);
    }
}

pub type ChannelHandle = Handle<Channel>;

#[derive(Debug)]
pub struct Channel {
    pub id: Uuid,
    pub num: u16,
    pub connection: ConnectionHandle,
    pub global_data: GlobalData,
}

impl Channel {
    pub fn new_handle(
        id: Uuid,
        num: u16,
        connection: ConnectionHandle,
        global_data: GlobalData,
    ) -> ChannelHandle {
        Arc::new(Mutex::new(Self {
            id,
            num,
            connection,
            global_data,
        }))
    }

    pub fn close(&self) {
        let mut global_data = self.global_data.lock();
        global_data.channels.remove(&self.id);
    }
}

pub fn gen_uuid() -> Uuid {
    Uuid::from_bytes(rand::random())
}

#[macro_export]
macro_rules! amqp_todo {
    () => {
        return Err(::amqp_core::error::ConException::NotImplemented.into())
    };
}
