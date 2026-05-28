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
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::Digest;

use crate::core::connection_registry::{update_connection_status, add_or_update_peer};
use crate::core::sync_engine::engine::{SYNC_ENGINE, emit_event, SyncEvent};
#[cfg(target_os = "linux")]
use crate::core::clipboard::linux::write_to_linux_clipboard;
use crate::core::crypto::{sign_message, verify_message_signature, compute_shared_secret, chacha_encrypt, chacha_decrypt, get_my_public_keys};
use crate::core::trust_store::{is_device_trusted, get_trusted_device};
use crate::core::session::{register_session_key, get_session_key, remove_session};
use crate::core::pairing::{handle_pairing_flow, PairingMessage};
use crate::core::clipboard_state::get_clipboard_state;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Message {
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
    #[serde(rename = "clipboard_state_exchange")]
    ClipboardStateExchange {
        packet_id: String,
        timestamp: i64,
    },
    #[serde(rename = "clipboard_state_request")]
    ClipboardStateRequest {
        packet_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum HandshakeMessage {
    #[serde(rename = "handshake_1")]
    Handshake1 {
        device_id: String,
        ephemeral_dh_pub: String, // base64
        signature: String,        // base64
    },
    #[serde(rename = "handshake_2")]
    Handshake2 {
        device_id: String,
        ephemeral_dh_pub: String, // base64
        signature: String,        // base64
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedEnvelope {
    #[serde(rename = "type")]
    pub msg_type: String, // "encrypted_payload"
    pub sender: String,
    pub nonce: String,      // base64
    pub ciphertext: String, // base64
}

pub type TxChannel = mpsc::UnboundedSender<WsMessage>;

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
    remove_session(device_id);
}

pub fn broadcast_clipboard_update(origin_device_id: String, packet_id: String, content: String, exclude_device_id: Option<String>) {
    let inner_msg = Message::ClipboardUpdate {
        packet_id,
        origin_device_id,
        content,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
    };

    let Ok(plaintext) = serde_json::to_vec(&inner_msg) else { return; };
    let local_id = SYNC_ENGINE.device_id.clone();

    let peers = ACTIVE_PEERS.lock().unwrap();
    for (id, tx) in peers.iter() {
        if let Some(ref exclude) = exclude_device_id {
            if id == exclude {
                continue;
            }
        }

        // Encrypt specifically for this peer using their session key
        if let Some(key) = get_session_key(id) {
            if let Ok((ciphertext, nonce)) = chacha_encrypt(&key, &plaintext) {
                let envelope = EncryptedEnvelope {
                    msg_type: "encrypted_payload".to_string(),
                    sender: local_id.clone(),
                    nonce: BASE64.encode(nonce),
                    ciphertext: BASE64.encode(ciphertext),
                };
                if let Ok(env_str) = serde_json::to_string(&envelope) {
                    let _ = tx.send(WsMessage::Text(env_str));
                }
            }
        }
    }
}

pub fn send_heartbeats() {
    let inner_msg = Message::Heartbeat {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    };

    let Ok(plaintext) = serde_json::to_vec(&inner_msg) else { return; };
    let local_id = SYNC_ENGINE.device_id.clone();

    let peers = ACTIVE_PEERS.lock().unwrap();
    for (id, tx) in peers.iter() {
        if let Some(key) = get_session_key(id) {
            if let Ok((ciphertext, nonce)) = chacha_encrypt(&key, &plaintext) {
                let envelope = EncryptedEnvelope {
                    msg_type: "encrypted_payload".to_string(),
                    sender: local_id.clone(),
                    nonce: BASE64.encode(nonce),
                    ciphertext: BASE64.encode(ciphertext),
                };
                if let Ok(env_str) = serde_json::to_string(&envelope) {
                    let _ = tx.send(WsMessage::Text(env_str));
                }
            }
        }
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
    println!("Rust Server: handle_incoming_connection called for incoming TCP connection from {}", ip_address);
    let ws_stream = accept_async(stream).await?;
    println!("Rust Server: WebSocket connection accepted from {}", ip_address);
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Read the first message. It determines whether this is a pairing request or a trusted handshake.
    let client_device_id = match ws_read.next().await {
        Some(Ok(WsMessage::Text(text))) => {
            println!("Rust Server: received first message payload: {}", text);
            if text.contains("pairing_request") {
                println!("Rust Server: payload matches pairing_request. Starting pairing flow.");
                if let Ok(PairingMessage::PairingRequest { device_id, device_name, public_signing_key, public_dh_key }) = serde_json::from_str::<PairingMessage>(&text) {
                    handle_pairing_flow(ws_write, device_id, device_name, public_signing_key, public_dh_key).await?;
                } else {
                    println!("Rust Server Error: Failed to parse PairingRequest JSON!");
                }
                return Ok(());
            } else if text.contains("handshake_1") {
                let Ok(HandshakeMessage::Handshake1 { device_id, ephemeral_dh_pub, signature }) = serde_json::from_str::<HandshakeMessage>(&text) else {
                    return Err("Failed to parse Handshake 1".into());
                };

                // Validate if untrusted
                if !is_device_trusted(&device_id) {
                    return Err(format!("Rejecting connection from untrusted peer: {}", device_id).into());
                }

                let peer = get_trusted_device(&device_id).unwrap();
                let Ok(client_ephemeral_pub_bytes) = BASE64.decode(&ephemeral_dh_pub) else { return Err("Invalid base64".into()); };
                let Ok(client_sig_bytes) = BASE64.decode(&signature) else { return Err("Invalid base64".into()); };

                let mut client_sig_arr = [0u8; 64];
                client_sig_arr.copy_from_slice(&client_sig_bytes);

                // Verify client's signature of their ephemeral public key
                if !verify_message_signature(&peer.public_signing_key, &client_ephemeral_pub_bytes, &client_sig_arr) {
                    return Err("Handshake 1 signature verification failed".into());
                }

                // Generate our ephemeral key
                let my_ephemeral_secret = x25519_dalek::StaticSecret::new(&mut rand::thread_rng());
                let my_ephemeral_pub = x25519_dalek::PublicKey::from(&my_ephemeral_secret);
                let my_ephemeral_pub_bytes = my_ephemeral_pub.as_bytes();

                // Sign our ephemeral public key
                let my_sig = sign_message(my_ephemeral_pub_bytes)?;

                let handshake2 = HandshakeMessage::Handshake2 {
                    device_id: SYNC_ENGINE.device_id.clone(),
                    ephemeral_dh_pub: BASE64.encode(my_ephemeral_pub_bytes),
                    signature: BASE64.encode(my_sig),
                };

                let handshake2_str = serde_json::to_string(&handshake2)?;
                ws_write.send(WsMessage::Text(handshake2_str)).await?;

                // Compute shared secret key
                let mut client_ephemeral_pub_arr = [0u8; 32];
                client_ephemeral_pub_arr.copy_from_slice(&client_ephemeral_pub_bytes);
                
                let shared_secret = my_ephemeral_secret.diffie_hellman(&x25519_dalek::PublicKey::from(client_ephemeral_pub_arr));
                
                let mut hasher = sha2::Sha256::new();
                hasher.update(shared_secret.as_bytes());
                let mut session_key = [0u8; 32];
                session_key.copy_from_slice(&hasher.finalize());

                register_session_key(device_id.clone(), session_key);
                add_or_update_peer(device_id.clone(), peer.device_name, ip_address, 0);
                
                device_id
            } else {
                return Err("Invalid protocol handshake packet".into());
            }
        }
        _ => return Err("Connection aborted during handshake".into()),
    };

    manage_connection_loops(client_device_id, ws_write, ws_read).await
}

pub async fn connect_to_peer(peer_id: String, ip: String, port: u16) {
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

            // 1. Generate local ephemeral keys
            let my_ephemeral_secret = x25519_dalek::StaticSecret::new(&mut rand::thread_rng());
            let my_ephemeral_pub = x25519_dalek::PublicKey::from(&my_ephemeral_secret);
            let my_ephemeral_pub_bytes = my_ephemeral_pub.as_bytes();

            // Sign
            let Ok(my_sig) = sign_message(my_ephemeral_pub_bytes) else {
                update_connection_status(&peer_id, "Disconnected");
                return;
            };

            // Send Handshake 1
            let handshake1 = HandshakeMessage::Handshake1 {
                device_id: SYNC_ENGINE.device_id.clone(),
                ephemeral_dh_pub: BASE64.encode(my_ephemeral_pub_bytes),
                signature: BASE64.encode(my_sig),
            };

            let Ok(h1_str) = serde_json::to_string(&handshake1) else { return; };
            if ws_write.send(WsMessage::Text(h1_str)).await.is_err() {
                update_connection_status(&peer_id, "Disconnected");
                return;
            }

            // 2. Read Handshake 2 from server
            let server_device_id = match ws_read.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    if let Ok(HandshakeMessage::Handshake2 { device_id, ephemeral_dh_pub, signature }) = serde_json::from_str::<HandshakeMessage>(&text) {
                        if !is_device_trusted(&device_id) {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        }

                        let peer = get_trusted_device(&device_id).unwrap();
                        let Ok(srv_ephemeral_pub_bytes) = BASE64.decode(&ephemeral_dh_pub) else { return; };
                        let Ok(srv_sig_bytes) = BASE64.decode(&signature) else { return; };

                        let mut srv_sig_arr = [0u8; 64];
                        srv_sig_arr.copy_from_slice(&srv_sig_bytes);

                        if !verify_message_signature(&peer.public_signing_key, &srv_ephemeral_pub_bytes, &srv_sig_arr) {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        }

                        // Compute shared session key
                        let mut srv_ephemeral_pub_arr = [0u8; 32];
                        srv_ephemeral_pub_arr.copy_from_slice(&srv_ephemeral_pub_bytes);
                        
                        let shared_secret = my_ephemeral_secret.diffie_hellman(&x25519_dalek::PublicKey::from(srv_ephemeral_pub_arr));
                        
                        let mut hasher = sha2::Sha256::new();
                        hasher.update(shared_secret.as_bytes());
                        let mut session_key = [0u8; 32];
                        session_key.copy_from_slice(&hasher.finalize());

                        register_session_key(device_id.clone(), session_key);
                        add_or_update_peer(device_id.clone(), peer.device_name, ip.clone(), port);
                        
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

            let _ = manage_connection_loops(server_device_id, ws_write, ws_read).await;
        }
        Err(_) => {
            update_connection_status(&peer_id, "Disconnected");
        }
    }
}

async fn manage_connection_loops<S>(
    peer_device_id: String,
    mut ws_write: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<S>, WsMessage>,
    mut ws_read: futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<S>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    if !register_peer(peer_device_id.clone(), tx.clone()) {
        return Ok(());
    }

    update_connection_status(&peer_device_id, "Connected");
    emit_event(SyncEvent::ConnectionStatus {
        connected: true,
        message: format!("Secure session established with {}", peer_device_id),
    });

    // Exchange clipboard state upon connection
    let local_state = get_clipboard_state();
    let state_msg = Message::ClipboardStateExchange {
        packet_id: local_state.packet_id.clone(),
        timestamp: local_state.timestamp,
    };
    if let Ok(plaintext) = serde_json::to_vec(&state_msg) {
        if let Some(session_key) = get_session_key(&peer_device_id) {
            if let Ok((ciphertext, nonce)) = chacha_encrypt(&session_key, &plaintext) {
                let envelope = EncryptedEnvelope {
                    msg_type: "encrypted_payload".to_string(),
                    sender: SYNC_ENGINE.device_id.clone(),
                    nonce: BASE64.encode(nonce),
                    ciphertext: BASE64.encode(ciphertext),
                };
                if let Ok(env_str) = serde_json::to_string(&envelope) {
                    let _ = tx.send(WsMessage::Text(env_str));
                }
            }
        }
    }

    let mut write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_write.send(msg).await.is_err() {
                break;
            }
        }
    });

    let peer_id_read = peer_device_id.clone();
    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(WsMessage::Text(text))) = ws_read.next().await {
            if let Ok(envelope) = serde_json::from_str::<EncryptedEnvelope>(&text) {
                if envelope.sender == peer_id_read {
                    if let Some(session_key) = get_session_key(&peer_id_read) {
                        let Ok(ciphertext) = BASE64.decode(&envelope.ciphertext) else { continue; };
                        let Ok(nonce_bytes) = BASE64.decode(&envelope.nonce) else { continue; };

                        let mut nonce_arr = [0u8; 12];
                        nonce_arr.copy_from_slice(&nonce_bytes);

                        if let Ok(plaintext) = chacha_decrypt(&session_key, &ciphertext, &nonce_arr) {
                            if let Ok(msg) = serde_json::from_slice::<Message>(&plaintext) {
                                match msg {
                                    Message::ClipboardUpdate { packet_id, origin_device_id, content, timestamp } => {
                                        if SYNC_ENGINE.process_incoming_packet(&packet_id) {
                                            crate::core::clipboard_state::update_clipboard_state(content.clone(), timestamp, packet_id.clone());

                                            #[cfg(target_os = "linux")]
                                            write_to_linux_clipboard(content.clone());

                                            emit_event(SyncEvent::ClipboardUpdated { content: content.clone() });

                                            // Re-encrypt and forward to other trusted devices
                                            broadcast_clipboard_update(origin_device_id, packet_id, content, Some(peer_id_read.clone()));
                                        }
                                    }
                                    Message::Heartbeat { .. } => {
                                        add_or_update_peer(peer_id_read.clone(), "".to_string(), "".to_string(), 0);
                                    }
                                    Message::ClipboardStateExchange { packet_id, timestamp } => {
                                        let local_state = get_clipboard_state();
                                        if timestamp > local_state.timestamp {
                                            // Remote has newer clipboard. Request it.
                                            let req_msg = Message::ClipboardStateRequest { packet_id };
                                            if let Ok(plaintext) = serde_json::to_vec(&req_msg) {
                                                if let Some(session_key) = get_session_key(&peer_id_read) {
                                                    if let Ok((ciphertext, nonce)) = chacha_encrypt(&session_key, &plaintext) {
                                                        let envelope = EncryptedEnvelope {
                                                            msg_type: "encrypted_payload".to_string(),
                                                            sender: SYNC_ENGINE.device_id.clone(),
                                                            nonce: BASE64.encode(nonce),
                                                            ciphertext: BASE64.encode(ciphertext),
                                                        };
                                                        if let Ok(env_str) = serde_json::to_string(&envelope) {
                                                            let peers = ACTIVE_PEERS.lock().unwrap();
                                                            if let Some(peer_tx) = peers.get(&peer_id_read) {
                                                                let _ = peer_tx.send(WsMessage::Text(env_str));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Message::ClipboardStateRequest { packet_id } => {
                                        let local_state = get_clipboard_state();
                                        if local_state.packet_id == packet_id && !local_state.content.is_empty() {
                                            let update_msg = Message::ClipboardUpdate {
                                                packet_id: local_state.packet_id,
                                                origin_device_id: SYNC_ENGINE.device_id.clone(),
                                                content: local_state.content,
                                                timestamp: local_state.timestamp,
                                            };
                                            if let Ok(plaintext) = serde_json::to_vec(&update_msg) {
                                                if let Some(session_key) = get_session_key(&peer_id_read) {
                                                    if let Ok((ciphertext, nonce)) = chacha_encrypt(&session_key, &plaintext) {
                                                        let envelope = EncryptedEnvelope {
                                                            msg_type: "encrypted_payload".to_string(),
                                                            sender: SYNC_ENGINE.device_id.clone(),
                                                            nonce: BASE64.encode(nonce),
                                                            ciphertext: BASE64.encode(ciphertext),
                                                        };
                                                        if let Ok(env_str) = serde_json::to_string(&envelope) {
                                                            let peers = ACTIVE_PEERS.lock().unwrap();
                                                            if let Some(peer_tx) = peers.get(&peer_id_read) {
                                                                let _ = peer_tx.send(WsMessage::Text(env_str));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
        message: format!("Session closed: {}", peer_device_id),
    });

    Ok(())
}
