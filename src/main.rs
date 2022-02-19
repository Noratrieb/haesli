use anyhow::Result;
use std::env;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    let mut level = Level::DEBUG;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--trace" => level = Level::TRACE,
            "ignore-this-clippy" => eprintln!("yes please"),
            _ => {}
        }
    }

    setup_tracing(level);
    amqp_transport::do_thing_i_guess().await
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
