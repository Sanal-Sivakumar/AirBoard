# AirBoard: Troubleshooting & Development History Log

This document chronicles the step-by-step evolution of AirBoard from a simple client-server prototype to a secure, decentralized cross-platform clipboard synchronization network. It details the technical challenges, OS constraints, network limitations, and system-level panics encountered in each phase, along with the engineering solutions developed to solve them.

---

## 🗺️ Chronological Development Journey

```
  Phase 1-2            Phases 3-4          Phases 5-7          Phases 8-10         Phases 11-15
  Client-Server  ===>  E2EE Security ===>  Background    ===>  Hotspot Sync  ===>  Aurora UI,
  to P2P Mesh          Pairing Keys        Loopbacks & BAL     & Audio Loops       Images & History
```

---

## 🛠️ Phase 1 & 2: Client-Server to P2P Mesh Sync

The project began as a basic Client-Server prototype: an Android host running a WebSocket server, and a Linux client connecting to it. We soon migrated this to a serverless Peer-to-Peer (P2P) architecture where every device ran a WebSocket server, a UDP announcement loop, and client connection routines.

### Challenge 1.1: The Echo Loop (Feedback Storm)
*   **Symptom**: Copying text on Device A successfully updated Device B. However, writing the text to B's system pasteboard triggered B's native clipboard listener, which interpreted it as a new copy, sent it back to A, and caused an infinite broadcast loop that crashed both client sockets.
*   **Root Cause**: Native operating system clipboard managers do not expose the identity of the process writing the text. Programmatic writes (updates) and user copies look identical to system listeners.
*   **Resolution**: We developed a local **Deduplication Engine** (`sync_engine/engine.rs`):
    1. The sync engine computes the SHA-256 hash of any incoming clipboard update before writing it to the system pasteboard.
    2. This hash is cached in a thread-safe `last_received_hash` register.
    3. The native clipboard listener checks any new clip changes against this register. If the hash matches, the change is ignored as a self-triggered update.

### Challenge 1.2: AP Isolation & Blocked UDP Broadcasts
*   **Symptom**: Auto-discovery failed when devices connected to public networks, university Wi-Fi, or enterprise routers.
*   **Root Cause**: For security and to reduce local network clutter, many routers implement **AP Client Isolation** or block **UDP Multicast / Broadcast** packets on the default discovery port (`45454`/`45455`).
*   **Resolution**: We implemented a manual connection bridge:
    1. The app settings pane queries local network interfaces to resolve and display the host's current local IP address.
    2. We added a manual IP address entry field in the UI.
    3. Entering a peer's IP directly opens a TCP unicast connection to port `45457`, completely bypassing the blocked UDP multicast layer.

### Challenge 1.3: Symmetric Peer Connection Race Conditions
*   **Symptom**: When A and B discovered each other simultaneously, they both attempted client connection tasks. This created two concurrent connections between the same pair of devices, causing duplicate packet syncing and thread contention.
*   **Root Cause**: In a decentralized P2P network, there is no coordinator server to manage who initiates a stream.
*   **Resolution**: We implemented a **Lexicographical Tie-Breaking Algorithm**:
    1. Every device generates a permanent, unique UUID (`device_id`) on startup.
    2. When A and B discover each other, they compare their `device_id` strings alphabetically.
    3. The device with the **alphabetically smaller** ID acts as the client initiator (initiating the WebSocket connection).
    4. The device with the **alphabetically larger** ID only listens for the incoming connection.
    5. This guarantees that exactly one stable TCP session is established between any two peers.

---

## 🔒 Phases 3 & 4: Secure Pairing & iPadOS Support

As the mesh was established, security and iPadOS platform compatibility became the primary goals.

### Challenge 3.1: Ephemeral Diffie-Hellman MITM Vulnerabilities
*   **Symptom**: While Diffie-Hellman exchanges allow two devices to agree on an encrypted session key over an insecure network, the key exchange alone is vulnerable to interceptors.
*   **Root Cause**: The raw key exchange has no built-in identity authentication. A malicious device on the Wi-Fi could intercept the public keys and establish separate encrypted sessions with both peers (Man-in-the-Middle).
*   **Resolution**: We implemented a **Zero-Trust Station-to-Station (STS) Protocol**:
    1. Every device maintains permanent Ed25519 (signing) and X25519 (DH) identity keypairs stored in secure hardware via `flutter_secure_storage`.
    2. During initial pairing, devices exchange public keys and generate a SHA-256 fingerprint.
    3. The user manually verifies this 16-byte hex fingerprint on both screens before approving the pairing, saving the peer's public key to a local `trust_store.json`.
    4. During session handshakes, peers sign their ephemeral DH keys using their private Ed25519 keys. The recipient verifies this signature using the stored public key from `trust_store.json`, validating the identity of the device.
    5. Payloads are encrypted using **ChaCha20-Poly1305** authenticated symmetric encryption, featuring random 96-bit nonces to prevent packet replay attacks.

### Challenge 4.1: iOS Background Suspensions
*   **Symptom**: When the iPadOS app was minimized, all active connections closed instantly.
*   **Root Cause**: Apple's operating system suspends or kills background sockets and threads within 10–30 seconds to conserve battery and memory.
*   **Resolution**:
    1. We restricted the iPadOS platform to run strictly as a client-only peer (avoiding background server bindings).
    2. We implemented lifecycle observers that suspend discovery and close active connections cleanly when backgrounded, then immediately trigger reconnection loops (`trigger_reconnect()`) when brought back to the foreground.

---

## ⚡ Phases 5 - 7: Tokio Runtime Panics & Android Background Sync

