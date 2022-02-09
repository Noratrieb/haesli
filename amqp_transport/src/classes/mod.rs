use std::collections::HashMap;

type ClassId = u16;

type ConsumerTag = String;

type DeliveryTag = u64;

/// must be shorter than 127, must match `^[a-zA-Z0-9-_.:]*$`
type ExchangeName = String;

type MethodId = u16;

type NoAck = u8;

type NoLocal = u8;

type NoWait = u8;

/// must be shorter than 127
type Path = String;

type PeerProperties = HashMap<Shortstr, (Octet, /* todo */ Box<dyn std::any::Any>)>;

/// must be shorter than 127, must match `^[a-zA-Z0-9-_.:]*$`
type QueueName = String;

type Redelivered = u8;

type MessageCount = u32;

type ReplyCode = u16;

type ReplyText = String;

type Bit = u8;

type Octet = u8;

type Short = u16;

type Long = u32;

type Longlong = u64;

type Shortstr = String;

type Longstr = String;

type Timestamp = u64;

type Table = HashMap<Shortstr, (Octet, /* todo */ Box<dyn std::any::Any>)>;

/// Index 10, handler = connection
pub enum Connection {
    /// Index 10
    Start {
        version_major: Option<Octet>,
        version_minor: Option<Octet>,
        server_properties: Option<PeerProperties>,
        mechanisms: Longstr,
        locales: Longstr,
    },
    /// Index 11
    StartOk {
        client_properties: Option<PeerProperties>,
        mechanism: Shortstr,
        response: Longstr,
        locale: Shortstr,
    },
    /// Index 20
    Secure {
        challenge: Option<Longstr>,
    },
    /// Index 21
    SecureOk {
        response: Longstr,
    },
    /// Index 30
    Tune {
        channel_max: Option<Short>,
        frame_max: Option<Long>,
        heartbeat: Option<Short>,
    },
    /// Index 31
    TuneOk {
        /// must be less than the tune field of the method channel-max
        channel_max: Short,
        frame_max: Option<Long>,
        heartbeat: Option<Short>,
    },
    /// Index 40
    Open {
        virtual_host: Path,
        reserved_1: Option<Shortstr>,
        reserved_2: Option<Bit>,
    },
    /// Index 41
    OpenOk {
        reserved_1: Option<Shortstr>,
    },
    /// Index 50
    Close {
        reply_code: ReplyCode,
        reply_text: ReplyText,
        class_id: Option<ClassId>,
        method_id: Option<MethodId>,
    },
    /// Index 51
    CloseOk,
    /// Index 60
    Blocked {
        reason: Option<Shortstr>,
    },
    /// Index 61
    Unblocked,
}
/// Index 20, handler = channel
pub enum Channel {
    /// Index 10
    Open {
        reserved_1: Option<Shortstr>,
    },
    /// Index 11
    OpenOk {
        reserved_1: Option<Longstr>,
    },
    /// Index 20
    Flow {
        active: Option<Bit>,
    },
    /// Index 21
    FlowOk {
        active: Option<Bit>,
    },
    /// Index 40
    Close {
        reply_code: ReplyCode,
        reply_text: ReplyText,
        class_id: Option<ClassId>,
        method_id: Option<MethodId>,
    },
    /// Index 41
    CloseOk,
}
/// Index 40, handler = channel
pub enum Exchange {
    /// Index 10
    Declare {
        reserved_1: Option<Short>,
        exchange: ExchangeName,
        r#type: Option<Shortstr>,
        passive: Option<Bit>,
        durable: Option<Bit>,
        reserved_2: Option<Bit>,
        reserved_3: Option<Bit>,
        no_wait: Option<NoWait>,
        arguments: Option<Table>,
    },
    /// Index 11
    DeclareOk,
    /// Index 20
    Delete {
        reserved_1: Option<Short>,
        exchange: ExchangeName,
        if_unused: Option<Bit>,
        no_wait: Option<NoWait>,
    },
    /// Index 21
    DeleteOk,
}
/// Index 50, handler = channel
pub enum Queue {
    /// Index 10
    Declare {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        passive: Option<Bit>,
        durable: Option<Bit>,
        exclusive: Option<Bit>,
        auto_delete: Option<Bit>,
        no_wait: Option<NoWait>,
        arguments: Option<Table>,
    },
    /// Index 11
    DeclareOk {
        queue: QueueName,
        message_count: Option<MessageCount>,
        consumer_count: Option<Long>,
    },
    /// Index 20
    Bind {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        exchange: Option<ExchangeName>,
        routing_key: Option<Shortstr>,
        no_wait: Option<NoWait>,
        arguments: Option<Table>,
    },
    /// Index 21
    BindOk,
    /// Index 50
    Unbind {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        exchange: Option<ExchangeName>,
        routing_key: Option<Shortstr>,
        arguments: Option<Table>,
    },
    /// Index 51
    UnbindOk,
    /// Index 30
    Purge {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        no_wait: Option<NoWait>,
    },
    /// Index 31
    PurgeOk {
        message_count: Option<MessageCount>,
    },
    /// Index 40
    Delete {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        if_unused: Option<Bit>,
        if_empty: Option<Bit>,
        no_wait: Option<NoWait>,
    },
    /// Index 41
    DeleteOk {
        message_count: Option<MessageCount>,
    },
}
/// Index 60, handler = channel
pub enum Basic {
    /// Index 10
    Qos {
        prefetch_size: Option<Long>,
        prefetch_count: Option<Short>,
        global: Option<Bit>,
    },
    /// Index 11
    QosOk,
    /// Index 20
    Consume {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        consumer_tag: Option<ConsumerTag>,
        no_local: Option<NoLocal>,
        no_ack: Option<NoAck>,
        exclusive: Option<Bit>,
        no_wait: Option<NoWait>,
        arguments: Option<Table>,
    },
    /// Index 21
    ConsumeOk {
        consumer_tag: Option<ConsumerTag>,
    },
    /// Index 30
    Cancel {
        consumer_tag: Option<ConsumerTag>,
        no_wait: Option<NoWait>,
    },
    /// Index 31
    CancelOk {
        consumer_tag: Option<ConsumerTag>,
    },
    /// Index 40
    Publish {
        reserved_1: Option<Short>,
        exchange: Option<ExchangeName>,
        routing_key: Option<Shortstr>,
        mandatory: Option<Bit>,
        immediate: Option<Bit>,
    },
    /// Index 50
    Return {
        reply_code: ReplyCode,
        reply_text: ReplyText,
        exchange: Option<ExchangeName>,
        routing_key: Option<Shortstr>,
    },
    /// Index 60
    Deliver {
        consumer_tag: Option<ConsumerTag>,
        delivery_tag: Option<DeliveryTag>,
        redelivered: Option<Redelivered>,
        exchange: Option<ExchangeName>,
        routing_key: Option<Shortstr>,
    },
    /// Index 70
    Get {
        reserved_1: Option<Short>,
        queue: Option<QueueName>,
        no_ack: Option<NoAck>,
    },
    /// Index 71
    GetOk {
        delivery_tag: Option<DeliveryTag>,
        redelivered: Option<Redelivered>,
        exchange: Option<ExchangeName>,
        routing_key: Option<Shortstr>,
        message_count: Option<MessageCount>,
    },
    /// Index 72
    GetEmpty {
        reserved_1: Option<Shortstr>,
    },
    /// Index 80
    Ack {
        delivery_tag: Option<DeliveryTag>,
        multiple: Option<Bit>,
    },
    /// Index 90
    Reject {
        delivery_tag: Option<DeliveryTag>,
        requeue: Option<Bit>,
    },
    /// Index 100
    RecoverAsync {
        requeue: Option<Bit>,
    },
    /// Index 110
    Recover {
        requeue: Option<Bit>,
    },
    /// Index 111
    RecoverOk,
}
/// Index 90, handler = channel
pub enum Tx {
    /// Index 10
    Select,
    /// Index 11
    SelectOk,
    /// Index 20
    Commit,
    /// Index 21
    CommitOk,
    /// Index 30
    Rollback,
    /// Index 31
    RollbackOk,
}
