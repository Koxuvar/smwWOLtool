use tracing::Level;

mod machine;
mod wake_on_lan;
mod server;
mod client;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    
    //Setup Logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();


    //start the server
    let addr = "0.0.0.0:9876".parse()?;
    let server = server::MachineServer::new(addr).await?;
    server.run().await

}

