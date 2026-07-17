# AirBoard Protocol v2 Technical Reference

This document describes the implementation in the current source tree. It intentionally separates implemented guarantees from platform limitations.

## 1. Architecture

AirBoard has two primary layers:

- Flutter owns the application UI, secure-storage integration, lifecycle observation, user approval, and mobile clipboard APIs.
- Rust owns LAN discovery, pairing validation, authenticated session establishment, encrypted transport, peer state, replay protection, mesh forwarding, and desktop clipboard polling.

Flutter and Rust communicate through `flutter_rust_bridge`.

```text
Flutter UI and platform APIs
        |
flutter_rust_bridge
        |
Rust API -> discovery / pairing / peer manager / session store
        |                                      |
 UDP :45454                            WebSocket TCP :45455
```

The static website in `web_showcase/` has no access to the clipboard engine.

## 2. Network topology and ports

AirBoard has no central coordinator. Every non-iOS peer listens for TCP connections, and any trusted peer with a known address can initiate a connection.

### Discovery

Every active device sends a JSON announcement every five seconds to UDP port `45454`. The announcement includes:

- message type;
- device name;
- device ID;
- platform;
- peer WebSocket port.

Announcements are metadata, not trusted authentication. A hostile LAN participant can forge them. AirBoard treats discovery only as a way to locate a candidate address; pairing and session authentication establish trust.

The broadcaster attempts the global IPv4 broadcast address, interface broadcast addresses available from the operating system, common `/24` and `/16` derived addresses, and the iOS hotspot broadcast address when applicable.

Manual IP pairing connects directly to TCP `45455` and is the fallback for AP isolation, blocked UDP, or asymmetric discovery.

### Active ports

| Port | Transport | Exposure | Purpose |
| --- | --- | --- | --- |
| `45454` | UDP | LAN | discovery announcements |
| `45455` | TCP/WebSocket | LAN | pairing and encrypted sessions |
| `45456` | TCP | Android loopback | incoming Rust-to-service bridge |
| `45457` | TCP | Android loopback | explicit Android clipboard send-to-Rust bridge |

## 3. Identity and storage

On first launch, Flutter creates 32 random bytes using `Random.secure()` for the Ed25519 signing seed and stores them with `flutter_secure_storage`. The active protocol derives the public signing key in Rust.

The public identity fingerprint is:

```text
SHA-256(ed25519_public_key)
```

It is rendered as all 32 digest bytes in colon-separated uppercase hexadecimal. Rust returns the canonical fingerprint to Flutter; the UI no longer calculates a shortened or substitute value.

The existing exported API and trust-store schema also retain an X25519 identity-DH field for migration compatibility. Protocol-v2 session secrecy uses fresh ephemeral X25519 keys, not the stored static DH secret.

Trusted public identities are stored in `trust_store.json` under the platform application-support directory. Writes use a temporary file, flush to disk, and rename into place. On Unix, newly created trust-store files request mode `0600`.

The trust store contains public keys and last-known LAN coordinates. Private keys remain in the platform credential store.

## 4. Mutual pairing

Pairing is a trust-on-first-verification operation. It is not automatic.

### Pairing request

The requester creates a fresh 256-bit random nonce and signs a length-delimited transcript containing:

```text
domain = airboard/pairing-request/v2
requester device ID
requester device name
requester Ed25519 public key
requester compatibility DH public key
request nonce
```

The receiver validates:

1. protocol version;
2. Base64 decoding;
3. exact key, nonce, and signature lengths;
4. the Ed25519 signature using the public key in the request.

This proves possession of the advertised signing key. The receiver then displays its SHA-256 fingerprint and waits up to two minutes for the user to compare it with the requester screen.

### Pairing response

After approval, the receiver creates a new 256-bit response nonce and signs:

```text
domain = airboard/pairing-response/v2
request nonce
response nonce
requester device ID
responder device ID
responder device name
responder Ed25519 public key
responder compatibility DH public key
```

The requester validates the echoed request nonce, expected discovered device ID when applicable, all field lengths, and the response signature. It then displays the responder fingerprint and requires a second user approval.

