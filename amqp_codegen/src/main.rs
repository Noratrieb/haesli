use std::fs;
use strong_xml::{XmlError, XmlRead};

#[derive(Debug, XmlRead)]
#[xml(tag = "amqp")]
struct Amqp {
    #[xml(child = "domain")]
    domains: Vec<Domain>,
    #[xml(child = "class")]
    classes: Vec<Class>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "domain")]
struct Domain {
    #[xml(attr = "name")]
    name: String,
    #[xml(attr = "type")]
    kind: String,
    #[xml(child = "assert")]
    asserts: Vec<Assert>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "assert")]
struct Assert {
    #[xml(attr = "check")]
    check: String,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "class")]
struct Class {
    #[xml(attr = "name")]
    name: String,
    #[xml(attr = "handler")]
    handler: String,
    #[xml(attr = "index")]
    index: u16,
    #[xml(child = "method")]
    methods: Vec<Method>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "method")]
struct Method {
    #[xml(attr = "name")]
    name: String,
    #[xml(child = "field")]
    fields: Vec<Field>,
    #[xml(attr = "index")]
    index: u16,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "field")]
struct Field {
    #[xml(attr = "name")]
    name: String,
    #[xml(attr = "domain")]
    domain: Option<String>,
    #[xml(child = "assert")]
    asserts: Vec<Assert>,
}

fn main() {
    let content = fs::read_to_string("./amqp-0-9-1.xml").unwrap();

    let amqp: Result<Amqp, XmlError> = Amqp::from_str(&content);

    match amqp {
        Ok(amqp) => {
            println!("{amqp:#?}");
        }
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    }
}
