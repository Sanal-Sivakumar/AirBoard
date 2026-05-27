use std::sync::Mutex;
use once_cell::sync::Lazy;
use rand::{thread_rng, RngCore};
use sha2::{Sha256, Digest};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{StaticSecret, PublicKey as XPublicKey};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, AeadCore, aead::Aead};

pub static MY_SIGNING_KEY: Lazy<Mutex<Option<SigningKey>>> = Lazy::new(|| Mutex::new(None));
pub static MY_DH_KEY: Lazy<Mutex<Option<StaticSecret>>> = Lazy::new(|| Mutex::new(None));

pub fn register_identity_keys(signing_key_bytes: [u8; 32], dh_key_bytes: [u8; 32]) -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::from_bytes(&signing_key_bytes);
    let dh_key = StaticSecret::from(dh_key_bytes);

    let pub_signing = signing_key.verifying_key().to_bytes().to_vec();
    let pub_dh = XPublicKey::from(&dh_key).as_bytes().to_vec();

    let mut sig_guard = MY_SIGNING_KEY.lock().unwrap();
    *sig_guard = Some(signing_key);

    let mut dh_guard = MY_DH_KEY.lock().unwrap();
    *dh_guard = Some(dh_key);

    (pub_signing, pub_dh)
}

pub fn get_my_public_keys() -> Option<(Vec<u8>, Vec<u8>)> {
    let sig_guard = MY_SIGNING_KEY.lock().unwrap();
    let dh_guard = MY_DH_KEY.lock().unwrap();
    
    if let (Some(sig), Some(dh)) = (sig_guard.as_ref(), dh_guard.as_ref()) {
        let pub_signing = sig.verifying_key().to_bytes().to_vec();
        let pub_dh = XPublicKey::from(dh).as_bytes().to_vec();
        Some((pub_signing, pub_dh))
    } else {
        None
    }
}

pub fn sign_message(message: &[u8]) -> Result<[u8; 64], String> {
    let sig_guard = MY_SIGNING_KEY.lock().unwrap();
    let signing_key = sig_guard.as_ref().ok_or("Signing key not registered")?;
    let signature = signing_key.sign(message);
    Ok(signature.to_bytes())
}

pub fn verify_message_signature(pub_key_bytes: &[u8; 32], message: &[u8], signature_bytes: &[u8; 64]) -> bool {
    if let Ok(verifying_key) = VerifyingKey::from_bytes(pub_key_bytes) {
        let signature = Signature::from_bytes(signature_bytes);
        verifying_key.verify(message, &signature).is_ok()
    } else {
        false
    }
}

pub fn compute_shared_secret(their_public_dh_bytes: &[u8; 32]) -> Result<[u8; 32], String> {
    let dh_guard = MY_DH_KEY.lock().unwrap();
    let my_dh = dh_guard.as_ref().ok_or("DH key not registered")?;
    
    let their_pub = XPublicKey::from(*their_public_dh_bytes);
    let shared = my_dh.diffie_hellman(&their_pub);
    
    // Hash shared secret with SHA-256 to derive session key
    let mut hasher = Sha256::new();
    hasher.update(shared.as_bytes());
    let mut session_key = [0u8; 32];
    session_key.copy_from_slice(&hasher.finalize());
    
    Ok(session_key)
}

pub fn chacha_encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), String> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let mut nonce = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce);
    
    let ciphertext = cipher.encrypt(&nonce.into(), plaintext)
        .map_err(|e| format!("Encryption error: {:?}", e))?;
        
    Ok((ciphertext, nonce))
}

pub fn chacha_decrypt(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    let cipher = ChaCha20Poly1305::new(key.into());
    
    let plaintext = cipher.decrypt(nonce.into(), ciphertext)
        .map_err(|e| format!("Decryption error: {:?}", e))?;
        
    Ok(plaintext)
}
