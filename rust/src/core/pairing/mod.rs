use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Message as WsMessage};

use crate::core::crypto::{
    decode_fixed, fingerprint, get_my_public_keys, sign_message, verify_message_signature,
    PROTOCOL_VERSION,
};
use crate::core::peer_manager::LOCAL_DEVICE_NAME;
use crate::core::sync_engine::engine::{emit_event, SyncEvent, SYNC_ENGINE};
use crate::core::trust_store::{add_trusted_device, TrustedDevice};

const PAIRING_TIMEOUT: Duration = Duration::from_secs(120);
const PAIRING_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_PAIRING_MESSAGE_SIZE: usize = 64 * 1024;

fn pairing_websocket_config() -> WebSocketConfig {
    WebSocketConfig {
        max_message_size: Some(MAX_PAIRING_MESSAGE_SIZE),
        max_frame_size: Some(MAX_PAIRING_MESSAGE_SIZE),
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum PairingMessage {
    #[serde(rename = "pairing_request")]
    PairingRequest {
        protocol_version: u16,
        device_id: String,
        device_name: String,
        public_signing_key: String,
        public_dh_key: String,
        pairing_nonce: String,
        signature: String,
    },
    #[serde(rename = "pairing_response")]
    PairingResponse {
        protocol_version: u16,
        status: String,
        device_id: String,
        device_name: String,
        public_signing_key: String,
        public_dh_key: String,
        request_nonce: String,
        response_nonce: String,
        signature: String,
    },
}

#[derive(Serialize)]
struct PairingPrompt<'a> {
    peer_id: &'a str,
    peer_name: &'a str,
    fingerprint: &'a str,
    direction: &'a str,
}

pub static PENDING_PAIRINGS: Lazy<Mutex<HashMap<String, oneshot::Sender<bool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    fingerprint(pub_signing_key_bytes)
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

fn pairing_request_transcript(
    device_id: &str,
    device_name: &str,
    public_signing_key: &[u8],
    public_dh_key: &[u8],
    nonce: &[u8],
) -> Vec<u8> {
    encode_parts(
        b"airboard/pairing-request/v2",
        &[
            device_id.as_bytes(),
            device_name.as_bytes(),
            public_signing_key,
            public_dh_key,
            nonce,
        ],
    )
}

fn pairing_response_transcript(
    request_nonce: &[u8],
    response_nonce: &[u8],
    requester_id: &str,
    responder_id: &str,
    responder_name: &str,
    responder_signing_key: &[u8],
    responder_dh_key: &[u8],
) -> Vec<u8> {
    encode_parts(
        b"airboard/pairing-response/v2",
        &[
            request_nonce,
            response_nonce,
            requester_id.as_bytes(),
            responder_id.as_bytes(),
            responder_name.as_bytes(),
            responder_signing_key,
            responder_dh_key,
        ],
    )
}

async fn request_user_verification(
    peer_id: &str,
    peer_name: &str,
    peer_fingerprint: &str,
    direction: &str,
) -> bool {
    let (tx, rx) = oneshot::channel::<bool>();
    {
        let mut pending = PENDING_PAIRINGS.lock().unwrap();
        if let Some(previous) = pending.insert(peer_id.to_string(), tx) {
            let _ = previous.send(false);
        }
    }

    let prompt = PairingPrompt {
        peer_id,
        peer_name,
        fingerprint: peer_fingerprint,
        direction,
    };
    if let Ok(json) = serde_json::to_string(&prompt) {
        emit_event(SyncEvent::Error {
            message: format!("PAIR_VERIFY:{json}"),
        });
    }

    let approved = timeout(PAIRING_TIMEOUT, rx)
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or(false);
    PENDING_PAIRINGS.lock().unwrap().remove(peer_id);
    approved
}

fn random_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

pub async fn initiate_pairing_flow(expected_peer_id: String, ip: String, port: u16) {
    let url = format!("ws://{ip}:{port}");
    emit_event(SyncEvent::ConnectionStatus {
        connected: false,
        message: format!("Initiating mutually verified pairing with {url}..."),
    });

    let result: Result<(), String> = async {
        let (ws_stream, _) = timeout(
            PAIRING_CONNECT_TIMEOUT,
            connect_async_with_config(&url, Some(pairing_websocket_config()), false),
        )
        .await
        .map_err(|_| "Pairing connection timed out".to_string())?
        .map_err(|error| format!("Pairing connection failed: {error}"))?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        let (public_signing_key, public_dh_key) =
            get_my_public_keys().ok_or("Identity keys are not registered")?;
        let request_nonce = random_nonce();
        let local_device_id = SYNC_ENGINE.device_id.clone();
        let local_device_name = LOCAL_DEVICE_NAME.lock().unwrap().clone();
        let request_transcript = pairing_request_transcript(
            &local_device_id,
            &local_device_name,
            &public_signing_key,
            &public_dh_key,
            &request_nonce,
        );
        let signature = sign_message(&request_transcript)?;

        let request = PairingMessage::PairingRequest {
            protocol_version: PROTOCOL_VERSION,
            device_id: local_device_id.clone(),
            device_name: local_device_name,
            public_signing_key: BASE64.encode(&public_signing_key),
            public_dh_key: BASE64.encode(&public_dh_key),
            pairing_nonce: BASE64.encode(request_nonce),
            signature: BASE64.encode(signature),
        };
        let request_json = serde_json::to_string(&request)
            .map_err(|error| format!("Pairing request serialization failed: {error}"))?;
        ws_write
            .send(WsMessage::Text(request_json))
            .await
            .map_err(|error| format!("Pairing request send failed: {error}"))?;

        let response_text = timeout(PAIRING_TIMEOUT, ws_read.next())
            .await
            .map_err(|_| "Pairing response timed out".to_string())?
            .ok_or("Peer closed the pairing connection")?
            .map_err(|error| format!("Pairing response failed: {error}"))?
            .into_text()
            .map_err(|_| "Pairing response was not text".to_string())?;

        let PairingMessage::PairingResponse {
            protocol_version,
            status,
            device_id,
            device_name,
            public_signing_key,
            public_dh_key,
            request_nonce: echoed_request_nonce,
            response_nonce,
            signature,
        } = serde_json::from_str::<PairingMessage>(&response_text)
            .map_err(|error| format!("Invalid pairing response: {error}"))?
        else {
            return Err("Peer returned an unexpected pairing message".to_string());
        };

        if protocol_version != PROTOCOL_VERSION || status != "approved" {
            return Err("Pairing was denied or the peer uses an incompatible protocol".to_string());
        }
        if expected_peer_id != "manual_connection" && device_id != expected_peer_id {
            return Err("Pairing response identity did not match the selected device".to_string());
        }

        let echoed_nonce = decode_fixed::<32>(
            BASE64
                .decode(echoed_request_nonce)
                .map_err(|_| "Invalid request nonce")?,
            "request nonce",
        )?;
        if echoed_nonce != request_nonce {
            return Err("Pairing response did not match this request".to_string());
        }
        let response_nonce = decode_fixed::<32>(
            BASE64
                .decode(response_nonce)
                .map_err(|_| "Invalid response nonce")?,
            "response nonce",
        )?;
        let responder_signing_key = decode_fixed::<32>(
            BASE64
                .decode(public_signing_key)
                .map_err(|_| "Invalid signing key")?,
            "signing key",
        )?;
        let responder_dh_key = decode_fixed::<32>(
            BASE64.decode(public_dh_key).map_err(|_| "Invalid DH key")?,
            "DH key",
        )?;
        let response_signature = decode_fixed::<64>(
            BASE64
                .decode(signature)
                .map_err(|_| "Invalid pairing signature")?,
            "pairing signature",
        )?;
        let response_transcript = pairing_response_transcript(
            &request_nonce,
            &response_nonce,
            &local_device_id,
            &device_id,
            &device_name,
            &responder_signing_key,
            &responder_dh_key,
        );
        if !verify_message_signature(
            &responder_signing_key,
            &response_transcript,
            &response_signature,
        ) {
            return Err("Pairing response signature verification failed".to_string());
        }

        let responder_fingerprint = fingerprint(&responder_signing_key);
        if !request_user_verification(&device_id, &device_name, &responder_fingerprint, "outgoing")
            .await
        {
            return Err("Responder fingerprint was not approved".to_string());
        }

        add_trusted_device(TrustedDevice {
            device_id: device_id.clone(),
            device_name: device_name.clone(),
            public_signing_key: responder_signing_key,
            public_dh_key: responder_dh_key,
            paired_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_ip: Some(ip),
            last_port: Some(port),
        });
        crate::core::reconnect::trigger_reconnect();
        emit_event(SyncEvent::ConnectionStatus {
            connected: false,
            message: format!("Mutually verified pairing completed with {device_name}"),
        });
        Ok(())
    }
    .await;

    if let Err(message) = result {
        emit_event(SyncEvent::Error { message });
    }
}

pub async fn handle_pairing_flow(
    mut ws_write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        WsMessage,
    >,
    request: PairingMessage,
    ip: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let PairingMessage::PairingRequest {
        protocol_version,
        device_id,
        device_name,
        public_signing_key,
        public_dh_key,
        pairing_nonce,
        signature,
    } = request
    else {
        return Err("Expected pairing request".into());
    };
    if protocol_version != PROTOCOL_VERSION {
        return Err("Incompatible pairing protocol".into());
    }

    let requester_signing_key =
        decode_fixed::<32>(BASE64.decode(public_signing_key)?, "signing key")?;
    let requester_dh_key = decode_fixed::<32>(BASE64.decode(public_dh_key)?, "DH key")?;
    let request_nonce = decode_fixed::<32>(BASE64.decode(pairing_nonce)?, "pairing nonce")?;
    let request_signature = decode_fixed::<64>(BASE64.decode(signature)?, "pairing signature")?;
    let request_transcript = pairing_request_transcript(
        &device_id,
        &device_name,
        &requester_signing_key,
        &requester_dh_key,
        &request_nonce,
    );
    if !verify_message_signature(
        &requester_signing_key,
        &request_transcript,
        &request_signature,
    ) {
        return Err("Pairing request signature verification failed".into());
    }

    let requester_fingerprint = fingerprint(&requester_signing_key);
    let approved =
        request_user_verification(&device_id, &device_name, &requester_fingerprint, "incoming")
            .await;

    let (my_public_signing_key, my_public_dh_key) =
        get_my_public_keys().ok_or("Identity keys are not registered")?;
    let response_nonce = random_nonce();
    let local_device_id = SYNC_ENGINE.device_id.clone();
    let local_device_name = LOCAL_DEVICE_NAME.lock().unwrap().clone();
    let response_transcript = pairing_response_transcript(
        &request_nonce,
        &response_nonce,
        &device_id,
        &local_device_id,
        &local_device_name,
        &my_public_signing_key,
        &my_public_dh_key,
    );
    let response_signature = sign_message(&response_transcript)?;

    let response = PairingMessage::PairingResponse {
        protocol_version: PROTOCOL_VERSION,
        status: if approved { "approved" } else { "denied" }.to_string(),
        device_id: local_device_id,
        device_name: local_device_name,
        public_signing_key: BASE64.encode(&my_public_signing_key),
        public_dh_key: BASE64.encode(&my_public_dh_key),
        request_nonce: BASE64.encode(request_nonce),
        response_nonce: BASE64.encode(response_nonce),
        signature: BASE64.encode(response_signature),
    };
    ws_write
        .send(WsMessage::Text(serde_json::to_string(&response)?))
        .await?;

    if approved {
        add_trusted_device(TrustedDevice {
            device_id,
            device_name,
            public_signing_key: requester_signing_key,
            public_dh_key: requester_dh_key,
            paired_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_ip: Some(ip),
            last_port: None,
        });
        crate::core::reconnect::trigger_reconnect();
    }

    Ok(())
}
