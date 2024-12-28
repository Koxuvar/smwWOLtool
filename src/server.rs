use crate::machine::Machine;
use crate::wake_on_lan::WakeOnLan;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

pub struct MachineServer {
    listener: TcpListener,
    machines: Arc<Mutex<HashMap<Uuid, Machine>>>,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    RegisterMachine { name: String, mac_address: MacAddr6 },
    WakeMachine { machine_id: Uuid },
    ListMachines,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    success: bool,
    message: String,
    data: Option<String>,
}

impl MachineServer {
    pub async fn new(addr: SocketAddr) -> Result<Self, anyhow::Error> {
        let listener = TcpListener::bind(addr).await?;

        info!("Server listening on {}", addr);

        Ok(Self {
            listener,
            machines: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn register_machine(
        &self,
        name: String,
        mac_address: MacAddr6,
        ip_address: Option<String>,
    ) -> Uuid {
        let machine: Machine = Machine::new(name, mac_address, ip_address);
        let machine_id = machine.id;

        let mut machines = self.machines.lock().await;
        machines.insert(machine_id, machine);

        machine_id
    }

    pub async fn wake_machine(&self, machine_id: Uuid) -> Result<(), anyhow::Error> {
        let machines = self.machines.lock().await;
        if let Some(machine) = machines.get(&machine_id) {
            WakeOnLan::send_magic_packet(machine.mac_address).await
        } else {
            Err(anyhow::anyhow!("Cant Find that machine on the network!"))
        }
    }

    pub async fn list_machines(&self) -> Result<Vec<Machine>, anyhow::Error> {
        let machines = self.machines.lock().await;
        let machlist: Vec<Machine> = machines.values().cloned().collect();
        Ok(machlist)
    }

    pub async fn listen(&self) {
        let (mut socket, _) = self.listener.accept().await.unwrap();

        let mut buffer = [0; 1024];
        loop {
            let n = socket.read(&mut buffer).await.unwrap();
            if n == 0 {
               continue;
            }

            let message: ServerMessage = serde_json::from_slice(&buffer[..n]).expect("Failed to deserialize message"); 

            match message {
                ServerMessage::RegisterMachine { name, mac_address } => {
                    let machine_id = self.register_machine(name, mac_address, None).await;
                    let response = Response {
                        success: true,
                        message: "Registered machine".to_string(),
                        data: Some(machine_id.to_string()),
                    };

                    let response = serde_json::to_vec(&response).unwrap();
                    socket.write_all(&response).await.unwrap();
                }
                ServerMessage::WakeMachine { machine_id } => {
                    let wake_success = self.wake_machine(machine_id).await;
                    let response = match wake_success {
                        Ok(_) => Response {
                            success: true,
                            message: "Machine woken".to_string(),
                            data: None,
                        },
                        Err(e) => Response {
                            success: false,
                            message: e.to_string(),
                            data: None,
                        },
                    };

                    let response = serde_json::to_vec(&response).unwrap();
                    socket.write_all(&response).await.unwrap();
                }
                ServerMessage::ListMachines => {
                    let machines = self.list_machines().await.unwrap();
                    let response = Response {
                        success: true,
                        message: "Machines listed".to_string(),
                        data: Some(
                            serde_json::to_string(&machines)
                                .expect("Failed to serialize machine list"),
                        ),
                    };
                    let response = serde_json::to_vec(&response).unwrap();
                    socket.write_all(&response).await.unwrap();
                }
            }
        }

    }
}
