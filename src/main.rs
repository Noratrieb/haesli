#![warn(rust_2018_idioms)]

use anyhow::Result;
use std::env;
use tracing::Level;
use tracing::{info_span, Instrument};

#[tokio::main]
async fn main() -> Result<()> {
    let mut dashboard = false;
    let mut level = Level::INFO;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--debug" => level = Level::DEBUG,
            "--trace" => level = Level::TRACE,
            "--dashboard" => dashboard = true,
            "ignore-this-clippy" => eprintln!("yes please"),
            _ => {}
        }
    }

    setup_tracing(level);

    let global_data = amqp_core::GlobalData::default();

    if dashboard {
        let dashboard_span = info_span!("dashboard");
        tokio::spawn(amqp_dashboard::dashboard(global_data.clone()).instrument(dashboard_span));
    }

    amqp_transport::do_thing_i_guess(global_data).await
}

fn setup_tracing(level: Level) {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_ansi(true)
        .with_thread_names(true)
        .with_max_level(level)
        .init()
}
