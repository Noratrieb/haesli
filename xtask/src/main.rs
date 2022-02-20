mod codegen;

fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("No task provided");
        help();
        std::process::exit(1);
    });

    match command.as_str() {
        "generate" | "gen" => codegen::main(),
        _ => eprintln!("Unknown command {command}."),
    }
}

fn help() {
    println!(
        "Available tasks:
generate - Generate amqp method code in `amqp_transport/src/methods/generated.rs.
           Dumps code to stdout and should be redirected manually."
    );
}
