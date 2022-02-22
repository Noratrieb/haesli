use crate::project_root;
use anyhow::{ensure, Context, Result};
use std::process::Command;

pub fn main() -> Result<()> {
    let test_js_root = project_root().join("test-js");
    println!("$ yarn");
    let status = Command::new("yarn")
        .current_dir(&test_js_root)
        .status()
        .context("yarn install tests")?;
    ensure!(status.success(), "yarn install failed");
    println!("$ yarn test");
    let status = Command::new("yarn")
        .arg("test")
        .current_dir(&test_js_root)
        .status()
        .context("yarn test tests")?;
    ensure!(status.success(), "yarn tests failed");

    Ok(())
}
