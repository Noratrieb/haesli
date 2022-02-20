use crate::error::{ConException, ProtocolError, Result};
use crate::frame::{Frame, FrameType};
use crate::{frame, methods, sasl};
use amqp_core::methods::{FieldValue, Method, Table};
use amqp_core::GlobalData;
use anyhow::Context;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;
use tracing::{debug, error, info, warn};
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

#[allow(dead_code)]
pub struct Channel {
    num: u16,
    channel_handle: amqp_core::ChannelHandle,
}

pub struct Connection {
    id: Uuid,
    stream: TcpStream,
    max_frame_size: usize,
    heartbeat_delay: u16,
    channel_max: u16,
    next_timeout: Pin<Box<time::Sleep>>,
    channels: HashMap<u16, Channel>,
    connection_handle: amqp_core::ConnectionHandle,
    global_data: GlobalData,
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

impl Connection {
    pub fn new(
        id: Uuid,
        stream: TcpStream,
        connection_handle: amqp_core::ConnectionHandle,
        global_data: GlobalData,
    ) -> Self {
        Self {
            id,
            stream,
            max_frame_size: FRAME_SIZE_MIN_MAX,
            heartbeat_delay: HEARTBEAT_DELAY,
            channel_max: CHANNEL_MAX,
            next_timeout: Box::pin(time::sleep(DEFAULT_TIMEOUT)),
            connection_handle,
            channels: HashMap::new(),
            global_data,
        }
    }

    pub async fn start_connection_processing(mut self) {
        match self.process_connection().await {
            Ok(()) => {}
            Err(err) => error!(%err, "Error during processing of connection"),
        }

        let connection_handle = self.connection_handle.lock();
        connection_handle.close();
    }

    pub async fn process_connection(&mut self) -> Result<()> {
        self.negotiate_version().await?;
        self.start().await?;
        self.tune().await?;
        self.open().await?;

        info!("Connection is ready for usage!");

        self.main_loop().await
    }

    async fn send_method(&mut self, channel: u16, method: Method) -> Result<()> {
        let mut payload = Vec::with_capacity(64);
        methods::write::write_method(method, &mut payload)?;
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

    async fn recv_method(&mut self) -> Result<Method> {
        let start_ok_frame = frame::read_frame(&mut self.stream, self.max_frame_size).await?;

        ensure_conn(start_ok_frame.kind == FrameType::Method)?;

        let method = methods::parse_method(&start_ok_frame.payload)?;
        Ok(method)
    }

    async fn start(&mut self) -> Result<()> {
        let start_method = Method::ConnectionStart {
            version_major: 0,
            version_minor: 9,
            server_properties: server_properties(
                self.stream
                    .local_addr()
                    .context("failed to get local_addr")?,
            ),
            mechanisms: "PLAIN".into(),
            locales: "en_US".into(),
        };

        debug!(?start_method, "Sending Start method");
        self.send_method(0, start_method).await?;

        let start_ok = self.recv_method().await?;
        debug!(?start_ok, "Received Start-Ok");

        if let Method::ConnectionStartOk {
            mechanism,
            locale,
            response,
            ..
        } = start_ok
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
        let tune_method = Method::ConnectionTune {
            channel_max: CHANNEL_MAX,
            frame_max: FRAME_SIZE_MAX,
            heartbeat: HEARTBEAT_DELAY,
        };

        debug!("Sending Tune method");
        self.send_method(0, tune_method).await?;

        let tune_ok = self.recv_method().await?;
        debug!(?tune_ok, "Received Tune-Ok method");

        if let Method::ConnectionTuneOk {
            channel_max,
            frame_max,
            heartbeat,
        } = tune_ok
        {
            self.channel_max = channel_max;
            self.max_frame_size = usize::try_from(frame_max).unwrap();
            self.heartbeat_delay = heartbeat;
            self.reset_timeout();
        }

        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        let open = self.recv_method().await?;
        debug!(?open, "Received Open method");

        if let Method::ConnectionOpen { virtual_host, .. } = open {
            ensure_conn(virtual_host == "/")?;
        }

        self.send_method(
            0,
            Method::ConnectionOpenOk {
                reserved_1: "".to_string(),
            },
        )
        .await?;

        Ok(())
    }

    async fn main_loop(&mut self) -> Result<()> {
        loop {
            debug!("Waiting for next frame");
            let frame = frame::read_frame(&mut self.stream, self.max_frame_size).await?;
            debug!(?frame);
            self.reset_timeout();

            match frame.kind {
                FrameType::Method => self.dispatch_method(frame).await?,
                FrameType::Heartbeat => {}
                _ => warn!(frame_type = ?frame.kind, "TODO"),
            }
        }
    }

    async fn dispatch_method(&mut self, frame: Frame) -> Result<()> {
        let method = methods::parse_method(&frame.payload)?;
        debug!(?method, "Received method");

        match method {
            Method::ConnectionClose { .. } => {
                // todo: handle closing
            }
            Method::ChannelOpen { .. } => self.channel_open(frame.channel).await?,
            _ => {
                let channel_handle = self
                    .channels
                    .get(&frame.channel)
                    .ok_or_else(|| ConException::Todo.into_trans())?
                    .channel_handle
                    .clone();

                tokio::spawn(amqp_messaging::methods::handle_method(
                    channel_handle,
                    method,
                ));
                // we don't handle this here, forward it to *somewhere*
            }
        }

        Ok(())
    }

    async fn channel_open(&mut self, num: u16) -> Result<()> {
        let id = Uuid::from_bytes(rand::random());
        let channel_handle = amqp_core::Channel::new_handle(
            id,
            num,
            self.connection_handle.clone(),
            self.global_data.clone(),
        );

        let channel = Channel {
            num,
            channel_handle: channel_handle.clone(),
        };

        let prev = self.channels.insert(num, channel);
        if let Some(prev) = prev {
            self.channels.insert(num, prev); // restore previous state
            return Err(ConException::ChannelError.into_trans());
        }

        {
            let mut global_data = self.global_data.lock();
            global_data.channels.insert(id, channel_handle.clone());
            global_data
                .connections
                .get_mut(&self.id)
                .unwrap()
                .lock()
                .channels
                .insert(num, channel_handle);
        }

        info!(%num, "Opened new channel");

        self.send_method(
            num,
            Method::ChannelOpenOk {
                reserved_1: Vec::new(),
            },
        )
        .await?;

        Ok(())
    }

    fn reset_timeout(&mut self) {
        if self.heartbeat_delay != 0 {
            let next = Duration::from_secs(u64::from(self.heartbeat_delay));
            self.next_timeout = Box::pin(time::sleep(next));
        }
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
            Err(ProtocolError::CloseNow.into())
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.connection_handle.lock().close();
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        self.channel_handle.lock().close();
    }
}

fn server_properties(host: SocketAddr) -> Table {
    fn ls(str: &str) -> FieldValue {
        FieldValue::LongString(str.into())
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
