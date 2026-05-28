use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Peer {
    pub device_id: String,
    pub device_name: String,
    pub ip_address: String,
    pub ws_port: u16,
    pub last_seen: u64,
    pub connection_status: String, // "Connected", "Disconnected", "Connecting"
}

pub static REGISTRY: Lazy<Mutex<HashMap<String, Peer>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn add_or_update_peer(
    device_id: String,
    device_name: String,
    ip_address: String,
    ws_port: u16,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut registry = REGISTRY.lock().unwrap();
    if let Some(peer) = registry.get_mut(&device_id) {
        peer.last_seen = now;
        if !ip_address.is_empty() {
            peer.ip_address = ip_address;
        }
        if ws_port > 0 {
            peer.ws_port = ws_port;
        }
        if !device_name.is_empty() {
            peer.device_name = device_name;
        }
    } else {
        registry.insert(
            device_id.clone(),
            Peer {
                device_id,
                device_name,
                ip_address,
                ws_port,
                last_seen: now,
                connection_status: "Disconnected".to_string(),
            },
        );
    }
}

pub fn update_connection_status(device_id: &str, status: &str) {
    let mut registry = REGISTRY.lock().unwrap();
    if let Some(peer) = registry.get_mut(device_id) {
        peer.connection_status = status.to_string();
    }
}

pub fn get_peers() -> Vec<Peer> {
    let registry = REGISTRY.lock().unwrap();
    registry.values().cloned().collect()
}

pub fn get_unconnected_peers() -> Vec<Peer> {
    let registry = REGISTRY.lock().unwrap();
    registry
        .values()
        .filter(|p| p.connection_status == "Disconnected")
        .cloned()
        .collect()
}

pub fn prune_inactive_peers(timeout_seconds: u64) -> Vec<String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut registry = REGISTRY.lock().unwrap();
    let mut to_remove = Vec::new();

    for (id, peer) in registry.iter() {
        if now - peer.last_seen > timeout_seconds {
            to_remove.push(id.clone());
        }
    }

    for id in &to_remove {
        registry.remove(id);
    }

    to_remove
}