Only after this mutual verification does either side persist the other identity.

### Why fingerprint comparison still matters

A signature made by a newly supplied key proves possession of that key but does not say who owns it. Comparing the fingerprint through a separate visual or verbal channel binds the cryptographic key to the physical device. Approving without comparison defeats initial-pairing authentication.

## 5. Authenticated session handshake

Trusted peers use a two-message ephemeral X25519 handshake.

### Handshake 1

The initiator sends:

- protocol version;
- initiator device ID;
- intended responder device ID;
- fresh ephemeral X25519 public key;
- fresh 256-bit initiator nonce;
- Ed25519 signature.

The signature covers a length-delimited `airboard/handshake-1/v2` transcript containing both identities, the ephemeral key, and nonce.

The responder rejects unknown identities, incorrect targets, incompatible versions, malformed field lengths, and invalid signatures.

### Handshake 2

The responder creates its own ephemeral X25519 key and nonce. Its signed `airboard/handshake-2/v2` transcript binds:

- initiator and responder identities;
- both ephemeral public keys;
- both nonces.

The initiator verifies the response against the public signing key saved during pairing and requires the responder ID to match the selected peer.

### Session-key derivation

Both devices compute X25519 Diffie-Hellman with their ephemeral secret and the peer's ephemeral public key. The raw shared secret is processed by HKDF-SHA-256:

```text
salt = SHA-256(handshake_2_transcript)
IKM  = X25519 shared secret
info = airboard/session/v2
L    = 32 bytes
```

The result is the ChaCha20-Poly1305 session key. Ephemeral secrets are not persisted, providing forward secrecy for completed sessions assuming endpoint memory is not compromised during the session.

## 6. Encrypted envelopes

Clipboard messages, state exchange, requests, and heartbeats are serialized as JSON and encrypted with ChaCha20-Poly1305.

Each outer envelope contains:

- protocol version;
- sender device ID;
- recipient device ID;
- monotonic sequence number;
- random 96-bit nonce;
- ciphertext and Poly1305 tag.

The following data is passed as AEAD associated data:

```text
airboard|v2|sender|recipient|sequence
```

Changing the sender, recipient, sequence, protocol version, ciphertext, or authenticated payload causes decryption to fail.

The outer sender and recipient remain visible to a LAN observer because AirBoard does not attempt to hide traffic metadata.

## 7. Replay and duplicate protection

Two independent controls are used:

1. Every session allocates monotonically increasing send sequence numbers. The receiver remembers accepted values in a 1,024-message sliding window, allowing bounded task reordering while rejecting duplicates and stale replay.
2. Clipboard updates carry random UUID packet IDs. A 4,096-entry set/deque cache prevents duplicate application and mesh-forwarding loops.

Sequence numbers are authenticated before they are added to the replay window, so an attacker cannot send an unauthenticated high sequence number to advance the window.

Random AEAD nonces prevent normal nonce reuse. Replay rejection is provided by authenticated sequences rather than by assuming random nonces alone stop replay.

## 8. Resource and parser limits

Network fields decoded into fixed arrays must have exactly the required size. Malformed keys, signatures, nonces, or ephemeral keys return errors rather than calling `copy_from_slice` with attacker-controlled lengths.

Limits:

- text clipboard payload: 512 KiB UTF-8;
- WebSocket message/frame: 1 MiB;
- pairing approval timeout: two minutes;
- mesh packet-ID cache: 4,096 entries;
- replay window: 1,024 sequences.

These limits reduce memory-exhaustion exposure. They are not a substitute for operating-system process limits or network firewalling.

## 9. Clipboard synchronization

### Local changes

The sync engine hashes clipboard text with SHA-256. If it matches the last synchronized content, the change is treated as the feedback generated by a remote write and is not rebroadcast. New local content receives a UUID packet ID and timestamp before encrypted fan-out to active trusted peers.

### Incoming changes

An incoming update must pass:

