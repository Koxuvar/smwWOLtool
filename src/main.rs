use anyhow::Error;
use tracing::Level;

mod client;
mod machine;
mod server;
mod wake_on_lan;

#[tokio::main]
async fn main() -> Result<(), Error> {
    //Setup Logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    //start the server
    let addr = "127.0.0.1:9876".parse()?;
    let server = server::MachineServer::new(addr).await?;

    server.listen().await;
    Ok(())
}
