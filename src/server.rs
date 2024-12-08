use crate::machine::Machine;
use crate::wake_on_lan::{WakeOnLan, WolError};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use mac_address::MacAddress;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use anyhow::Error;
use tracing::info;
use uuid::Uuid;

pub struct MachineServer {
    listener: TcpListener,
    machines: Arc<Mutex<HashMap<Uuid, Machine>>>,
}

impl MachineServer {
    pub async fn new(addr: SocketAddr) -> Result<Self, Error> {
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
        mac_address: MacAddress,
        ip_address: Option<String>,
    ) -> Uuid {
        let machine: Machine = Machine::new(name, mac_address, ip_address);
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

// impl MachineServer {
//  //     pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
//         let machines = Arc::clone(&self.machines);
//         let listener = self.listener;
//         loop {
//             let (socket, addr) = listener.accept().await?;
//
//             // Clone the machines Arc for the handler
//             let machines_clone = Arc::clone(&machines);
//
//             // Spawn a new task to handle the connection
//             tokio::spawn(async move {
//                 if let Err(e) = handle_connection(socket, machines_clone).await {
//                     error!("Error handling connection from {}: {}", addr, e);
//                 }
//             });
//         }
//     }
//
//    //
//     pub async fn wake_machine(&self, machine_id: Uuid) -> Result<(), WolError> {
//         let machines = self.machines.lock().await;
//         if let Some(machine) = machines.get(&machine_id) {
//             WakeOnLan::send_magic_packet(machine.mac_address).await
//         } else {
//             Err(WolError::InvalidMacAddress)
//         }
//     }
// }
//
// #[derive(Serialize, Deserialize)]
// enum ServerMessage {
//     RegisterMachine {
//         name: String,
//         mac_address: MacAddress,
//     },
//     WakeMachine {
//         machine_id: Uuid,
//     },
//     ListMachines,
//     Response {
//         success: bool,
//         message: String,
//         data: Option<String>,
//     },
// }
//
// async fn handle_connection(
//     mut socket: TcpStream,
//     machines: Arc<Mutex<HashMap<Uuid, Machine>>>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     // Connection handling logic
//
//     let mut buffer = [0; 1024];
//
//     let n = socket.read(&mut buffer).await?;
//     if n == 0 {
//         return Ok(());
//     }
//
//     let message: ServerMessage = match serde_json::from_slice(&buffer[..n]) {
//         Ok(msg) => msg,
//         Err(e) => {
//             error!("Failed to parse message: {}", e);
//             let response = ServerMessage::Response {
//                 success: false,
//                 message: "Invalid message format".to_string(),
//                 data: None,
//             };
//             socket.write_all(&serde_json::to_vec(&response)?).await?;
//             return Ok(());
//         }
//     };
//
//     let response = match message {
//         ServerMessage::RegisterMachine { name, mac_address } => {
//             // Parse MAC address
//             let mac = match mac_address.bytes().map(|x| x.to_string()){
//                 Ok(mac) => mac,
//                 Err(_) => ServerMessage::Response {
//                     success: false,
//                     message: "Invalid MAC address".to_string(),
//                     data: None,
//                 },
//             };
//
//             // Register the machine
//             let mut machines_lock = machines.lock().await;
//             let machine = Machine::new(name, mac, None);
//             let machine_id = machine.id;
//             machines_lock.insert(machine_id, machine);
//
//             ServerMessage::Response {
//                 success: true,
//                 message: "Machine registered successfully".to_string(),
//                 data: Some(machine_id.to_string()),
//             }
//         }
//         ServerMessage::WakeMachine { machine_id } => {
//             let machines_lock = machines.lock().await;
//             match machines_lock.get(&machine_id) {
//                 Some(machine) => {
//                     // Send wake-on-lan packet
//                     match WakeOnLan::send_magic_packet(machine.mac_address).await {
//                         Ok(_) => ServerMessage::Response {
//                             success: true,
//                             message: "Wake packet sent".to_string(),
//                             data: None,
//                         },
//                         Err(_) => ServerMessage::Response {
//                             success: false,
//                             message: "Failed to send wake packet".to_string(),
//                             data: None,
//                         },
//                     }
//                 }
//                 None => ServerMessage::Response {
//                     success: false,
//                     message: "Machine not found".to_string(),
//                     data: None,
//                 },
//             }
//         }
//         ServerMessage::ListMachines => {
//             let machines_lock = machines.lock().await;
//             let machine_list: Vec<String> = machines_lock
//                 .values()
//                 .map(|machine| format!("{}: {}", machine.id, machine.name))
//                 .collect();
//
//             ServerMessage::Response {
//                 success: true,
//                 message: "Machines listed".to_string(),
//                 data: Some(machine_list.join(", ")),
//             }
//         }
//     };
//
//     // Send response back to client
//     socket.write_all(&serde_json::to_vec(&response)?).await?;
//     socket.flush().await?;
//
//     Ok(())
// }
