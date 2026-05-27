use tokio::sync::broadcast;
use once_cell::sync::Lazy;

pub mod server;
pub mod client;

pub static OUTBOUND_BROADCAST: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(100);
    tx
});
