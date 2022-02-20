use super::{Amqp, Assert, Class, Domain, Method};
use crate::codegen::Codegen;
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

impl Codegen {
    pub(super) fn codegen_parser(&mut self, amqp: &Amqp) {
        writeln!(
            self.output,
            "pub mod parse {{
use amqp_core::methods::*;
use crate::methods::parse_helper::*;
use crate::error::TransError;
use nom::{{branch::alt, bytes::complete::tag}};
use regex::Regex;
use once_cell::sync::Lazy;

pub type IResult<'a, T> = nom::IResult<&'a [u8], T, TransError>;
"
        )
        .ok();
        writeln!(
            self.output,
            "pub fn parse_method(input: &[u8]) -> Result<(&[u8], Method), nom::Err<TransError>> {{
    alt(({}))(input)
}}",
            amqp.classes
                .iter()
                .map(|class| class.name.to_snake_case())
                .join(", ")
        )
        .ok();

        for domain in &amqp.domains {
            self.domain_parser(domain);
        }

        for class in &amqp.classes {
            let class_name = class.name.to_snake_case();

            self.function(&class_name, "Method");

            let class_index = class.index;
            let all_methods = class
                .methods
                .iter()
                .map(method_function_name(&class_name))
                .join(", ");
            let class_name_raw = &class.name;
            writeln!(
                self.output,
                r#"    let (input, _) = tag({class_index}_u16.to_be_bytes())(input)?;
    alt(({all_methods}))(input).map_err(fail_err("class {class_name_raw}"))"#
            )
            .ok();

            writeln!(self.output, "}}").ok();

            for method in &class.methods {
                self.method_parser(class, method, amqp);
            }
        }

        writeln!(self.output, "\n}}").ok();
    }

    fn domain_parser(&mut self, domain: &Domain) {
        let fn_name = domain_function_name(&domain.name);
        let type_name = domain.kind.to_snake_case();
        // don't even bother with bit domains, do them manually at call site
        if type_name != "bit" {
            self.function(&fn_name, &domain.name.to_upper_camel_case());

            if domain.asserts.is_empty() {
                writeln!(self.output, "    {type_name}(input)").ok();
            } else {
                writeln!(
                    self.output,
                    "    let (input, result) = {type_name}(input)?;"
                )
                .ok();

                for assert in &domain.asserts {
                    // channel.close requires a reply code, but there exists no reply code for
                    // a regular shutdown, and pythons `pika` just sends 0, even though the spec
                    // technically says that reply-code must be nonnull. Ignore that here.
                    if domain.name != "reply-code" {
                        self.assert_check(assert, &type_name, "result");
                    }
                }
                writeln!(self.output, "    Ok((input, result))").ok();
            }

            writeln!(self.output, "}}").ok();
        }
    }

    fn method_parser(&mut self, class: &Class, method: &Method, amqp: &Amqp) {
        let class_name = class.name.to_snake_case();
        let method_name_raw = &method.name;

        let function_name = method_function_name(&class_name)(method);
        self.function(&function_name, "Method");
        let method_index = method.index;
        writeln!(
            self.output,
            r#"    let (input, _) = tag({method_index}_u16.to_be_bytes())(input)?;"#
        )
        .ok();
        let mut iter = method.fields.iter().peekable();
        while let Some(field) = iter.next() {
            let field_name_raw = &field.name;
            let type_name = self.resolve_type_from_domain(amqp, self.field_type(field));

            if type_name == "bit" {
                let fields_with_bit = self.subsequent_bit_fields(field, &mut iter, amqp);

                let amount = fields_with_bit.len();
                writeln!(
                    self.output,
                    r#"    let (input, bits) = bit(input, {amount}).map_err(fail_err("field {field_name_raw} in method {method_name_raw}"))?;"#
                ).ok();

                for (i, field) in fields_with_bit.iter().enumerate() {
                    let field_name = self.snake_case(&field.name);
                    writeln!(self.output, "    let {field_name} = bits[{i}];").ok();
                }
            } else {
                let fn_name = domain_function_name(self.field_type(field));
                let field_name = self.snake_case(&field.name);
                writeln!(
                    self.output,
                    r#"    let (input, {field_name}) = {fn_name}(input).map_err(fail_err("field {field_name_raw} in method {method_name_raw}"))?;"#
                ).ok();

                for assert in &field.asserts {
                    self.assert_check(assert, &type_name, &field_name);
                }
            }
        }
        let class_name = class_name.to_upper_camel_case();
        let method_name = method.name.to_upper_camel_case();
        writeln!(
            self.output,
            "    Ok((input, Method::{class_name}{method_name} {{"
        )
        .ok();
        for field in &method.fields {
            let field_name = self.snake_case(&field.name);
            writeln!(self.output, "        {field_name},").ok();
        }
        writeln!(self.output, "    }}))").ok();

        writeln!(self.output, "}}").ok();
    }

    fn assert_check(&mut self, assert: &Assert, type_name: &str, var_name: &str) {
        match &*assert.check {
            "notnull" => match type_name {
                "shortstr" | "longstr" => {
                    writeln!(
                        self.output,
                        r#"    if {var_name}.is_empty() {{ fail!("string was null for field {var_name}") }}"#
                    ).ok();
                }
                "short" => {
                    writeln!(
                        self.output,
                        r#"    if {var_name} == 0 {{ fail!("number was 0 for field {var_name}") }}"#
                    )
                    .ok();
                }
                _ => unimplemented!(),
            },
            "regexp" => {
                let value = assert.value.as_ref().unwrap();
                writeln!(
                    self.output,
                    r#"    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"{value}").unwrap());"#
                ).ok();
                let cause = format!("regex `{value}` did not match value for field {var_name}");
                writeln!(
                    self.output,
                    r#"    if !REGEX.is_match(&{var_name}) {{ fail!(r"{cause}") }}"#
                )
                .ok();
            }
            "le" => {} // can't validate this here
            "length" => {
                let length = assert.value.as_ref().unwrap();
                let cause = format!("value is shorter than {length} for field {var_name}");
                writeln!(
                    self.output,
                    r#"    if {var_name}.len() > {length} {{ fail!("{cause}") }}"#
                )
                .ok();
            }
            _ => unimplemented!(),
        }
    }

    fn function(&mut self, name: &str, ret_ty: &str) {
        writeln!(
            self.output,
            "fn {name}(input: &[u8]) -> IResult<'_, {ret_ty}> {{"
        )
        .ok();
    }
}
