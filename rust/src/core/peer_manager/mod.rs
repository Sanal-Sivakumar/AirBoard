use std::collections::HashMap;
use std::sync::Mutex;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::core::connection_registry::{update_connection_status, add_or_update_peer};
use crate::core::sync_engine::engine::{SYNC_ENGINE, emit_event, SyncEvent};
use crate::core::clipboard::linux::write_to_linux_clipboard;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "init")]
    Init {
        device_id: String,
        device_name: String,
    },
    #[serde(rename = "clipboard_update")]
    ClipboardUpdate {
        packet_id: String,
        origin_device_id: String,
        content: String,
        timestamp: i64,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: i64,
    },
}

pub type TxChannel = mpsc::UnboundedSender<Message>;

pub static ACTIVE_PEERS: Lazy<Mutex<HashMap<String, TxChannel>>> = Lazy::new(|| Mutex::new(HashMap::new()));
pub static LOCAL_DEVICE_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("Device".to_string()));

pub fn register_peer(device_id: String, tx: TxChannel) -> bool {
    let mut peers = ACTIVE_PEERS.lock().unwrap();
    if peers.contains_key(&device_id) {
        return false;
    }
    peers.insert(device_id, tx);
    true
}

pub fn deregister_peer(device_id: &str) {
    let mut peers = ACTIVE_PEERS.lock().unwrap();
    peers.remove(device_id);
}

pub fn broadcast_clipboard_update(origin_device_id: String, packet_id: String, content: String, exclude_device_id: Option<String>) {
    let msg = Message::ClipboardUpdate {
        packet_id,
        origin_device_id,
        content,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    };

    let peers = ACTIVE_PEERS.lock().unwrap();
    for (id, tx) in peers.iter() {
        if let Some(ref exclude) = exclude_device_id {
            if id == exclude {
                continue;
            }
        }
        let _ = tx.send(msg.clone());
    }
}

pub async fn start_p2p_server(port: u16) -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let bound_port = listener.local_addr()?.port();

    tokio::spawn(async move {
        while let Ok((stream, src_addr)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = handle_incoming_connection(stream, src_addr.ip().to_string()).await {
                    eprintln!("Error handling incoming connection: {}", e);
                }
            });
        }
    });

    Ok(bound_port)
}

async fn handle_incoming_connection(stream: TcpStream, ip_address: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // 1. Send Init
    let local_id = SYNC_ENGINE.device_id.clone();
    let local_name = LOCAL_DEVICE_NAME.lock().unwrap().clone();
    let init_msg = Message::Init {
        device_id: local_id.clone(),
        device_name: local_name,
    };
    let init_str = serde_json::to_string(&init_msg)?;
    ws_write.send(WsMessage::Text(init_str)).await?;

    // 2. Wait for Client's Init
    let client_device_id = match ws_read.next().await {
        Some(Ok(WsMessage::Text(text))) => {
            if let Ok(Message::Init { device_id, device_name }) = serde_json::from_str::<Message>(&text) {
                // Register in connection registry
                add_or_update_peer(device_id.clone(), device_name, ip_address, 0);
                device_id
            } else {
                return Err("Invalid initialization message".into());
            }
        }
        _ => return Err("Connection closed during handshake".into()),
    };

    manage_connection_loops(client_device_id, ws_write, ws_read).await
}

pub async fn connect_to_peer(peer_id: String, ip: String, port: u16) {
    // Connection tie-breaker
    let local_id = SYNC_ENGINE.device_id.clone();
    if local_id >= peer_id {
        // Skip connecting, wait for smaller device ID to initiate connection to us
        return;
    }

    // Check if already connected
    {
        let peers = ACTIVE_PEERS.lock().unwrap();
        if peers.contains_key(&peer_id) {
            return;
        }
    }

    update_connection_status(&peer_id, "Connecting");

    let url = format!("ws://{}:{}", ip, port);
    match connect_async(&url).await {
        Ok((ws_stream, _)) => {
            let (mut ws_write, mut ws_read) = ws_stream.split();

            // 1. Wait for Server Init
            let server_device_id = match ws_read.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    if let Ok(Message::Init { device_id, device_name }) = serde_json::from_str::<Message>(&text) {
                        add_or_update_peer(device_id.clone(), device_name, ip.clone(), port);
                        device_id
                    } else {
                        update_connection_status(&peer_id, "Disconnected");
                        return;
                    }
                }
                _ => {
                    update_connection_status(&peer_id, "Disconnected");
                    return;
                }
            };

            if server_device_id != peer_id {
                update_connection_status(&peer_id, "Disconnected");
                return;
            }

            // 2. Send local Init
            let local_id = SYNC_ENGINE.device_id.clone();
            let local_name = LOCAL_DEVICE_NAME.lock().unwrap().clone();
            let init_msg = Message::Init {
                device_id: local_id,
                device_name: local_name,
            };
            if let Ok(init_str) = serde_json::to_string(&init_msg) {
                if ws_write.send(WsMessage::Text(init_str)).await.is_err() {
                    update_connection_status(&peer_id, "Disconnected");
                    return;
                }
            }

            let _ = manage_connection_loops(server_device_id, ws_write, ws_read).await;
        }
        Err(_) => {
            update_connection_status(&peer_id, "Disconnected");
        }
    }
}

async fn manage_connection_loops(
    peer_device_id: String,
    mut ws_write: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, WsMessage>,
    mut ws_read: futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<TcpStream>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Register active channel
    if !register_peer(peer_device_id.clone(), tx) {
        // Connection duplicate exists
        return Ok(());
    }

    update_connection_status(&peer_device_id, "Connected");
    emit_event(SyncEvent::ConnectionStatus {
        connected: true,
        message: format!("Peer connected: {}", peer_device_id),
    });

    let peer_id_write = peer_device_id.clone();
    let mut write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if ws_write.send(WsMessage::Text(json_str)).await.is_err() {
                    break;
                }
            }
        }
    });

    let peer_id_read = peer_device_id.clone();
    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(WsMessage::Text(text))) = ws_read.next().await {
            if let Ok(msg) = serde_json::from_str::<Message>(&text) {
                match msg {
                    Message::ClipboardUpdate { packet_id, origin_device_id, content, .. } => {
                        // Check if packet processed already
                        if SYNC_ENGINE.process_incoming_packet(&packet_id) {
                            // Update system clipboard
                            #[cfg(target_os = "linux")]
                            write_to_linux_clipboard(content.clone());

                            // Emit event to Flutter
                            emit_event(SyncEvent::ClipboardUpdated { content: content.clone() });

                            // Forward packet to all other connected peers (excluding sender)
                            broadcast_clipboard_update(origin_device_id, packet_id, content, Some(peer_id_read.clone()));
                        }
                    }
                    Message::Heartbeat { .. } => {
                        // Heartbeat updates registry last_seen timestamp
                        add_or_update_peer(peer_id_read.clone(), "".to_string(), "".to_string(), 0);
                    }
                    _ => {}
                }
            }
        }
    });

    tokio::select! {
        _ = &mut write_task => {},
        _ = &mut read_task => {},
    }

    deregister_peer(&peer_device_id);
    update_connection_status(&peer_device_id, "Disconnected");
    emit_event(SyncEvent::ConnectionStatus {
        connected: false,
        message: format!("Peer disconnected: {}", peer_device_id),
    });

    Ok(())
}
