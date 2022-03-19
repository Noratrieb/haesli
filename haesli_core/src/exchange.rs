pub enum ExchangeType {
    /// Routes a message to a queue if the routing-keys are equal
    Direct,
    /// Always routes the message to a queue
    Fanout,
    /// Routes a message to a queue if the routing key matches the pattern
    Topic,
    /// Is bound with a table of headers and values, and matches if the message headers
    /// match up with the binding headers
    Headers,
    /// The message is sent to the server system service with the name of the routing-key
    System,
}
