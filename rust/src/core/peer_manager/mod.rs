use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Message as WsMessage};
use tokio_tungstenite::{accept_async_with_config, connect_async_with_config};

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use crate::core::clipboard::desktop::write_to_desktop_clipboard;
use crate::core::clipboard_state::get_clipboard_state;
use crate::core::connection_registry::{add_or_update_peer, update_connection_status};
use crate::core::crypto::{
    chacha_decrypt, chacha_encrypt, decode_fixed, derive_session_key, handshake_aad, sign_message,
    verify_message_signature, PROTOCOL_VERSION,
};
use crate::core::pairing::{handle_pairing_flow, PairingMessage};
use crate::core::session::{
    accept_received_sequence, get_session_key, next_send_sequence, register_session_key,
    remove_session,
};
use crate::core::sync_engine::engine::{emit_event, SyncEvent, SYNC_ENGINE};
use crate::core::trust_store::{get_trusted_device, is_device_trusted};

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
    Heartbeat { timestamp: i64 },
    #[serde(rename = "clipboard_state_exchange")]
    ClipboardStateExchange { packet_id: String, timestamp: i64 },
    #[serde(rename = "clipboard_state_request")]
    ClipboardStateRequest { packet_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum HandshakeMessage {
    #[serde(rename = "handshake_1")]
    Handshake1 {
        protocol_version: u16,
        device_id: String,
        target_device_id: String,
        ephemeral_dh_pub: String, // base64
        nonce: String,
        signature: String, // base64
    },
    #[serde(rename = "handshake_2")]
    Handshake2 {
        protocol_version: u16,
        device_id: String,
        target_device_id: String,
        ephemeral_dh_pub: String, // base64
        initiator_nonce: String,
        responder_nonce: String,
        signature: String, // base64
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedEnvelope {
    #[serde(rename = "type")]
    pub msg_type: String, // "encrypted_payload"
    pub protocol_version: u16,
    pub sender: String,
    pub recipient: String,
    pub sequence: u64,
    pub nonce: String,      // base64
    pub ciphertext: String, // base64
}

pub type TxChannel = mpsc::UnboundedSender<WsMessage>;

pub struct PeerConnection {
    pub tx: TxChannel,
    pub cancel_tx: tokio::sync::oneshot::Sender<()>,
}

pub static ACTIVE_PEERS: Lazy<Mutex<HashMap<String, PeerConnection>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub static LOCAL_DEVICE_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("Device".to_string()));

const MAX_WIRE_MESSAGE_SIZE: usize = 1024 * 1024;
pub const MAX_CLIPBOARD_BYTES: usize = 512 * 1024;

fn websocket_config() -> WebSocketConfig {
    WebSocketConfig {
        write_buffer_size: 128 * 1024,
        max_write_buffer_size: 2 * 1024 * 1024,
        max_message_size: Some(MAX_WIRE_MESSAGE_SIZE),
        max_frame_size: Some(MAX_WIRE_MESSAGE_SIZE),
        accept_unmasked_frames: false,
        ..Default::default()
    }
}

fn encode_parts(domain: &[u8], parts: &[&[u8]]) -> Vec<u8> {
    let mut output =
        Vec::with_capacity(domain.len() + parts.iter().map(|part| part.len() + 8).sum::<usize>());
    output.extend_from_slice(domain);
    for part in parts {
        output.extend_from_slice(&(part.len() as u64).to_be_bytes());
        output.extend_from_slice(part);
    }
    output
}

fn handshake1_transcript(
    device_id: &str,
    target_device_id: &str,
    ephemeral_public_key: &[u8],
    nonce: &[u8],
) -> Vec<u8> {
    encode_parts(
        b"airboard/handshake-1/v2",
        &[
            device_id.as_bytes(),
            target_device_id.as_bytes(),
            ephemeral_public_key,
            nonce,
        ],
    )
}

fn handshake2_transcript(
    initiator_id: &str,
    responder_id: &str,
    initiator_ephemeral: &[u8],
    responder_ephemeral: &[u8],
    initiator_nonce: &[u8],
    responder_nonce: &[u8],
) -> Vec<u8> {
    encode_parts(
        b"airboard/handshake-2/v2",
        &[
            initiator_id.as_bytes(),
            responder_id.as_bytes(),
            initiator_ephemeral,
            responder_ephemeral,
            initiator_nonce,
            responder_nonce,
        ],
    )
}

fn encrypted_envelope_for_peer(peer_id: &str, plaintext: &[u8]) -> Option<WsMessage> {
    let key = get_session_key(peer_id)?;
    let sequence = next_send_sequence(peer_id)?;
    let sender = SYNC_ENGINE.device_id.clone();
    let aad = handshake_aad(&sender, peer_id, sequence);
    let (ciphertext, nonce) = chacha_encrypt(&key, plaintext, &aad).ok()?;
    let envelope = EncryptedEnvelope {
        msg_type: "encrypted_payload".to_string(),
        protocol_version: PROTOCOL_VERSION,
        sender,
        recipient: peer_id.to_string(),
        sequence,
        nonce: BASE64.encode(nonce),
        ciphertext: BASE64.encode(ciphertext),
    };
    serde_json::to_string(&envelope).ok().map(WsMessage::Text)
}

pub fn register_peer(
    device_id: String,
    tx: TxChannel,
    cancel_tx: tokio::sync::oneshot::Sender<()>,
    session_key: [u8; 32],
) -> bool {
    let mut peers = ACTIVE_PEERS.lock().unwrap();
    if peers.contains_key(&device_id) {
        return false;
    }
    register_session_key(device_id.clone(), session_key);
    peers.insert(device_id, PeerConnection { tx, cancel_tx });
    true
}

pub fn deregister_peer(device_id: &str) {
    let mut peers = ACTIVE_PEERS.lock().unwrap();
    peers.remove(device_id);
    remove_session(device_id);
}

pub fn broadcast_clipboard_update(
    origin_device_id: String,
    packet_id: String,
    content: String,
    exclude_device_id: Option<String>,
) {
    if content.len() > MAX_CLIPBOARD_BYTES {
        emit_event(SyncEvent::Error {
            message: format!(
                "Clipboard content exceeds the {} KiB safety limit",
                MAX_CLIPBOARD_BYTES / 1024
            ),
        });
        return;
    }
    let inner_msg = Message::ClipboardUpdate {
        packet_id,
        origin_device_id,
        content,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
    };

    let Ok(plaintext) = serde_json::to_vec(&inner_msg) else {
        return;
    };
    let peers = ACTIVE_PEERS.lock().unwrap();
    for (id, conn) in peers.iter() {
        if let Some(ref exclude) = exclude_device_id {
            if id == exclude {
                continue;
            }
        }

        if let Some(message) = encrypted_envelope_for_peer(id, &plaintext) {
            let _ = conn.tx.send(message);
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

    let Ok(plaintext) = serde_json::to_vec(&inner_msg) else {
        return;
    };
    let peers = ACTIVE_PEERS.lock().unwrap();
    for (id, conn) in peers.iter() {
        if let Some(message) = encrypted_envelope_for_peer(id, &plaintext) {
            let _ = conn.tx.send(message);
        }
    }
}

pub async fn start_p2p_server(
    port: u16,
) -> Result<(u16, tokio::task::JoinHandle<()>), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let bound_port = listener.local_addr()?.port();

    let handle = tokio::spawn(async move {
        while let Ok((stream, src_addr)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = handle_incoming_connection(stream, src_addr.ip().to_string()).await
                {
                    eprintln!("Error handling incoming connection: {}", e);
                }
            });
        }
    });

    Ok((bound_port, handle))
}

async fn handle_incoming_connection(
    stream: TcpStream,
    ip_address: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Rust Server: handle_incoming_connection called for incoming TCP connection from {}",
        ip_address
    );
    let ws_stream = accept_async_with_config(stream, Some(websocket_config())).await?;
    println!(
        "Rust Server: WebSocket connection accepted from {}",
        ip_address
    );
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Read the first message. It determines whether this is a pairing request or a trusted handshake.
    let client_device_id = match ws_read.next().await {
        Some(Ok(WsMessage::Text(text))) => {
            println!("Rust Server: received first message payload: {}", text);
            if text.contains("pairing_request") {
                println!("Rust Server: payload matches pairing_request. Starting pairing flow.");
                if let Ok(request @ PairingMessage::PairingRequest { .. }) =
                    serde_json::from_str::<PairingMessage>(&text)
                {
                    handle_pairing_flow(ws_write, request, ip_address.clone()).await?;
                } else {
                    println!("Rust Server Error: Failed to parse PairingRequest JSON!");
                }
                return Ok(());
            } else if text.contains("handshake_1") {
                let Ok(HandshakeMessage::Handshake1 {
                    protocol_version,
                    device_id,
                    target_device_id,
                    ephemeral_dh_pub,
                    nonce,
                    signature,
                }) = serde_json::from_str::<HandshakeMessage>(&text)
                else {
                    return Err("Failed to parse Handshake 1".into());
                };

                if protocol_version != PROTOCOL_VERSION || target_device_id != SYNC_ENGINE.device_id
                {
                    return Err("Handshake target or protocol version mismatch".into());
                }

                // Validate if untrusted
                if !is_device_trusted(&device_id) {
                    return Err(
                        format!("Rejecting connection from untrusted peer: {}", device_id).into(),
                    );
                }

                let peer = get_trusted_device(&device_id)
                    .ok_or("Trusted peer disappeared during handshake")?;
                let client_ephemeral_pub_arr = decode_fixed::<32>(
                    BASE64
                        .decode(&ephemeral_dh_pub)
                        .map_err(|_| "Invalid ephemeral key encoding")?,
                    "ephemeral key",
                )?;
                let client_nonce = decode_fixed::<32>(
                    BASE64
                        .decode(&nonce)
                        .map_err(|_| "Invalid handshake nonce encoding")?,
                    "handshake nonce",
                )?;
                let client_sig_arr = decode_fixed::<64>(
                    BASE64
                        .decode(&signature)
                        .map_err(|_| "Invalid signature encoding")?,
                    "signature",
                )?;

                let handshake1_transcript = handshake1_transcript(
                    &device_id,
                    &target_device_id,
                    &client_ephemeral_pub_arr,
                    &client_nonce,
                );

                // Verify client's signature of their ephemeral public key
                if !verify_message_signature(
                    &peer.public_signing_key,
                    &handshake1_transcript,
                    &client_sig_arr,
                ) {
                    return Err("Handshake 1 signature verification failed".into());
                }

                // Generate our ephemeral key
                let my_ephemeral_secret =
                    x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
                let my_ephemeral_pub = x25519_dalek::PublicKey::from(&my_ephemeral_secret);
                let my_ephemeral_pub_bytes = my_ephemeral_pub.as_bytes();

                let mut responder_nonce = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut responder_nonce);

                let handshake2_transcript = handshake2_transcript(
                    &device_id,
                    &SYNC_ENGINE.device_id,
                    &client_ephemeral_pub_arr,
                    my_ephemeral_pub_bytes,
                    &client_nonce,
                    &responder_nonce,
                );
                let my_sig = sign_message(&handshake2_transcript)?;

                let handshake2 = HandshakeMessage::Handshake2 {
                    protocol_version: PROTOCOL_VERSION,
                    device_id: SYNC_ENGINE.device_id.clone(),
                    target_device_id: device_id.clone(),
                    ephemeral_dh_pub: BASE64.encode(my_ephemeral_pub_bytes),
                    initiator_nonce: BASE64.encode(client_nonce),
                    responder_nonce: BASE64.encode(responder_nonce),
                    signature: BASE64.encode(my_sig),
                };

                let handshake2_str = serde_json::to_string(&handshake2)?;
                ws_write.send(WsMessage::Text(handshake2_str)).await?;

                // Compute shared secret key
                let shared_secret = my_ephemeral_secret
                    .diffie_hellman(&x25519_dalek::PublicKey::from(client_ephemeral_pub_arr));
                let session_key =
                    derive_session_key(shared_secret.as_bytes(), &handshake2_transcript)?;

                add_or_update_peer(device_id.clone(), peer.device_name, ip_address.clone(), 0);
                crate::core::trust_store::update_trusted_device_ip_port(&device_id, ip_address, 0);

                (device_id, session_key)
            } else {
                return Err("Invalid protocol handshake packet".into());
            }
        }
        _ => return Err("Connection aborted during handshake".into()),
    };

    manage_connection_loops(client_device_id.0, client_device_id.1, ws_write, ws_read).await
}

