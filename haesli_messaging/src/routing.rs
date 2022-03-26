use haesli_core::{
    exchange::{Exchange, ExchangeType, TopicSegment},
    queue::Queue,
};

fn parse_topic(topic: &str) -> Vec<TopicSegment> {
    topic
        .split('.')
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

/// Route a message to a queue. Returns the queues to send it to, or `None` if it can't be matched
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
            // todo: optimizing this is a fun problem

            Some(match_topic(bindings, routing_key))
        }
        ExchangeType::Headers => None, // unsupported
        ExchangeType::System => None,  // unsupported
    }
}

fn match_topic<Q: Clone>(patterns: &[(Vec<TopicSegment>, Q)], routing_key: &str) -> Vec<Q> {
    patterns
        .iter()
        .filter_map(|(pattern, value)| {
            let mut key_segments = routing_key.split('.');
            let mut pat_segments = pattern.iter();

            loop {
                let key = key_segments.next();
                let pat = pat_segments.next();

                match (pat, key) {
                    (Some(TopicSegment::Word(pat)), Some(key)) if pat != key => return None,
                    (Some(TopicSegment::Word(_)), Some(_)) => {}
                    (Some(TopicSegment::SingleWildcard), Some(_)) => {}
                    (Some(TopicSegment::MultiWildcard), _) => {
                        let pat = pat_segments.next();
                        match pat {
                            Some(pat) => {
                                // loop until we find the next pat segment in the key
                                loop {
                                    let key = key_segments.next();
                                    match (pat, key) {
                                        (_, None) => return None, // we are expecting something after the wildcard
                                        (TopicSegment::Word(pat), Some(key)) if pat == key => {
                                            break; // we matched all of the `#` and the segment afterwards
                                        }
                                        (TopicSegment::SingleWildcard, Some(_)) => {
                                            break; // `#.*` is a cursed pattern, match `#` for 1
                                        }
                                        (TopicSegment::MultiWildcard, Some(_)) => {
                                            break; // at this point I don't even care, who the fuck
                                                   // would use `#.#`, just only match 2 which is
                                                   // wrong so todo I guess lol
                                        }
                                        _ => continue,
                                    }
                                }
                            }
                            None => break, // pattern ends with `#`, it certainly matches
                        }
                    }
                    (Some(_), None) => return None,
                    (None, Some(_)) => return None,
                    (None, None) => break,
                }
            }

            Some(value.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::routing::{match_topic, parse_topic};

    macro_rules! match_topics_test {
        ($name:ident {
            patterns: $($pattern:expr),*;
            routing_key: $routing_key:expr;
            expected: $($expected:expr),*;
        }) => {
            #[test]
            fn $name() {
                fn inc(x: &mut u64) -> u64 { let tmp = *x; *x += 1; tmp }

                let mut n = 0;
                let n = &mut n;

                // assign each pattern a number
                let patterns = [$((parse_topic($pattern), inc(n))),*];

                let matched = match_topic(&patterns, $routing_key);
                let expected = vec![$($expected),*];

                assert_eq!(matched, expected);
            }
        };
    }

    match_topics_test!(match_spec_example_1 {
        patterns: "*.stock.#";
        routing_key: "usd.stock";
        expected: 0;
    });

    match_topics_test!(match_spec_example_2 {
        patterns: "*.stock.#";
        routing_key: "eur.stock.db";
        expected: 0;
    });

    match_topics_test!(match_spec_example_3 {
        patterns: "*.stock.#";
        routing_key: "stock.nasdaq";
        expected: ;
    });

    match_topics_test!(match_no_wildcards {
        patterns: "na.stock.usd", "sa.stock.peso", "stock.nasdaq", "usd.stock.na";
        routing_key: "na.stock.usd";
        expected: 0;
    });

    match_topics_test!(match_cursed_wildcards {
        patterns: "*.*.*", "#.usd", "#.stock.*", "*.#", "#", "na.*";
        routing_key: "na.stock.usd";
        expected: 0, 1, 2, 3, 4;
    });

    match_topics_test!(match_empty_topic {
        patterns: "", "bad";
        routing_key: "";
        expected: 0;
    });
}
