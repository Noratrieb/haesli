use std::sync::Arc;

use bytes::Bytes;
use smallvec::SmallVec;

use crate::{connection::ContentHeader, newtype_id};

pub type Message = Arc<MessageInner>;

newtype_id!(pub MessageId);

#[derive(Debug)]
pub struct MessageInner {
    pub id: MessageId,
    pub header: ContentHeader,
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
