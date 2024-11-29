use tokio::sync::Mutex;
use tokio::net::{TcpStream, TcpListener};
use std::sync::Arc;
use crate::wake_on_lan::{WakeOnLan, WolError};
use uuid::Uuid;
use crate::machine::Machine;
use std::collections::HashMap;
use std::net::SocketAddr;
use mac_address::MacAddress;
use tracing::{info, error};

pub struct MachineServer {
    listener: TcpListener,
    machines: Arc<Mutex<HashMap<Uuid, Machine>>>,
}

impl MachineServer {
    pub async fn new(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr).await?;
        
        info!("Server listening on {}", addr);

        Ok(Self {
            listener,
            machines: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            
            // Clone the machines Arc for the handler
            let machines_clone = Arc::clone(&self.machines);
            
            // Spawn a new task to handle the connection
            tokio::spawn(async move {
                if let Err(e) = self.handle_connection(socket, machines_clone).await {
                    error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_connection(
        &self, 
        mut socket: TcpStream, 
        machines: Arc<Mutex<HashMap<Uuid, Machine>>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Connection handling logic
        // TODO: Implement protocol for registering machines, sending wake commands, etc.
        Ok(())
    }

    pub async fn register_machine(
        &self, 
        name: String, 
        mac_address: MacAddress, 
        ip_address: Option<String>
    ) -> Uuid {
        let machine = Machine::new(name, mac_address, ip_address);
        let machine_id = machine.id;

        let mut machines = self.machines.lock().await;
        machines.insert(machine_id, machine);

        machine_id
    }

    pub async fn wake_machine(&self, machine_id: Uuid) -> Result<(), WolError> {
        let machines = self.machines.lock().await;
        if let Some(machine) = machines.get(&machine_id) {
            WakeOnLan::send_magic_packet(machine.mac_address).await
        } else {
            Err(WolError::InvalidMacAddress)
        }
    }
}

