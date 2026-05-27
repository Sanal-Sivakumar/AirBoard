use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};
use crate::core::sync_engine::engine::SYNC_ENGINE;
use crate::core::connection_registry::add_or_update_peer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceAnnouncement {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub device_name: String,
    pub device_id: String,
    pub platform: String,
    pub ws_port: u16,
}

pub async fn start_udp_announcer(device_name: String, platform: String, ws_port: u16) {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind UDP announcer: {}", e);
            return;
        }
    };

    if let Err(e) = socket.set_broadcast(true) {
        eprintln!("Failed to set UDP broadcast: {}", e);
        return;
    }

    let broadcast_addr: SocketAddr = "255.255.255.255:45454".parse().unwrap();
    let local_device_id = SYNC_ENGINE.device_id.clone();

    loop {
        let announcement = DeviceAnnouncement {
            msg_type: "device_announcement".to_string(),
            device_name: device_name.clone(),
            device_id: local_device_id.clone(),
            platform: platform.clone(),
            ws_port,
        };

        if let Ok(json_str) = serde_json::to_string(&announcement) {
            let _ = socket.send_to(json_str.as_bytes(), broadcast_addr).await;
        }

        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn start_udp_listener() {
    let socket = match UdpSocket::bind("0.0.0.0:45454").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind UDP listener: {}", e);
            return;
        }
    };

    let mut buf = [0u8; 1024];
    let local_device_id = SYNC_ENGINE.device_id.clone();

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src_addr)) => {
                let data = &buf[..len];
                if let Ok(announcement) = serde_json::from_slice::<DeviceAnnouncement>(data) {
                    if announcement.device_id != local_device_id {
                        let ip_str = src_addr.ip().to_string();
                        add_or_update_peer(
                            announcement.device_id,
                            announcement.device_name,
                            ip_str,
                            announcement.ws_port,
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("UDP listener error: {}", e);
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
}
