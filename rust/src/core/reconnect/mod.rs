use crate::core::connection_registry::get_unconnected_peers;
use crate::core::peer_manager::connect_to_peer;
use tokio::time::{sleep, Duration};

pub async fn start_reconnect_loop() {
    loop {
        sleep(Duration::from_secs(10)).await;
        trigger_reconnect();
    }
}

pub fn trigger_reconnect() {
    // 1. Reconnect using discovered peers registry
    let unconnected = get_unconnected_peers();
    for peer in unconnected {
        let peer_id = peer.device_id;
        let ip = peer.ip_address;
        let port = peer.ws_port;

        if port > 0 && crate::core::trust_store::is_device_trusted(&peer_id) {
            crate::api::RUNTIME.spawn(async move {
                connect_to_peer(peer_id, ip, port).await;
            });
        }
    }

    // 2. Reconnect using persisted trust store coordinates for offline/undiscovered trusted devices
    let active_ids: std::collections::HashSet<String> = {
        let peers = crate::core::peer_manager::ACTIVE_PEERS.lock().unwrap();
        peers.keys().cloned().collect()
    };

    let trusted_devices = crate::core::trust_store::get_all_trusted_devices();
    for device in trusted_devices {
        if !active_ids.contains(&device.device_id) {
            if let (Some(ip), Some(port)) = (device.last_ip, device.last_port) {
                if port > 0 {
                    let peer_id = device.device_id;
                    crate::api::RUNTIME.spawn(async move {
                        connect_to_peer(peer_id, ip, port).await;
                    });
                }
            }
        }
    }
}
