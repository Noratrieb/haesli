use crate::{
    error::{ConException, ProtocolError, Result, TransError},
    frame,
    frame::{parse_content_header, Frame, FrameType},
    methods, sasl,
};
use amqp_core::{
    amqp_todo,
    connection::{
        Channel, ChannelInner, ChannelNum, ConEventReceiver, ConEventSender, Connection,
        ConnectionEvent, ConnectionId, ContentHeader,
    },
    message::{MessageId, RawMessage, RoutingInformation},
    methods::{
        BasicPublish, ChannelClose, ChannelCloseOk, ChannelOpenOk, ConnectionClose,
        ConnectionCloseOk, ConnectionOpen, ConnectionOpenOk, ConnectionStart, ConnectionStartOk,
        ConnectionTune, ConnectionTuneOk, FieldValue, Method, ReplyCode, ReplyText, Table,
    },
    GlobalData,
};
use anyhow::{anyhow, Context};
use bytes::Bytes;
use smallvec::SmallVec;
use std::{
    cmp::Ordering, collections::HashMap, net::SocketAddr, pin::Pin, sync::Arc, time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select, time,
};
use tracing::{debug, error, info, trace, warn};

fn ensure_conn(condition: bool) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(ConException::Todo.into())
    }
}

const FRAME_SIZE_MIN_MAX: usize = 4096;
const CHANNEL_MAX: u16 = 0;
const FRAME_SIZE_MAX: u32 = 0;
const HEARTBEAT_DELAY: u16 = 0;

const BASIC_CLASS_ID: u16 = 60;

pub struct TransportChannel {
    /// A handle to the global channel representation. Used to remove the channel when it's dropped
    global_chan: Channel,
    /// The current status of the channel, whether it has sent a method that expects a body
    status: ChannelStatus,
}

pub struct TransportConnection {
    id: ConnectionId,
    stream: TcpStream,
    max_frame_size: usize,
    heartbeat_delay: u16,
    channel_max: u16,
    /// When the next heartbeat expires
    next_timeout: Pin<Box<time::Sleep>>,
    channels: HashMap<ChannelNum, TransportChannel>,
    global_con: Connection,
    global_data: GlobalData,
    /// Only here to forward to other futures so they can send events
    event_sender: ConEventSender,
    /// To receive events from other futures
    event_receiver: ConEventReceiver,
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

enum ChannelStatus {
    Default,
    NeedHeader(u16, Box<Method>),
    NeedsBody(Box<Method>, ContentHeader, SmallVec<[Bytes; 1]>),
}

impl ChannelStatus {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Default)
    }
}

impl TransportConnection {
    pub fn new(
        id: ConnectionId,
        stream: TcpStream,
        global_con: Connection,
        global_data: GlobalData,
        method_queue_send: ConEventSender,
        method_queue_recv: ConEventReceiver,
    ) -> Self {
        Self {
            id,
            stream,
            max_frame_size: FRAME_SIZE_MIN_MAX,
            heartbeat_delay: HEARTBEAT_DELAY,
            channel_max: CHANNEL_MAX,
            next_timeout: Box::pin(time::sleep(DEFAULT_TIMEOUT)),
            global_con,
            channels: HashMap::with_capacity(4),
            global_data,
            event_sender: method_queue_send,
            event_receiver: method_queue_recv,
        }
    }

    pub async fn start_connection_processing(mut self) {
        let process_result = self.process_connection().await;

        match process_result {
            Ok(()) => {}
            Err(TransError::Protocol(ProtocolError::GracefullyClosed)) => {
                /* do nothing, remove below */
            }
            Err(TransError::Protocol(ProtocolError::ConException(ex))) => {
                warn!(%ex, "Connection exception occurred. This indicates a faulty client.");
                let close_result = self.close(ex.reply_code(), ex.reply_text()).await;

                match close_result {
                    Ok(()) => {}
                    Err(err) => {
                        error!(%ex, %err, "Failed to close connection after ConnectionException");
                    }
                }
            }
            Err(err) => error!(%err, "Error during processing of connection"),
        }

        // global connection is closed on drop
    }

    pub async fn process_connection(&mut self) -> Result<()> {
        self.negotiate_version().await?;
        self.start().await?;
        self.tune().await?;
        self.open().await?;

        info!("Connection is ready for usage!");

        self.main_loop().await
    }

    async fn send_method_content(
        &mut self,
        channel: ChannelNum,
        method: &Method,
        header: ContentHeader,
        _body: SmallVec<[Bytes; 1]>,
    ) -> Result<()> {
        self.send_method(channel, method).await?;

        let mut header_buf = Vec::new();
        frame::write_content_header(&mut header_buf, header)?;
        frame::write_frame(
            &Frame {
                kind: FrameType::Method,
                channel,
                payload: header_buf.into(),
            },
            &mut self.stream,
        )
        .await?;

        amqp_todo!()
    }

