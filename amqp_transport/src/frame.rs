use crate::error::{ConException, ProtocolError, TransError};
use anyhow::Context;
use tokio::io::AsyncReadExt;

const REQUIRED_FRAME_END: u8 = 0xCE;

mod frame_type {
    pub const METHOD: u8 = 1;
    pub const HEADER: u8 = 2;
    pub const BODY: u8 = 3;
    pub const HEARTBEAT: u8 = 4;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The type of the frame including its parsed metadata.
    kind: FrameType,
    channel: u16,
    /// Includes the whole payload, also including the metadata from each type.
    payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameType {
    /// 1
    Method { class_id: u16, method_id: u16 },
    /// 2
    Header {
        class_id: u16,
        body_size: u64,
        /// Ordered from high to low    
        property_flags: u16,
    },
    /// 3
    Body,
    /// 4
    Heartbeat,
}

pub async fn read_frame<R>(r: &mut R, max_frame_size: usize) -> Result<Frame, TransError>
where
    R: AsyncReadExt + Unpin,
{
    let kind = r.read_u8().await.context("read type")?;
    let channel = r.read_u16().await.context("read channel")?;
    let size = r.read_u32().await.context("read size")?;

    let mut payload = vec![0; size.try_into().unwrap()];
    r.read_exact(&mut payload).await.context("read payload")?;

    let frame_end = r.read_u8().await.context("read frame end")?;

    if frame_end != REQUIRED_FRAME_END {
        return Err(ProtocolError::Fatal.into());
    }

    if payload.len() > max_frame_size {
        return Err(ProtocolError::ConException(ConException::FrameError).into());
    }

    let kind = parse_frame_type(kind, &payload, channel)?;

    Ok(Frame {
        kind,
        channel,
        payload,
    })
}

fn parse_frame_type(kind: u8, payload: &[u8], channel: u16) -> Result<FrameType, TransError> {
    match kind {
        frame_type::METHOD => {
            let class_id = u16::from_be_bytes(payload[0..2].try_into().unwrap());
            let method_id = u16::from_be_bytes(payload[2..4].try_into().unwrap());

            Ok(FrameType::Method {
                class_id,
                method_id,
            })
        }
        frame_type::HEADER => {
            let class_id = u16::from_be_bytes(payload[0..2].try_into().unwrap());
            let weight = u16::from_be_bytes(payload[2..4].try_into().unwrap());
            // weight is unused and must always be 0
            if weight != 0 {
                return Err(ProtocolError::ConException(ConException::FrameError).into());
            }

            let body_size = u64::from_be_bytes(payload[4..12].try_into().unwrap());
            let property_flags = u16::from_be_bytes(payload[12..14].try_into().unwrap());

            Ok(FrameType::Header {
                class_id,
                body_size,
                property_flags,
            })
        }
        frame_type::BODY => Ok(FrameType::Body),
        frame_type::HEARTBEAT => {
            if channel != 0 {
                Err(ProtocolError::ConException(ConException::FrameError).into())
            } else {
                Ok(FrameType::Heartbeat)
            }
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{Frame, FrameType};

    #[tokio::test]
    async fn read_small_body() {
        let mut bytes: &[u8] = &[
            /*type*/ 1,
            /*channel*/ 0,
            0,
            /*size*/ 0,
            0,
            0,
            3,
            /*payload*/ 1,
            2,
            3,
            /*frame-end*/ super::REQUIRED_FRAME_END,
        ];

        let frame = super::read_frame(&mut bytes, 10000).await.unwrap();
        assert_eq!(
            frame,
            Frame {
                kind: FrameType::Method,
                channel: 0,
                size: 3,
                payload: vec![1, 2, 3],
            }
        );
    }
}
