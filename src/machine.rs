use uuid::Uuid;
use macaddr::MacAddr6;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: Uuid,
    pub name: String,
    pub mac_address: MacAddr6,
    pub ip_address: Option<String>, 
    pub registered_at: SystemTime,
}

impl Machine {
    pub fn new(name: String, mac_address: MacAddr6 , ip_address: Option<String>) -> Self {
        Self{
            id: Uuid::new_v4(),
            name,
            mac_address,
            ip_address,
            registered_at: SystemTime::now(),

        }
    }
}
