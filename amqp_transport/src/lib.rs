#![warn(rust_2018_idioms)]

mod connection;
mod error;
mod frame;
mod methods;
mod sasl;
#[cfg(test)]
mod tests;

// TODO: handle big types

use std::{future::Future, net::SocketAddr};

use amqp_core::{connection::ConnectionEvent, queue::QueueEvent, GlobalData};
use anyhow::Context;
use tokio::{net, net::TcpStream, select};
use tracing::{info, info_span, Instrument};

use crate::connection::TransportConnection;

pub async fn do_thing_i_guess(
    global_data: GlobalData,
    terminate: impl Future + Send,
) -> anyhow::Result<()> {
    select! {
        res = accept_cons(global_data.clone()) => {
            res
        }
        _ = terminate => {
            handle_shutdown(global_data).await
        }
    }
}

async fn accept_cons(global_data: GlobalData) -> anyhow::Result<()> {
    info!("Binding TCP listener...");
    let listener = net::TcpListener::bind(("127.0.0.1", 5672)).await?;
    info!(addr = ?listener.local_addr()?, "Successfully bound TCP listener");

    loop {
        let connection = listener.accept().await?;
        handle_con(global_data.clone(), connection);
    }
}

fn handle_con(global_data: GlobalData, connection: (TcpStream, SocketAddr)) {
    let (stream, peer_addr) = connection;
    let id = rand::random();

    info!(local_addr = ?stream.local_addr(), %id, "Accepted new connection");
    let span = info_span!("client-connection", %id);

    let (method_send, method_recv) = tokio::sync::mpsc::channel(10);

    let connection_handle = amqp_core::connection::ConnectionInner::new(
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

async fn handle_shutdown(global_data: GlobalData) -> anyhow::Result<()> {
    info!("Shutting down...");

    let lock = global_data.lock();

    for con in lock.connections.values() {
        con.event_sender
            .try_send(ConnectionEvent::Shutdown)
            .context("failed to stop connection")?;
    }

    for queue in lock.queues.values() {
        queue
            .event_send
            .try_send(QueueEvent::Shutdown)
            .context("failed to stop queue worker")?;
    }

    // todo: here we should wait for everything to close
    // https://github.com/tokio-rs/mini-redis/blob/4b4ecf0310e6bca43d336dde90a06d9dcad00d6c/src/server.rs#L51

    info!("Finished shutdown");

    Ok(())
}
