use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    amqp_transport::do_thing_i_guess().await
}
