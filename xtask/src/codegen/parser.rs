use super::{
    field_type, resolve_type_from_domain, snake_case, subsequent_bit_fields, Amqp, Assert, Class,
    Domain, Method,
};
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

pub(super) fn codegen_parser(amqp: &Amqp) {
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
        domain_parser(domain);
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
            let class_name_raw = &class.name;
            println!(
                r#"    let (input, _) = tag({class_index}_u16.to_be_bytes())(input).map_err(err("invalid tag for class {class_name_raw}"))?;
    alt(({all_methods}))(input).map_err(err("class {class_name_raw}")).map_err(failure)"#
            );
        });

        for method in &class.methods {
            method_parser(amqp, class, method);
        }
    }

    println!("\n}}");
}

fn domain_parser(domain: &Domain) {
    let fn_name = domain_function_name(&domain.name);
    let type_name = domain.kind.to_snake_case();
    // don't even bother with bit domains, do them manually at call site
    if type_name != "bit" {
        function(&fn_name, &domain.name.to_upper_camel_case(), || {
            if domain.asserts.is_empty() {
                println!("    {type_name}(input)");
            } else {
                println!("    let (input, result) = {type_name}(input)?;");

                for assert in &domain.asserts {
                    assert_check(assert, &type_name, "result");
                }
                println!("    Ok((input, result))");
            }
        });
    }
}

fn method_parser(amqp: &Amqp, class: &Class, method: &Method) {
    let class_name = class.name.to_snake_case();
    let method_name_raw = &method.name;

    let function_name = method_function_name(&class_name)(method);
    function(&function_name, "Class", || {
        let method_index = method.index;
        println!(
            r#"    let (input, _) = tag({method_index}_u16.to_be_bytes())(input).map_err(err("parsing method index"))?;"#
        );
        let mut iter = method.fields.iter().peekable();
        while let Some(field) = iter.next() {
            let field_name_raw = &field.name;
            let type_name = resolve_type_from_domain(amqp, field_type(field));

            if type_name == "bit" {
                let fields_with_bit = subsequent_bit_fields(amqp, field, &mut iter);

                let amount = fields_with_bit.len();
                println!(
                    r#"    let (input, bits) = bit(input, {amount}).map_err(err("field {field_name_raw} in method {method_name_raw}")).map_err(failure)?;"#
                );

                for (i, field) in fields_with_bit.iter().enumerate() {
                    let field_name = snake_case(&field.name);
                    println!("    let {field_name} = bits[{i}];");
                }
            } else {
                let fn_name = domain_function_name(field_type(field));
                let field_name = snake_case(&field.name);
                println!(
                    r#"    let (input, {field_name}) = {fn_name}(input).map_err(err("field {field_name_raw} in method {method_name_raw}")).map_err(failure)?;"#
                );

                for assert in &field.asserts {
                    assert_check(assert, &type_name, &field_name);
                }
            }
        }
        let class_name = class_name.to_upper_camel_case();
        let method_name = method.name.to_upper_camel_case();
        println!("    Ok((input, Class::{class_name}({class_name}::{method_name} {{");
        for field in &method.fields {
            let field_name = snake_case(&field.name);
            println!("        {field_name},");
        }
        println!("    }})))");
    });
}

fn assert_check(assert: &Assert, type_name: &str, var_name: &str) {
    match &*assert.check {
        "notnull" => match type_name {
            "shortstr" | "longstr" => {
                println!(
                    r#"    if {var_name}.is_empty() {{ fail!("string was null for field {var_name}") }}"#
                );
            }
            "short" => {
                println!(
                    r#"    if {var_name} == 0 {{ fail!("number was 0 for field {var_name}") }}"#
                );
            }
            _ => unimplemented!(),
        },
        "regexp" => {
            let value = assert.value.as_ref().unwrap();
            println!(
                r#"    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"{value}").unwrap());"#
            );
            let cause = format!("regex `{value}` did not match value for field {var_name}");
            println!(r#"    if !REGEX.is_match(&{var_name}) {{ fail!(r"{cause}") }}"#);
        }
        "le" => {} // can't validate this here
        "length" => {
            let length = assert.value.as_ref().unwrap();
            let cause = format!("value is shorter than {length} for field {var_name}");
            println!(r#"    if {var_name}.len() > {length} {{ fail!("{cause}") }}"#);
        }
        _ => unimplemented!(),
    }
}

fn function<F>(name: &str, ret_ty: &str, body: F)
where
    F: FnOnce(),
{
    println!("fn {name}(input: &[u8]) -> IResult<{ret_ty}> {{");
    body();
    println!("}}");
}
