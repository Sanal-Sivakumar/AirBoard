pub mod simple;

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use crate::core::clipboard::desktop::start_desktop_clipboard_monitor;
use crate::core::connection_registry::REGISTRY;
use crate::core::connection_registry::{get_peers, Peer};
use crate::core::crypto::register_identity_keys;
use crate::core::discovery::{start_udp_announcer, start_udp_listener};
use crate::core::heartbeat::start_heartbeat_loop;
use crate::core::pairing::{compute_fingerprint, initiate_pairing_flow, respond_to_pairing};
use crate::core::peer_manager::{
    broadcast_clipboard_update, start_p2p_server, ACTIVE_PEERS, LOCAL_DEVICE_NAME,
};
use crate::core::reconnect::start_reconnect_loop;
use crate::core::sync_engine::engine::{emit_event, SyncEvent, EVENT_SINK, SYNC_ENGINE};
use crate::core::trust_store::{get_all_trusted_devices, init_trust_store, remove_trusted_device};
use crate::StreamSink;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

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
    if signing_key_bytes.len() != 32 || dh_key_bytes.len() != 32 {
        return Vec::new();
    }
    let mut sig_arr = [0u8; 32];
    let mut dh_arr = [0u8; 32];
    sig_arr.copy_from_slice(&signing_key_bytes[..32]);
    dh_arr.copy_from_slice(&dh_key_bytes[..32]);

    let (pub_sig, pub_dh) = register_identity_keys(sig_arr, dh_arr);

    let fingerprint = crate::core::crypto::fingerprint(&pub_sig);
    vec![BASE64.encode(pub_sig), BASE64.encode(pub_dh), fingerprint]
}

use crate::core::lifecycle::{register_initial_handles, set_client_only};

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
        let (bound_port, server_handle) = if !is_ios {
            match start_p2p_server(45455).await {
                Ok((port, handle)) => (port, Some(handle)),
                Err(e) => {
                    crate::core::sync_engine::engine::emit_event(SyncEvent::Error {
                        message: format!("Server failed to start: {}", e),
                    });
                    return;
                }
            }
        } else {
            (0, None)
        };

        let h_announcer = tokio::spawn(start_udp_announcer(
            device_name.clone(),
            platform.clone(),
            bound_port,
        ));
        let h_listener = tokio::spawn(start_udp_listener());
        let h_heartbeat = tokio::spawn(start_heartbeat_loop());
        let h_reconnect = tokio::spawn(start_reconnect_loop());

        emit_event(SyncEvent::ConnectionStatus {
            connected: false,
            message: if is_ios {
                "AirBoard networking started in iPadOS/iOS client mode. Pair from this device using the other device's LAN IP.".to_string()
            } else {
                format!(
                    "AirBoard is listening for trusted peers on TCP {bound_port} and discovery on UDP 45454."
                )
            },
        });

        let clipboard_handle = {
            #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
            {
                if platform == "linux" || platform == "windows" || platform == "macos" {
                    Some(tokio::spawn(start_desktop_clipboard_monitor()))
                } else {
                    None
                }
            }
            #[cfg(target_os = "android")]
            {
                if platform == "android" {
                    Some(tokio::spawn(
                        crate::core::clipboard::android::start_android_local_receiver(),
                    ))
                } else {
                    None
                }
            }
            #[cfg(not(any(
                target_os = "linux",
                target_os = "windows",
                target_os = "macos",
                target_os = "android"
            )))]
            {
                None
            }
        };

        register_initial_handles(
            server_handle,
            h_announcer,
            h_listener,
            h_heartbeat,
            h_reconnect,
            clipboard_handle,
        );
    });
}

pub fn handle_app_foreground() {
    crate::core::lifecycle::handle_app_foreground();
}

pub fn handle_app_background() {
    crate::core::lifecycle::handle_app_background();
}

pub fn send_local_clipboard_update(content: String) {
    if content.len() > crate::core::peer_manager::MAX_CLIPBOARD_BYTES {
        emit_event(SyncEvent::Error {
            message: "Clipboard content exceeds the 512 KiB safety limit".to_string(),
        });
        return;
    }
    let (is_new, packet_id, timestamp) = SYNC_ENGINE.process_local_change(&content);
    if is_new {
        crate::core::clipboard_state::update_clipboard_state(
            content.clone(),
            timestamp,
            packet_id.clone(),
        );
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
    println!(
        "Rust API: initiate_pairing called with peer_id = {}",
        peer_id
    );
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
            emit_event(SyncEvent::Error {
                message: "iOS/iPadOS devices cannot accept incoming connections. Please initiate pairing from the iOS/iPadOS device instead.".to_string(),
            });
        }
    });
}

pub fn approve_pairing(peer_id: String, approve: bool) {
    respond_to_pairing(peer_id, approve);
}

pub fn unpair_device(peer_id: String) {
    remove_trusted_device(&peer_id);

    let conn_opt = {
        let mut peers = ACTIVE_PEERS.lock().unwrap();
        peers.remove(&peer_id)
    };
    if let Some(conn) = conn_opt {
        let _ = conn.cancel_tx.send(());
    }
}

pub fn initiate_pairing_to_ip(ip_or_addr: String) {
    println!(
        "Rust API: initiate_pairing_to_ip called with {}",
        ip_or_addr
    );
    RUNTIME.spawn(async move {
        let (ip, port) = if let Some(pos) = ip_or_addr.find(':') {
            let ip = ip_or_addr[..pos].to_string();
            let port = ip_or_addr[pos + 1..].parse::<u16>().unwrap_or(45455);
            (ip, port)
        } else {
            (ip_or_addr, 45455)
        };
        println!(
            "Rust API: initiating manual pairing flow with {}:{}",
            ip, port
        );
        initiate_pairing_flow("manual_connection".to_string(), ip, port).await;
    });
}

pub fn update_local_ip(ip: String) {
    let mut guard = crate::core::discovery::DYNAMIC_LOCAL_IP.lock().unwrap();
    *guard = Some(ip);
}
