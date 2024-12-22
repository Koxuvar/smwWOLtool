use tokio::net::TcpStream;
use std::net::SocketAddr;
use serde::{Serialize, Deserialize};
use macaddr::MacAddr6;
use uuid::Uuid;
use anyhow::Error;

#[derive(Serialize, Deserialize)]
enum ClientMessage {
    RegisterMachine {
        name: String,
        mac_address: MacAddr6,
    },
    WakeMachine {
        machine_id: Uuid,
    },
}

pub struct RemoteMachineClient {
    server_addr: SocketAddr,
}

impl RemoteMachineClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }

    pub async fn connect(&self) -> Result<TcpStream, Error> {
        Ok(TcpStream::connect(self.server_addr).await?)
    }
}