1. WebSocket size enforcement;
2. envelope parsing and version/identity checks;
3. exact nonce decoding;
4. ChaCha20-Poly1305 verification with associated data;
5. replay-window acceptance;
6. message parsing;
7. clipboard payload-size enforcement;
8. packet-ID deduplication.

It is then written to the local clipboard and re-encrypted separately for other active peers, excluding the immediate sender.

### Reconnection state exchange

Peers exchange the current packet ID and timestamp after establishing a session. A peer that sees a newer advertised timestamp requests the corresponding encrypted content. This is a last-writer-wins convenience mechanism and still relies on reasonably synchronized device clocks. It is not a conflict-free replicated data type.

## 10. Platform integration

### Linux, Windows, and macOS

The Rust `arboard` adapter polls text every 500 ms and writes verified incoming text directly. macOS is included in the same compile-time target set. The macOS sandbox has both network-client and network-server entitlements in debug/profile and release configurations.

### Android

Rust runs the LAN protocol and two loopback-only clipboard bridges. A Kotlin foreground service holds networking resources and exposes notification actions.

Android 10+ restricts clipboard reads and writes when an app is not focused. Protocol v2 does not request `SYSTEM_ALERT_WINDOW` and does not launch transparent activities automatically. When Android enforces the restriction, the user taps **Sync** to read/send or **Copy** to apply a received value. Notification text does not include clipboard contents.

### iOS and iPadOS

The iOS runner implements the Flutter method channel used to read `UIPasteboard.changeCount`, read foreground text, and create privacy-preserving notifications. `NSLocalNetworkUsageDescription` explains LAN access.

iOS is client-only and foreground-oriented. On suspension, AirBoard closes discovery and peer tasks. On resume, it restarts discovery and reconnection and resumes clipboard polling. It does not use silent audio to evade operating-system suspension and does not claim uninterrupted background clipboard access.

## 11. Lifecycle and duplicate connections

Server, discovery, heartbeat, reconnect, and platform clipboard tasks are tracked by the lifecycle task manager. Disabling synchronization aborts those tasks and closes active sessions.

If two authenticated connections race, peer registration and session-key installation occur together under the active-peer registry lock. The first connection wins; the losing connection cannot overwrite the live session key.

## 12. Diagnostics and privacy

Diagnostic logs report clipboard lengths and state transitions, not clipboard previews. Android notification text reports that content arrived from a trusted peer without exposing the value on the lock screen.

The in-app clipboard history intentionally displays recent clipboard text in process memory. Users should treat the visible history as sensitive and close or pause AirBoard when screen sharing.

## 13. Threat model and limitations

### Protected

- clipboard plaintext against passive packet capture;
- payload modification, sender/recipient substitution, and version/sequence modification;
- session impersonation by devices that do not possess a paired Ed25519 key;
- initial MITM when users correctly compare both fingerprints;
- replay within and beyond the bounded live-session reorder window;
- malformed fixed-length protocol fields and oversized messages.

### Not protected

- compromised endpoints or malicious paired devices;
- users approving unverified fingerprints;
- local malware, accessibility services, clipboard managers, screenshots, or keyloggers;
- traffic metadata;
- public-internet exposure or NAT traversal;
- denial of service from a hostile LAN;
- device-clock conflicts in reconnection state selection.

AirBoard should be used on a LAN and TCP `45455` should not be port-forwarded to the internet.

## 14. Verification

The repository provides Rust tests for:

- encryption/decryption round trips;
- ciphertext tampering;
- incorrect associated data;
- transcript-bound HKDF output;
- malformed fixed-size fields;
- replay rejection and bounded reordering;
- monotonic send sequences;
- handshake transcript identity/role binding;
- local and incoming clipboard deduplication.

Flutter widget tests exercise desktop and narrow mobile layouts. `flutter analyze` excludes the bundled Cargokit build-tool source because it is a separate Dart package with its own dependency graph.

Use the commands in [README.md](README.md) and consult [TROUBLESHOOTING.md](TROUBLESHOOTING.md) when a platform toolchain prevents a native build.
