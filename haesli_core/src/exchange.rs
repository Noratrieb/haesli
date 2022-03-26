use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use crate::{newtype, Queue};

#[derive(Debug)]
pub enum TopicSegment {
    Word(String),
    SingleWildcard,
    MultiWildcard,
}

#[derive(Debug)]
pub enum ExchangeType {
    /// Routes a message to a queue if the routing-keys are equal
    Direct { bindings: HashMap<String, Queue> },
    /// Always routes the message to a queue
    Fanout { bindings: Vec<Queue> },
    /// Routes a message to a queue if the routing key matches the pattern
    Topic {
        bindings: Vec<(Vec<TopicSegment>, Queue)>,
    },
    /// Is bound with a table of headers and values, and matches if the message headers
    /// match up with the binding headers
    ///
    /// Unsupported for now.
    Headers,
    /// The message is sent to the server system service with the name of the routing-key
    ///
    /// Unsupported for now.
    System,
}

newtype!(
    /// The name of a queue. A newtype wrapper around `Arc<str>`, which guarantees cheap clones.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub ExchangeName: Arc<str>
);

impl Borrow<str> for ExchangeName {
    fn borrow(&self) -> &str {
        Borrow::borrow(&self.0)
    }
}

#[derive(Debug)]
pub struct Exchange {
    pub name: ExchangeName,
    pub kind: ExchangeType,
    pub durable: bool,
}

pub fn default_exchanges() -> HashMap<ExchangeName, Exchange> {
    // 3.1.3 - The spec requires a few default exchanges to exist

    let empty_name = ExchangeName::new("".to_owned().into());
    let empty = Exchange {
        name: empty_name.clone(),
        kind: ExchangeType::Direct {
            bindings: HashMap::new(),
        },
        durable: true,
    };

    let direct_name = ExchangeName::new("amqp.direct".to_owned().into());
    let direct = Exchange {
        name: direct_name.clone(),
        kind: ExchangeType::Direct {
            bindings: HashMap::new(),
        },
        durable: true,
    };

    let fanout_name = ExchangeName::new("amqp.fanout".to_owned().into());
    let fanout = Exchange {
        name: fanout_name.clone(),
        kind: ExchangeType::Fanout {
            bindings: Vec::new(),
        },
        durable: true,
    };

    let topic_name = ExchangeName::new("amqp.topic".to_owned().into());
    let topic = Exchange {
        name: topic_name.clone(),
        kind: ExchangeType::Topic {
            bindings: Vec::new(),
        },
        durable: true,
    };

    // we don't implement headers (yet), so don't provide the default exchange for it

    HashMap::from([
        (empty_name, empty),
        (direct_name, direct),
        (fanout_name, fanout),
        (topic_name, topic),
    ])
}
