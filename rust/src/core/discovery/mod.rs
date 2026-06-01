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

fn get_local_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

fn get_broadcast_addresses() -> Vec<SocketAddr> {
    let mut addrs = vec!["255.255.255.255:45454".parse().unwrap()];
    if let Some(local_ip) = get_local_ip() {
        if let std::net::IpAddr::V4(ipv4) = local_ip {
            let octets = ipv4.octets();
            if !ipv4.is_loopback() && !ipv4.is_unspecified() {
                // Add standard /24 subnet broadcast
                let b24 = std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], 255);
                addrs.push(SocketAddr::new(std::net::IpAddr::V4(b24), 45454));

                // Add standard /16 subnet broadcast
                let b16 = std::net::Ipv4Addr::new(octets[0], octets[1], 255, 255);
                addrs.push(SocketAddr::new(std::net::IpAddr::V4(b16), 45454));

                // Special handling for iOS/iPadOS hotspot subnet: 172.20.10.0/28
                if octets[0] == 172 && octets[1] == 20 && octets[2] == 10 {
                    let ios_hotspot = std::net::Ipv4Addr::new(172, 20, 10, 15);
                    addrs.push(SocketAddr::new(std::net::IpAddr::V4(ios_hotspot), 45454));
                }
            }
        }
    }
    addrs.sort();
    addrs.dedup();
    addrs
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
            let broadcast_addrs = get_broadcast_addresses();
            for addr in broadcast_addrs {
                let _ = socket.send_to(json_str.as_bytes(), addr).await;
            }
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