Deploying the code in release environments exposed async thread conflicts and background clipboard restrictions on Android 10+.

### Challenge 5.1: Tokio Reactor Panic
*   **Symptom**: Minimizing and reopening the app on Android or Linux caused the application to crash with a `PanicException`: `"no reactor running; tokio::spawn must be called from the context of a Tokio 1.x runtime"`.
*   **Root Cause**: Dart-to-Rust calls (like triggering reconnections) execute on external threads managed by the Dart VM. Calling `tokio::spawn` from these foreign OS threads failed because they lacked access to the static Tokio event loop running inside the Rust environment.
*   **Resolution**: We bound task spawning directly to our custom globally initialized static runtime pointer (`crate::api::RUNTIME`):
    ```rust
    crate::api::RUNTIME.spawn(async move { ... });
    ```

### Challenge 5.2: Android 10+ Background Clipboard Blocks
*   **Symptom**: The Android background service remained connected to the network, but incoming clipboard updates could not be written to the system clipboard when the app was minimized.
*   **Root Cause**: Starting with Android 10, Google restricts background applications from reading/writing to the system pasteboard for user privacy.
*   **Resolution**:
    1. **Direct TCP Loopback (`127.0.0.1:45457`)**: The background Rust core runs a local TCP server on localhost.
    2. **Background Activities**: When Android is minimized, incoming clipboard packets from the network are received by Rust and sent over localhost to the Kotlin foreground service (`ClipboardSyncService`).
    3. **Overlay & Focus Grabbing**: Kotlin checks for "Display over other apps" (Overlay/`SYSTEM_ALERT_WINDOW`) permission. If active, it launches an invisible, transparent activity (`ClipboardWriteActivity`) that briefly takes window focus in the background, writes the synced text to the clipboard manager, and terminates instantly.
    4. **Manual Notification Action Fallback**: If overlay permission is disabled, the app falls back to updating the persistent notification with a **"Copy"** button that manually invokes the transparent activity to write the text when clicked.

---

## 📡 Phases 8 - 10: Host Hotspots, Audio Loops & Connection Persistence

To enable iPad-to-Android direct syncing under mobile hotspot scenarios, we had to address network broadcast locks.

### Challenge 8.1: Mobile Hotspot Discovery Isolation
*   **Symptom**: An iPad and an Android device connected directly via a mobile hotspot could not discover each other, as discovery packets were dropped.
*   **Root Cause**: Mobile operating systems shut down Wi-Fi multicast and broadcast capabilities during lock screens and sleep cycles to extend battery. Additionally, hotspot interfaces route packets differently than home Wi-Fi networks.
*   **Resolution**:
    1. **Android Multicast Lock**: We added `CHANGE_WIFI_MULTICAST_STATE` to the Android Manifest. The Kotlin service acquires a native `WifiManager.MulticastLock` on startup, forcing the Wi-Fi hardware to remain open to UDP announcements.
    2. **iOS Background Audio Loop**: We added the `audio` background mode key in `Info.plist`. The app plays a silent, infinite audio loop via `AVAudioPlayer` in the background, preventing iOS from suspending the WebSocket client connection.
    3. **IP Persistence**: We modified `trust_store.json` to store `last_ip` and `last_port` for each paired peer. If UDP discovery fails, the reconnection loop tries direct TCP connections to these coordinates automatically.
    4. **Notification Replacement ID**: To prevent sync alert notifications from cluttering the iOS Notification Center, we replaced the random UUID request identifier with a static string (`"airboard_clipboard_sync"`), ensuring only the latest synced message remains visible.

---

## 🎨 Phases 11 - 15: Aurora Redesign, Subnets & Image Synchronization

The final stages introduced a responsive UI redesign, subnet discovery adjustments, and multi-format pasteboard synchronization.

### Challenge 11.1: Asymmetric Network Discovery
*   **Symptom**: Paired devices remained "Disconnected" because Device A discovered B, but B could not discover A. Because the tie-breaker rule (`local_id >= peer_id`) blocked A from initiating the connection, no connection was ever opened.
*   **Root Cause**: One-way UDP packet routing on isolated local networks.
*   **Resolution**: We removed the static lexicographical tie-breaker check from `connect_to_peer()`. Since duplicate connection attempts are already handled by thread-safe `ACTIVE_PEERS` registry checks, allowing either side to initiate the TCP socket resolved the connection block.

### Challenge 11.2: Multi-Platform Image Synchronization
*   **Symptom**: Users wanted to sync images and screenshots, but our network protocol was built for text messages.
*   **Root Cause**: Sending binary streams requires changing the underlying serialization libraries, which can break compatibility with older clients.
*   **Resolution**:
    1. We added image capture hooks on all platforms. If a user copies an image, the clipboard monitor encodes it to a Base64 PNG string (`data:image/png;base64,...`).
    2. This text string is sent over the normal cryptographic transport.
    3. When a peer receives the payload, if it detects the image header, it decodes the base64 string back into raw image bytes and writes it natively using the system's image clipboard APIs (`arboard::Clipboard::set_image` on Desktop and `UIPasteboard.general.image` on iOS).

### Challenge 11.3: Subnet Discovery Routing
*   **Symptom**: Devices connected to different subnet masks (e.g. Wi-Fi repeaters) were isolated from UDP broadcasts.
*   **Root Cause**: The global broadcast address (`255.255.255.255`) is blocked by many network repeaters and switches.
*   **Resolution**: We modified the discovery module (`discovery/mod.rs`) to query the OS routing table for the active interface's IP, calculate the local subnet broadcast address (e.g. `192.168.1.255`), and broadcast to both the global and subnet-specific addresses.
