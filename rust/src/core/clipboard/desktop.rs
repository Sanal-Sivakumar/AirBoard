use std::time::Duration;
use tokio::time::sleep;
use arboard::Clipboard;
use crate::core::sync_engine::engine::{SYNC_ENGINE, emit_event, SyncEvent};
use crate::core::peer_manager::broadcast_clipboard_update;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

fn rgba_to_png(width: usize, height: usize, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    }
    Ok(png_bytes)
}

fn png_to_rgba(png_bytes: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let bytes = buf[..info.buffer_size()].to_vec();
    Ok((info.width as usize, info.height as usize, bytes))
}

pub async fn start_desktop_clipboard_monitor() {
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

        // 1. Try to get text
        match clipboard.get_text() {
            Ok(content) => {
                if content != last_content && !content.is_empty() {
                    last_content = content.clone();
                    let (is_new, packet_id, timestamp) = SYNC_ENGINE.process_local_change(&content);
                    if is_new {
                        crate::core::clipboard_state::update_clipboard_state(content.clone(), timestamp, packet_id.clone());
                        broadcast_clipboard_update(SYNC_ENGINE.device_id.clone(), packet_id, content.clone(), None);
                        emit_event(SyncEvent::ClipboardUpdated { content, is_local: true });
                    }
                }
            }
            Err(_) => {
                // If it is not text, try to get image
                match clipboard.get_image() {
                    Ok(img) => {
                        if let Ok(png_bytes) = rgba_to_png(img.width, img.height, &img.bytes) {
                            let base64_str = format!("data:image/png;base64,{}", BASE64.encode(&png_bytes));
                            if base64_str != last_content {
                                last_content = base64_str.clone();
                                let (is_new, packet_id, timestamp) = SYNC_ENGINE.process_local_change(&base64_str);
                                if is_new {
                                    crate::core::clipboard_state::update_clipboard_state(base64_str.clone(), timestamp, packet_id.clone());
                                    broadcast_clipboard_update(SYNC_ENGINE.device_id.clone(), packet_id, base64_str.clone(), None);
                                    emit_event(SyncEvent::ClipboardUpdated { content: base64_str, is_local: true });
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore other clipboard types
                    }
                }
            }
        }
    }
}

pub fn write_to_desktop_clipboard(content: String) {
    match Clipboard::new() {
        Ok(mut cb) => {
            if content.starts_with("data:image/png;base64,") {
                let base64_part = &content["data:image/png;base64,".len()..];
                if let Ok(png_bytes) = BASE64.decode(base64_part) {
                    if let Ok((width, height, rgba)) = png_to_rgba(&png_bytes) {
                        let img = arboard::ImageData {
                            width,
                            height,
                            bytes: std::borrow::Cow::Owned(rgba),
                        };
                        if let Err(e) = cb.set_image(img) {
                            emit_event(SyncEvent::Error {
                                message: format!("Failed to write image to clipboard: {}", e),
                            });
                        }
                        return;
                    }
                }
            }
            // Fallback to text
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
