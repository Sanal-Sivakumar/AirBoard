use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use serde_json;

use crate::core::sync_engine::engine::{SYNC_ENGINE, emit_event, SyncEvent};
use crate::core::protocol::models::ClipboardUpdate;
use crate::core::networking::OUTBOUND_BROADCAST;

static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

pub async fn start_websocket_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    emit_event(SyncEvent::ConnectionStatus {
        connected: false,
        message: format!("Server listening on port {}", port),
    });

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
                emit_event(SyncEvent::ConnectionStatus {
                    connected: true,
                    message: format!("Client connected. Active: {}", ACTIVE_CONNECTIONS.load(Ordering::SeqCst)),
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
                                    let _ = OUTBOUND_BROADCAST.send(update.content.clone());
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

                ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
                let active = ACTIVE_CONNECTIONS.load(Ordering::SeqCst);
                emit_event(SyncEvent::ConnectionStatus {
                    connected: active > 0,
                    message: if active > 0 {
                        format!("Client disconnected. Active: {}", active)
                    } else {
                        "Disconnected".to_string()
                    },
                });
            }
        });
    }

    Ok(())
}
