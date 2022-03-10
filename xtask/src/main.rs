use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{ensure, Context, Result};

mod check_fmt;
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
    /// Check the formatting
    CheckFmt,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    match args.command {
        Commands::Generate => codegen::main(),
        Commands::TestJs => test_js::main(),
        Commands::Fmt => fmt::main(),
        Commands::CheckFmt => check_fmt::main(),
    }
}

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("project root path")
        .to_path_buf()
}

pub fn yarn_install(path: &Path) -> Result<()> {
    let status = Command::new("yarn")
        .arg("install")
        .current_dir(path)
        .status()
        .context("run yarn install failed")?;

    ensure!(status.success(), "Failed to build frontend");
    Ok(())
}
