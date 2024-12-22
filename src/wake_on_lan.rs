use tokio::net::UdpSocket;
use macaddr::MacAddr6;
use tracing::info;
use anyhow::Error;

pub struct WakeOnLan;

impl WakeOnLan {
    pub async fn send_magic_packet(mac_address:MacAddr6) -> Result<(), Error> {
        let mut packet = vec![0xFF; 6];

        let mac_bytes = mac_address.as_bytes();
        for _ in 0..16 {
            packet.extend_from_slice(&mac_bytes);
        }

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        
        socket.send_to(
            &packet,
            "255.255.255.255:9"
            ).await?;

        info!("Wake-on-Lan packet sent to {}", mac_address);
        Ok(())
    }
}
