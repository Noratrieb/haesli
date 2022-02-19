extern crate core;

mod classes;
mod connection;
mod error;
mod frame;
mod sasl;
#[cfg(test)]
mod tests;

use crate::connection::Connection;
use anyhow::Result;
use tokio::net;
use tracing::{info, info_span, Instrument};
use uuid::Uuid;

pub async fn do_thing_i_guess() -> Result<()> {
    info!("Binding TCP listener...");
    let listener = net::TcpListener::bind(("127.0.0.1", 5672)).await?;
    info!(addr = ?listener.local_addr()?, "Successfully bound TCP listener");

    loop {
        let (stream, _) = listener.accept().await?;

        let id = Uuid::from_bytes(rand::random());

        info!(local_addr = ?stream.local_addr(), %id, "Accepted new connection");
        let span = info_span!("client-connection", %id);

        let connection = Connection::new(stream);

        tokio::spawn(connection.start_connection_processing().instrument(span));
    }
}
