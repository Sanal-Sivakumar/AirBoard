# SyncBoard - Peer-to-Peer Local Network Clipboard Sync (Flutter + Rust)

SyncBoard is a high-performance, low-latency, plain text clipboard synchronization system operating in a decentralized, Peer-to-Peer (P2P) network mesh. It auto-discovers and synchronizes clipboards between Linux desktops and Android devices connected on the same local network (WiFi or Hotspot).

---

## 1. P2P Architecture Details

SyncBoard implements a fully decentralized mesh topology:

1. **UDP Discovery (Port 45454)**:
   - Every active device broadcasts a UDP packet announcing its `device_name`, `device_id`, and `ws_port` every 5 seconds to `255.255.255.255:45454`.
   - At the same time, each device listens on `0.0.0.0:45454` for announcements from other peers to register them in its Connection Registry.

2. **WebSocket Core (Port 45455 / Ephemeral)**:
   - Each device hosts its own WebSocket server (spawning on `45455` or next free port).
   - Once a peer is discovered, the device initiates a client connection to the peer's WebSocket server *if and only if* its own `device_id` is alphabetically smaller than the peer's `device_id` (tie-breaking mechanism).
   - This prevents duplicate socket connections and ensures a single stable link.

3. **Routing & Loop Prevention**:
   - Every clipboard update is packaged with a unique `packet_id` (UUID) and `origin_device_id`.
   - When a peer receives an update, it checks its local sliding-window cache of the last 100 packet IDs.
   - If the packet is new, it updates the system clipboard, logs the event, and floods (forwards) the packet to all other connected peers (excluding the sender).
   - If the packet was already processed, it is silently dropped.

4. **Heartbeats & Dead Peer Detection**:
   - Heartbeat packets are exchanged every 10 seconds.
   - If a peer fails to send a UDP announcement or a WebSocket heartbeat for more than 30 seconds, it is marked as `Disconnected`, connection sockets are pruned, and the registry is updated.

---

## 2. Installation & Prerequisites

### Linux Dependencies
Compile tools and X11/xcb headers are required to build on Linux:
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libx11-dev libxcb1-dev
```

### Rust and Targets
Configure compilation targets for both local desktop and Android architectures:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Add Android targets
rustup target add aarch64-linux-android      # Modern physical Android devices
rustup target add x86_64-linux-android       # Android Studio Emulator
```

### CLI Code Generator
```bash
cargo install flutter_rust_bridge_codegen --version 2.0.0
```

---

## 3. How to Build & Run

1. **Perform Code Generation**:
   Generate Dart/Rust serialization glue by running the command in the root folder:
   ```bash
   flutter_rust_bridge_codegen generate
   ```

2. **Run on Linux**:
   ```bash
   flutter run -d linux
   ```

3. **Run on Android**:
   ```bash
   flutter run -d android
   ```

---

## 4. Hotspot Compatibility & Troubleshooting

SyncBoard is designed to work reliably on standard LAN routers as well as Android Hotspots:

- **Android Hotspot Mode**:
  - Turn on hotspot on Android and connect your Linux desktop to it.
  - Due to security restrictions on some mobile operating systems, UDP broadcasts from the Linux client to the Android hotspot might be filtered at the interface level, but **broadcasts from the Android hotspot (host) to connected clients are fully forwarded**.
  - As soon as the Linux client receives the Android device's broadcast, it registers the Android IP and establishes the WebSocket connection.
- **Firewall Warnings**:
  - Ensure your Linux firewall (e.g. `ufw`) allows UDP packets on port `45454` and TCP packets on port `45455`:
    ```bash
    sudo ufw allow 45454/udp
    sudo ufw allow 45455/tcp
    ```
