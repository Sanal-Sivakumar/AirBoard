use crate::core::peer_manager::android_log;
use crate::core::peer_manager::broadcast_clipboard_update;
use crate::core::peer_manager::MAX_CLIPBOARD_BYTES;
use crate::core::sync_engine::engine::SYNC_ENGINE;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

pub async fn start_android_local_receiver() {
    android_log("Rust: starting Android local socket receiver on 127.0.0.1:45457");
    let addr: SocketAddr = "127.0.0.1:45457".parse().unwrap();
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            android_log(&format!("Rust local receiver bind failed: {:?}", e));
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((mut socket, _)) => {
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    let mut temp = [0u8; 1024];
                    loop {
                        match socket.read(&mut temp).await {
                            Ok(0) => break,
                            Ok(n) => buffer.extend_from_slice(&temp[..n]),
                            Err(e) => {
                                android_log(&format!(
                                    "Rust local receiver socket read error: {:?}",
                                    e
                                ));
                                return;
                            }
                        }
                        if buffer.len() > MAX_CLIPBOARD_BYTES {
                            android_log(
                                "Rust local receiver rejected clipboard content over 512 KiB",
                            );
                            return;
                        }
                    }
                    if let Ok(content) = String::from_utf8(buffer) {
                        if !content.is_empty() {
                            android_log(&format!(
                                "Rust local receiver received clip of {} bytes",
                                content.len()
                            ));
                            let (is_new, packet_id, timestamp) =
                                SYNC_ENGINE.process_local_change(&content);
                            if is_new {
                                crate::core::clipboard_state::update_clipboard_state(
                                    content.clone(),
                                    timestamp,
                                    packet_id.clone(),
                                );
                                broadcast_clipboard_update(
                                    SYNC_ENGINE.device_id.clone(),
                                    packet_id,
                                    content,
                                    None,
                                );
                            }
                        }
                    }
                });
            }
            Err(e) => {
                android_log(&format!("Rust local receiver accept failed: {:?}", e));
            }
        }
    }
}
