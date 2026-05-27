use tokio::time::{sleep, Duration};
use crate::core::connection_registry::get_unconnected_peers;
use crate::core::peer_manager::connect_to_peer;

pub async fn start_reconnect_loop() {
    loop {
        sleep(Duration::from_secs(10)).await;

        let unconnected = get_unconnected_peers();
        for peer in unconnected {
            let peer_id = peer.device_id;
            let ip = peer.ip_address;
            let port = peer.ws_port;

            if port > 0 {
                tokio::spawn(async move {
                    connect_to_peer(peer_id, ip, port).await;
                });
            }
        }
    }
}
