# Implementation Plan: Mobile Peer Discovery Improvements & Mesh Sync Verification

This plan outlines the changes required to resolve discovery isolation between mobile devices (Android <-> iPad) and details the clipboard mesh synchronization behavior.

## User Review Required

> [!IMPORTANT]
> - **Discovery Mechanism**: We will enhance UDP discovery to bind to the active interface's IP (preventing OS routing to cellular data networks) and broadcast to both the global address (`255.255.255.255`) and the local subnet broadcast address (`192.168.x.255`), which bypasses many wireless AP isolation filters.
> - **Clipboard Mesh**: The Rust core already forwards incoming clipboard updates to all other active connected peers (excluding the sender). Therefore, as long as mobile devices connect to a shared peer (like a Linux PC), their clipboards sync seamlessly across the network.

---

## Proposed Changes

### [Rust Core Engine]

#### [MODIFY] [discovery/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/discovery/mod.rs)
- **Local IP Detection**: Add a connectionless socket resolver (`get_local_ip`) to query the OS routing table for the active LAN interface's IP.
- **Subnet Broadcast Calculation**: Add a helper (`get_subnet_broadcast`) to calculate the subnet broadcast address (replacing the last octet with `.255`).
- **Update Announcer**:
  - Dynamically bind the announcer socket to the resolved local IP address to direct traffic on the correct network interface.
  - Broadcast the JSON packet to both `255.255.255.255:45454` and the calculated subnet broadcast address.

---

## Verification Plan

### Automated Tests
- Run `cargo check` inside the `rust/` directory to verify the Rust engine compiles cleanly.
- Run `flutter analyze` inside the root workspace to confirm Dart bindings are unmodified and lint-free.

### Manual Verification
1. Run the Linux and Android applications.
2. Verify that Android and iPad are now able to discover each other directly when on the same subnet.
3. Verify that if multiple devices (e.g., Android and iPad) are paired with the Linux PC:
   - Copying text on Android successfully syncs to Linux.
   - Linux automatically forwards the payload to iPad, updating its clipboard seamlessly without a direct Android-to-iPad connection.
