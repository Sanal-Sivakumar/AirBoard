use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use serde_json;

use crate::core::sync_engine::engine::{SYNC_ENGINE, emit_event, SyncEvent};
use crate::core::protocol::models::ClipboardUpdate;
use crate::core::networking::OUTBOUND_BROADCAST;
use crate::core::clipboard::linux::write_to_linux_clipboard;

pub async fn start_websocket_client(server_ip: String, port: u16) {
    let url = format!("ws://{}:{}", server_ip, port);
    
    loop {
        emit_event(SyncEvent::ConnectionStatus {
            connected: false,
            message: format!("Connecting to {}...", url),
        });

        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                emit_event(SyncEvent::ConnectionStatus {
                    connected: true,
                    message: "Connected to server".to_string(),
                });

                let (mut ws_write, mut ws_read) = ws_stream.split();
                let mut broadcast_rx = OUTBOUND_BROADCAST.subscribe();
                let local_device_id = SYNC_ENGINE.device_id.clone();

                let write_device_id = local_device_id.clone();
                let mut write_task = tokio::spawn(async move {
                    while let Ok(content) = broadcast_rx.recv().await {
                        let update = ClipboardUpdate::new(write_device_id.clone(), content);
                        if let Ok(json_str) = serde_json::to_string(&update) {
                            if ws_write.send(tokio_tungstenite::tungstenite::Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        }
                    }
                });

                let mut read_task = tokio::spawn(async move {
                    while let Some(Ok(msg)) = ws_read.next().await {
                        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                            if let Ok(update) = serde_json::from_str::<ClipboardUpdate>(&text) {
                                if SYNC_ENGINE.process_incoming_sync(&update.device_id, &update.content) {
                                    write_to_linux_clipboard(update.content.clone());
                                    emit_event(SyncEvent::ClipboardUpdated { content: update.content });
                                }
                            }
                        }
                    }
                });

                tokio::select! {
                    _ = &mut write_task => {},
                    _ = &mut read_task => {},
                }

                emit_event(SyncEvent::ConnectionStatus {
                    connected: false,
                    message: "Connection lost. Reconnecting in 3s...".to_string(),
                });
            }
            Err(e) => {
                emit_event(SyncEvent::ConnectionStatus {
                    connected: false,
                    message: format!("Connection failed: {}. Retrying in 3s...", e),
                });
            }
        }

        sleep(Duration::from_secs(3)).await;
    }
}
