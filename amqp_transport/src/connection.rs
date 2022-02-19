use crate::error::{ConException, ProtocolError, Result};
use crate::frame::{Frame, FrameType};
use crate::{classes, frame, sasl};
use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info};
use uuid::Uuid;

fn ensure_conn(condition: bool) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(ConException::Todo.into_trans())
    }
}

const FRAME_SIZE_MIN_MAX: usize = 4096;
const CHANNEL_MAX: u16 = 0;
const FRAME_SIZE_MAX: u32 = 0;
const HEARTBEAT_DELAY: u16 = 0;

pub struct Connection {
    stream: TcpStream,
    max_frame_size: usize,
    heartbeat_delay: u16,
    channel_max: u16,
    id: Uuid,
}

impl Connection {
    pub fn new(stream: TcpStream, id: Uuid) -> Self {
        Self {
            stream,
            max_frame_size: FRAME_SIZE_MIN_MAX,
            heartbeat_delay: HEARTBEAT_DELAY,
            channel_max: CHANNEL_MAX,
            id,
        }
    }

    pub async fn start_connection_processing(mut self) {
        match self.process_connection().await {
            Ok(()) => {}
            Err(err) => error!(%err, "Error during processing of connection"),
        }
    }

    pub async fn process_connection(&mut self) -> Result<()> {
        self.negotiate_version().await?;
        self.start().await?;
        self.tune().await?;
        self.open().await?;

        info!("Connection is ready for usage!");

        loop {
            let method = self.recv_method().await?;
            debug!(?method, "Received method");
        }
    }

    async fn send_method(&mut self, channel: u16, method: classes::Class) -> Result<()> {
        let mut payload = Vec::with_capacity(64);
        classes::write::write_method(method, &mut payload)?;
        frame::write_frame(
            &Frame {
                kind: FrameType::Method,
                channel,
                payload,
            },
            &mut self.stream,
        )
        .await
    }

    async fn recv_method(&mut self) -> Result<classes::Class> {
        let start_ok_frame = frame::read_frame(&mut self.stream, self.max_frame_size).await?;

        ensure_conn(start_ok_frame.kind == FrameType::Method)?;

        let class = classes::parse_method(&start_ok_frame.payload)?;
        Ok(class)
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
            mechanisms: "PLAIN".into(),
            locales: "en_US".into(),
        });

        debug!(?start_method, "Sending Start method");
        self.send_method(0, start_method).await?;

        let start_ok = self.recv_method().await?;
        debug!(?start_ok, "Received Start-Ok");

        if let classes::Class::Connection(classes::Connection::StartOk {
            mechanism,
            locale,
            response,
            ..
        }) = start_ok
        {
            ensure_conn(mechanism == "PLAIN")?;
            ensure_conn(locale == "en_US")?;
            let plain_user = sasl::parse_sasl_plain_response(&response)?;
            info!(username = %plain_user.authentication_identity, "SASL Authentication successful")
        } else {
            return Err(ConException::Todo.into_trans());
        }

        Ok(())
    }

    async fn tune(&mut self) -> Result<()> {
        let tune_method = classes::Class::Connection(classes::Connection::Tune {
            channel_max: CHANNEL_MAX,
            frame_max: FRAME_SIZE_MAX,
            heartbeat: HEARTBEAT_DELAY,
        });

        debug!("Sending Tune method");
        self.send_method(0, tune_method).await?;

        let tune_ok = self.recv_method().await?;
        debug!(?tune_ok, "Received Tune-Ok method");

        if let classes::Class::Connection(classes::Connection::TuneOk {
            channel_max,
            frame_max,
            heartbeat,
        }) = tune_ok
        {
            self.channel_max = channel_max;
            self.max_frame_size = usize::try_from(frame_max).unwrap();
            self.heartbeat_delay = heartbeat;
        }

        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        let open = self.recv_method().await?;
        debug!(?open, "Received Open method");

        if let classes::Class::Connection(classes::Connection::Open { virtual_host, .. }) = open {
            ensure_conn(virtual_host == "/")?;
        }

        self.send_method(
            0,
            classes::Class::Connection(classes::Connection::OpenOk {
                reserved_1: "".to_string(),
            }),
        )
        .await?;

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
    fn ls(str: &str) -> classes::FieldValue {
        classes::FieldValue::LongString(str.into())
    }

    let host_str = host.ip().to_string();
    HashMap::from([
        ("host".to_string(), ls(&host_str)),
        ("product".to_string(), ls("no name yet")),
        ("version".to_string(), ls("0.1.0")),
        ("platform".to_string(), ls("microsoft linux")),
        ("copyright".to_string(), ls("MIT")),
        ("information".to_string(), ls("hello reader")),
        ("uwu".to_string(), ls("owo")),
    ])
}
