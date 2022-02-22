use std::path::PathBuf;

mod codegen;
mod fmt;
mod test_js;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate method definitions/parser/writer
    Generate,
    /// Run Javascript integration tests
    TestJs,
    /// Format all code
    Fmt,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    match args.command {
        Commands::Generate => codegen::main(),
        Commands::TestJs => test_js::main(),
        Commands::Fmt => fmt::main(),
    }
}

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("project root path")
        .to_path_buf()
}