    async fn send_method(&mut self, channel: ChannelNum, method: &Method) -> Result<()> {
        trace!(%channel, ?method, "Sending method");

        let mut payload = Vec::with_capacity(64);
        methods::write::write_method(method, &mut payload)?;
        frame::write_frame(
            &Frame {
                kind: FrameType::Method,
                channel,
                payload: payload.into(),
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
        let start_method = Method::ConnectionStart(ConnectionStart {
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
        self.send_method(ChannelNum::zero(), &start_method).await?;

        let start_ok = self.recv_method().await?;
        debug!(?start_ok, "Received Start-Ok");

        if let Method::ConnectionStartOk(ConnectionStartOk {
            mechanism,
            locale,
            response,
            ..
        }) = start_ok
        {
            ensure_conn(mechanism == "PLAIN")?;
            ensure_conn(locale == "en_US")?;
            let plain_user = sasl::parse_sasl_plain_response(&response)?;
            info!(username = %plain_user.authentication_identity, "SASL Authentication successful");
        } else {
            return Err(ConException::Todo.into());
        }

        Ok(())
    }

    async fn tune(&mut self) -> Result<()> {
        let tune_method = Method::ConnectionTune(ConnectionTune {
            channel_max: CHANNEL_MAX,
            frame_max: FRAME_SIZE_MAX,
            heartbeat: HEARTBEAT_DELAY,
        });

        debug!("Sending Tune method");
        self.send_method(ChannelNum::zero(), &tune_method).await?;

        let tune_ok = self.recv_method().await?;
        debug!(?tune_ok, "Received Tune-Ok method");

        if let Method::ConnectionTuneOk(ConnectionTuneOk {
            channel_max,
            frame_max,
            heartbeat,
        }) = tune_ok
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

        if let Method::ConnectionOpen(ConnectionOpen { virtual_host, .. }) = open {
            ensure_conn(virtual_host == "/")?;
        }

        self.send_method(
            ChannelNum::zero(),
            &Method::ConnectionOpenOk(ConnectionOpenOk {
                reserved_1: "".to_owned(),
            }),
        )
        .await?;

        Ok(())
    }

    async fn main_loop(&mut self) -> Result<()> {
        loop {
            select! {
                frame = frame::read_frame(&mut self.stream, self.max_frame_size) => {
                    let frame = frame?;
                    self.handle_frame(frame).await?;
                }
                queued_method = self.event_receiver.recv() => {
                    match queued_method {
                        Some(ConnectionEvent::Method(channel, method)) => self.send_method(channel, &method).await?,
                        Some(ConnectionEvent::MethodContent(channel, method, header, body)) => self.send_method_content(channel, &method, header, body).await?,
                        Some(ConnectionEvent::Shutdown) => return self.close(0, "".to_owned()).await,
                        None => {}
                    }
                }
            }
        }
    }

    async fn handle_frame(&mut self, frame: Frame) -> Result<()> {
        let channel = frame.channel;
        self.reset_timeout();

        let result = match frame.kind {
            FrameType::Method => self.dispatch_method(frame).await,
            FrameType::Heartbeat => {
                Ok(()) /* Nothing here, just the `reset_timeout` above  */
            }
            FrameType::Header => self.dispatch_header(frame),
            FrameType::Body => self.dispatch_body(frame),
        };

        match result {
            Ok(()) => Ok(()),
            Err(TransError::Protocol(ProtocolError::ChannelException(ex))) => {
                warn!(%ex, "Channel exception occurred");
                self.send_method(
                    channel,
                    &Method::ChannelClose(ChannelClose {
                        reply_code: ex.reply_code(),
                        reply_text: ex.reply_text(),
                        class_id: 0, // todo: do this
                        method_id: 0,
                    }),
                )
                .await?;
                drop(self.channels.remove(&channel));
                Ok(())
            }
            Err(other_err) => Err(other_err),
        }
    }

    async fn dispatch_method(&mut self, frame: Frame) -> Result<()> {
        let method = methods::parse_method(&frame.payload)?;
        debug!(?method, "Received method");

        // Sending a method implicitly cancels the content frames that might be ongoing
        self.channels
            .get_mut(&frame.channel)
            .map(|channel| channel.status.take());

        match method {
            Method::ConnectionClose(ConnectionClose {
                reply_code,
                reply_text,
                class_id,
                method_id,
            }) => {
                info!(%reply_code, %reply_text, %class_id, %method_id, "Closing connection");
                self.send_method(
                    ChannelNum::zero(),
                    &Method::ConnectionCloseOk(ConnectionCloseOk),
                )
                .await?;
                return Err(ProtocolError::GracefullyClosed.into());
            }
            Method::ChannelOpen { .. } => self.channel_open(frame.channel).await?,
            Method::ChannelClose { .. } => self.channel_close(frame.channel, method).await?,
            Method::BasicPublish { .. } => match self.channels.get_mut(&frame.channel) {
                Some(channel) => {
                    channel.status = ChannelStatus::NeedHeader(BASIC_CLASS_ID, Box::new(method));
                }
                None => return Err(ConException::Todo.into()),
            },
            _ => {
                let channel_handle = self
                    .channels
                    .get(&frame.channel)
                    .ok_or(ConException::Todo)?
                    .global_chan
                    .clone();

                // call into amqp_messaging to handle the method
                // it returns the response method that we are supposed to send
                // maybe this might become an `Option` in the future
                let return_method =
                    amqp_messaging::methods::handle_method(channel_handle, method).await?;
                self.send_method(frame.channel, &return_method).await?;
            }
        }
        Ok(())
    }

    fn dispatch_header(&mut self, frame: Frame) -> Result<()> {
        self.channels
            .get_mut(&frame.channel)
            .ok_or_else(|| ConException::Todo.into())
            .and_then(|channel| match channel.status.take() {
                ChannelStatus::Default => {
                    warn!(channel = %frame.channel, "unexpected header");
                    Err(ConException::UnexpectedFrame.into())
                }
                ChannelStatus::NeedHeader(class_id, method) => {
                    let header = parse_content_header(&frame.payload)?;
                    ensure_conn(header.class_id == class_id)?;

                    channel.status = ChannelStatus::NeedsBody(method, header, SmallVec::new());
                    Ok(())
                }
                ChannelStatus::NeedsBody(_, _, _) => {
                    warn!(channel = %frame.channel, "already got header");
                    Err(ConException::UnexpectedFrame.into())
                }
            })
    }

    fn dispatch_body(&mut self, frame: Frame) -> Result<()> {
        let channel = self
            .channels
            .get_mut(&frame.channel)
            .ok_or(ConException::Todo)?;

        match channel.status.take() {
            ChannelStatus::Default => {
                warn!(channel = %frame.channel, "unexpected body");
                Err(ConException::UnexpectedFrame.into())
            }
            ChannelStatus::NeedHeader(_, _) => {
                warn!(channel = %frame.channel, "unexpected body");
                Err(ConException::UnexpectedFrame.into())
            }
            ChannelStatus::NeedsBody(method, header, mut vec) => {
                vec.push(frame.payload);
                match vec
                    .iter()
                    .map(Bytes::len)
                    .sum::<usize>()
                    .cmp(&usize::try_from(header.body_size).unwrap())
                {
                    Ordering::Equal => {
                        self.process_method_with_body(*method, header, vec, frame.channel)
                    }
                    Ordering::Greater => Err(ConException::Todo.into()),
                    Ordering::Less => Ok(()), // wait for next body
                }
            }
        }
    }

    fn process_method_with_body(
        &mut self,
        method: Method,
        header: ContentHeader,
        payloads: SmallVec<[Bytes; 1]>,
        channel: ChannelNum,
    ) -> Result<()> {
        // The only method with content that is sent to the server is Basic.Publish.
        ensure_conn(header.class_id == BASIC_CLASS_ID)?;

        if let Method::BasicPublish(BasicPublish {
            exchange,
            routing_key,
            mandatory,
            immediate,
            ..
        }) = method
        {
            let message = RawMessage {
                id: MessageId::random(),
                header,
                routing: RoutingInformation {
                    exchange,
                    routing_key,
                    mandatory,
                    immediate,
                },
                content: payloads,
            };
            let message = Arc::new(message);

            let channel = self.channels.get(&channel).ok_or(ConException::Todo)?;

            // Spawn the handler for the publish. The connection task goes back to handling
            // just the connection.
            tokio::spawn(amqp_messaging::methods::handle_basic_publish(
                channel.global_chan.clone(),
                message,
            ));
            Ok(())
        } else {
            Err(ConException::Todo.into())
        }
    }

    async fn channel_open(&mut self, channel_num: ChannelNum) -> Result<()> {
        let id = rand::random();
        let channel_handle = ChannelInner::new(
            id,
            channel_num,
            self.global_con.clone(),
            self.global_data.clone(),
            self.event_sender.clone(),
        );

        let channel = TransportChannel {
            global_chan: channel_handle.clone(),
            status: ChannelStatus::Default,
        };

        let prev = self.channels.insert(channel_num, channel);
        if let Some(prev) = prev {
            self.channels.insert(channel_num, prev); // restore previous state
            return Err(ConException::ChannelError.into());
        }

        {
            let mut global_data = self.global_data.lock();
            global_data.channels.insert(id, channel_handle.clone());
            global_data
                .connections
                .get_mut(&self.id)
                .unwrap()
                .channels
                .lock()
                .insert(channel_num, channel_handle);
        }

        info!(%channel_num, "Opened new channel");

        self.send_method(
            channel_num,
            &Method::ChannelOpenOk(ChannelOpenOk {
                reserved_1: Vec::new(),
            }),
        )
        .await?;

        Ok(())
    }

    async fn channel_close(&mut self, channel_id: ChannelNum, method: Method) -> Result<()> {
        if let Method::ChannelClose(ChannelClose {
            reply_code: code,
            reply_text: reason,
            ..
        }) = method
        {
            info!(%code, %reason, "Closing channel");

            if let Some(channel) = self.channels.remove(&channel_id) {
                drop(channel);
                self.send_method(channel_id, &Method::ChannelCloseOk(ChannelCloseOk))
                    .await?;
            } else {
                return Err(ConException::Todo.into());
            }
        } else {
            unreachable!()
        }
        Ok(())
    }

    fn reset_timeout(&mut self) {
        if self.heartbeat_delay != 0 {
            let next = Duration::from_secs(u64::from(self.heartbeat_delay / 2));
            self.next_timeout = Box::pin(time::sleep(next));
        }
    }

    async fn negotiate_version(&mut self) -> Result<()> {
        const HEADER_SIZE: usize = 8;
        const SUPPORTED_PROTOCOL_VERSION: &[u8] = &[0, 9, 1];
        const AMQP_PROTOCOL: &[u8] = b"AMQP";
        const OWN_PROTOCOL_HEADER: &[u8] = b"AMQP\0\0\x09\x01";

        debug!("Negotiating version");

        let mut read_header_buf = [0; HEADER_SIZE];

        self.stream
            .read_exact(&mut read_header_buf)
            .await
            .context("read protocol header")?;

        debug!(received_header = ?read_header_buf,"Received protocol header");

        let protocol = &read_header_buf[0..4];
        let version = &read_header_buf[5..8];

        if protocol != AMQP_PROTOCOL {
            self.stream
                .write_all(OWN_PROTOCOL_HEADER)
                .await
                .context("write protocol header")?;
            debug!(?protocol, "Version negotiation failed");
            return Err(ProtocolError::ProtocolNegotiationFailed.into());
        }

        if &read_header_buf[0..5] == b"AMQP\0" && version == SUPPORTED_PROTOCOL_VERSION {
            debug!(?version, "Version negotiation successful");
            Ok(())
        } else {
            self.stream
                .write_all(OWN_PROTOCOL_HEADER)
                .await
                .context("write protocol header")?;
            debug!(?version, expected_version = ?SUPPORTED_PROTOCOL_VERSION, "Version negotiation failed");
            Err(ProtocolError::ProtocolNegotiationFailed.into())
        }
    }

    async fn close(&mut self, reply_code: ReplyCode, reply_text: ReplyText) -> Result<()> {
        self.send_method(
            ChannelNum::zero(),
            &Method::ConnectionClose(ConnectionClose {
                reply_code,
                reply_text,
                class_id: 0, // todo: do this
                method_id: 0,
            }),
        )
        .await?;

        match self.recv_method().await {
            Ok(Method::ConnectionCloseOk(_)) => Ok(()),
            Ok(method) => {
                return Err(TransError::Other(anyhow!(
                    "Received wrong method after closing, method: {method:?}"
                )));
            }
            Err(err) => {
                return Err(TransError::Other(anyhow!(
                    "Failed to receive Connection.CloseOk method after closing, err: {err}"
                )));
            }
        }
    }
}

impl Drop for TransportConnection {
    fn drop(&mut self) {
        self.global_con.close();
    }
}

impl Drop for TransportChannel {
    fn drop(&mut self) {
        self.global_chan.close();
    }
}

fn server_properties(host: SocketAddr) -> Table {
    fn ls(str: &str) -> FieldValue {
        FieldValue::LongString(str.into())
    }

    let host_str = host.ip().to_string();
    HashMap::from([
        ("host".to_owned(), ls(&host_str)),
        ("product".to_owned(), ls("no name yet")),
        ("version".to_owned(), ls("0.1.0")),
        ("platform".to_owned(), ls("microsoft linux")),
        ("copyright".to_owned(), ls("MIT")),
        ("information".to_owned(), ls("hello reader")),
        ("uwu".to_owned(), ls("owo")),
    ])
}
