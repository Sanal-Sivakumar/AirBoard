use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

pub static SESSION_KEYS: Lazy<Mutex<HashMap<String, [u8; 32]>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_session_key(device_id: String, key: [u8; 32]) {
    let mut keys = SESSION_KEYS.lock().unwrap();
    keys.insert(device_id, key);
}

pub fn get_session_key(device_id: &str) -> Option<[u8; 32]> {
    let keys = SESSION_KEYS.lock().unwrap();
    keys.get(device_id).cloned()
}

pub fn remove_session(device_id: &str) {
    let mut keys = SESSION_KEYS.lock().unwrap();
    keys.remove(device_id);
}

pub fn clear_all_sessions() {
    let mut keys = SESSION_KEYS.lock().unwrap();
    keys.clear();
}
