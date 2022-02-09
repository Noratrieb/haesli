use anyhow::Result;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    r#type: u8,
    channel: u16,
    size: u32,
    payload: Vec<u8>,
    frame_end: u8,
}

pub async fn read_frame<R>(r: &mut R) -> Result<Frame>
where
    R: AsyncReadExt + Unpin,
{
    let r#type = r.read_u8().await?;
    let channel = r.read_u16().await?;
    let size = r.read_u32().await?;

    let mut payload = vec![0; size.try_into().unwrap()];
    r.read_exact(&mut payload).await?;

    let frame_end = r.read_u8().await?;

    Ok(Frame {
        r#type,
        channel,
        size,
        payload,
        frame_end,
    })
}

#[cfg(test)]
mod tests {
    use crate::frame::Frame;

    #[tokio::test]
    async fn read_small_body() {
        let mut bytes: &[u8] = &[
            /*type*/ 1, /*channel*/ 0, 0, /*size*/ 0, 0, 0, 3, /*payload*/ 1,
            2, 3, /*frame-end*/ 0,
        ];

        let frame = super::read_frame(&mut bytes).await.unwrap();
        assert_eq!(
            frame,
            Frame {
                r#type: 1,
                channel: 0,
                size: 3,
                payload: vec![1, 2, 3],
                frame_end: 0
            }
        );
    }
}
