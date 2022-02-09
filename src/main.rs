use anyhow::Result;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    amqp_transport::do_thing_i_guess().await
}

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_ansi(true)
        .with_thread_names(true)
        .with_max_level(Level::DEBUG)
        .init()
}
