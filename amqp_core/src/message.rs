use crate::connection::ContentHeader;
use crate::newtype_id;
use bytes::Bytes;
use smallvec::SmallVec;
use std::sync::Arc;

pub type Message = Arc<RawMessage>;

newtype_id!(pub MessageId);

#[derive(Debug)]
pub struct RawMessage {
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