pub async fn connect_to_peer(peer_id: String, ip: String, port: u16) {
    {
        let peers = ACTIVE_PEERS.lock().unwrap();
        if peers.contains_key(&peer_id) {
            return;
        }
    }
    // Check registry status to avoid concurrent attempts
    {
        let registry = crate::core::connection_registry::REGISTRY.lock().unwrap();
        if let Some(peer) = registry.get(&peer_id) {
            if peer.connection_status == "Connecting" {
                return;
            }
        }
    }

    update_connection_status(&peer_id, "Connecting");

    let url = format!("ws://{}:{}", ip, port);
    match connect_async_with_config(&url, Some(websocket_config()), false).await {
        Ok((ws_stream, _)) => {
            let (mut ws_write, mut ws_read) = ws_stream.split();

            // 1. Generate local ephemeral keys
            let my_ephemeral_secret =
                x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
            let my_ephemeral_pub = x25519_dalek::PublicKey::from(&my_ephemeral_secret);
            let my_ephemeral_pub_bytes = my_ephemeral_pub.as_bytes();

            let mut initiator_nonce = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut initiator_nonce);

            let handshake1_transcript = handshake1_transcript(
                &SYNC_ENGINE.device_id,
                &peer_id,
                my_ephemeral_pub_bytes,
                &initiator_nonce,
            );
            let Ok(my_sig) = sign_message(&handshake1_transcript) else {
                update_connection_status(&peer_id, "Disconnected");
                return;
            };

            // Send Handshake 1
            let handshake1 = HandshakeMessage::Handshake1 {
                protocol_version: PROTOCOL_VERSION,
                device_id: SYNC_ENGINE.device_id.clone(),
                target_device_id: peer_id.clone(),
                ephemeral_dh_pub: BASE64.encode(my_ephemeral_pub_bytes),
                nonce: BASE64.encode(initiator_nonce),
                signature: BASE64.encode(my_sig),
            };

            let Ok(h1_str) = serde_json::to_string(&handshake1) else {
                return;
            };
            if ws_write.send(WsMessage::Text(h1_str)).await.is_err() {
                update_connection_status(&peer_id, "Disconnected");
                return;
            }

            // 2. Read Handshake 2 from server
            let server_device_id = match ws_read.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    if let Ok(HandshakeMessage::Handshake2 {
                        protocol_version,
                        device_id,
                        target_device_id,
                        ephemeral_dh_pub,
                        initiator_nonce: echoed_initiator_nonce,
                        responder_nonce,
                        signature,
                    }) = serde_json::from_str::<HandshakeMessage>(&text)
                    {
                        if protocol_version != PROTOCOL_VERSION
                            || target_device_id != SYNC_ENGINE.device_id
                            || device_id != peer_id
                        {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        }
                        if !is_device_trusted(&device_id) {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        }

                        let Some(peer) = get_trusted_device(&device_id) else {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        };
                        let Ok(srv_ephemeral_pub_arr) = BASE64
                            .decode(&ephemeral_dh_pub)
                            .map_err(|_| ())
                            .and_then(|bytes| {
                                decode_fixed::<32>(bytes, "ephemeral key").map_err(|_| ())
                            })
                        else {
                            return;
                        };
                        let Ok(echoed_nonce) = BASE64
                            .decode(&echoed_initiator_nonce)
                            .map_err(|_| ())
                            .and_then(|bytes| {
                                decode_fixed::<32>(bytes, "initiator nonce").map_err(|_| ())
                            })
                        else {
                            return;
                        };
                        if echoed_nonce != initiator_nonce {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        }
                        let Ok(responder_nonce_arr) = BASE64
                            .decode(&responder_nonce)
                            .map_err(|_| ())
                            .and_then(|bytes| {
                                decode_fixed::<32>(bytes, "responder nonce").map_err(|_| ())
                            })
                        else {
                            return;
                        };
                        let Ok(srv_sig_arr) =
                            BASE64.decode(&signature).map_err(|_| ()).and_then(|bytes| {
                                decode_fixed::<64>(bytes, "signature").map_err(|_| ())
                            })
                        else {
                            return;
                        };

                        let handshake2_transcript = handshake2_transcript(
                            &SYNC_ENGINE.device_id,
                            &device_id,
                            my_ephemeral_pub_bytes,
                            &srv_ephemeral_pub_arr,
                            &initiator_nonce,
                            &responder_nonce_arr,
                        );
                        if !verify_message_signature(
                            &peer.public_signing_key,
                            &handshake2_transcript,
                            &srv_sig_arr,
                        ) {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        }

                        // Compute shared session key
                        let shared_secret = my_ephemeral_secret
                            .diffie_hellman(&x25519_dalek::PublicKey::from(srv_ephemeral_pub_arr));
                        let Ok(session_key) =
                            derive_session_key(shared_secret.as_bytes(), &handshake2_transcript)
                        else {
                            update_connection_status(&peer_id, "Disconnected");
                            return;
                        };

                        add_or_update_peer(device_id.clone(), peer.device_name, ip.clone(), port);
                        crate::core::trust_store::update_trusted_device_ip_port(
                            &device_id,
                            ip.clone(),
                            port,
                        );

                        (device_id, session_key)
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

            if server_device_id.0 != peer_id {
                update_connection_status(&peer_id, "Disconnected");
                return;
            }

            let _ =
                manage_connection_loops(server_device_id.0, server_device_id.1, ws_write, ws_read)
                    .await;
        }
        Err(_) => {
            update_connection_status(&peer_id, "Disconnected");
        }
    }
}

