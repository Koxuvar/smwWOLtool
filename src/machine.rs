use uuid::Uuid;
use mac_address::MacAddress;
use std::time::SystemTime;

pub struct Machine {
    pub id: Uuid,
    pub name: String,
    pub mac_address: MacAddress,
    pub ip_address: Option<String>, 
    pub registered_at: SystemTime,
}

impl Machine {
    pub fn new(name: String, mac_address: MacAddress, ip_address: Option<String>) -> Self {
        Self{
            id: Uuid::new_v4(),
            name,
            mac_address,
            ip_address,
            registered_at: SystemTime::now(),

        }
    }
}
