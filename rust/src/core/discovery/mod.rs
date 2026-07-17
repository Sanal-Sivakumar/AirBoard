use crate::core::connection_registry::add_or_update_peer;
use crate::core::sync_engine::engine::{emit_event, SyncEvent, SYNC_ENGINE};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceAnnouncement {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub device_name: String,
    pub device_id: String,
    pub platform: String,
    pub ws_port: u16,
}

const DISCOVERY_PORT: u16 = 45454;
const DISCOVERY_MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 255, 45, 54);

fn get_local_broadcasts_from_ip_cmd() -> Vec<std::net::IpAddr> {
    let mut broadcasts = Vec::new();
    if let Ok(output) = std::process::Command::new("ip").arg("addr").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("inet ") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() > 3 && parts[2] == "brd" {
                        if let Ok(ip) = parts[3].parse::<std::net::IpAddr>() {
                            if !ip.is_loopback() && !ip.is_unspecified() {
                                broadcasts.push(ip);
                            }
                        }
                    }
                }
            }
        }
    }
    broadcasts
}

pub static DYNAMIC_LOCAL_IP: once_cell::sync::Lazy<std::sync::Mutex<Option<String>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

fn get_local_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

fn get_broadcast_addresses() -> Vec<SocketAddr> {
    let mut addrs = vec![
        SocketAddr::from((Ipv4Addr::BROADCAST, DISCOVERY_PORT)),
        SocketAddr::from((DISCOVERY_MULTICAST, DISCOVERY_PORT)),
    ];

    // 1. Check dynamic local IP set from Dart side
    let mut resolved_ip = None;
    if let Ok(guard) = DYNAMIC_LOCAL_IP.lock() {
        if let Some(ref ip_str) = *guard {
            if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                resolved_ip = Some(ip);
            }
        }
    }

    // 2. Try spawning "ip addr" command first (supported on Android/Linux)
    let parsed_broadcasts = get_local_broadcasts_from_ip_cmd();
    if !parsed_broadcasts.is_empty() {
        for ip in parsed_broadcasts {
            addrs.push(SocketAddr::new(ip, DISCOVERY_PORT));
        }
    }

    // 3. Fallback/Supplement with connectionless UDP routing resolver
    let local_ip = resolved_ip.or_else(get_local_ip);
    if let Some(std::net::IpAddr::V4(ipv4)) = local_ip {
        let octets = ipv4.octets();
        if !ipv4.is_loopback() && !ipv4.is_unspecified() {
            // Add standard /24 subnet broadcast
            let b24 = std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], 255);
            addrs.push(SocketAddr::new(std::net::IpAddr::V4(b24), DISCOVERY_PORT));

            // Add standard /16 subnet broadcast
            let b16 = std::net::Ipv4Addr::new(octets[0], octets[1], 255, 255);
            addrs.push(SocketAddr::new(std::net::IpAddr::V4(b16), DISCOVERY_PORT));

            // Special handling for iOS/iPadOS hotspot subnet: 172.20.10.0/28
            if octets[0] == 172 && octets[1] == 20 && octets[2] == 10 {
                let ios_hotspot = std::net::Ipv4Addr::new(172, 20, 10, 15);
                addrs.push(SocketAddr::new(
                    std::net::IpAddr::V4(ios_hotspot),
                    DISCOVERY_PORT,
                ));
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
    let destinations = get_broadcast_addresses();

    emit_event(SyncEvent::ConnectionStatus {
        connected: false,
        message: format!(
            "LAN discovery announcing on UDP {DISCOVERY_PORT} to {} broadcast/multicast destinations.",
            destinations.len()
        ),
    });

    loop {
        let announcement = DeviceAnnouncement {
            msg_type: "device_announcement".to_string(),
            device_name: device_name.clone(),
            device_id: local_device_id.clone(),
            platform: platform.clone(),
            ws_port,
        };

        if let Ok(json_str) = serde_json::to_string(&announcement) {
            for addr in &destinations {
                let _ = socket.send_to(json_str.as_bytes(), addr).await;
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn start_udp_listener() {
    let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT)).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind UDP listener: {}", e);
            emit_event(SyncEvent::Error {
                message: format!(
                    "LAN discovery could not listen on UDP {DISCOVERY_PORT}: {e}. Check for another AirBoard process and firewall rules."
                ),
            });
            return;
        }
    };

    if let Err(error) = socket.join_multicast_v4(DISCOVERY_MULTICAST, Ipv4Addr::UNSPECIFIED) {
        emit_event(SyncEvent::ConnectionStatus {
            connected: false,
            message: format!(
                "Multicast discovery is unavailable ({error}); continuing with LAN broadcast."
            ),
        });
    }

    let mut buf = [0u8; 1024];
    let local_device_id = SYNC_ENGINE.device_id.clone();

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src_addr)) => {
                let data = &buf[..len];
                if let Ok(announcement) = serde_json::from_slice::<DeviceAnnouncement>(data) {
                    if announcement.msg_type == "device_announcement"
                        && !announcement.device_id.is_empty()
                        && announcement.device_id.len() <= 128
                        && announcement.device_id != local_device_id
                    {
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
