use std::collections::HashMap;
use std::sync::Mutex;
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};

use crate::core::crypto::{get_my_public_keys, verify_message_signature};
use crate::core::trust_store::{add_trusted_device, TrustedDevice};
use crate::core::sync_engine::engine::{emit_event, SyncEvent, SYNC_ENGINE};
use crate::core::peer_manager::LOCAL_DEVICE_NAME;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum PairingMessage {
    #[serde(rename = "pairing_request")]
    PairingRequest {
        device_id: String,
        device_name: String,
        public_signing_key: String, // base64
        public_dh_key: String,      // base64
    },
    #[serde(rename = "pairing_response")]
    PairingResponse {
        status: String,             // "approved" or "denied"
        device_id: String,
        device_name: String,
        public_signing_key: String, // base64
        public_dh_key: String,      // base64
    },
}

pub static PENDING_PAIRINGS: Lazy<Mutex<HashMap<String, oneshot::Sender<bool>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn respond_to_pairing(device_id: String, approve: bool) -> bool {
    let mut pending = PENDING_PAIRINGS.lock().unwrap();
    if let Some(tx) = pending.remove(&device_id) {
        let _ = tx.send(approve);
        true
    } else {
        false
    }
}

pub fn compute_fingerprint(pub_signing_key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pub_signing_key_bytes);
    let hash = hasher.finalize();
    hash.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(":")
}

pub async fn initiate_pairing_flow(peer_id: String, ip: String, port: u16) {
    let url = format!("ws://{}:{}", ip, port);
    println!("Rust Pairing: initiate_pairing_flow starting for {}", url);
    emit_event(SyncEvent::ConnectionStatus {
        connected: false,
        message: format!("Initiating pairing request to {}...", url),
    });

    match connect_async(&url).await {
        Ok((ws_stream, _)) => {
            println!("Rust Pairing: connected successfully to {}", url);
            let (mut ws_write, mut ws_read) = ws_stream.split();
            
            // 1. Send PairingRequest
            let Some((pub_sig, pub_dh)) = get_my_public_keys() else {
                println!("Rust Pairing Error: Keys not registered!");
                emit_event(SyncEvent::Error { message: "Keys not registered".to_string() });
                return;
            };

            let req = PairingMessage::PairingRequest {
                device_id: SYNC_ENGINE.device_id.clone(),
                device_name: LOCAL_DEVICE_NAME.lock().unwrap().clone(),
                public_signing_key: BASE64.encode(pub_sig),
                public_dh_key: BASE64.encode(pub_dh),
            };

            if let Ok(req_str) = serde_json::to_string(&req) {
                println!("Rust Pairing: sending pairing request payload to {}", url);
                if ws_write.send(WsMessage::Text(req_str)).await.is_err() {
                    println!("Rust Pairing Error: Failed to write payload to WebSocket!");
                    emit_event(SyncEvent::Error { message: "Failed to send pairing request".to_string() });
                    return;
                }
            }

            // 2. Await Response
            println!("Rust Pairing: awaiting pairing response from {}", url);
            match ws_read.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    println!("Rust Pairing: received response string: {}", text);
                    if let Ok(PairingMessage::PairingResponse { status, device_id, device_name, public_signing_key, public_dh_key }) = serde_json::from_str::<PairingMessage>(&text) {
                        println!("Rust Pairing: parsed response status: '{}' from '{}'", status, device_name);
                        if status == "approved" {
                            let Ok(sig_bytes) = BASE64.decode(public_signing_key) else { return; };
                            let Ok(dh_bytes) = BASE64.decode(public_dh_key) else { return; };
                            
                            let mut signing_arr = [0u8; 32];
                            let mut dh_arr = [0u8; 32];
                            signing_arr.copy_from_slice(&sig_bytes);
                            dh_arr.copy_from_slice(&dh_bytes);

                            add_trusted_device(TrustedDevice {
                                device_id: device_id.clone(),
                                device_name: device_name.clone(),
                                public_signing_key: signing_arr,
                                public_dh_key: dh_arr,
                                paired_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                            });

                            emit_event(SyncEvent::ConnectionStatus {
                                connected: false,
                                message: format!("Pairing successful with {}!", device_name),
                            });
                        } else {
                            emit_event(SyncEvent::Error { message: format!("Pairing denied by {}!", device_name) });
                        }
                    }
                }
                Some(Ok(other)) => {
                    println!("Rust Pairing Error: Received non-text message: {:?}", other);
                    emit_event(SyncEvent::Error { message: "Pairing aborted by remote peer".to_string() });
                }
                Some(Err(e)) => {
                    println!("Rust Pairing Error: WebSocket read error: {}", e);
                    emit_event(SyncEvent::Error { message: "Pairing aborted by remote peer".to_string() });
                }
                None => {
                    println!("Rust Pairing Error: Connection closed by remote peer while awaiting response!");
                    emit_event(SyncEvent::Error { message: "Pairing aborted by remote peer".to_string() });
                }
            }
        }
        Err(e) => {
            println!("Rust Pairing Error: Connection to {} failed: {}", url, e);
            emit_event(SyncEvent::Error { message: format!("Connection failed: {}", e) });
        }
    }
}

pub async fn handle_pairing_flow(
    mut ws_write: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, WsMessage>,
    req_device_id: String,
    req_device_name: String,
    pub_sig_base64: String,
    pub_dh_base64: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Ok(sig_bytes) = BASE64.decode(pub_sig_base64) else { return Ok(()); };
    let Ok(dh_bytes) = BASE64.decode(pub_dh_base64) else { return Ok(()); };
    
    let fingerprint = compute_fingerprint(&sig_bytes);

    let (tx, rx) = oneshot::channel::<bool>();
    {
        let mut pending = PENDING_PAIRINGS.lock().unwrap();
        pending.insert(req_device_id.clone(), tx);
    }

    // Emit event to Flutter containing string-serialized details so FRB exposes it nicely
    emit_event(SyncEvent::Error {
        message: format!("PAIR_REQ:{}:{}:{}", req_device_id, req_device_name, fingerprint),
    });

    // Await user action
    let approved = rx.await.unwrap_or(false);

    if approved {
        let Some((my_pub_sig, my_pub_dh)) = get_my_public_keys() else {
            return Ok(());
        };

        let mut signing_arr = [0u8; 32];
        let mut dh_arr = [0u8; 32];
        signing_arr.copy_from_slice(&sig_bytes);
        dh_arr.copy_from_slice(&dh_bytes);

        add_trusted_device(TrustedDevice {
            device_id: req_device_id.clone(),
            device_name: req_device_name.clone(),
            public_signing_key: signing_arr,
            public_dh_key: dh_arr,
            paired_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        });

        let resp = PairingMessage::PairingResponse {
            status: "approved".to_string(),
            device_id: SYNC_ENGINE.device_id.clone(),
            device_name: LOCAL_DEVICE_NAME.lock().unwrap().clone(),
            public_signing_key: BASE64.encode(my_pub_sig),
            public_dh_key: BASE64.encode(my_pub_dh),
        };

        let resp_str = serde_json::to_string(&resp)?;
        ws_write.send(WsMessage::Text(resp_str)).await?;
    } else {
        let resp = PairingMessage::PairingResponse {
            status: "denied".to_string(),
            device_id: SYNC_ENGINE.device_id.clone(),
            device_name: LOCAL_DEVICE_NAME.lock().unwrap().clone(),
            public_signing_key: String::new(),
            public_dh_key: String::new(),
        };

        let resp_str = serde_json::to_string(&resp)?;
        ws_write.send(WsMessage::Text(resp_str)).await?;
    }

    Ok(())
}
