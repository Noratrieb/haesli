use std::process::Command;

use anyhow::ensure;

use crate::{project_root, yarn_install};

pub fn main() -> anyhow::Result<()> {
    println!("$ cargo fmt");
    let status = Command::new("cargo")
        .arg("fmt")
        .current_dir(project_root())
        .status()?;
    ensure!(status.success(), "`cargo fmt` did not exit successfully");

    let test_js = project_root().join("test-js");
    yarn_install(&test_js)?;
    println!("$ yarn fmt");
    let status = Command::new("yarn")
        .arg("fmt")
        .current_dir(test_js)
        .status()?;
    ensure!(status.success(), "`yarn fmt` did not exist successfully");

    let frontend = project_root().join("haesli_dashboard/frontend");
    yarn_install(&frontend)?;
    println!("$ yarn fmt");
    let status = Command::new("yarn")
        .arg("fmt")
        .current_dir(frontend)
        .status()?;
    ensure!(status.success(), "`yarn fmt` did not exist successfully");

    Ok(())
}
