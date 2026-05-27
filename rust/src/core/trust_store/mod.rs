use std::sync::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrustedDevice {
    pub device_id: String,
    pub device_name: String,
    pub public_signing_key: [u8; 32],
    pub public_dh_key: [u8; 32],
    pub paired_at: u64,
}

pub static TRUST_STORE: Lazy<Mutex<HashMap<String, TrustedDevice>>> = Lazy::new(|| Mutex::new(HashMap::new()));
pub static DB_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

pub fn init_trust_store(storage_dir: String) {
    let mut path = PathBuf::from(storage_dir);
    path.push("trust_store.json");
    
    let mut db_guard = DB_PATH.lock().unwrap();
    *db_guard = Some(path.clone());
    
    if path.exists() {
        if let Ok(mut file) = File::open(&path) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(loaded) = serde_json::from_str::<HashMap<String, TrustedDevice>>(&content) {
                    let mut store = TRUST_STORE.lock().unwrap();
                    *store = loaded;
                }
            }
        }
    }
}

pub fn save_trust_store() {
    let path_guard = DB_PATH.lock().unwrap();
    if let Some(ref path) = *path_guard {
        let store = TRUST_STORE.lock().unwrap();
        if let Ok(json_str) = serde_json::to_string(&*store) {
            if let Ok(mut file) = File::create(path) {
                let _ = file.write_all(json_str.as_bytes());
            }
        }
    }
}

pub fn add_trusted_device(device: TrustedDevice) {
    {
        let mut store = TRUST_STORE.lock().unwrap();
        store.insert(device.device_id.clone(), device);
    }
    save_trust_store();
}

pub fn remove_trusted_device(device_id: &str) {
    {
        let mut store = TRUST_STORE.lock().unwrap();
        store.remove(device_id);
    }
    save_trust_store();
}

pub fn is_device_trusted(device_id: &str) -> bool {
    let store = TRUST_STORE.lock().unwrap();
    store.contains_key(device_id)
}

pub fn get_trusted_device(device_id: &str) -> Option<TrustedDevice> {
    let store = TRUST_STORE.lock().unwrap();
    store.get(device_id).cloned()
}

pub fn get_all_trusted_devices() -> Vec<TrustedDevice> {
    let store = TRUST_STORE.lock().unwrap();
    store.values().cloned().collect()
}
