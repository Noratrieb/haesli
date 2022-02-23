#![allow(dead_code)]

use crate::methods;
use bytes::Bytes;
use smallvec::SmallVec;
use std::sync::Arc;
use uuid::Uuid;

pub type Message = Arc<RawMessage>;

#[derive(Debug)]
pub struct RawMessage {
    pub id: Uuid,
    pub properties: methods::Table,
    pub routing: RoutingInformation,
    pub content: SmallVec<[Bytes; 1]>,
}

#[derive(Debug)]
pub struct RoutingInformation {
    pub exchange: String,
    pub routing_key: String,
    pub mandatory: bool,
    pub immediate: bool,
}
