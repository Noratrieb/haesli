use crate::codegen::{Amqp, Codegen};
use heck::ToUpperCamelCase;

impl Codegen {
    pub fn codegen_write(&mut self, amqp: &Amqp) {
        writeln!(
            self.output,
            "pub mod write {{
use amqp_core::methods::*;
use crate::error::TransError;
use crate::methods::write_helper::*;
use std::io::Write;

pub fn write_method<W: Write>(method: Method, mut writer: W) -> Result<(), TransError> {{
    match method {{"
        )
        .ok();

        for class in &amqp.classes {
            let class_name = class.name.to_upper_camel_case();
            let class_index = class.index;
            for method in &class.methods {
                let method_name = method.name.to_upper_camel_case();
                let method_index = method.index;
                writeln!(
                    self.output,
                    "        Method::{class_name}{method_name}({class_name}{method_name} {{"
                )
                .ok();
                for field in &method.fields {
                    let field_name = self.snake_case(&field.name);
                    writeln!(self.output, "            {field_name},").ok();
                }
                writeln!(self.output, "        }}) => {{").ok();
                let [ci0, ci1] = class_index.to_be_bytes();
                let [mi0, mi1] = method_index.to_be_bytes();
                writeln!(
                    self.output,
                    "            writer.write_all(&[{ci0}, {ci1}, {mi0}, {mi1}])?;"
                )
                .ok();
                let mut iter = method.fields.iter().peekable();

                while let Some(field) = iter.next() {
                    let field_name = self.snake_case(&field.name);
                    let type_name = self.resolve_type_from_domain(amqp, self.field_type(field));
                    if type_name == "bit" {
                        let fields_with_bit = self.subsequent_bit_fields(field, &mut iter, amqp);
                        write!(self.output, "            bit(&[").ok();
                        for field in fields_with_bit {
                            let field_name = self.snake_case(&field.name);
                            write!(self.output, "{field_name}, ").ok();
                        }
                        writeln!(self.output, "], &mut writer)?;").ok();
                    } else {
                        writeln!(
                            self.output,
                            "            {type_name}({field_name}, &mut writer)?;"
                        )
                        .ok();
                    }
                }
                writeln!(self.output, "        }}").ok();
            }
        }

        writeln!(
            self.output,
            "    }}
    Ok(())
}}
}}"
        )
        .ok();
    }
}
