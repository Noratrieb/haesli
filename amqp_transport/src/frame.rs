use crate::error::{ConException, ProtocolError, Result};
use amqp_core::connection::{ChannelNum, ContentHeader};
use anyhow::Context;
use bytes::Bytes;
use std::{
    fmt::{Debug, Formatter},
    num::NonZeroUsize,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

const REQUIRED_FRAME_END: u8 = 0xCE;

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
    pub channel: ChannelNum,
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

mod content_header_parse {
    use crate::{
        error::TransError,
        methods::parse_helper::{octet, shortstr, table, timestamp},
    };
    use amqp_core::{
        connection::ContentHeader,
        methods,
        methods::FieldValue::{FieldTable, ShortShortUInt, ShortString, Timestamp},
    };
    use nom::number::{
        complete::{u16, u64},
        Endianness::Big,
    };

    type IResult<'a, T> = nom::IResult<&'a [u8], T, TransError>;

    pub fn basic_properties(flags: u16, input: &[u8]) -> IResult<'_, methods::Table> {
        macro_rules! parse_property {
            (if $flags:ident >> $n:literal, $parser:ident($input:ident)?, $map:ident.insert($name:expr, $ctor:path)) => {
                if (($flags >> $n) & 1) == 1 {
                    let (input, value) = $parser($input)?;
                    $map.insert(String::from($name), $ctor(value));
                    input
                } else {
                    $input
                }
            };
        }

        let mut map = methods::Table::new();

        let input = parse_property!(if flags >> 15, shortstr(input)?, map.insert("content-type", ShortString));
        let input = parse_property!(if flags >> 14, shortstr(input)?, map.insert("content-encoding", ShortString));
        let input =
            parse_property!(if flags >> 13, table(input)?, map.insert("headers", FieldTable));
        let input = parse_property!(if flags >> 12, octet(input)?, map.insert("delivery-mode", ShortShortUInt));
        let input =
            parse_property!(if flags >> 11, octet(input)?, map.insert("priority", ShortShortUInt));
        let input = parse_property!(if flags >> 10, shortstr(input)?, map.insert("correlation-id", ShortString));
        let input =
            parse_property!(if flags >> 9, shortstr(input)?, map.insert("reply-to", ShortString));
        let input =
            parse_property!(if flags >> 8, shortstr(input)?, map.insert("expiration", ShortString));
        let input =
            parse_property!(if flags >> 7, shortstr(input)?, map.insert("message-id", ShortString));
        let input =
            parse_property!(if flags >> 6, timestamp(input)?, map.insert("timestamp", Timestamp));
        let input =
            parse_property!(if flags >> 5, shortstr(input)?, map.insert("type", ShortString));
        let input =
            parse_property!(if flags >> 4, shortstr(input)?, map.insert("user-id", ShortString));
        let input =
            parse_property!(if flags >> 3, shortstr(input)?, map.insert("app-id", ShortString));
        let input =
            parse_property!(if flags >> 2, shortstr(input)?, map.insert("reserved", ShortString));

        Ok((input, map))
    }

    pub fn header(input: &[u8]) -> IResult<'_, ContentHeader> {
        let (input, class_id) = u16(Big)(input)?;
        let (input, weight) = u16(Big)(input)?;
        let (input, body_size) = u64(Big)(input)?;

        // I do not quite understand this here. Apparently, there can be more than 15 flags?
        // But the Basic class only specifies 15, so idk. Don't care about this for now
        // Todo: But probably later.
        let (input, property_flags) = u16(Big)(input)?;
        let (input, property_fields) = basic_properties(property_flags, input)?;

        Ok((
            input,
            ContentHeader {
                class_id,
                weight,
                body_size,
                property_fields,
            },
        ))
    }
}

pub fn parse_content_header(input: &[u8]) -> Result<ContentHeader> {
    match content_header_parse::header(input) {
        Ok(([], header)) => Ok(header),
        Ok((_, _)) => {
            Err(ConException::SyntaxError(vec!["could not consume all input".to_owned()]).into())
        }
        Err(nom::Err::Incomplete(_)) => {
            Err(ConException::SyntaxError(vec!["there was not enough data".to_owned()]).into())
        }
        Err(nom::Err::Failure(err) | nom::Err::Error(err)) => Err(err),
    }
}

mod content_header_write {
    use crate::{
        error::Result,
        methods::write_helper::{longlong, octet, short, shortstr, table, timestamp},
    };
    use amqp_core::{
        connection::ContentHeader,
        methods::{
            FieldValue::{FieldTable, ShortShortUInt, ShortString, Timestamp},
            Table,
        },
    };
    use std::io::Write;

