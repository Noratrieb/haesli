use crate::error::{ConError, ConException, ProtocolError};
use anyhow::Context;
use tokio::io::AsyncReadExt;

const REQUIRED_FRAME_END: u8 = 0xCE;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Method = 1,
    Header = 2,
    Body = 3,
    Heartbeat = 4,
}

impl TryFrom<u8> for FrameType {
    type Error = ConError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Method,
            2 => Self::Header,
            3 => Self::Body,
            4 => Self::Heartbeat,
            _ => return Err(ProtocolError::Fatal.into()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    r#type: FrameType,
    channel: u16,
    size: u32,
    payload: Vec<u8>,
}

pub async fn read_frame<R>(r: &mut R, max_frame_size: usize) -> Result<Frame, ConError>
where
    R: AsyncReadExt + Unpin,
{
    let r#type = r.read_u8().await.context("read type")?;
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

    Ok(Frame {
        r#type: r#type.try_into()?,
        channel,
        size,
        payload,
    })
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
                r#type: FrameType::Method,
                channel: 0,
                size: 3,
                payload: vec![1, 2, 3],
            }
        );
    }
}
