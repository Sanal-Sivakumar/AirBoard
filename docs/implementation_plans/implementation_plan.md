# Clipboard Synchronization Application Implementation Plan (Flutter + Rust)

This plan details the design and implementation of a minimal working prototype to synchronize plain text clipboards between a Linux desktop (WebSocket client) and an Android device (WebSocket server).

---

## Architecture Overview

We will construct a Flutter application containing a Rust-based backend core linked via `flutter_rust_bridge` (FRB) v2.

```mermaid
graph TD
    subgraph Android Device (Server)
        A_UI[Flutter UI] <-->|FRB v2| A_Rust[Rust Core]
        A_UI <-->|Platform Channel| A_Kotlin[Kotlin Foreground Service]
        A_Rust <-->|TCP Server| A_WS[WebSocket Server]
        A_UI <-->|Method Channel| A_Clip[Android Clipboard API]
    end

    subgraph Linux Desktop (Client)
        L_UI[Flutter UI] <-->|FRB v2| L_Rust[Rust Core]
        L_Rust <-->|arboard crate| L_Clip[Linux Clipboard]
        L_Rust <-->|TCP Client| L_WS[WebSocket Client]
    end

    A_WS <-->|WiFi / Hotspot| L_WS
```

### Module Structure (Rust)
The Rust core will reside in the `rust` subdirectory of the project. Its internal structure will be:
```
rust/src/
 ├── lib.rs                 # Bridge Entrypoint & Exports
 ├── api/
 │    └── mod.rs            # Exposes interface functions to Flutter (FRB v2)
 └── core/
      ├── mod.rs
      ├── networking/
      │    ├── mod.rs
      │    ├── server.rs    # Tokio WebSocket Server (Android)
      │    └── client.rs    # Tokio WebSocket Client (Linux)
      ├── clipboard/
      │    ├── mod.rs
      │    └── linux.rs     # Linux clipboard monitoring using `arboard`
      ├── protocol/
      │    ├── mod.rs
      │    └── models.rs    # JSON Serialization structures
      ├── sync_engine/
      │    ├── mod.rs
      │    └── engine.rs    # Clipboard Loop Prevention & State manager
      └── utils/
           ├── mod.rs
           └── helpers.rs   # Device ID generation & hashes
```

---

## Detailed Component Specifications

### 1. JSON Protocol (`protocol/models.rs`)
```json
{
  "type": "clipboard_update",
  "device_id": "unique-device-id",
  "content": "clipboard text",
  "timestamp": 1712345678
}
```

### 2. Clipboard Loop Prevention (`sync_engine/engine.rs`)
To prevent feedback loops (e.g., Device A writes to clipboard -> triggers change event -> sends to Device B -> Device B writes to clipboard -> triggers change event -> sends to Device A):
- **Local Cache**: Track the SHA-256 hash of the most recently *written* (synced) clipboard text.
- **Source Filter**: Compare the incoming `device_id` with the local device ID. If they match, discard the update.
- **Deduplication**: If the incoming text hash matches the local cache, ignore the change.
- **Local Monitor Ignore**: When the clipboard monitor detects a new change:
  - If the new content hash matches `last_synced_content_hash`, do NOT send it (it was written by the sync engine).
  - Otherwise, broadcast it and update `last_seen_local_text`.

### 3. Android Foreground Service
We will implement an Android foreground service in Kotlin to keep the application active in the background. It will display a persistent notification: *"Clipboard Sync Active"*.
- **Service Name**: `ClipboardSyncService`
- **Foreground Type**: `dataSync` (Android 14+ compatible)
- **Lifecycle**: Started when the sync server starts; stopped when the server stops.
- **Communication**: Android native clipboard changes are listened to by the Flutter frontend (since clipboard monitoring in Rust on Android requires complex JNI binds). Flutter will forward updates to Rust via FRB.

### 4. Linux Clipboard Monitoring
For Linux, we will run a Tokio task in the background that polls `arboard` every 500ms. Since we run inside a background thread in Rust, it does not block the UI. If a change is detected (and it's not the last synced hash), it broadcasts to the server.

---

## Proposed Changes

We will create a new Flutter project in `/home/sanal-sivakumar/Documents/clipboard` and initialize the Rust bridge.

### [NEW] [pubspec.yaml](file:///home/sanal-sivakumar/Documents/clipboard/pubspec.yaml)
Includes dependencies:
- `flutter_rust_bridge: ^2.0.0`
- `uuid: ^4.0.0`

### [NEW] [Cargo.toml](file:///home/sanal-sivakumar/Documents/clipboard/rust/Cargo.toml)
Includes dependencies:
- `flutter_rust_bridge = "=2.0.0"`
- `tokio = { version = "1", features = ["full"] }`
- `tokio-tungstenite = "0.21"`
- `serde = { version = "1.0", features = ["derive"] }`
- `serde_json = "1.0"`
- `arboard = "3.3"`
- `sha2 = "0.10"`
- `uuid = { version = "1.0", features = ["v4"] }`
- `futures-util = "0.3"`

### [NEW] Rust Core Modules
- [lib.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/lib.rs)
- [api/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/api/mod.rs)
- [core/networking/server.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/networking/server.rs)
- [core/networking/client.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/networking/client.rs)
- [core/clipboard/linux.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/linux.rs)
- [core/protocol/models.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/protocol/models.rs)
- [core/sync_engine/engine.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/sync_engine/engine.rs)
- [core/utils/helpers.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/utils/helpers.rs)

### [NEW] Android Native Code
- [AndroidManifest.xml](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/AndroidManifest.xml): Registers `ClipboardSyncService` and permissions.
- [ClipboardSyncService.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardSyncService.kt): Kotlin foreground service implementation.
- [MainActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/MainActivity.kt): Set up method channels for launching the service.

### [NEW] Flutter UI & App Logic
- [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart): Layout with status indicators, IP configuration, and connection controls. Relies on platform-specific channels to handle clipboard access on Android, and uses the Rust bridge stream for remote events.

---

## Verification Plan

### Manual Verification
1. **Host Prerequisites**:
   - Install Flutter SDK.
   - Install Rust and the Android targets (`aarch64-linux-android`, `x86_64-linux-android`).
   - Install `flutter_rust_bridge_codegen`.
   - Install Linux X11 development dependencies (`libx11-dev` and `libxcb1-dev`).
2. **Execution Test**:
   - Start the Android app, check the server IP (displayed on screen), and start the service (ensuring the notification appears).
   - Start the Linux client app, input the server IP, and connect.
   - Copy text on Linux; verify it appears on the Android clipboard immediately.
   - Copy text on Android; verify it appears on the Linux clipboard immediately.
   - Verify copying multiple times does not trigger an infinite loop of clipboard sync updates.
