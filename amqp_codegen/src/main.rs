use anyhow::{Context, Result};
use heck::ToUpperCamelCase;
use std::fs;
use strong_xml::XmlRead;

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
    #[xml(attr = "method")]
    method: Option<String>,
    #[xml(attr = "field")]
    field: Option<String>,
    #[xml(attr = "value")]
    value: Option<String>,
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
    #[xml(attr = "type")]
    kind: Option<String>,
    #[xml(child = "assert")]
    asserts: Vec<Assert>,
}

fn main() -> Result<()> {
    let content = fs::read_to_string("./amqp-0-9-1.xml").unwrap();

    let amqp = Amqp::from_str(&content)?;
    codegen(&amqp)
}

fn codegen(amqp: &Amqp) -> Result<()> {
    println!("use std::collections::HashMap;\n");
    domain_defs(amqp)?;
    class_defs(amqp)
}

fn domain_defs(amqp: &Amqp) -> Result<()> {
    for domain in &amqp.domains {
        let invariants = invariants(domain.asserts.iter());

        if !invariants.is_empty() {
            println!("/// {invariants}");
        }
        println!(
            "type {} = {};\n",
            domain.name.to_upper_camel_case(),
            amqp_type_to_rust_type(&domain.kind),
        );
    }

    Ok(())
}

fn class_defs(amqp: &Amqp) -> Result<()> {
    for class in &amqp.classes {
        let enum_name = class.name.to_upper_camel_case();
        println!("/// Index {}, handler = {}", class.index, class.handler);
        println!("pub enum {enum_name} {{");
        for method in &class.methods {
            let method_name = method.name.to_upper_camel_case();
            println!("    /// Index {}", method.index);
            print!("    {method_name}");
            if method.fields.len() > 0 {
                println!(" {{");
                for field in &method.fields {
                    let field_name = snake_case(&field.name);
                    let (field_type, field_docs) = resolve_type(
                        amqp,
                        &field.domain.as_ref().or(field.kind.as_ref()).unwrap(),
                        field.asserts.as_ref(),
                    )?;
                    if !field_docs.is_empty() {
                        println!("        /// {field_docs}");
                    }
                    println!("        {field_name}: {field_type},");
                }
                println!("    }},");
            } else {
                println!(",");
            }
        }
        println!("}}");
    }

    Ok(())
}

fn amqp_type_to_rust_type<'a>(amqp_type: &str) -> &'static str {
    match amqp_type {
        "octet" => "u8",
        "short" => "u16",
        "long" => "u32",
        "longlong" => "u64",
        "bit" => "u8",
        "shortstr" | "longstr" => "String",
        "timestamp" => "u64",
        "table" => "HashMap<Shortstr, (Octet, /* todo */ Box<dyn std::any::Any>)>",
        _ => unreachable!("invalid type {}", amqp_type),
    }
}

/// returns (type name, invariant docs)
fn resolve_type(amqp: &Amqp, domain: &str, asserts: &[Assert]) -> Result<(String, String)> {
    let kind = amqp
        .domains
        .iter()
        .find(|d| &d.name == domain)
        .context("domain not found")?;

    let is_nonnull = is_nonnull(asserts.iter().chain(kind.asserts.iter()));

    let additional_docs = invariants(asserts.iter());

    let type_name = domain.to_upper_camel_case();

    Ok((
        if is_nonnull {
            type_name
        } else {
            format!("Option<{type_name}>")
        },
        additional_docs,
    ))
}

fn is_nonnull<'a>(mut asserts: impl Iterator<Item = &'a Assert>) -> bool {
    asserts.find(|assert| assert.check == "notnull").is_some()
}

fn snake_case(ident: &str) -> String {
    use heck::ToSnakeCase;

    if ident == "type" {
        "r#type".to_string()
    } else {
        ident.to_snake_case()
    }
}

fn invariants<'a>(asserts: impl Iterator<Item = &'a Assert>) -> String {
    asserts
        .filter_map(|assert| match &*assert.check {
            "notnull" => None,
            "length" => Some(format!(
                "must be shorter than {}",
                assert.value.as_ref().unwrap()
            )),
            "regexp" => Some(format!("must match `{}`", assert.value.as_ref().unwrap())),
            "le" => Some(format!(
                "must be less than the {} field of the method {}",
                assert.method.as_ref().unwrap(),
                assert.field.as_ref().unwrap()
            )),
            _ => unimplemented!(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
