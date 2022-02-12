use crate::{Amqp, Class, Domain, Method};
use anyhow::Result;
use heck::{ToSnakeCase, ToUpperCamelCase};
use itertools::Itertools;

fn method_function_name(class_name: &str) -> impl Fn(&Method) -> String + '_ {
    move |method| {
        let method_name = method.name.to_snake_case();
        format!("{class_name}_{method_name}")
    }
}

fn domain_function_name(domain_name: &str) -> String {
    let domain_name = domain_name.to_snake_case();
    format!("domain_{domain_name}")
}

pub(crate) fn codegen_parser(amqp: &Amqp) -> Result<()> {
    println!(
        "pub mod parse {{
use super::*;
use crate::classes::parse_helper::*;
use crate::error::TransError;
use nom::{{branch::alt, bytes::complete::tag}};
use regex::Regex;
use once_cell::sync::Lazy;

pub type IResult<'a, T> = nom::IResult<&'a [u8], T, TransError>;
"
    );
    println!(
        "pub fn parse_method(input: &[u8]) -> Result<(&[u8], Class), nom::Err<TransError>> {{
    alt(({}))(input)
}}",
        amqp.classes
            .iter()
            .map(|class| class.name.to_snake_case())
            .join(", ")
    );

    for domain in &amqp.domains {
        domain_parser(domain)?;
    }

    for class in &amqp.classes {
        let class_name = class.name.to_snake_case();

        function(&class_name, "Class", || {
            let class_index = class.index;
            let all_methods = class
                .methods
                .iter()
                .map(method_function_name(&class_name))
                .join(", ");
            println!(
                "    let (input, _) = tag([{class_index}])(input)?;
    alt(({all_methods}))(input)"
            );

            Ok(())
        })?;

        for method in &class.methods {
            method_parser(class, method)?;
        }
    }

    println!("\n}}");
    Ok(())
}

fn domain_parser(domain: &Domain) -> Result<()> {
    let fn_name = domain_function_name(&domain.name);
    let type_name = domain.kind.to_snake_case();
    function(&fn_name, &domain.name.to_upper_camel_case(), || {
        if domain.asserts.is_empty() {
            if type_name == "bit" {
                println!("    todo!() // bit")
            } else {
                println!("    {type_name}(input)");
            }
        } else {
            println!("    let (input, result) = {type_name}(input)?;");

            for assert in &domain.asserts {
                match &*assert.check {
                    "notnull" => { /* todo */ }
                    "regexp" => {
                        let value = assert.value.as_ref().unwrap();
                        println!(
                            r#"    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"{value}").unwrap());"#
                        );
                        println!("    if !REGEX.is_match(&result) {{ fail!() }}");
                    }
                    "le" => {} // can't validate this here
                    "length" => {
                        let length = assert.value.as_ref().unwrap();
                        println!("    if result.len() > {length} {{ fail!() }}");
                    }
                    _ => unimplemented!(),
                }
            }
            println!("    Ok((input, result))");
        }
        Ok(())
    })
}

fn method_parser(class: &Class, method: &Method) -> Result<()> {
    let class_name = class.name.to_snake_case();

    let function_name = method_function_name(&class_name)(method);
    function(&function_name, "Class", || {
        let method_index = method.index;
        println!("    let (input, _) = tag([{method_index}])(input)?;");
        println!("    todo!()");
        for _field in &method.fields {}
        Ok(())
    })?;

    Ok(())
}

fn function<F>(name: &str, ret_ty: &str, body: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    println!("fn {name}(input: &[u8]) -> IResult<{ret_ty}> {{");
    body()?;
    println!("}}");

    Ok(())
}
