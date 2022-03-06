#![warn(rust_2018_idioms)]

use amqp_core::GlobalData;
use axum::{
    body::{boxed, Full},
    http::Method,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

const INDEX_HTML: &str = include_str!("../assets/index.html");
const SCRIPT_JS: &str = include_str!("../assets/script.js");
const STYLE_CSS: &str = include_str!("../assets/style.css");

pub async fn dashboard(global_data: GlobalData) {
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(get_index_html))
        .route("/script.js", get(get_script_js))
        .route("/style.css", get(get_style_css))
        .route("/api/data", get(move || get_data(global_data)).layer(cors));

    let socket_addr = "0.0.0.0:8080".parse().unwrap();

    info!(%socket_addr, "Starting up dashboard on address");

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_index_html() -> impl IntoResponse {
    info!("Requesting index.html");
    Html(INDEX_HTML)
}

async fn get_script_js() -> Response {
    Response::builder()
        .header("content-type", "application/javascript")
        .body(boxed(Full::from(SCRIPT_JS)))
        .unwrap()
}

async fn get_style_css() -> Response {
    Response::builder()
        .header("content-type", "text/css")
        .body(boxed(Full::from(STYLE_CSS)))
        .unwrap()
}

#[derive(Serialize)]
struct Data {
    connections: Vec<Connection>,
    queues: Vec<Queue>,
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
        })
        .collect();

    let data = Data {
        connections,
        queues,
    };

    Json(data)
}
