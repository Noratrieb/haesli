use crate::{field_type, resolve_type_from_domain, snake_case, Amqp};
use heck::ToUpperCamelCase;

pub(crate) fn codegen_write(amqp: &Amqp) {
    println!(
        "mod write {{
use super::*;
use crate::classes::write_helper::*;
use crate::error::TransError;
use std::io::Write;

pub fn write_method<W: Write>(class: Class, mut writer: W) -> Result<(), TransError> {{
    match class {{"
    );

    for class in &amqp.classes {
        let class_name = class.name.to_upper_camel_case();
        let class_index = class.index;
        for method in &class.methods {
            let method_name = method.name.to_upper_camel_case();
            let method_index = method.index;
            println!("        Class::{class_name}({class_name}::{method_name} {{");
            for field in &method.fields {
                let field_name = snake_case(&field.name);
                println!("            {field_name},");
            }
            println!("        }}) => {{");
            println!("            writer.write_all(&[{class_index}, {method_index}])?;");
            for field in &method.fields {
                let field_name = snake_case(&field.name);
                let field_type = resolve_type_from_domain(amqp, field_type(field));
                if field_type == "bit" {
                    println!("            todo!();");
                } else {
                    println!("            {field_type}({field_name}, &mut writer)?;");
                }
            }
            println!("        }}");
        }
    }

    println!(
        "    }}
    Ok(())
}}
}}"
    );
}
