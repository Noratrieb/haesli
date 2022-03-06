use crate::project_root;
use anyhow::ensure;
use std::process::Command;

pub fn main() -> anyhow::Result<()> {
    println!("$ cargo +nightly fmt --check");
    let status = Command::new("cargo")
        .arg("+nightly")
        .arg("fmt")
        .arg("--check")
        .current_dir(project_root())
        .status()?;
    ensure!(
        status.success(),
        "`cargo +nightly fmt --check` did not exit successfully"
    );

    println!("$ yarn");
    let status = Command::new("yarn")
        .arg("check-fmt")
        .current_dir(project_root().join("test-js"))
        .status()?;
    ensure!(status.success(), "`yarn fmt` did not exist successfully");

    println!("$ yarn");
    let status = Command::new("yarn")
        .arg("check-fmt")
        .current_dir(project_root().join("amqp_dashboard/frontend"))
        .status()?;
    ensure!(status.success(), "`yarn fmt` did not exist successfully");

    ensure!(status.success(), "`prettier .` did not exist successfully");

    Ok(())
}
