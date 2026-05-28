pub mod simple;

use crate::core::sync_engine::engine::{SyncEvent, EVENT_SINK, SYNC_ENGINE};
use crate::core::connection_registry::{get_peers, Peer};
use crate::core::peer_manager::{start_p2p_server, broadcast_clipboard_update, LOCAL_DEVICE_NAME, ACTIVE_PEERS};
use crate::core::discovery::{start_udp_announcer, start_udp_listener};
use crate::core::reconnect::start_reconnect_loop;
use crate::core::heartbeat::start_heartbeat_loop;
#[cfg(target_os = "linux")]
use crate::core::clipboard::linux::start_linux_clipboard_monitor;
use crate::core::crypto::register_identity_keys;
use crate::core::trust_store::{init_trust_store, get_all_trusted_devices, remove_trusted_device};
use crate::core::pairing::{initiate_pairing_flow, respond_to_pairing, compute_fingerprint};
use crate::core::connection_registry::REGISTRY;
use crate::StreamSink;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

#[derive(Debug, Clone)]
pub struct TrustedPeer {
    pub device_id: String,
    pub device_name: String,
    pub fingerprint: String,
    pub paired_at: u64,
}

pub fn init_app(sink: StreamSink<SyncEvent>) {
    let mut guard = EVENT_SINK.lock().unwrap();
    *guard = Some(sink);
}

pub fn register_keys(signing_key_bytes: Vec<u8>, dh_key_bytes: Vec<u8>) -> Vec<String> {
    let mut sig_arr = [0u8; 32];
    let mut dh_arr = [0u8; 32];
    sig_arr.copy_from_slice(&signing_key_bytes[..32]);
    dh_arr.copy_from_slice(&dh_key_bytes[..32]);

    let (pub_sig, pub_dh) = register_identity_keys(sig_arr, dh_arr);
    
    vec![BASE64.encode(pub_sig), BASE64.encode(pub_dh)]
}

use crate::core::lifecycle::{set_client_only, register_initial_handles};

pub fn start_sync(storage_dir: String, device_name: String, platform: String, device_id: String) {
    init_trust_store(storage_dir);
    crate::core::sync_engine::engine::set_my_device_id(device_id);

    {
        let mut name_guard = LOCAL_DEVICE_NAME.lock().unwrap();
        *name_guard = device_name.clone();
    }

    let is_ios = platform == "ios";
    set_client_only(is_ios);

    RUNTIME.spawn(async move {
        let bound_port = if !is_ios {
            match start_p2p_server(45455).await {
                Ok(p) => p,
                Err(e) => {
                    crate::core::sync_engine::engine::emit_event(SyncEvent::Error {
                        message: format!("Server failed to start: {}", e),
                    });
                    return;
                }
            }
        } else {
            0
        };

        let h_announcer = tokio::spawn(start_udp_announcer(device_name.clone(), platform.clone(), bound_port));
        let h_listener = tokio::spawn(start_udp_listener());
        let h_heartbeat = tokio::spawn(start_heartbeat_loop());
        let h_reconnect = tokio::spawn(start_reconnect_loop());

        register_initial_handles(h_announcer, h_listener, h_heartbeat, h_reconnect);

        #[cfg(target_os = "linux")]
        if platform == "linux" {
            tokio::spawn(start_linux_clipboard_monitor());
        }
    });
}

pub fn handle_app_foreground() {
    crate::core::lifecycle::handle_app_foreground();
}

pub fn handle_app_background() {
    crate::core::lifecycle::handle_app_background();
}

pub fn send_local_clipboard_update(content: String) {
    let (is_new, packet_id, timestamp) = SYNC_ENGINE.process_local_change(&content);
    if is_new {
        crate::core::clipboard_state::update_clipboard_state(content.clone(), timestamp, packet_id.clone());
        broadcast_clipboard_update(SYNC_ENGINE.device_id.clone(), packet_id, content, None);
    }
}

pub fn get_device_id() -> String {
    SYNC_ENGINE.device_id.clone()
}

pub fn get_discovered_peers() -> Vec<Peer> {
    get_peers()
}

pub fn get_trusted_peers() -> Vec<TrustedPeer> {
    get_all_trusted_devices()
        .into_iter()
        .map(|d| TrustedPeer {
            device_id: d.device_id,
            device_name: d.device_name,
            fingerprint: compute_fingerprint(&d.public_signing_key),
            paired_at: d.paired_at,
        })
        .collect()
}

pub fn initiate_pairing(peer_id: String) {
    println!("Rust API: initiate_pairing called with peer_id = {}", peer_id);
    RUNTIME.spawn(async move {
        let (ip, port) = {
            let registry = REGISTRY.lock().unwrap();
            if let Some(peer) = registry.get(&peer_id) {
                (peer.ip_address.clone(), peer.ws_port)
            } else {
                println!("Rust API: peer_id '{}' not found in discovered registry!", peer_id);
                return;
            }
        };
        println!("Rust API: calling initiate_pairing_flow with {}:{} for peer '{}'", ip, port, peer_id);
        if port > 0 {
            initiate_pairing_flow(peer_id, ip, port).await;
        } else {
            println!("Rust API: Port is 0, skipping connection.");
        }
    });
}

pub fn approve_pairing(peer_id: String, approve: bool) {
    respond_to_pairing(peer_id, approve);
}

pub fn unpair_device(peer_id: String) {
    remove_trusted_device(&peer_id);
    
    let mut peers = ACTIVE_PEERS.lock().unwrap();
    peers.remove(&peer_id);
}
