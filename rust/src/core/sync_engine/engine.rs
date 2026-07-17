use crate::core::utils::helpers::compute_hash;
use once_cell::sync::{Lazy, OnceCell};
use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncEvent {
    ClipboardUpdated { content: String, is_local: bool },
    ConnectionStatus { connected: bool, message: String },
    Error { message: String },
}

pub static DEVICE_ID: OnceCell<String> = OnceCell::new();

pub fn set_my_device_id(id: String) {
    let _ = DEVICE_ID.set(id);
}

pub struct SyncEngine {
    pub device_id: String,
    last_synced_hash: Mutex<String>,
    processed_packet_ids: Mutex<(HashSet<String>, VecDeque<String>)>,
}

impl SyncEngine {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            last_synced_hash: Mutex::new(String::new()),
            processed_packet_ids: Mutex::new((HashSet::new(), VecDeque::new())),
        }
    }

    pub fn process_incoming_packet(&self, packet_id: &str) -> bool {
        let mut cache = self.processed_packet_ids.lock().unwrap();
        if cache.0.contains(packet_id) {
            return false;
        }

        cache.0.insert(packet_id.to_string());
        cache.1.push_back(packet_id.to_string());
        if cache.1.len() > 4096 {
            if let Some(expired) = cache.1.pop_front() {
                cache.0.remove(&expired);
            }
        }
        true
    }

    pub fn process_local_change(&self, content: &str) -> (bool, String, i64) {
        let local_hash = compute_hash(content);
        let mut last_hash = self.last_synced_hash.lock().unwrap();

        if local_hash == *last_hash {
            return (false, String::new(), 0);
        }

        *last_hash = local_hash;

        let packet_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let mut cache = self.processed_packet_ids.lock().unwrap();
        cache.0.insert(packet_id.clone());
        cache.1.push_back(packet_id.clone());
        if cache.1.len() > 4096 {
            if let Some(expired) = cache.1.pop_front() {
                cache.0.remove(&expired);
            }
        }

        (true, packet_id, now)
    }
}

pub static SYNC_ENGINE: Lazy<SyncEngine> = Lazy::new(|| {
    let id = DEVICE_ID
        .get()
        .cloned()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    SyncEngine::new(id)
});

pub static EVENT_SINK: Lazy<Mutex<Option<crate::StreamSink<SyncEvent>>>> =
    Lazy::new(|| Mutex::new(None));

pub fn emit_event(event: SyncEvent) {
    if let Some(sink) = EVENT_SINK.lock().unwrap().as_ref() {
        let _ = sink.add(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppresses_duplicate_local_content_and_packet_ids() {
        let engine = SyncEngine::new("test-device".to_string());
        assert!(engine.process_local_change("first").0);
        assert!(!engine.process_local_change("first").0);
        assert!(engine.process_local_change("second").0);

        assert!(engine.process_incoming_packet("packet-1"));
        assert!(!engine.process_incoming_packet("packet-1"));
    }
}
