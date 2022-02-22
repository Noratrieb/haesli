use crate::error::{ConException, ProtocolError, Result};
use amqp_core::methods;
use anyhow::Context;
use bytes::Bytes;
use std::fmt::{Display, Formatter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

const REQUIRED_FRAME_END: u8 = 0xCE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(u16);

impl ChannelId {
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

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

mod frame_type {
    pub const METHOD: u8 = 1;
    pub const HEADER: u8 = 2;
    pub const BODY: u8 = 3;
    pub const HEARTBEAT: u8 = 8;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The type of the frame including its parsed metadata.
    pub kind: FrameType,
    pub channel: ChannelId,
    /// Includes the whole payload, also including the metadata from each type.
    pub payload: Bytes,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Method = 1,
    Header = 2,
    Body = 3,
    Heartbeat = 8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicProperties {
    content_type: Option<methods::Shortstr>,
    content_encoding: Option<methods::Shortstr>,
    headers: Option<methods::Table>,
    delivery_mode: Option<methods::Octet>,
    priority: Option<methods::Octet>,
    correlation_id: Option<methods::Shortstr>,
    reply_to: Option<methods::Shortstr>,
    expiration: Option<methods::Shortstr>,
    message_id: Option<methods::Shortstr>,
    timestamp: Option<methods::Timestamp>,
    r#type: Option<methods::Shortstr>,
    user_id: Option<methods::Shortstr>,
    app_id: Option<methods::Shortstr>,
    reserved: Option<methods::Shortstr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentHeader {
    pub class_id: u16,
    pub weight: u16,
    pub body_size: u64,
    pub property_fields: BasicProperties,
}

mod content_header_parse {
    use crate::error::TransError;
    use crate::frame::{BasicProperties, ContentHeader};
    use nom::number::complete::{u16, u64};
    use nom::number::Endianness::Big;

    type IResult<'a, T> = nom::IResult<&'a [u8], T, TransError>;

    pub fn basic_properties(_property_flags: u16, _input: &[u8]) -> IResult<'_, BasicProperties> {
        todo!()
    }

    pub fn header(input: &[u8]) -> IResult<'_, Box<ContentHeader>> {
        let (input, class_id) = u16(Big)(input)?;
        let (input, weight) = u16(Big)(input)?;
        let (input, body_size) = u64(Big)(input)?;

        // I do not quite understand this here. Apparently, there can be more than 15 flags?
        // But the Basic class only specifies 15, so idk. Don't care about this for now
        let (input, property_flags) = u16(Big)(input)?;
        let (input, property_fields) = basic_properties(property_flags, input)?;

        Ok((
            input,
            Box::new(ContentHeader {
                class_id,
                weight,
                body_size,
                property_fields,
            }),
        ))
    }
}

impl ContentHeader {
    pub fn parse(input: &[u8]) -> Result<Box<Self>> {
        match content_header_parse::header(input) {
            Ok(([], header)) => Ok(header),
            Ok((_, _)) => {
                Err(
                    ConException::SyntaxError(vec!["could not consume all input".to_string()])
                        .into_trans(),
                )
            }
            Err(nom::Err::Incomplete(_)) => {
                Err(
                    ConException::SyntaxError(vec!["there was not enough data".to_string()])
                        .into_trans(),
                )
            }
            Err(nom::Err::Failure(err) | nom::Err::Error(err)) => Err(err),
        }
    }
}

pub async fn write_frame<W>(frame: &Frame, mut w: W) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    trace!(?frame, "Sending frame");

    w.write_u8(frame.kind as u8).await?;
    w.write_u16(frame.channel.num()).await?;
    w.write_u32(u32::try_from(frame.payload.len()).context("frame size too big")?)
        .await?;
    w.write_all(&frame.payload).await?;
    w.write_u8(REQUIRED_FRAME_END).await?;

    Ok(())
}

pub async fn read_frame<R>(r: &mut R, max_frame_size: usize) -> Result<Frame>
where
    R: AsyncReadExt + Unpin,
{
    let kind = r.read_u8().await.context("read type")?;
    let channel = r.read_u16().await.context("read channel")?;
    let channel = ChannelId(channel);
    let size = r.read_u32().await.context("read size")?;

    let mut payload = vec![0; size.try_into().unwrap()];
    r.read_exact(&mut payload).await.context("read payload")?;

    let frame_end = r.read_u8().await.context("read frame end")?;

    if frame_end != REQUIRED_FRAME_END {
        return Err(ProtocolError::Fatal.into());
    }

    if max_frame_size != 0 && payload.len() > max_frame_size {
        return Err(ConException::FrameError.into_trans());
    }

    let kind = parse_frame_type(kind, channel)?;

    let frame = Frame {
        kind,
        channel,
        payload: payload.into(),
    };

    trace!(?frame, "Received frame");

    Ok(frame)
}

fn parse_frame_type(kind: u8, channel: ChannelId) -> Result<FrameType> {
    match kind {
        frame_type::METHOD => Ok(FrameType::Method),
        frame_type::HEADER => Ok(FrameType::Header),
        frame_type::BODY => Ok(FrameType::Body),
        frame_type::HEARTBEAT => {
            if channel.is_zero() {
                Ok(FrameType::Heartbeat)
            } else {
                Err(ProtocolError::ConException(ConException::FrameError).into())
            }
        }
        _ => Err(ConException::FrameError.into_trans()),
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{ChannelId, Frame, FrameType};
    use bytes::Bytes;

    #[tokio::test]
    async fn read_small_body() {
        let mut bytes: &[u8] = &[
            /*type*/
            1,
            /*channel*/
            0,
            0,
            /*size*/
            0,
            0,
            0,
            3,
            /*payload*/
            1,
            2,
            3,
            /*frame-end*/
            super::REQUIRED_FRAME_END,
        ];

        let frame = super::read_frame(&mut bytes, 10000).await.unwrap();
        assert_eq!(
            frame,
            Frame {
                kind: FrameType::Method,
                channel: ChannelId(0),
                payload: Bytes::from_static(&[1, 2, 3]),
            }
        );
    }
}
