# IP Persistence & Automatic Reconnection for Direct iPad-to-Android Sync

This implementation plan addresses the issue where clipboard synchronization fails when connecting an iPad directly to an Android device without a desktop peer. The root cause is that local networks on mobile devices (e.g. mobile Wi-Fi, hotspot) frequently block UDP broadcast/multicast packets. Because the app relies entirely on UDP announcements to populate the connection registry and trigger the reconnect loop, once the devices disconnect, they can never discover each other or reconnect automatically.

To solve this, we will persist the last-known IP address and port of trusted devices in the secure trust store, and update the reconnection loop to attempt direct connections to trusted peers using these saved coordinates.

## User Review Required

> [!IMPORTANT]
> - **IP Address Storage**: We are adding `last_ip` (string) and `last_port` (u16) fields to the serialized `TrustedDevice` struct in the `trust_store.json` database. These fields are marked with `#[serde(default)]` so that existing databases are parsed correctly without compatibility issues.
> - **Pairing & Connection Initiation**: The pairing connection is closed immediately after pairing succeeds. This plan triggers an immediate reconnection attempt using the newly saved IP address to start the sync session without waiting.

## Open Questions

None. The design is fully specified and backward-compatible.

## Proposed Changes

### Rust Core

---

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/trust_store/mod.rs)
- Update `TrustedDevice` to include `last_ip` and `last_port` as optional fields with `#[serde(default)]`.
- Implement `update_trusted_device_ip_port(device_id: &str, ip: String, port: u16)` to update these persisted coordinates whenever a connection is established.

---

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/pairing/mod.rs)
- Update `initiate_pairing_flow` to save the remote peer's `ip` and `port` to the trust store upon successful pairing.
- Modify `handle_pairing_flow` to accept the peer's IP address and store it.
- Trigger an immediate reconnect attempt via `crate::core::reconnect::trigger_reconnect()` at the end of successful pairing.

---

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/peer_manager/mod.rs)
- Pass the source `ip_address` into `handle_pairing_flow` from the listener loop.
- Prevent duplicate/concurrent connection tasks in `connect_to_peer` by checking if the peer connection status is currently `"Connecting"`.
- Update `last_ip` and `last_port` in the trust store upon successful handshake in both `connect_to_peer` and `handle_incoming_connection`.

---

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/reconnect/mod.rs)
- Enhance `trigger_reconnect()` to retrieve trusted devices from the trust store.
- If a trusted device is not currently active (`ACTIVE_PEERS`) and has valid saved `last_ip`/`last_port` coordinates (where `port > 0`), spawn a reconnection task to it.

---

## Verification Plan

### Automated Tests
- Run `cargo check` inside the `/home/sanal-sivakumar/Documents/clipboard/rust` directory to verify Rust compiles without errors.
- Run `flutter analyze` inside the workspace to confirm the Flutter app is in a clean state.

### Manual Verification
1. Open the app on Android and iPad.
2. Manually pair the iPad to the Android device by entering Android's IP in the iPad's manual connection input.
3. Confirm that the pairing completes, and they immediately establish a sync connection.
4. Copy text on iPad and confirm it appears on Android.
5. Minimize the iPad app or disconnect/reconnect Wi-Fi to force a disconnection.
6. Return to the app and verify that the iPad automatically reconnects to the Android device without requiring a manual repair or UDP broadcast discovery.
