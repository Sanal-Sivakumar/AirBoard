use tokio::time::{sleep, Duration};
use crate::core::peer_manager::{ACTIVE_PEERS, Message};
use crate::core::connection_registry::{prune_inactive_peers, update_connection_status};
use crate::core::sync_engine::engine::{emit_event, SyncEvent};

pub async fn start_heartbeat_loop() {
    loop {
        sleep(Duration::from_secs(10)).await;

        {
            let peers = ACTIVE_PEERS.lock().unwrap();
            let heartbeat_msg = Message::Heartbeat {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            };

            for (_, tx) in peers.iter() {
                let _ = tx.send(heartbeat_msg.clone());
            }
        }

        let pruned = prune_inactive_peers(30);
        for id in pruned {
            {
                let mut peers = ACTIVE_PEERS.lock().unwrap();
                peers.remove(&id);
            }
            update_connection_status(&id, "Disconnected");
            emit_event(SyncEvent::ConnectionStatus {
                connected: false,
                message: format!("Peer timed out: {}", id),
            });
        }
    }
}
