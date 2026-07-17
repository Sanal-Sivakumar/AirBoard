use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Clone)]
pub struct SessionState {
    pub key: [u8; 32],
    pub next_send_sequence: u64,
    pub highest_received_sequence: u64,
    pub received_sequences: HashSet<u64>,
}

pub static SESSIONS: Lazy<Mutex<HashMap<String, SessionState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_session_key(device_id: String, key: [u8; 32]) {
    let mut sessions = SESSIONS.lock().unwrap();
    sessions.insert(
        device_id,
        SessionState {
            key,
            next_send_sequence: 1,
            highest_received_sequence: 0,
            received_sequences: HashSet::new(),
        },
    );
}

pub fn get_session_key(device_id: &str) -> Option<[u8; 32]> {
    let sessions = SESSIONS.lock().unwrap();
    sessions.get(device_id).map(|session| session.key)
}

pub fn next_send_sequence(device_id: &str) -> Option<u64> {
    let mut sessions = SESSIONS.lock().unwrap();
    let session = sessions.get_mut(device_id)?;
    let sequence = session.next_send_sequence;
    session.next_send_sequence = session.next_send_sequence.checked_add(1)?;
    Some(sequence)
}

pub fn accept_received_sequence(device_id: &str, sequence: u64) -> bool {
    let mut sessions = SESSIONS.lock().unwrap();
    let Some(session) = sessions.get_mut(device_id) else {
        return false;
    };
    const REPLAY_WINDOW: u64 = 1024;
    if sequence == 0
        || sequence
            <= session
                .highest_received_sequence
                .saturating_sub(REPLAY_WINDOW)
        || session.received_sequences.contains(&sequence)
    {
        return false;
    }
    session.received_sequences.insert(sequence);
    session.highest_received_sequence = session.highest_received_sequence.max(sequence);
    let oldest_allowed = session
        .highest_received_sequence
        .saturating_sub(REPLAY_WINDOW);
    session
        .received_sequences
        .retain(|seen| *seen > oldest_allowed);
    true
}

pub fn remove_session(device_id: &str) {
    let mut sessions = SESSIONS.lock().unwrap();
    sessions.remove(device_id);
}

pub fn clear_all_sessions() {
    let mut sessions = SESSIONS.lock().unwrap();
    sessions.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_window_rejects_duplicates_and_accepts_bounded_reordering() {
        let peer = format!("test-peer-{}", uuid::Uuid::new_v4());
        register_session_key(peer.clone(), [3u8; 32]);

        assert!(accept_received_sequence(&peer, 2));
        assert!(accept_received_sequence(&peer, 1));
        assert!(!accept_received_sequence(&peer, 2));
        assert!(accept_received_sequence(&peer, 2_000));
        assert!(!accept_received_sequence(&peer, 1));

        remove_session(&peer);
    }

    #[test]
    fn send_sequences_are_monotonic() {
        let peer = format!("test-peer-{}", uuid::Uuid::new_v4());
        register_session_key(peer.clone(), [4u8; 32]);
        assert_eq!(next_send_sequence(&peer), Some(1));
        assert_eq!(next_send_sequence(&peer), Some(2));
        remove_session(&peer);
    }
}
