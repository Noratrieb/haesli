#![allow(dead_code)]

use crate::methods;
use bytes::Bytes;
use smallvec::SmallVec;
use std::sync::Arc;
use uuid::Uuid;

pub type Message = Arc<RawMessage>;

pub struct RawMessage {
    id: Uuid,
    properties: methods::Table,
    routing: RoutingInformation,
    content: SmallVec<[Bytes; 1]>,
}

pub struct RoutingInformation {
    pub exchange: String,
    pub routing_key: String,
    pub mandatory: bool,
    pub immediate: bool,
}
