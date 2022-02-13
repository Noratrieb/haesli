use crate::{field_type, resolve_type_from_domain, snake_case, subsequent_bit_fields, Amqp};
use heck::ToUpperCamelCase;

pub(crate) fn codegen_write(amqp: &Amqp) {
    println!(
        "pub mod write {{
use super::*;
use crate::classes::write_helper::*;
use crate::error::TransError;
use tokio::io::AsyncWriteExt;

pub async fn write_method<W: AsyncWriteExt + Unpin>(class: Class, mut writer: W) -> Result<(), TransError> {{
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
            println!("            writer.write_all(&[{class_index}, {method_index}]).await?;");
            let mut iter = method.fields.iter().peekable();

            while let Some(field) = iter.next() {
                let field_name = snake_case(&field.name);
                let type_name = resolve_type_from_domain(amqp, field_type(field));
                if type_name == "bit" {
                    let fields_with_bit = subsequent_bit_fields(amqp, field, &mut iter);
                    print!("            bit(&[");
                    for field in fields_with_bit {
                        let field_name = snake_case(&field.name);
                        print!("{field_name}, ");
                    }
                    println!("], &mut writer).await?;");
                } else {
                    println!("            {type_name}({field_name}, &mut writer).await?;");
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
