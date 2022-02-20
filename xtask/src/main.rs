mod codegen;

fn main() -> anyhow::Result<()> {
    let command = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Error: No task provided");
        help();
        std::process::exit(1);
    });

    match command.as_str() {
        "generate" | "gen" => codegen::main(),
        _ => {
            eprintln!("Unknown command {command}.");
            Ok(())
        }
    }
}

fn help() {
    println!(
        "Available tasks:
   generate, gen - Generate amqp method code in `amqp_transport/src/methods/generated.rs and amqp_core/src/methods/generated.rs"
    );
}
