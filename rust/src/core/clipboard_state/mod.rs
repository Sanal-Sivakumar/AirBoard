use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClipboardState {
    pub content: String,
    pub timestamp: i64,
    pub packet_id: String,
}

pub static CLIPBOARD_STATE: Lazy<Mutex<ClipboardState>> = Lazy::new(|| {
    Mutex::new(ClipboardState {
        content: String::new(),
        timestamp: 0,
        packet_id: String::new(),
    })
});

pub fn get_clipboard_state() -> ClipboardState {
    let state = CLIPBOARD_STATE.lock().unwrap();
    state.clone()
}

pub fn update_clipboard_state(content: String, timestamp: i64, packet_id: String) {
    let mut state = CLIPBOARD_STATE.lock().unwrap();
    if timestamp > state.timestamp {
        state.content = content;
        state.timestamp = timestamp;
        state.packet_id = packet_id;
    }
}
