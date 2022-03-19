use crate::{newtype_id, Channel, Queue};

newtype_id!(
    pub ConsumerId
);

#[derive(Debug, Clone)]
pub struct Consumer {
    pub id: ConsumerId,
    pub tag: String,
    pub channel: Channel,
    pub queue: Queue,
}
