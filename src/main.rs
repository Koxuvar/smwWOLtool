use tracing::Level;
use anyhow::Error;

mod machine;
mod wake_on_lan;
mod server;
mod client;


#[tokio::main]
async fn main() -> Result<(), Error> {    
    
    //Setup Logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();


    //start the server
    let addr = "127.0.0.1:9876".parse()?;
    let server = server::MachineServer::new(addr).await?;
    server.run().await

}

