use crate::{newtype_id, ChannelHandle};

newtype_id!(
    pub ConsumerId
);

#[derive(Debug)]
pub struct Consumer {
    pub id: ConsumerId,
    pub tag: String,
    pub channel: ChannelHandle,
}
