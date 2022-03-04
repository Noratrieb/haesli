#![warn(rust_2018_idioms)]

mod connection;
mod error;
mod frame;
pub mod methods;
mod sasl;
#[cfg(test)]
mod tests;

// TODO: handle big types

use crate::connection::TransportConnection;
use amqp_core::GlobalData;
use anyhow::Result;
use tokio::net;
use tracing::{info, info_span, Instrument};

pub async fn do_thing_i_guess(global_data: GlobalData) -> Result<()> {
    info!("Binding TCP listener...");
    let listener = net::TcpListener::bind(("127.0.0.1", 5672)).await?;
    info!(addr = ?listener.local_addr()?, "Successfully bound TCP listener");

    loop {
        let (stream, peer_addr) = listener.accept().await?;

        let id = rand::random();

        info!(local_addr = ?stream.local_addr(), %id, "Accepted new connection");
        let span = info_span!("client-connection", %id);

        let (method_send, method_recv) = tokio::sync::mpsc::channel(10);

        let connection_handle = amqp_core::connection::ConnectionInner::new_handle(
            id,
            peer_addr,
            global_data.clone(),
            method_send.clone(),
        );

        let mut global_data_guard = global_data.lock();
        global_data_guard
            .connections
            .insert(id, connection_handle.clone());

        let connection = TransportConnection::new(
            id,
            stream,
            connection_handle,
            global_data.clone(),
            method_send,
            method_recv,
        );

        tokio::spawn(connection.start_connection_processing().instrument(span));
    }
}
