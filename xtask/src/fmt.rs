use crate::project_root;
use anyhow::ensure;
use std::process::Command;

pub fn main() -> anyhow::Result<()> {
    println!("$ cargo fmt");
    let status = Command::new("cargo")
        .arg("fmt")
        .current_dir(project_root())
        .status()?;
    ensure!(status.success(), "`cargo fmt` did not exit successfully");

    println!("$ yarn fmt");
    let status = Command::new("yarn")
        .arg("fmt")
        .current_dir(project_root().join("test-js"))
        .status()?;
    ensure!(status.success(), "`yarn fmt` did not exist successfully");

    println!("$ prettier -w .");
    let status = Command::new("prettier")
        .arg("-w")
        .arg(".")
        .current_dir(project_root().join("amqp_dashboard/assets"))
        .status()?;
    ensure!(status.success(), "`prettier .` did not exist successfully");

    Ok(())
}
