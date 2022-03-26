#![warn(rust_2018_idioms)]

mod archive;

use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, get_service},
    Json, Router,
};
use haesli_core::{exchange::ExchangeType, GlobalData};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use crate::archive::StaticFileService;

const DATA_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/frontend.zip"));

pub async fn start_dashboard(global_data: GlobalData) {
    match dashboard(global_data).await {
        Ok(()) => {}
        Err(err) => error!(%err, "Failed to start dashboard"),
    }
}

#[tracing::instrument(skip(global_data))]
pub async fn dashboard(global_data: GlobalData) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_origin(Any);

    let static_file_service =
        tokio::task::spawn_blocking(|| StaticFileService::new(DATA_ZIP)).await??;

    let static_file_service = get_service(static_file_service).handle_error(|error| async move {
        error!(?error, "Error in static file service");
        StatusCode::INTERNAL_SERVER_ERROR
    });

    let app = Router::new()
        .route("/api/data", get(move || get_data(global_data)).layer(cors))
        .fallback(static_file_service);

    let socket_addr = "0.0.0.0:8080".parse().unwrap();

    info!(%socket_addr, "Starting up dashboard on address");

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[derive(Serialize)]
struct Data {
    connections: Vec<Connection>,
    queues: Vec<Queue>,
    exchanges: Vec<Exchange>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Connection {
    id: String,
    peer_addr: String,
    channels: Vec<Channel>,
}

#[derive(Serialize)]
struct Channel {
    id: String,
    number: u16,
}

#[derive(Serialize)]
struct Queue {
    id: String,
    name: String,
    durable: bool,
    messages: usize,
    consumers: Vec<Consumer>,
}

#[derive(Serialize)]
struct Consumer {
    tag: String,
    channel: String,
}

#[derive(Serialize)]
struct Exchange {
    name: String,
    durable: bool,
    bindings: Vec<Binding>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Binding {
    queue: String,
    routing_key: String,
}

async fn get_data(global_data: GlobalData) -> impl IntoResponse {
    let global_data = global_data.lock();

    let connections = global_data
        .connections
        .values()
        .map(|conn| Connection {
            id: conn.id.to_string(),
            peer_addr: conn.peer_addr.to_string(),
            channels: conn
                .channels
                .lock()
                .values()
                .map(|chan| Channel {
                    id: chan.id.to_string(),
                    number: chan.num.num(),
                })
                .collect(),
        })
        .collect();

    let queues = global_data
        .queues
        .values()
        .map(|queue| Queue {
            id: queue.id.to_string(),
            name: queue.name.to_string(),
            durable: queue.durable,
            messages: queue.messages.len(),
            consumers: queue
                .consumers
                .lock()
                .values()
                .map(|consumer| Consumer {
                    tag: consumer.tag.to_string(),
                    channel: consumer.channel.id.to_string(),
                })
                .collect(),
        })
        .collect();

    let exchanges = global_data.exchanges.values().map(map_exchange).collect();

    let data = Data {
        connections,
        queues,
        exchanges,
    };

    Json(data)
}

fn map_exchange(exch: &haesli_core::exchange::Exchange) -> Exchange {
    Exchange {
        name: exch.name.to_string(),
        durable: exch.durable,
        bindings: match &exch.kind {
            ExchangeType::Direct { bindings } => bindings
                .iter()
                .map(|(name, _)| Binding {
                    queue: name.clone(),
                    routing_key: name.clone(),
                })
                .collect(),
            ExchangeType::Fanout { bindings } => bindings
                .iter()
                .map(|q| Binding {
                    queue: q.name.to_string(),
                    routing_key: "".to_owned(),
                })
                .collect(),
            ExchangeType::Topic { bindings } => bindings
                .iter()
                .map(|(segs, q)| Binding {
                    queue: q.name.to_string(),
                    routing_key: segs
                        .iter()
                        .map(|seg| seg.to_string())
                        .collect::<Vec<_>>()
                        .join("."),
                })
                .collect(),
            ExchangeType::Headers => Vec::new(),
            ExchangeType::System => Vec::new(),
        },
    }
}
