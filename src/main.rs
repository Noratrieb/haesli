#![warn(rust_2018_idioms)]

use anyhow::Result;
use clap::Parser;
use tracing::{info, info_span, Instrument};

/// An AMQP 0-9-1 broker implementation.
#[derive(Parser)]
struct Args {
    /// Whether to serve the dashboard on localhost. Port defaults to 3000.
    #[clap(short, long)]
    dashboard: bool,
    /// The log level of the application. Overwrites the `RUST_LOG` env var.
    #[clap(long)]
    log_level: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_tracing(&args);

    let global_data = amqp_core::GlobalData::default();

    if args.dashboard {
        let dashboard_span = info_span!("dashboard");
        tokio::spawn(amqp_dashboard::dashboard(global_data.clone()).instrument(dashboard_span));
    }

    amqp_transport::do_thing_i_guess(global_data).await
}

fn setup_tracing(args: &Args) {
    const DEFAULT_LOG: &str = "hyper=info,debug";

    let log_filter = args
        .log_level
        .clone()
        .or_else(|| std::env::var("RUST_LOG").ok())
        .unwrap_or_else(|| DEFAULT_LOG.to_owned());

    tracing_subscriber::fmt()
        .with_level(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_ansi(true)
        .with_thread_names(true)
        .with_env_filter(&log_filter)
        .init();

    info!(%log_filter, "Using log filter level");
}
