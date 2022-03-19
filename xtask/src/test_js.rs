use std::{path::Path, process::Command, thread, time::Duration};

use anyhow::{ensure, Context, Result};

use crate::{project_root, yarn_install};

pub fn main() -> Result<()> {
    let project_root = project_root();
    let test_js_root = project_root.join("test-js");

    println!("$ cargo build");
    let status = Command::new("cargo")
        .arg("build")
        .status()
        .context("cargo build")?;
    ensure!(status.success(), "cargo build failed");

    let server_binary = project_root.join("target/debug/haesli");
    let mut server_process = Command::new(&server_binary)
        .env("RUST_LOG", "trace")
        .spawn()
        .context(server_binary.display().to_string())
        .context("run server binary")?;

    // give it time for startup
    thread::sleep(Duration::from_secs(1));

    let test_result = run_js(&test_js_root);

    server_process.kill().context("killing amqp server")?;

    test_result
}

fn run_js(test_js_root: &Path) -> Result<()> {
    yarn_install(test_js_root)?;

    println!("$ yarn test");
    let status = Command::new("yarn")
        .arg("test")
        .current_dir(&test_js_root)
        .status()
        .context("yarn test tests")?;
    ensure!(status.success(), "yarn tests failed");

    Ok(())
}
