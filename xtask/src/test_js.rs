use crate::project_root;
use anyhow::{bail, Result};
use std::process::Command;

pub fn main() -> Result<()> {
    let test_js_root = project_root().join("tests-js");
    let status = Command::new("yarn").current_dir(&test_js_root).status()?;
    if !status.success() {
        bail!("yarn install failed");
    }
    let status = Command::new("yarn")
        .arg("test")
        .current_dir(&test_js_root)
        .status()?;
    if !status.success() {
        bail!("yarn tests failed");
    }

    Ok(())
}