async fn manage_connection_loops<S>(
    peer_device_id: String,
    session_key: [u8; 32],
    mut ws_write: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<S>, WsMessage>,
    mut ws_read: futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<S>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();
    let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

    if !register_peer(peer_device_id.clone(), tx.clone(), cancel_tx, session_key) {
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
        if let Some(message) = encrypted_envelope_for_peer(&peer_device_id, &plaintext) {
            let _ = tx.send(message);
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
                if envelope.protocol_version == PROTOCOL_VERSION
                    && envelope.sender == peer_id_read
                    && envelope.recipient == SYNC_ENGINE.device_id
                {
                    if let Some(session_key) = get_session_key(&peer_id_read) {
                        let Ok(ciphertext) = BASE64.decode(&envelope.ciphertext) else {
                            continue;
                        };
                        let Ok(nonce_arr) = BASE64
                            .decode(&envelope.nonce)
                            .map_err(|_| ())
                            .and_then(|bytes| decode_fixed::<12>(bytes, "nonce").map_err(|_| ()))
                        else {
                            continue;
                        };
                        let aad =
                            handshake_aad(&envelope.sender, &envelope.recipient, envelope.sequence);

                        if let Ok(plaintext) =
                            chacha_decrypt(&session_key, &ciphertext, &nonce_arr, &aad)
                        {
                            if !accept_received_sequence(&peer_id_read, envelope.sequence) {
                                continue;
                            }
                            if let Ok(msg) = serde_json::from_slice::<Message>(&plaintext) {
                                match msg {
                                    Message::ClipboardUpdate {
                                        packet_id,
                                        origin_device_id,
                                        content,
                                        timestamp,
                                    } => {
                                        if content.len() <= MAX_CLIPBOARD_BYTES
                                            && SYNC_ENGINE.process_incoming_packet(&packet_id)
                                        {
                                            crate::core::clipboard_state::update_clipboard_state(
                                                content.clone(),
                                                timestamp,
                                                packet_id.clone(),
                                            );

                                            #[cfg(any(
                                                target_os = "linux",
                                                target_os = "windows",
                                                target_os = "macos"
                                            ))]
                                            write_to_desktop_clipboard(content.clone());

                                            #[cfg(target_os = "android")]
                                            {
                                                let content_clone = content.clone();
                                                tokio::spawn(async move {
                                                    android_log(&format!("Rust connecting to local socket bridge with {} bytes...", content_clone.len()));
                                                    match tokio::net::TcpStream::connect(
                                                        "127.0.0.1:45456",
                                                    )
                                                    .await
                                                    {
                                                        Ok(mut stream) => {
                                                            use tokio::io::AsyncWriteExt;
                                                            if let Err(e) = stream
                                                                .write_all(content_clone.as_bytes())
                                                                .await
                                                            {
                                                                android_log(&format!("Rust local socket write error: {:?}", e));
                                                            } else {
                                                                let _ = stream.flush().await;
                                                                let _ = stream.shutdown().await;
                                                                android_log("Rust local socket successfully wrote and shutdown stream");
                                                            }
                                                        }
                                                        Err(e) => {
                                                            android_log(&format!("Rust local socket connection failed: {:?}", e));
                                                        }
                                                    }
                                                });
                                            }

                                            emit_event(SyncEvent::ClipboardUpdated {
                                                content: content.clone(),
                                                is_local: false,
                                            });

                                            // Re-encrypt and forward to other trusted devices
                                            broadcast_clipboard_update(
                                                origin_device_id,
                                                packet_id,
                                                content,
                                                Some(peer_id_read.clone()),
                                            );
                                        }
                                    }
                                    Message::Heartbeat { .. } => {
                                        add_or_update_peer(
                                            peer_id_read.clone(),
                                            "".to_string(),
                                            "".to_string(),
                                            0,
                                        );
                                    }
                                    Message::ClipboardStateExchange {
                                        packet_id,
                                        timestamp,
                                    } => {
                                        let local_state = get_clipboard_state();
                                        if timestamp > local_state.timestamp {
                                            // Remote has newer clipboard. Request it.
                                            let req_msg =
                                                Message::ClipboardStateRequest { packet_id };
                                            if let Ok(plaintext) = serde_json::to_vec(&req_msg) {
                                                if let Some(message) = encrypted_envelope_for_peer(
                                                    &peer_id_read,
                                                    &plaintext,
                                                ) {
                                                    let peers = ACTIVE_PEERS.lock().unwrap();
                                                    if let Some(peer_tx) = peers.get(&peer_id_read)
                                                    {
                                                        let _ = peer_tx.tx.send(message);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Message::ClipboardStateRequest { packet_id } => {
                                        let local_state = get_clipboard_state();
                                        if local_state.packet_id == packet_id
                                            && !local_state.content.is_empty()
                                        {
                                            let update_msg = Message::ClipboardUpdate {
                                                packet_id: local_state.packet_id,
                                                origin_device_id: SYNC_ENGINE.device_id.clone(),
                                                content: local_state.content,
                                                timestamp: local_state.timestamp,
                                            };
                                            if let Ok(plaintext) = serde_json::to_vec(&update_msg) {
                                                if let Some(message) = encrypted_envelope_for_peer(
                                                    &peer_id_read,
                                                    &plaintext,
                                                ) {
                                                    let peers = ACTIVE_PEERS.lock().unwrap();
                                                    if let Some(peer_tx) = peers.get(&peer_id_read)
                                                    {
                                                        let _ = peer_tx.tx.send(message);
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
        _ = &mut cancel_rx => {},
    }

    deregister_peer(&peer_device_id);
    update_connection_status(&peer_device_id, "Disconnected");
    emit_event(SyncEvent::ConnectionStatus {
        connected: false,
        message: format!("Session closed: {}", peer_device_id),
    });

    Ok(())
}

#[cfg(target_os = "android")]
extern "C" {
    fn __android_log_print(
        prio: std::os::raw::c_int,
        tag: *const std::os::raw::c_char,
        fmt: *const std::os::raw::c_char,
        ...
    ) -> std::os::raw::c_int;
}

#[cfg(target_os = "android")]
pub fn android_log(message: &str) {
    use std::ffi::CString;
    let Ok(tag) = CString::new("RustAirBoard") else {
        return;
    };
    let Ok(msg) = CString::new(message.replace('\0', "�")) else {
        return;
    };
    unsafe {
        __android_log_print(4, tag.as_ptr(), msg.as_ptr());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_transcript_binds_roles_identities_and_nonces() {
        let ephemeral = [1u8; 32];
        let nonce = [2u8; 32];
        let base = handshake1_transcript("a", "b", &ephemeral, &nonce);
        assert_ne!(base, handshake1_transcript("b", "a", &ephemeral, &nonce));

        let responder_ephemeral = [3u8; 32];
        let responder_nonce = [4u8; 32];
        let response = handshake2_transcript(
            "a",
            "b",
            &ephemeral,
            &responder_ephemeral,
            &nonce,
            &responder_nonce,
        );
        assert_ne!(base, response);
        assert_ne!(
            response,
            handshake2_transcript(
                "a",
                "c",
                &ephemeral,
                &responder_ephemeral,
                &nonce,
                &responder_nonce,
            )
        );
    }
}