    pub fn write_content_header<W: Write>(buf: &mut W, header: &ContentHeader) -> Result<()> {
        short(&header.class_id, buf)?;
        short(&header.weight, buf)?;
        longlong(&header.body_size, buf)?;

        write_content_header_props(buf, &header.property_fields)
    }

    pub fn write_content_header_props<W: Write>(writer: &mut W, header: &Table) -> Result<()> {
        let mut flags = 0_u16;
        // todo: don't allocate for no reason here
        let mut temp_buf = Vec::new();
        let buf = &mut temp_buf;

        buf.extend_from_slice(&flags.to_be_bytes()); // placeholder

        if let Some(ShortString(value)) = header.get("content-type") {
            flags |= 1 << 15;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("content-encoding") {
            flags |= 1 << 14;
            shortstr(value, buf)?;
        }
        if let Some(FieldTable(value)) = header.get("headers") {
            flags |= 1 << 13;
            table(value, buf)?;
        }
        if let Some(ShortShortUInt(value)) = header.get("delivery-mode") {
            flags |= 1 << 12;
            octet(value, buf)?;
        }
        if let Some(ShortShortUInt(value)) = header.get("priority") {
            flags |= 1 << 11;
            octet(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("correlation-id") {
            flags |= 1 << 10;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("reply-to") {
            flags |= 1 << 9;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("expiration") {
            flags |= 1 << 8;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("message-id") {
            flags |= 1 << 7;
            shortstr(value, buf)?;
        }
        if let Some(Timestamp(value)) = header.get("timestamp") {
            flags |= 1 << 6;
            timestamp(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("type") {
            flags |= 1 << 5;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("user-id") {
            flags |= 1 << 4;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("app-id") {
            flags |= 1 << 3;
            shortstr(value, buf)?;
        }
        if let Some(ShortString(value)) = header.get("reserved") {
            flags |= 1 << 2;
            shortstr(value, buf)?;
        }

        let [a, b] = flags.to_be_bytes();
        buf[0] = a;
        buf[1] = b;

        writer.write_all(&temp_buf)?;

        Ok(())
    }
}

pub fn write_content_header(buf: &mut Vec<u8>, content_header: &ContentHeader) -> Result<()> {
    content_header_write::write_content_header(buf, content_header)
}

#[derive(Clone, Copy)]
pub struct MaxFrameSize(Option<NonZeroUsize>);

impl MaxFrameSize {
    pub const fn new(size: usize) -> Self {
        Self(NonZeroUsize::new(size))
    }

    pub fn as_usize(&self) -> usize {
        self.0.map(NonZeroUsize::get).unwrap_or(usize::MAX)
    }
}

impl Debug for MaxFrameSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub async fn write_frame<W>(
    mut w: W,
    kind: FrameType,
    channel: ChannelNum,
    payload: &[u8],
) -> Result<()>
where
    W: AsyncWriteExt + Unpin + Send,
{
    trace!(?kind, ?channel, ?payload, "Sending frame");

    w.write_u8(kind as u8).await?;
    w.write_u16(channel.num()).await?;
    w.write_u32(u32::try_from(payload.len()).context("frame size too big")?)
        .await?;
    w.write_all(payload).await?;
    w.write_u8(REQUIRED_FRAME_END).await?;

    Ok(())
}

pub async fn read_frame<R>(r: &mut R, max_frame_size: MaxFrameSize) -> Result<Frame>
where
    R: AsyncReadExt + Unpin + Send,
{
    let kind = r.read_u8().await.context("read type")?;
    let channel = r.read_u16().await.context("read channel")?;
    let channel = ChannelNum::new(channel);
    let size = r.read_u32().await.context("read size")?;

    let mut payload = vec![0; size.try_into().unwrap()];
    r.read_exact(&mut payload).await.context("read payload")?;

    let frame_end = r.read_u8().await.context("read frame end")?;

    if frame_end != REQUIRED_FRAME_END {
        return Err(ProtocolError::Fatal.into());
    }

    if payload.len() > max_frame_size.as_usize() {
        return Err(ConException::FrameError.into());
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

fn parse_frame_type(kind: u8, channel: ChannelNum) -> Result<FrameType> {
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
        _ => Err(ConException::FrameError.into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{ChannelNum, Frame, FrameType, MaxFrameSize};
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

        let frame = super::read_frame(&mut bytes, MaxFrameSize::new(10000))
            .await
            .unwrap();
        assert_eq!(
            frame,
            Frame {
                kind: FrameType::Method,
                channel: ChannelNum::new(0),
                payload: Bytes::from_static(&[1, 2, 3]),
            }
        );
    }
}
