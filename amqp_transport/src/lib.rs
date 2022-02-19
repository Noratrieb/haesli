extern crate core;

mod classes;
mod connection;
mod error;
mod frame;
mod sasl;
#[cfg(test)]
mod tests;

use crate::connection::Connection;
use amqp_core::GlobalData;
use anyhow::Result;
use tokio::net;
use tracing::{info, info_span, Instrument};
use uuid::Uuid;

pub async fn do_thing_i_guess(global_data: GlobalData) -> Result<()> {
    info!("Binding TCP listener...");
    let listener = net::TcpListener::bind(("127.0.0.1", 5672)).await?;
    info!(addr = ?listener.local_addr()?, "Successfully bound TCP listener");

    loop {
        let (stream, peer_addr) = listener.accept().await?;

        let id = Uuid::from_bytes(rand::random());

        info!(local_addr = ?stream.local_addr(), %id, "Accepted new connection");
        let span = info_span!("client-connection", %id);

        let connection_handle =
            amqp_core::Connection::new_handle(id, peer_addr, global_data.clone());

        let mut global_data = global_data.lock();
        global_data
            .connections
            .insert(id, connection_handle.clone());

        let connection = Connection::new(stream, connection_handle);

        tokio::spawn(connection.start_connection_processing().instrument(span));
    }
}
