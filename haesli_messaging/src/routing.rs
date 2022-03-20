use haesli_core::{
    exchange::{Exchange, ExchangeType},
    queue::Queue,
};

pub fn bind(exchange: &mut Exchange, routing_key: String, queue: Queue) {
    match &mut exchange.kind {
        ExchangeType::Direct { bindings } => {
            bindings.insert(routing_key, queue);
        }
        ExchangeType::Fanout { bindings } => bindings.push(queue),
        ExchangeType::Topic { bindings } => bindings.push((routing_key, queue)),
        ExchangeType::Headers => {} // unsupported
        ExchangeType::System => {}  // unsupported
    }
}

/// Route a message to a queue. Returns the queue to send it to, or `None` if it can't be matched
pub fn route_message(exchange: &Exchange, routing_key: &str) -> Option<Queue> {
    match &exchange.kind {
        ExchangeType::Direct { bindings } => {
            // 3.1.3.1 - routing-key = routing-key
            bindings.get(routing_key).cloned()
        }
        ExchangeType::Fanout { .. } => None,
        ExchangeType::Topic { .. } => None,
        ExchangeType::Headers => None, // unsupported
        ExchangeType::System => None,  // unsupported
    }
}
