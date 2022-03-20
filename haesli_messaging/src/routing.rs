use std::sync::Arc;

use haesli_core::{
    exchange::{Exchange, ExchangeType, TopicSegment},
    queue::Queue,
};

fn parse_topic(topic: &str) -> Vec<TopicSegment> {
    topic
        .split(".")
        .map(|segment| match segment {
            "*" => TopicSegment::SingleWildcard,
            "#" => TopicSegment::MultiWildcard,
            word => TopicSegment::Word(word.to_owned()),
        })
        .collect()
}

pub fn bind(exchange: &mut Exchange, routing_key: String, queue: Queue) {
    match &mut exchange.kind {
        ExchangeType::Direct { bindings } => {
            bindings.insert(routing_key, queue);
        }
        ExchangeType::Fanout { bindings } => bindings.push(queue),
        ExchangeType::Topic { bindings } => bindings.push((parse_topic(&routing_key), queue)),
        ExchangeType::Headers => {} // unsupported
        ExchangeType::System => {}  // unsupported
    }
}

/// Route a message to a queue. Returns the queue to send it to, or `None` if it can't be matched
pub fn route_message(exchange: &Exchange, routing_key: &str) -> Option<Vec<Queue>> {
    match &exchange.kind {
        ExchangeType::Direct { bindings } => {
            // 3.1.3.1 - routing-key = routing-key
            bindings.get(routing_key).cloned().map(|q| vec![q])
        }
        ExchangeType::Fanout { bindings } => {
            // 3.1.3.2 - unconditionally
            Some(bindings.clone()) // see, this is actually Not That Bad I Hope
        }
        ExchangeType::Topic { bindings } => {
            let topic = parse_topic(routing_key);
            // todo: optimizing this is a fun problem

            Some(match_topic(bindings, topic))
        }
        ExchangeType::Headers => None, // unsupported
        ExchangeType::System => None,  // unsupported
    }
}

fn match_topic<Q: Clone>(
    patterns: &[(Vec<TopicSegment>, Q)],
    routing_key: Vec<TopicSegment>,
) -> Vec<Q> {
    patterns
        .iter()
        .filter_map(|(pattern, value)| {
            let mut queue_segments = routing_key.iter();

            for segment in pattern {}

            Some(value.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::routing::{match_topic, parse_topic};

    macro_rules! match_topics {
        (patterns: $($pattern:expr),*) => {};
    }

    #[test]
    #[ignore]
    fn match_empty_topic() {
        let patterns = [(parse_topic(""), 1), (parse_topic("BAD"), 2)];
        let routing_key = parse_topic("");

        let matched = match_topic(&patterns, routing_key);

        assert_eq!(matched, vec![1])
    }
}
