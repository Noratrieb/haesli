use crate::methods::Method;
use crate::{methods, newtype_id, GlobalData, Handle, Queue};
use bytes::Bytes;
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

newtype_id!(pub ConnectionId);
newtype_id!(pub ChannelId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelNum(u16);

impl ChannelNum {
    #[must_use]
    pub fn new(num: u16) -> Self {
        Self(num)
    }

    #[must_use]
    pub fn num(self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn zero() -> Self {
        Self(0)
    }
}

impl Display for ChannelNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub type ConnectionHandle = Handle<Connection>;

#[derive(Debug)]
pub struct Connection {
    pub id: ConnectionId,
    pub peer_addr: SocketAddr,
    pub global_data: GlobalData,
    pub channels: HashMap<ChannelNum, ChannelHandle>,
    pub exclusive_queues: Vec<Queue>,
    _method_queue: MethodSender,
}

#[derive(Debug)]
pub enum QueuedMethod {
    Normal(Method),
    WithContent(Method, ContentHeader, SmallVec<[Bytes; 1]>),
}

pub type MethodSender = mpsc::Sender<(ChannelNum, QueuedMethod)>;
pub type MethodReceiver = mpsc::Receiver<(ChannelNum, QueuedMethod)>;

impl Connection {
    #[must_use]
    pub fn new_handle(
        id: ConnectionId,
        peer_addr: SocketAddr,
        global_data: GlobalData,
        method_queue: MethodSender,
    ) -> ConnectionHandle {
        Arc::new(Mutex::new(Self {
            id,
            peer_addr,
            global_data,
            channels: HashMap::new(),
            exclusive_queues: vec![],
            _method_queue: method_queue,
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
    pub id: ChannelId,
    pub num: ChannelNum,
    pub connection: ConnectionHandle,
    pub global_data: GlobalData,
    method_queue: MethodSender,
}

impl Channel {
    #[must_use]
    pub fn new_handle(
        id: ChannelId,
        num: ChannelNum,
        connection: ConnectionHandle,
        global_data: GlobalData,
        method_queue: MethodSender,
    ) -> ChannelHandle {
        Arc::new(Mutex::new(Self {
            id,
            num,
            connection,
            global_data,
            method_queue,
        }))
    }

    pub fn close(&self) {
        let mut global_data = self.global_data.lock();
        global_data.channels.remove(&self.id);
    }

    pub fn queue_method(&self, method: QueuedMethod) {
        // todo: this is a horrible hack around the lock chaos
        self.method_queue
            .try_send((self.num, method))
            .expect("could not send method to channel, RIP");
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentHeader {
    pub class_id: u16,
    pub weight: u16,
    pub body_size: u64,
    pub property_fields: methods::Table,
}
