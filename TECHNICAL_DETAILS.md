# AirBoard: Comprehensive Technical Reference Guide

Welcome to the technical reference manual for **AirBoard**. This document is written as a structured, textbook-style guide. It is designed to explain the networking, cryptographic, systems engineering, and multi-language communication protocols used in AirBoard in a way that is accessible to developers and students alike—even if you have zero background in computer networking or cryptography.

---

## Table of Contents
1. [Network Topology: Decentralized Peer-to-Peer (P2P)](#1-network-topology-decentralized-peer-to-peer-p2p)
2. [Peer Discovery Protocol: UDP Broadcast & IP Fallback](#2-peer-discovery-protocol-udp-broadcast--ip-fallback)
3. [Zero-Trust Cryptography & End-to-End Encryption (E2EE)](#3-zero-trust-cryptography--end-to-end-encryption-e2ee)
4. [Cross-Language Architecture: Flutter & Rust Bridge (FRB v2)](#4-cross-language-architecture-flutter--rust-bridge-frb-v2)
5. [Platform native integrations & Clipboard Monitors](#5-platform-native-integrations--clipboard-monitors)
6. [Deduplication & Feedback Loop Prevention](#6-deduplication--feedback-loop-prevention)

---

## 1. Network Topology: Decentralized Peer-to-Peer (P2P)

### Client-Server vs. Peer-to-Peer
Most modern applications use a **Client-Server** model. If you copy a text on your phone and want to paste it on your laptop, the standard approach is:
1. **Client (Phone)** uploads the text to a central database server (e.g., hosted on AWS or Google Cloud).
2. **Server** stores the text in a database.
3. **Client (Laptop)** polls the server or listens for updates, and downloads the text.

```
Client-Server Model:
[Phone (Client)] -----(Uploads Text)-----> [Central Cloud Server]
                                                    |
[Laptop (Client)] <---(Downloads Text)--------------+
```

While simple, this model has massive downsides:
*   **Privacy Exposure**: Your private clipboard (often containing passwords, keys, or sensitive texts) passes through a third-party server.
*   **Internet Dependency**: If your internet is down, or if the server goes offline, synchronization fails—even if your phone and laptop are sitting right next to each other on the same desk.
*   **Latency**: The data must travel to a remote datacenter and back, causing visible lag.

### The Peer-to-Peer (P2P) Solution
AirBoard operates on a **decentralized, serverless P2P** topology. Devices communicate directly with one another over the Local Area Network (LAN). 

```
Peer-to-Peer (P2P) Model:
[Phone (Peer)] <=========(Direct Encrypted LAN Connection)=========> [Laptop (Peer)]
```

When you copy text on your phone, it transmits the payload directly to your laptop over your local Wi-Fi or hotspot.
*   **Zero Middlemen**: Your clipboard never leaves your physical local network.
*   **Offline Operation**: You can sync clipboards in a cabin in the woods using a portable router or phone hotspot without any internet connectivity.
*   **Sub-50ms Latency**: Sockets communicate directly, eliminating round-trips to the cloud.

---

## 2. Peer Discovery Protocol: UDP Broadcast & IP Fallback

Before devices can sync, they must find each other. In a traditional client-server system, discovery is trivial because the server has a fixed, public internet address. In a P2P network, devices receive random local IP addresses (like `192.168.1.15` and `192.168.1.22`) from the router, and these addresses change frequently.

To solve this without a server, AirBoard uses **UDP Broadcasts** alongside a **Manual IP Fallback**.

### What is UDP vs. TCP?
Think of communication protocols as different ways to send mail:
*   **TCP (Transmission Control Protocol)** is like a *registered phone call*. Before speaking, a connection is formally established (a "handshake"). If a packet is lost, it is automatically retransmitted. It is extremely reliable but requires knowing the exact address of the recipient beforehand.
*   **UDP (User Datagram Protocol)** is like *throwing postcards*. There is no pre-established connection. You write the data on the postcard, throw it, and hope it arrives. It is faster and supports sending one message to multiple recipients at once.

### UDP Broadcasting (Port 45455)
Every device running AirBoard opens a UDP socket on a dedicated port (`45455`). 
1. **Announcing Presence**: Every few seconds, each device sends a UDP broadcast packet to the local sub-network's broadcast address (e.g., `192.168.1.255`).
2. **The Metaphor**: This is like walking into a crowded room with a megaphone and shouting: *"I am Device A! My IP address is 192.168.1.15, and my cryptographic fingerprint is XYZ!"*
3. **Listening**: Other devices on the network listen on port `45455`. When they receive this broadcast postcard, they read the sender's IP address and add them to their "Discovered Devices" panel.

### Manual IP Fallback (TCP Bridge)
In corporate offices, public hotspots, or university Wi-Fi networks, routers block UDP broadcasts (called **Wi-Fi isolation**) to prevent security threats and network congestion. 

To bypass this restriction, AirBoard includes a **Manual IP Connection**. If auto-discovery fails, a user can look up their laptop's IP address on the AirBoard UI and type it directly into their phone. The phone then bypasses UDP broadcasting entirely, initiating a direct TCP connection straight to the laptop on port `45457`.

---

## 3. Zero-Trust Cryptography & End-to-End Encryption (E2EE)

Because P2P networks broadcast data over shared Wi-Fi (such as a public coffee shop network), anyone running a packet-sniffing tool (like Wireshark) could read your clipboard packets in plaintext. 

AirBoard implements a **Zero-Trust Security Model** to protect against these threats. No device is trusted automatically; they must be cryptographically paired, and all traffic must be encrypted end-to-end.

```
AirBoard Cryptographic Stack:
+--------------------------------------------------------+
|           Application Payload (Plaintext Clip)          |
+--------------------------------------------------------+
|      ChaCha20-Poly1305 (Symmetric Payload Encryption)  |
+--------------------------------------------------------+
|      X25519 (ECDH - Ephemeral Session Key Exchange)    |
+--------------------------------------------------------+
|      Ed25519 (Digital Signatures / Handshake Proof)    |
+--------------------------------------------------------+
```

### Key Exchange (X25519)
How can two devices agree on a shared secret password (a "symmetric key") to encrypt their packets without a hacker intercepting that secret password during transmission?

We solve this using **Elliptic Curve Diffie-Hellman (ECDH)** via the **X25519** protocol. 

#### The Color-Mixing Analogy
Imagine Device A and Device B want to agree on a secret paint color without a hacker (Eve) seeing it.
1. **Agreeing on a Base**: They agree publicly on a base color (e.g., Yellow). Eve knows this.
2. **Private Colors**: 
   * A chooses a secret color (e.g., Red).
   * B chooses a secret color (e.g., Blue). 
   * *These private colors represent Private Keys and are never sent over the network.*
3. **Mixing & Transmitting**:
   * A mixes Yellow + Red to get Orange. A sends Orange to B.
   * B mixes Yellow + Blue to get Green. B sends Green to A.
   * *Orange and Green represent Public Keys.* Eve sees these, but she cannot easily extract the original secret Red or Blue from them.
4. **The Final Secret**:
   * A takes B's Green (Yellow+Blue) and adds its secret Red -> Purple (Yellow+Blue+Red).
   * B takes A's Orange (Yellow+Red) and adds its secret Blue -> Purple (Yellow+Red+Blue).
   * Both devices now possess the identical final color (Purple), but Eve cannot recreate it because she lacks the secret Red and Blue.

In AirBoard, X25519 performs this mathematical "color mixing" to produce a shared 256-bit key used to encrypt all session data.

### Authentication & Digital Signatures (Ed25519)
The color-mixing protocol alone is vulnerable to a **Man-in-the-Middle (MITM)** attack. A hacker sitting between the devices could pretend to be B when talking to A, and pretend to be A when talking to B.

To prevent this, AirBoard uses **Ed25519** digital signatures for identity verification:
1. **Identity Keys**: On first boot, every device generates an Ed25519 signature keypair.
2. **Pairing Handshake**: When you tap "Pair Device", the devices exchange their public Ed25519 keys and generate a unique SHA-256 fingerprint based on them.
3. **Fingerprint Verification**: The user manually compares the 16-byte hex fingerprints displayed on both screens. Tapping "Approve" stores the peer's public key in a local `trust_store.json`.
4. **Session Authentication**: During subsequent connections, the peers sign their ephemeral X25519 keys with their private Ed25519 key. By verifying the signature against the stored public key, each device verifies that the peer is indeed the same trusted hardware that was paired.

### Symmetric Payload Encryption (ChaCha20-Poly1305)
Once the ephemeral X25519 key exchange establishes a shared session key, the actual clipboard updates are encrypted using **ChaCha20-Poly1305**.

*   **ChaCha20** is a stream cipher that encrypts the data quickly.
*   **Poly1305** is an authenticator (MAC - Message Authentication Code) that acts like a tamper-evident seal on a package. If a hacker alters even a single bit of the encrypted clipboard packet in transit, the signature check fails, and the packet is discarded.
*   **Nonces (Number Used Once)**: Every encrypted packet is assigned a unique, random 96-bit value called a *nonce*. Even if you copy the word "Hello" twice, the resulting encrypted ciphertext will look completely different because a new nonce is used each time. This protects against **Replay Attacks** (where an eavesdropper records an encrypted packet and plays it back later to write to your clipboard).

---

## 4. Cross-Language Architecture: Flutter & Rust Bridge (FRB v2)

AirBoard combines the user interface capabilities of **Flutter/Dart** with the performance, safety, and system-level access of **Rust**.

```
System Bridge Boundary:
+-----------------------------------+
|          FLUTTER (DART)           |  <--- High-level UI, platform plugins
+-----------------------------------+
|======= flutter_rust_bridge =======|  <--- Zero-copy asynchronous boundary
+-----------------------------------+
|            RUST CORE              |  <--- Cryptography, network loops, sockets
+-----------------------------------+
```

### The Challenge of Polyglot Runtimes
Dart is a garbage-collected language that manages memory automatically by periodically cleaning up unused objects. Rust is a system language with no garbage collector; it manages memory via a compile-time ownership model with strict borrowing rules.

Usually, passing data between two such environments requires copying the data into a temporary C-style memory buffer, which is slow and memory-intensive.

### Zero-Copy Interoperability via FRB v2
AirBoard uses `flutter_rust_bridge` version 2 to link Dart and Rust.
*   **Rust as the Core Engine**: All high-performance operations (listening to TCP streams, executing cryptography algorithms, checking trust stores) run in Rust.
*   **Zero-Copy Serialization**: When large clips are copied, the bridge uses native pointers and zero-copy binary serialization to read the raw memory bytes directly, preventing memory fragmentation and keeping the app's RAM footprint below **40MB**.
*   **Async Event Streams**: Rust uses the `StreamSink` wrapper to send real-time clipboard updates as asynchronous events straight into Flutter's Dart event loop, updating the UI instantly.

---

## 5. Platform Native Integrations & Clipboard Monitors

Accessing the operating system's clipboard and maintaining a connection when the application is closed requires distinct platform integrations.

### Android Foreground Service & Local TCP Loopback
Android 10+ restricts background clipboard operations. A background application is blocked from writing to or reading from the primary system clipboard for security reasons.

AirBoard bypasses this restriction through a custom hybrid architecture:
1. **Foreground Service**: AirBoard launches a native Kotlin service (`ClipboardSyncService`) that shows a persistent notification. This tells Android that the application is actively running a core user utility, protecting the process from being terminated by the OS.
2. **Local Loopback Sockets (`127.0.0.1:45457`)**: The background Rust core cannot access the clipboard on Android directly. However, the foreground Kotlin service *can* access it when the app is minimized. AirBoard establishes a local loopback TCP connection inside the device.
   * When Kotlin detects a local copy, it sends the text over the local socket `127.0.0.1:45457` to the Rust core.
   * Rust then encrypts the text and broadcasts it to the network.
   * When an incoming update is received, Rust passes it to Kotlin over the socket, which triggers a background Activity (`ClipboardWriteActivity`) to write the data directly to the native clipboard.
3. **Multicast Lock**: During sleep, Android shuts down Wi-Fi multicast capabilities to save power. AirBoard locks the multicast state natively, allowing discovery packets to arrive even when the phone's screen is turned off.

### Linux Clipboard Polling (`arboard`)
On Linux, AirBoard uses the `arboard` crate to interface directly with the windowing clipboard systems (X11 and Wayland). A dedicated background loop polls the clipboard state periodically:
* It reads the system clipboard every 500ms.
* If a new text string is found, it immediately forwards it to the Rust sync engine.
* Because this loop runs in an asynchronous Tokio task on a background thread, it does not block or slow down the graphical UI.

### iPadOS & iOS Background Keep-Alives
iOS terminates applications within seconds of them entering the background. AirBoard bypasses this to keep the P2P connection alive:
*   **Silent Audio Loop (`AVFoundation`)**: When minimized, the iPadOS build triggers a silent, low-resource audio player loop. This indicates to the OS that the app is performing active tasks, extending background execution times.
*   **Buffered Syncing**: If the iPad pasteboard is blocked in the background, incoming clipboard updates are buffered in memory and a local notification is fired. Tapping the notification immediately flushes the memory buffer into the system pasteboard.

---

## 6. Deduplication & Feedback Loop Prevention

A critical issue in real-time clipboard synchronizers is the **Feedback Loop** (or "Echo Effect").

```
The Feedback Loop Problem:
Device A (Copy "Hello") ===(Network Sync)===> Device B Writes "Hello"
                                                    ||
Device A Receives "Hello" <==(Network Sync)=== Device B Triggers PrimaryClipChanged
```

If left unchecked, this triggers an infinite loop of network traffic, freezing both devices.

### The Prevention Algorithm
AirBoard implements a deduplication state machine:

1. **SHA-256 Hashing**: Instead of comparing long texts directly, the sync engine computes the SHA-256 hash of the clipboard text.
2. **Tracking the Last Sent Hash**:
   * When a local user copies a text, the engine updates `last_sent_hash` to `SHA256(text)`.
   * It broadcasts the text across the network.
3. **Tracking the Last Received Hash**:
   * When an incoming packet is received from the network, the engine updates `last_received_hash` to `SHA256(received_text)`.
   * It writes the text to the system clipboard.
4. **Ignoring Self-Triggered Clipboard Events**:
   * When the native clipboard listener fires (because a new clip was written):
     * The engine checks: Is `SHA256(new_clip)` equal to `last_received_hash`?
     * If yes, the event is ignored because the change was caused by the sync engine itself writing an incoming payload.
     * If no, it checks: Is `SHA256(new_clip)` equal to `last_sent_hash`?
     * If yes, the event is ignored (deduplicated).
     * If it matches neither, it is treated as a genuine new copy action by the user, and is sent over the network.
