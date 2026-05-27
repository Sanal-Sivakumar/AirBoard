use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::core::utils::helpers::compute_hash;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncEvent {
    ClipboardUpdated { content: String },
    ConnectionStatus { connected: bool, message: String },
    Error { message: String },
}

pub struct SyncEngine {
    pub device_id: String,
    last_synced_hash: Mutex<String>,
    processed_packet_ids: Mutex<Vec<String>>,
}

impl SyncEngine {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            last_synced_hash: Mutex::new(String::new()),
            processed_packet_ids: Mutex::new(Vec::new()),
        }
    }

    pub fn process_incoming_packet(&self, packet_id: &str) -> bool {
        let mut cache = self.processed_packet_ids.lock().unwrap();
        if cache.contains(&packet_id.to_string()) {
            return false;
        }

        cache.push(packet_id.to_string());
        if cache.len() > 100 {
            cache.remove(0);
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
        cache.push(packet_id.clone());
        if cache.len() > 100 {
            cache.remove(0);
        }

        (true, packet_id, now)
    }
}

pub static SYNC_ENGINE: Lazy<SyncEngine> = Lazy::new(|| {
    let device_id = uuid::Uuid::new_v4().to_string();
    SyncEngine::new(device_id)
});

pub static EVENT_SINK: Lazy<Mutex<Option<crate::StreamSink<SyncEvent>>>> = Lazy::new(|| Mutex::new(None));

pub fn emit_event(event: SyncEvent) {
    if let Some(sink) = EVENT_SINK.lock().unwrap().as_ref() {
        let _ = sink.add(event);
    }
}
