use tokio::time::{sleep, Duration};
use crate::core::peer_manager::{send_heartbeats, ACTIVE_PEERS};
use crate::core::connection_registry::{prune_inactive_peers, update_connection_status};
use crate::core::sync_engine::engine::{emit_event, SyncEvent};

pub async fn start_heartbeat_loop() {
    loop {
        sleep(Duration::from_secs(10)).await;

        // Broadcast encrypted heartbeats
        send_heartbeats();

        // Prune inactive peers (unresponsive for more than 30 seconds)
        let pruned = prune_inactive_peers(30);
        for id in pruned {
            let conn_opt = {
                let mut peers = ACTIVE_PEERS.lock().unwrap();
                peers.remove(&id)
            };
            update_connection_status(&id, "Disconnected");
            if let Some(conn) = conn_opt {
                let _ = conn.cancel_tx.send(());
            }
            emit_event(SyncEvent::ConnectionStatus {
                connected: false,
                message: format!("Peer timed out: {}", id),
            });
        }
    }
}
