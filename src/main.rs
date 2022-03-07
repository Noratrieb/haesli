#![warn(rust_2018_idioms)]

use anyhow::Result;
use clap::Parser;
use std::str::FromStr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// An AMQP 0-9-1 broker implementation.
#[derive(Parser)]
struct Args {
    /// Whether to serve the dashboard on localhost. Port defaults to 3000.
    #[clap(short, long)]
    dashboard: bool,

    /// Displays logs in a flat structure, otherwise as a tree
    #[clap(long)]
    flat_log: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_tracing(&args);

    let global_data = amqp_core::GlobalData::default();

    if args.dashboard {
        let global_data = global_data.clone();
        tokio::spawn(async move { amqp_dashboard::start_dashboard(global_data).await });
    }

    let res = amqp_transport::do_thing_i_guess(global_data, terminate()).await;

    info!("Bye!");

    res
}

fn setup_tracing(args: &Args) {
    const DEFAULT_LOG: &str = "hyper=info,debug"; // set hyper to info because I really don't care about hyper

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG.to_owned());

    let registry = Registry::default().with(EnvFilter::from_str(&log_filter).unwrap());

    if args.flat_log {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_level(true)
            .with_timer(tracing_subscriber::fmt::time::time())
            .with_ansi(true)
            .with_thread_names(true);

        registry.with(fmt_layer).init();
    } else {
        let tree_layer = tracing_tree::HierarchicalLayer::new(2)
            .with_targets(true)
            .with_bracketed_fields(true);

        registry.with(tree_layer).init();
    };

    info!(%log_filter, "Using log filter level");
}

async fn terminate() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c signal handler");
}
