use crate::{newtype_id, GlobalData, Handle, Queue};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;

newtype_id!(pub ConnectionId);
newtype_id!(pub ChannelId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelNum(u16);

impl ChannelNum {
    pub fn new(num: u16) -> Self {
        Self(num)
    }

    pub fn num(self) -> u16 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

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
    pub channels: HashMap<u16, ChannelHandle>,
    pub exclusive_queues: Vec<Queue>,
}

impl Connection {
    pub fn new_handle(
        id: ConnectionId,
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
    pub id: ChannelId,
    pub num: u16,
    pub connection: ConnectionHandle,
    pub global_data: GlobalData,
}

impl Channel {
    pub fn new_handle(
        id: ChannelId,
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
