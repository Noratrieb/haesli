use anyhow::Result;
use heck::{ToSnakeCase, ToUpperCamelCase};
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

fn main() -> Result<()> {
    let content = fs::read_to_string("./amqp-0-9-1.xml").unwrap();

    let amqp = Amqp::from_str(&content)?;
    codegen(&amqp)
}

fn codegen(amqp: &Amqp) -> Result<()> {
    for class in &amqp.classes {
        let enum_name = class.name.to_upper_camel_case();
        println!("///////// ---- Class {enum_name}");
        println!("enum {enum_name} {{");
        for method in &class.methods {
            let method_name = method.name.to_upper_camel_case();
            print!("    {method_name}");
            if method.fields.len() > 0 {
                println!(" {{");
                for field in &method.fields {
                    let field_name = field.name.to_snake_case();
                    println!("        {field_name}: (),");
                }
                println!("    }}");
            } else {
                println!(",");
            }
        }
        println!("}}");
    }

    Ok(())
}
