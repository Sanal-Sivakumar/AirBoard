# Implementation Plan: iPadOS Support & Client Lifecycle Management

This plan implements secure clipboard synchronization support for iPadOS, respecting iOS/iPadOS lifecycle behavior, battery saving rules, local network permission policies, and latest-state clipboard conflict reconciliation.

## User Review Required

> [!WARNING]
> iPadOS cannot reliably host a background WebSocket server due to Apple's background execution rules. Consequently, iPadOS operates strictly as a WebSocket Client (reconnecting peer). It only performs real-time synchronization while active in the foreground, and pauses connections/discovery loops cleanly when placed in the background to avoid battery drain and system-induced terminations.

## Open Questions

> [!IMPORTANT]
> 1. Should we define a custom Bonjour service type name (like `_syncboard._tcp`) or are standard `_http._tcp` / `_ws._tcp` service registrations acceptable?
> 2. How frequently should the iOS general pasteboard changeCount be polled? 1000ms is proposed to ensure low energy impact.

---

## Proposed Changes

### iOS Native Integration
Allows the Flutter app to fetch and write clipboard contents on Apple devices using Swift and UIPasteboard APIs directly, avoiding Flutter clipboard constraints and enabling efficient `changeCount` checking.

#### [NEW] [AppDelegate.swift](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/AppDelegate.swift)
- Setup custom FlutterMethodChannel `com.example.clipboard/clipboard`.
- Implement `getClipboardText`, `setClipboardText`, and `getChangeCount` via native iOS `UIPasteboard` API.

#### [NEW] [Info.plist](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/Info.plist)
- Add `NSLocalNetworkUsageDescription` describing the P2P discovery.
- Add `NSBonjourServices` declaring local service types (`_http._tcp`, `_ws._tcp`) to enable local discovery.

---

### Rust Core Modules
Implements client-only behavior, lifecycle-driven task suspensions, and timestamp-based conflict reconciliation in the Rust core.

#### [MODIFY] [core/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/mod.rs)
- Declare all core submodules to ensure successful compilation (including `crypto`, `trust_store`, `pairing`, `session`, `networking`).
- Register Phase 4 modules: `ios`, `lifecycle`, `clipboard_state`.

#### [NEW] [core/ios/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/ios/mod.rs)
- Stub module definition representing Apple/iOS target configurations.

#### [NEW] [core/lifecycle/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/lifecycle/mod.rs)
- Implement `handle_app_foreground()` and `handle_app_background()`.
- Abort active tokio tasks (UDP announcer, listener, heartbeats, reconnects) and clear connections when entering background.
- Re-spawn discovery/networking tasks when entering foreground.

#### [NEW] [core/clipboard_state/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard_state/mod.rs)
- Track local clipboard state: `content`, `timestamp` (Unix epoch milliseconds), and `packet_id`.
- Implement getters/setters for reconciliation.

#### [MODIFY] [core/reconnect/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/reconnect/mod.rs)
- Implement `trigger_reconnect()` to immediately spawn connections to unconnected peers on foreground transition.

#### [MODIFY] [core/peer_manager/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/peer_manager/mod.rs)
- In `connect_to_peer`, bypass `local_id >= peer_id` rule if running as client-only/iOS.
- Add `ClipboardStateExchange` and `ClipboardStateRequest` to the `Message` protocol.
- Perform clipboard reconciliation by exchanging state hashes and timestamps on connection initialization, requesting missing updates from the newer peer.

#### [MODIFY] [api/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/api/mod.rs)
- Modify `start_sync` to accept a `platform: String` instead of `is_android: bool`. Set `IS_CLIENT_ONLY` if the platform is `"ios"`.
- Expose `handle_app_foreground()` and `handle_app_background()` endpoints for FRB v2.

---

### Flutter UI & Observer

#### [MODIFY] [lib/main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- Include `WidgetsBindingObserver` to report lifecycle updates to Rust.
- Use `com.example.clipboard/clipboard` method channel on iOS for clipboard operations.
- Update UI to present iPad connection details, lifecycle states, reconnect states, and last sync timestamps.

---

## Verification Plan

### Automated/Unit Tests
Since local Cargo/Flutter tooling is unavailable on this container, verification will rely on static checks and structural compilation readiness:
- Run compile/build commands once environment is configured.

### Manual Verification Instructions
1. **Local Network Permissions**:
   - Install the app on iPadOS. Open it and trigger sync. Confirm the "Local Network Permission" system dialog is shown.
2. **Lifecycle Transitions**:
   - Enable sync on iPadOS and connect to a server (Linux/Android).
   - Put iPadOS app in background. Verify logs on server show "Session closed" (clean disconnect).
   - Bring iPadOS app back to foreground. Verify it immediately reconnects and displays the connection status.
3. **Reconciliation Test**:
   - Turn off sync on iPadOS (or put app in background). Copy text "iPad Offline Text" on iPad.
   - Copy text "Server Text" on Linux (which is running the server).
   - Bring iPadOS app to the foreground. Since Linux had a newer copy timestamp, iPadOS clipboard should reconcile and update to "Server Text".
   - Turn off sync, copy "iPad Newer Text" on iPad. Let some time pass, copy "Server Older Text" on Linux (using an older timestamp). Re-activate iPad. The Linux clipboard should reconcile and update to "iPad Newer Text".
4. **Hotspot Compatibility Tests**:
   - Verify connection under a mobile hotspot configuration where routing multicast/broadcast packets may be filtered.
