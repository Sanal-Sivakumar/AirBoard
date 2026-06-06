# Phase 4 Implementation Plan: iPadOS Client Integration & Lifecycle Reconciliation

This plan implements secure clipboard synchronization support for iPadOS, respecting iOS/iPadOS lifecycle behavior, background constraints, and local network permissions.

---

## 1. iPadOS Integration Architecture

Due to Apple's strict background execution constraints, the iPadOS app will run purely as a WebSocket Client (reconnecting peer) and will only sync in real-time while in the foreground.

```mermaid
graph TD
    subgraph iPadOS (Foreground Client)
        I_UI[Flutter UI + WidgetsBindingObserver] -->|Lifecycle Updates| I_Rust[Rust Core]
        I_UI <-->|UIPasteboard API| I_Clip[Flutter Clipboard]
        I_Rust -->|Client Socket| I_Conn[WebSocket Connection]
    end

    subgraph Desktop / Android (Server)
        S_WS[WebSocket Server]
    end

    I_Conn <-->|WiFi / Hotspot| S_WS
```

### App Lifecycle Transitions
- **Foreground (Active/Resumed)**:
  - Flutter notifies Rust (`AppLifecycleState.resumed`).
  - Rust initializes UDP discovery listener and begins announcer broadcasts.
  - Rust triggers the reconnect loop immediately to link with any active servers.
  - The client performs a state reconciliation check (exchanges timestamps to sync offline updates).
- **Background (Paused/Inactive)**:
  - Flutter notifies Rust (`AppLifecycleState.paused`).
  - Rust cleanly shuts down all active WebSocket client connections to avoid socket leaks.
  - Rust suspends UDP discovery sockets and announcer loops to conserve battery.

---

## 2. iOS Local Network Permission (`Info.plist`)

To support local discovery and connections, we must configure specific keys in `ios/Runner/Info.plist`:
- `NSLocalNetworkUsageDescription`: Description explaining that the app needs to discover other devices on the local WiFi network.
- `NSBonjourServices`: Registry of Bonjour/mDNS service types if needed (though UDP discovery on 45454 is our primary discovery mechanism, this ensures local discovery APIs work).

---

## 3. Clipboard Conflict & State Reconciliation

To resolve conflicts when a device has been offline/suspended and has copied new text:
- Every clipboard update packet will now include a `timestamp` (Unix timestamp in milliseconds).
- **Reconciliation Protocol**:
  1. Once the authenticated session key is established, the client and server exchange their current clipboard timestamp and packet ID.
  2. The peer with the **older** timestamp updates its clipboard with the text sent by the peer with the **newer** timestamp.
  3. This ensures that whichever device copied text last (even while disconnected) wins the reconciliation, propagating changes accurately upon reconnection.

---

## 4. Proposed Changes

### iOS Native Settings
- [NEW] [Info.plist](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/Info.plist): Setup local network description and Bonjour service declarations.

### Rust Core Modules
- [MODIFY] [core/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/mod.rs)
- [NEW] [core/ios/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/ios/mod.rs): iPadOS platform definitions (stub/client config).
- [NEW] [core/lifecycle/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/lifecycle/mod.rs): Functions to suspend/resume P2P sockets and listeners.
- [NEW] [core/clipboard_state/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard_state/mod.rs): Clipboard state tracking, timestamping, and reconciliation.
- [MODIFY] [core/peer_manager/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/peer_manager/mod.rs): Add timestamp-based conflict resolution checks upon handshaking.
- [MODIFY] [api/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/api/mod.rs): Expose lifecycle notification triggers.

### Flutter UI & Observer
- [MODIFY] [lib/main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart): Add `WidgetsBindingObserver` to capture lifecycle events, display last sync timestamp, and show current lifecycle states.
