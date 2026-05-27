use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardUpdate {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub device_id: String,
    pub content: String,
    pub timestamp: i64,
}

impl ClipboardUpdate {
    pub fn new(device_id: String, content: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Self {
            msg_type: "clipboard_update".to_string(),
            device_id,
            content,
            timestamp,
        }
    }
}
