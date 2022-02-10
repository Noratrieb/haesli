use crate::project_root;
use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn main() -> Result<()> {
    let test_js_root = project_root().join("test-js");
    let status = Command::new("yarn")
        .current_dir(&test_js_root)
        .status()
        .context("yarn install tests")?;
    if !status.success() {
        bail!("yarn install failed");
    }
    let status = Command::new("yarn")
        .arg("test")
        .current_dir(&test_js_root)
        .status()
        .context("yarn test tests")?;
    if !status.success() {
        bail!("yarn tests failed");
    }

    Ok(())
}
