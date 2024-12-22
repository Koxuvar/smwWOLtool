use crate::machine::Machine;
use crate::wake_on_lan::WakeOnLan;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

pub struct MachineServer {
    listener: TcpListener,
    machines: Arc<Mutex<HashMap<Uuid, Machine>>>,
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

    pub async fn run(self) -> Result<(), anyhow::Error> {
        let listener = self.listener;
        loop {
            let machines = Arc::clone(&self.machines);
            let (socket, _) = listener.accept().await?;
            tokio::spawn(async move {
                handle_connection(socket, &machines).await;
            });
        }
    }
}

#[derive(Serialize, Deserialize)]
enum ServerMessage {
    RegisterMachine {
        name: String,
        mac_address: MacAddr6,
    },
    WakeMachine {
        machine_id: Uuid,
    },
    ListMachines,
    Response {
        success: bool,
        message: String,
        data: Option<String>,
    },
}

async fn handle_connection(
    mut socket: TcpStream,
    machines: &Arc<Mutex<HashMap<Uuid, Machine>>>,
) -> Result<(), anyhow::Error> {
    // Connection handling logic

    let mut buffer = [0; 1024];

    let n = socket.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let message: ServerMessage = match serde_json::from_slice(&buffer[..n]) {
        Ok(msg) => msg,
        Err(_e) => {
            let response = ServerMessage::Response {
                success: false,
                message: "Invalid message format".to_string(),
                data: None,
            };
            socket.write(&serde_json::to_vec(&response)?).await?;
            return Ok(());
        }
    };

    let response = match message {
        ServerMessage::RegisterMachine { name, mac_address } => {
            // Parse MAC addres
            if !mac_address.is_nil() {
                ServerMessage::Response {
                    success: false,
                    message: "Mac address is not correct".to_string(),
                    data: None,
                }
            } else {

            // Register the machine
            let mut machines_lock = machines.lock().await;
            let machine = Machine::new(name, mac_address, None);
            let machine_id = machine.id;
            machines_lock.insert(machine_id, machine);

            ServerMessage::Response {
                success: true,
                message: "Machine registered successfully".to_string(),
                data: Some(machine_id.to_string()),
            }
            }
        }
        ServerMessage::WakeMachine { machine_id } => {
            let machines_lock = machines.lock().await;
            match machines_lock.get(&machine_id) {
                Some(machine) => {
                    // Send wake-on-lan packet
                    match WakeOnLan::send_magic_packet(machine.mac_address).await {
                        Ok(_) => ServerMessage::Response {
                            success: true,
                            message: "Wake packet sent".to_string(),
                            data: None,
                        },
                        Err(_) => ServerMessage::Response {
                            success: false,
                            message: "Failed to send wake packet".to_string(),
                            data: None,
                        },
                    }
                }
                None => ServerMessage::Response {
                    success: false,
                    message: "Machine not found".to_string(),
                    data: None,
                },
            }
        }
        ServerMessage::ListMachines => {
            let machines_lock = machines.lock().await;
            let machine_list: Vec<String> = machines_lock
                .values()
                .map(|machine| format!("{}: {}", machine.id, machine.name))
                .collect();

            ServerMessage::Response {
                success: true,
                message: "Machines listed".to_string(),
                data: Some(machine_list.join(", ")),
            }
        }
        _ => ServerMessage::Response {
                success: false,
                message: "nothing to do".to_string(),
                data:None,
        }
    };

    // Send response back to client
    socket.write_all(&serde_json::to_vec(&response)?).await?;
    socket.flush().await?;

    Ok(())
}
