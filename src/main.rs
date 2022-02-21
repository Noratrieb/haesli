#![warn(rust_2018_idioms)]

use anyhow::Result;
use std::env;
use tracing::{info, info_span, Instrument};

#[tokio::main]
async fn main() -> Result<()> {
    let mut dashboard = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--dashboard" => dashboard = true,
            "ignore-this-clippy" => eprintln!("yes please"),
            _ => {}
        }
    }

    setup_tracing();

    let global_data = amqp_core::GlobalData::default();

    if dashboard {
        let dashboard_span = info_span!("dashboard");
        tokio::spawn(amqp_dashboard::dashboard(global_data.clone()).instrument(dashboard_span));
    }

    amqp_transport::do_thing_i_guess(global_data).await
}

fn setup_tracing() {
    let rust_log = std::env::var("RUST_LOG");
    const DEFAULT_LOG: &str = "hyper=info,debug";

    tracing_subscriber::fmt()
        .with_level(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_ansi(true)
        .with_thread_names(true)
        .with_env_filter(rust_log.clone().unwrap_or_else(|_| DEFAULT_LOG.to_string()))
        .init();

    if let Ok(rust_log) = rust_log {
        info!(%rust_log, "Using custom log level");
    } else {
        info!(%DEFAULT_LOG, "Using default log level");
    }
}
