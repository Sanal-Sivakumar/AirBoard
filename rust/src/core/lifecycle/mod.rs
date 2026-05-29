use std::sync::Mutex;
use once_cell::sync::Lazy;
use tokio::task::JoinHandle;
use crate::core::discovery::{start_udp_announcer, start_udp_listener};
use crate::core::heartbeat::start_heartbeat_loop;
use crate::core::reconnect::{start_reconnect_loop, trigger_reconnect};
use crate::core::peer_manager::{ACTIVE_PEERS, LOCAL_DEVICE_NAME};

pub static IS_CLIENT_ONLY: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
pub static IS_ACTIVE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));

struct TaskManager {
    udp_announcer: Option<JoinHandle<()>>,
    udp_listener: Option<JoinHandle<()>>,
    heartbeat_loop: Option<JoinHandle<()>>,
    reconnect_loop: Option<JoinHandle<()>>,
}

static TASK_MANAGER: Lazy<Mutex<TaskManager>> = Lazy::new(|| Mutex::new(TaskManager {
    udp_announcer: None,
    udp_listener: None,
    heartbeat_loop: None,
    reconnect_loop: None,
}));

pub fn set_client_only(client_only: bool) {
    let mut guard = IS_CLIENT_ONLY.lock().unwrap();
    *guard = client_only;
}

pub fn register_initial_handles(
    udp_announcer: JoinHandle<()>,
    udp_listener: JoinHandle<()>,
    heartbeat_loop: JoinHandle<()>,
    reconnect_loop: JoinHandle<()>,
) {
    let mut tm = TASK_MANAGER.lock().unwrap();
    
    // Stop any stale tasks first
    if let Some(h) = tm.udp_announcer.take() { h.abort(); }
    if let Some(h) = tm.udp_listener.take() { h.abort(); }
    if let Some(h) = tm.heartbeat_loop.take() { h.abort(); }
    if let Some(h) = tm.reconnect_loop.take() { h.abort(); }

    tm.udp_announcer = Some(udp_announcer);
    tm.udp_listener = Some(udp_listener);
    tm.heartbeat_loop = Some(heartbeat_loop);
    tm.reconnect_loop = Some(reconnect_loop);
}

pub fn handle_app_foreground() {
    let mut active = IS_ACTIVE.lock().unwrap();
    if *active {
        return; // already active
    }
    *active = true;

    let mut tm = TASK_MANAGER.lock().unwrap();
    
    // Stop any stale tasks first
    if let Some(h) = tm.udp_announcer.take() { h.abort(); }
    if let Some(h) = tm.udp_listener.take() { h.abort(); }
    if let Some(h) = tm.heartbeat_loop.take() { h.abort(); }
    if let Some(h) = tm.reconnect_loop.take() { h.abort(); }

    let device_name = {
        let name_guard = LOCAL_DEVICE_NAME.lock().unwrap();
        name_guard.clone()
    };
    
    let is_client_only = {
        let guard = IS_CLIENT_ONLY.lock().unwrap();
        *guard
    };

    let platform = if is_client_only {
        "ios".to_string()
    } else if cfg!(target_os = "android") {
        "android".to_string()
    } else {
        "linux".to_string()
    };

    let ws_port = if is_client_only { 0 } else { 45455 };

    // Spawn tasks
    let rt = crate::api::RUNTIME.handle();
    tm.udp_announcer = Some(rt.spawn(start_udp_announcer(device_name, platform, ws_port)));
    tm.udp_listener = Some(rt.spawn(start_udp_listener()));
    tm.heartbeat_loop = Some(rt.spawn(start_heartbeat_loop()));
    tm.reconnect_loop = Some(rt.spawn(start_reconnect_loop()));

    // Trigger immediate reconnect
    trigger_reconnect();
}

pub fn handle_app_background() {
    let mut active = IS_ACTIVE.lock().unwrap();
    if !*active {
        return; // already paused
    }
    *active = false;

    // Abort tasks
    let mut tm = TASK_MANAGER.lock().unwrap();
    if let Some(h) = tm.udp_announcer.take() { h.abort(); }
    if let Some(h) = tm.udp_listener.take() { h.abort(); }
    if let Some(h) = tm.heartbeat_loop.take() { h.abort(); }
    if let Some(h) = tm.reconnect_loop.take() { h.abort(); }

    // Close connections
    let mut peers = ACTIVE_PEERS.lock().unwrap();
    for (_, conn) in peers.drain() {
        let _ = conn.cancel_tx.send(());
    }
}
