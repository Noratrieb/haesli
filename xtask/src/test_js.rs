use crate::project_root;
use anyhow::{ensure, Context, Result};
use std::{path::Path, process::Command};

pub fn main() -> Result<()> {
    let project_root = project_root();
    let test_js_root = project_root.join("test-js");

    let mut amqp_server = Command::new("cargo")
        .arg("run")
        .env("RUST_LOG", "trace")
        .spawn()
        .context("`cargo run` amqp")?;

    let test_result = run_js(&test_js_root);

    amqp_server.kill()?;

    test_result
}

fn run_js(test_js_root: &Path) -> Result<()> {
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
