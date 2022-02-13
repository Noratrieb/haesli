use crate::classes::FieldValue;
use crate::error::{ConException, ProtocolError, Result};
use crate::frame::{Frame, FrameType};
use crate::{classes, frame};
use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error};

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
            server_properties: server_properties(
                self.stream
                    .local_addr()
                    .context("failed to get local_addr")?,
            ),
            mechanisms: "PLAIN".to_string().into(),
            locales: "en_US".to_string().into(),
        });

        debug!(?start_method, "Sending start method");

        let mut payload = Vec::with_capacity(64);
        classes::write::write_method(start_method, &mut payload)?;
        frame::write_frame(
            &mut self.stream,
            &Frame {
                kind: FrameType::Method,
                channel: 0,
                payload,
            },
        )
        .await?;

        let start_ok_frame = frame::read_frame(&mut self.stream, self.max_frame_size).await?;
        debug!(?start_ok_frame, "Received Start-Ok frame");

        if start_ok_frame.kind != FrameType::Method {
            return Err(ProtocolError::ConException(ConException::Todo).into());
        }

        let class = classes::parse_method(&start_ok_frame.payload)?;

        debug!(?class, "extracted method");

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

fn server_properties(host: SocketAddr) -> classes::Table {
    fn ss(str: &str) -> FieldValue {
        FieldValue::ShortString(str.to_string())
    }

    let host_str = host.ip().to_string();
    let _host_value = if host_str.len() < 256 {
        FieldValue::ShortString(host_str)
    } else {
        FieldValue::LongString(host_str.into())
    };

    // todo: fix

    //HashMap::from([
    //    ("host".to_string(), host_value),
    //    ("product".to_string(), ss("no name yet")),
    //    ("version".to_string(), ss("0.1.0")),
    //    ("platform".to_string(), ss("microsoft linux")),
    //    ("copyright".to_string(), ss("MIT")),
    //    ("information".to_string(), ss("hello reader")),
    //    ("uwu".to_string(), ss("owo")),
    //])
    HashMap::new()
}
