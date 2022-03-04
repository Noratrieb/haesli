use crate::{methods, methods::Method, newtype_id, GlobalData, Queue};
use bytes::Bytes;
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    net::SocketAddr,
    sync::Arc,
};
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

pub type Connection = Arc<ConnectionInner>;

#[derive(Debug)]
pub struct ConnectionInner {
    pub id: ConnectionId,
    pub peer_addr: SocketAddr,
    pub global_data: GlobalData,
    pub channels: Mutex<HashMap<ChannelNum, Channel>>,
    pub exclusive_queues: Vec<Queue>,
    _events: ConEventSender,
}

#[derive(Debug)]
pub enum QueuedMethod {
    Normal(Method),
    WithContent(Method, ContentHeader, SmallVec<[Bytes; 1]>),
}

pub type ConEventSender = mpsc::Sender<(ChannelNum, QueuedMethod)>;
pub type ConEventReceiver = mpsc::Receiver<(ChannelNum, QueuedMethod)>;

impl ConnectionInner {
    #[must_use]
    pub fn new(
        id: ConnectionId,
        peer_addr: SocketAddr,
        global_data: GlobalData,
        method_queue: ConEventSender,
    ) -> Connection {
        Arc::new(Self {
            id,
            peer_addr,
            global_data,
            channels: Mutex::new(HashMap::new()),
            exclusive_queues: vec![],
            _events: method_queue,
        })
    }

    pub fn close(&self) {
        let mut global_data = self.global_data.lock();
        global_data.connections.remove(&self.id);
    }
}

pub type Channel = Arc<ChannelInner>;

#[derive(Debug)]
pub struct ChannelInner {
    pub id: ChannelId,
    pub num: ChannelNum,
    pub connection: Connection,
    pub global_data: GlobalData,
    method_queue: ConEventSender,
}

impl ChannelInner {
    #[must_use]
    pub fn new(
        id: ChannelId,
        num: ChannelNum,
        connection: Connection,
        global_data: GlobalData,
        method_queue: ConEventSender,
    ) -> Channel {
        Arc::new(Self {
            id,
            num,
            connection,
            global_data,
            method_queue,
        })
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

/// A content frame header.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentHeader {
    pub class_id: u16,
    pub weight: u16,
    pub body_size: u64,
    pub property_fields: methods::Table,
}
