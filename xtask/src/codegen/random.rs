use crate::codegen::{Amqp, Codegen};
use heck::ToUpperCamelCase;

impl Codegen {
    pub fn codegen_random(&mut self, amqp: &Amqp) {
        writeln!(
            self.output,
            "
mod random {{
use rand::Rng;
use amqp_core::methods::*;
use crate::methods::RandomMethod;
"
        )
        .ok();

        writeln!(
            self.output,
            "impl<R: Rng> RandomMethod<R> for Method {{
    #[allow(unused_variables)]
    fn random(rng: &mut R) -> Self {{"
        )
        .ok();

        let class_lens = amqp.classes.len();
        writeln!(
            self.output,
            "        match rng.gen_range(0u32..{class_lens}) {{"
        )
        .ok();
        for (i, class) in amqp.classes.iter().enumerate() {
            let class_name = class.name.to_upper_camel_case();
            writeln!(self.output, "            {i} => {{").ok();

            let method_len = class.methods.len();
            writeln!(
                self.output,
                "                match rng.gen_range(0u32..{method_len}) {{"
            )
            .ok();

            for (i, method) in class.methods.iter().enumerate() {
                let method_name = method.name.to_upper_camel_case();
                writeln!(
                    self.output,
                    "                    {i} => Method::{class_name}{method_name} {{"
                )
                .ok();
                for field in &method.fields {
                    let field_name = self.snake_case(&field.name);
                    writeln!(
                        self.output,
                        "                        {field_name}: RandomMethod::random(rng),"
                    )
                    .ok();
                }
                writeln!(self.output, "                    }},").ok();
            }
            writeln!(
                self.output,
                "                    _ => unreachable!(),
                }}"
            )
            .ok();

            writeln!(self.output, "            }}").ok();
        }
        writeln!(
            self.output,
            "            _ => unreachable!(),
        }}"
        )
        .ok();
        writeln!(self.output, "    }}\n}}").ok();

        writeln!(self.output, "}}").ok();
    }
}
