use std::path::PathBuf;

mod codegen;
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
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    match args.command {
        Commands::Generate => codegen::main(),
        Commands::TestJs => test_js::main(),
    }
}

pub fn project_root() -> PathBuf {
    PathBuf::from(file!())
        .parent()
        .expect("src directory path")
        .parent()
        .expect("xtask root path")
        .parent()
        .expect("project root path")
        .to_path_buf()
}
