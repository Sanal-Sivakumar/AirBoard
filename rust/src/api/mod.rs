use crate::core::sync_engine::engine::{SyncEvent, EVENT_SINK, SYNC_ENGINE};
use crate::core::connection_registry::{get_peers, Peer};
use crate::core::peer_manager::{start_p2p_server, broadcast_clipboard_update, LOCAL_DEVICE_NAME};
use crate::core::discovery::{start_udp_announcer, start_udp_listener};
use crate::core::reconnect::start_reconnect_loop;
use crate::core::heartbeat::start_heartbeat_loop;
use crate::core::clipboard::linux::start_linux_clipboard_monitor;
use flutter_rust_bridge::StreamSink;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

pub fn init_app(sink: StreamSink<SyncEvent>) {
    let mut guard = EVENT_SINK.lock().unwrap();
    *guard = Some(sink);
}

pub fn start_sync(device_name: String, is_android: bool) {
    {
        let mut name_guard = LOCAL_DEVICE_NAME.lock().unwrap();
        *name_guard = device_name.clone();
    }

    RUNTIME.spawn(async move {
        let bound_port = match start_p2p_server(45455).await {
            Ok(p) => p,
            Err(e) => {
                crate::core::sync_engine::engine::emit_event(SyncEvent::Error {
                    message: format!("Server failed to start: {}", e),
                });
                return;
            }
        };

        tokio::spawn(start_udp_announcer(device_name, is_android, bound_port));
        tokio::spawn(start_udp_listener());
        tokio::spawn(start_heartbeat_loop());
        tokio::spawn(start_reconnect_loop());

        if !is_android {
            tokio::spawn(start_linux_clipboard_monitor());
        }
    });
}

pub fn send_local_clipboard_update(content: String) {
    let (is_new, packet_id) = SYNC_ENGINE.process_local_change(&content);
    if is_new {
        broadcast_clipboard_update(SYNC_ENGINE.device_id.clone(), packet_id, content, None);
    }
}

pub fn get_device_id() -> String {
    SYNC_ENGINE.device_id.clone()
}

pub fn get_discovered_peers() -> Vec<Peer> {
    get_peers()
}
