use std::time::Duration;
use tokio::time::sleep;
use arboard::Clipboard;
use crate::core::sync_engine::engine::{SYNC_ENGINE, emit_event, SyncEvent};
use crate::core::peer_manager::broadcast_clipboard_update;

pub async fn start_linux_clipboard_monitor() {
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            emit_event(SyncEvent::Error {
                message: format!("Failed to initialize clipboard: {}", e),
            });
            return;
        }
    };

    let mut last_content = String::new();

    loop {
        sleep(Duration::from_millis(500)).await;

        match clipboard.get_text() {
            Ok(content) => {
                if content != last_content {
                    last_content = content.clone();
                    let (is_new, packet_id, timestamp) = SYNC_ENGINE.process_local_change(&content);
                    if is_new {
                        crate::core::clipboard_state::update_clipboard_state(content.clone(), timestamp, packet_id.clone());
                        broadcast_clipboard_update(SYNC_ENGINE.device_id.clone(), packet_id, content.clone(), None);
                        emit_event(SyncEvent::ClipboardUpdated { content });
                    }
                }
            }
            Err(_) => {
                // Ignore errors related to non-text content types
            }
        }
    }
}

pub fn write_to_linux_clipboard(content: String) {
    match Clipboard::new() {
        Ok(mut cb) => {
            if let Err(e) = cb.set_text(content) {
                emit_event(SyncEvent::Error {
                    message: format!("Failed to write to clipboard: {}", e),
                });
            }
        }
        Err(e) => {
            emit_event(SyncEvent::Error {
                message: format!("Failed to initialize clipboard: {}", e),
            });
        }
    }
}
