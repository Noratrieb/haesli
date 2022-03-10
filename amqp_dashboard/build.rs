use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{ensure, Context, Result};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

// inspired by https://fasterthanli.me/series/dont-shell-out/part-8

fn main() -> Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let frontend_dir = PathBuf::from(manifest_dir).join("frontend");

    // this is not always fully correct, but we need to ignore `build` from the rerun or else it
    // will always trigger itself
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("src").display()
    );

    build_frontend(&frontend_dir)
}

fn build_frontend(path: &Path) -> Result<()> {
    let status = Command::new("yarn")
        .arg("install")
        .current_dir(path)
        .status()
        .context("run yarn install failed")?;

    ensure!(status.success(), "Failed to install frontend dependencies");

    let status = Command::new("yarn")
        .arg("build")
        .current_dir(path)
        .status()
        .context("run yarn build")?;

    ensure!(status.success(), "Failed to build frontend");

    let yarn_build_dir = path.join("build");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let zip_path = out_dir.join("frontend.zip");

    let mut zw = ZipWriter::new(File::create(&zip_path).unwrap());

    for entry in WalkDir::new(&yarn_build_dir) {
        let entry = entry.context("walk build directory")?;

        let disk_path = entry.path();
        let rel_path = disk_path
            .strip_prefix(&yarn_build_dir)
            .unwrap()
            .to_string_lossy();

        let meta = entry.metadata().context("entry metadata")?;

        if meta.is_dir() {
            zw.add_directory(rel_path, FileOptions::default()).unwrap();
        } else if meta.is_file() {
            zw.start_file(rel_path, Default::default()).unwrap();
            std::io::copy(&mut File::open(disk_path).unwrap(), &mut zw).unwrap();
        } else {
            println!("cargo:warning=Ignoring entry {}", disk_path.display());
        }
    }

    Ok(())
}
