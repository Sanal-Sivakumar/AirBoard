# SyncBoard - Secure End-to-End Encrypted Clipboard Sync (Flutter + Rust)

SyncBoard is a highly secure, local-network Peer-to-Peer (P2P) clipboard synchronization system. It features automatic local network device discovery, secure manual device pairing, and end-to-end (E2E) payload encryption.

---

## 1. Security Architecture

SyncBoard utilizes a trusted-device model to ensure clipboard contents are never leaked or intercepted:

1. **Cryptographic Identity**:
   - Each device generates a permanent UUID and two keypairs on its first startup:
     - **Ed25519** keypair for connection handshakes and payload signatures.
     - **X25519** keypair for ephemeral Diffie-Hellman session key exchanges.
   - Keys are stored securely on the host platform using the Android Keystore (Android) and Linux Keyring/Keychain (Linux) via `flutter_secure_storage`.

2. **Manual Device Pairing Flow**:
   - Discovered devices are initially flagged as **Unpaired**.
   - Initiating pairing opens a temporary connection to exchange public keys and display a visual SHA-256 fingerprint (`SHA256(public_signing_key)`) on both screens.
   - Once approved by the recipient, public keys are persisted permanently in `trust_store.json`.

3. **Authenticated Session Handshake (Station-to-Station)**:
   - When connecting, trusted peers exchange signed ephemeral X25519 public keys (`Handshake1` and `Handshake2`).
   - Signature checks verify that both devices own the private keys corresponding to their paired public keys.
   - A shared session key is derived via Diffie-Hellman, hashed using SHA-256, and used to encrypt all data traffic.

4. **Payload Encryption**:
   - All clipboard contents and heartbeats are encrypted using **ChaCha20-Poly1305** authenticated symmetric encryption.
   - Sockets transmit only encrypted envelopes:
     ```json
     {
       "type": "encrypted_payload",
       "sender": "sender-device-id",
       "nonce": "base64-nonce",
       "ciphertext": "base64-ciphertext"
     }
     ```
   - When a peer forwards a clipboard update to other mesh nodes, it decrypts it, validates it, and re-encrypts it using the specific session key of each destination peer. Plaintext is never forwarded or exposed.

---

## 2. Installation & Prerequisites

### Linux Dependencies
Install standard compilation headers for Linux:
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libx11-dev libxcb1-dev
```

### Rust Cryptography Crates
SyncBoard uses high-performance, compilation-friendly, pure-Rust cryptographic backends:
- `chacha20poly1305`
- `ed25519-dalek`
- `x25519-dalek`
- `rand`
- `base64`

Configure targets for compilation:
```bash
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
```

---

## 3. Running & Testing

1. **Perform Code Generation**:
   ```bash
   flutter_rust_bridge_codegen generate
   ```

2. **Run the Apps**:
   - On Linux: `flutter run -d linux`
   - On Android: `flutter run -d android`

3. **Security Testing Steps**:
   - **Step 1 (Discovery)**: Enable sync on both devices. Verify they show up in each other's "Devices" list as unpaired.
   - **Step 2 (Clipboard Isolation)**: Copy text on Device A. Verify that Device B does **NOT** receive the text (since they are unpaired).
   - **Step 3 (Pairing)**: Tap "Pair Device" on Device A. Verify a modal dialog pops up on B displaying A's device name and fingerprint.
   - **Step 4 (Fingerprint Check)**: Match B's prompt fingerprint against A's header fingerprint. Tap **Approve** on B.
   - **Step 5 (Encrypted Sync)**: Verify both devices list each other in "Trusted Peers" and establish a "Secure session". Copy text on A and verify B's clipboard updates instantly.
   - **Step 6 (Unpairing)**: Go to "Trusted Peers" tab on B and tap the delete button. Confirm unpairing. Copy text on A and verify B does not receive it anymore.
