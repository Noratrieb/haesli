use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;

pub async fn do_thing_i_guess() -> Result<()> {
    tokio::io::stdout()
        .write(b"hello async world lol\n")
        .await
        .context("failed to write")
        .map(drop)
}
