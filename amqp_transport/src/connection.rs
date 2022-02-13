use crate::error::{ProtocolError, Result};
use crate::frame::{Frame, FrameType};
use crate::{classes, frame};
use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, warn};

const MIN_MAX_FRAME_SIZE: usize = 4096;

pub struct Connection {
    stream: TcpStream,
    max_frame_size: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            max_frame_size: MIN_MAX_FRAME_SIZE,
        }
    }

    pub async fn open_connection(mut self) {
        match self.run().await {
            Ok(()) => {}
            Err(err) => error!(%err, "Error during processing of connection"),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.negotiate_version().await?;
        self.start().await?;

        loop {
            let frame = frame::read_frame(&mut self.stream, self.max_frame_size).await?;
            debug!(?frame, "received frame");
            if frame.kind == FrameType::Method {
                let class = super::classes::parse_method(&frame.payload)?;
                debug!(?class, "was method frame");
            }
        }
    }

    async fn start(&mut self) -> Result<()> {
        let start_method = classes::Class::Connection(classes::Connection::Start {
            version_major: 0,
            version_minor: 9,
            server_properties: Default::default(),
            mechanisms: vec![],
            locales: vec![],
        });

        let fut = classes::write::write_method(start_method, &mut self.stream);
        warn!(size = %std::mem::size_of_val(&fut), "that future is big");
        // todo fix out_buffer buffer things :spiral_eyes:
        // maybe have a `size` method on `Class` and use `AsyncWrite`? oh god no that's horrible
        // frame::write_frame(&mut self.stream, Frame {})?;

        Ok(())
    }

    async fn negotiate_version(&mut self) -> Result<()> {
        const HEADER_SIZE: usize = 8;
        const SUPPORTED_PROTOCOL_VERSION: &[u8] = &[0, 9, 1];
        const OWN_PROTOCOL_HEADER: &[u8] = b"AMQP\0\0\x09\x01";

        debug!("Negotiating version");

        let mut read_header_buf = [0; HEADER_SIZE];

        self.stream
            .read_exact(&mut read_header_buf)
            .await
            .context("read protocol header")?;

        debug!(received_header = ?read_header_buf,"Received protocol header");

        let version = &read_header_buf[5..8];

        self.stream
            .write_all(OWN_PROTOCOL_HEADER)
            .await
            .context("write protocol header")?;

        if &read_header_buf[0..5] == b"AMQP\0" && version == SUPPORTED_PROTOCOL_VERSION {
            debug!(?version, "Version negotiation successful");
            Ok(())
        } else {
            debug!(?version, expected_version = ?SUPPORTED_PROTOCOL_VERSION, "Version negotiation failed, unsupported version");
            Err(ProtocolError::OtherCloseConnection.into())
        }
    }
}
