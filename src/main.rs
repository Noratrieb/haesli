#![warn(rust_2018_idioms)]

use anyhow::Result;
use std::env;
use tracing::{info_span, Instrument};

#[tokio::main]
async fn main() -> Result<()> {
    let mut dashboard = false;
    let mut console = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--dashboard" => dashboard = true,
            "--console" => console = true,
            "ignore-this-clippy" => eprintln!("yes please"),
            _ => {}
        }
    }

    setup_tracing(console);

    let global_data = amqp_core::GlobalData::default();

    if dashboard {
        let dashboard_span = info_span!("dashboard");
        tokio::task::Builder::new()
            .name("dashboard")
            .spawn(amqp_dashboard::dashboard(global_data.clone()).instrument(dashboard_span));
    }

    amqp_transport::do_thing_i_guess(global_data).await
}

fn setup_tracing(console: bool) {
    if console {
        console_subscriber::init();
    } else {
        tracing_subscriber::fmt()
            .with_level(true)
            .with_timer(tracing_subscriber::fmt::time::time())
            .with_ansi(true)
            .with_thread_names(true)
            .with_env_filter(
                std::env::var("RUST_LOG")
                    .unwrap_or_else(|_| "hyper=info,tokio=trace,runtime=trace,debug".to_string()),
            )
            .init();
    }
}
