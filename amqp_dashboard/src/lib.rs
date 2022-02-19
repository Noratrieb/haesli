use amqp_core::GlobalData;
use axum::body::{boxed, Full};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tracing::info;

const INDEX_HTML: &str = include_str!("../assets/index.html");
const SCRIPT_JS: &str = include_str!("../assets/script.js");
const STYLE_CSS: &str = include_str!("../assets/style.css");

pub async fn dashboard(global_data: GlobalData) {
    let app = Router::new()
        .route("/", get(get_index_html))
        .route("/script.js", get(get_script_js))
        .route("/style.css", get(get_style_css))
        .route("/api/data", get(move || get_data(global_data)));

    let socket_addr = "0.0.0.0:3000".parse().unwrap();

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
}

#[derive(Serialize)]
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

async fn get_data(global_data: GlobalData) -> impl IntoResponse {
    let global_data = global_data.lock();

    let connections = global_data
        .connections
        .values()
        .map(|conn| {
            let conn = conn.lock();
            Connection {
                id: conn.id.to_string(),
                peer_addr: conn.peer_addr.to_string(),
                channels: conn
                    .channels
                    .values()
                    .map(|chan| {
                        let chan = chan.lock();
                        Channel {
                            id: chan.id.to_string(),
                            number: chan.num,
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    let data = Data { connections };

    Json(data)
}
