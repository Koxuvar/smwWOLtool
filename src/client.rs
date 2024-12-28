use anyhow::Error;
use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use uuid::Uuid;
use crate::server::Response;

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    RegisterMachine { name: String, mac_address: MacAddr6 },
    WakeMachine { machine_id: Uuid },
    ListMachines,
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

    pub async fn send_message(&self, message: ClientMessage) -> Result<Response, Error> {
        let stream = self.connect().await?;
        let message = serde_json::to_vec(&message)?;
        stream.try_write(&message)?;

        let mut buffer = [0; 1024];
        let n = stream.try_read(&mut buffer)?;
        let response: Response = serde_json::from_slice(&buffer[..n])?;

        Ok(response)
    }
}
