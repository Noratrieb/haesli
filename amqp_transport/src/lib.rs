#![allow(dead_code)]

mod connection;
mod error;
mod frame;

use crate::connection::Connection;
use anyhow::Result;
use tokio::net;
use tracing::info;

pub async fn do_thing_i_guess() -> Result<()> {
    info!("Binding TCP listener...");
    let listener = net::TcpListener::bind(("127.0.0.1", 5672)).await?;
    info!(addr = ?listener.local_addr()?, "Successfully bound TCP listener");

    loop {
        let (stream, _) = listener.accept().await?;

        info!(local_addr = ?stream.local_addr(), "Accepted new connection");

        let connection = Connection::new(stream);

        tokio::spawn(connection.start());
    }
}
