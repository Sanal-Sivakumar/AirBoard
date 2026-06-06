# Fix Disconnected Pairing Status (Tie-Breaker Removal)

This plan resolves the issue where paired devices remain "Disconnected" because of a static connection tie-breaker combined with one-way network discovery.

## User Review Required

> [!IMPORTANT]
> * **The Issue**: In USB tethering/hotspot setups, one-way UDP discovery is extremely common (e.g. PC discovers Phone, or Phone discovers PC, but not both). Currently, a static connection tie-breaker (`local_id >= peer_id`) prevents the device with the larger ID from initiating connections. If that device is the only one that discovered the other, no connection is ever initiated.
> * **The Solution**: We will remove the static `local_id >= peer_id` tie-breaker. The codebase already natively handles duplicate connection attempts via the thread-safe `ACTIVE_PEERS` registration checks, so removing this check is completely safe and guarantees successful connection in one-way discovery scenarios.

---

## Proposed Changes

### Rust Core

#### [MODIFY] [peer_manager/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/peer_manager/mod.rs)
- Remove the `if !is_client_only && local_id >= peer_id { return; }` check from `connect_to_peer()`. This allows either device to initiate a connection as long as it has discovered the other device and is not currently connected to it.

---

## Verification Plan

### Automated/Manual Tests
1. **Compilation**: Verify the project compiles cleanly using `cargo check` and `flutter analyze`.
2. **E2E Connection Check**: Build and deploy to the phone, then verify that the status changes to **Connected** on both the Linux PC and the Phone.
3. **Clipboard Sync Test**: Copy text on Linux and verify it propagates to the phone, and vice versa.
