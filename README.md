# AirBoard

AirBoard is a local-network, peer-to-peer text clipboard synchronizer for Linux, Windows, macOS, Android, iOS, and iPadOS. Devices communicate directly over the LAN; there is no AirBoard cloud service, account, analytics endpoint, or clipboard database.

The current source implements protocol v2. Clipboard payloads are sent only after mutual pairing approval and an authenticated ephemeral-key handshake.

> **Release notice:** binaries already present in `releases/` and older GitHub Pages downloads predate protocol v2. They are not compatible with or security-equivalent to the current source. Rebuild every participating device from this revision and pair again. Do not mix legacy and v2 clients.

## What is implemented

- UDP discovery on port `45454`, plus manual IP connection when broadcast discovery is blocked.
- Direct WebSocket connections on TCP port `45455`.
- Ed25519 device identity keys stored through `flutter_secure_storage`.
- Full SHA-256 identity fingerprints shown for manual verification.
- Signed pairing requests and responses proving possession of the advertised identity keys.
- Mutual fingerprint approval: both the requester and responder must approve the other device.
- Fresh ephemeral X25519 keys for every connected session.
- Transcript-bound HKDF-SHA-256 session-key derivation.
- ChaCha20-Poly1305 authenticated encryption for clipboard data and heartbeats.
- Authenticated protocol version, sender, recipient, and sequence number on every encrypted envelope.
- A 1,024-message replay window and a 4,096-packet mesh deduplication cache.
- 512 KiB clipboard and 1 MiB wire-message limits.
- Atomic trust-store writes with owner-only permissions on Unix systems.
- Clipboard monitoring on Linux, Windows, and macOS while AirBoard is running.
- Android foreground networking with explicit notification actions when the OS blocks background clipboard access.
- Foreground clipboard synchronization and lifecycle-safe reconnection on iOS/iPadOS.

## Platform behavior

| Platform | Clipboard behavior | Background behavior |
| --- | --- | --- |
| Linux | Automatic text read/write while AirBoard runs | Continues while the process is running |
| Windows | Automatic text read/write while AirBoard runs | Continues while the process is running |
| macOS | Automatic text read/write while AirBoard runs | Continues while the app is running; sandbox network client/server entitlements are included |
| Android | Automatic while the app is foregrounded | Network service remains available. Android 10+ may require tapping **Sync** to send or **Copy** to apply clipboard text. AirBoard does not request overlay permission. |
| iOS/iPadOS | Automatic while AirBoard is foregrounded | Connections pause when suspended and reconnect on foreground. Continuous background clipboard access is not claimed because iOS does not permit it for this app category. |

AirBoard currently synchronizes text only. Older documents that mentioned image synchronization described planned work, not the active protocol.

## Security model

AirBoard is designed to protect clipboard plaintext from passive LAN observers and active network attackers after users verify pairing fingerprints correctly.

It does not protect against:

- a compromised or malicious paired device;
- malware, accessibility services, clipboard managers, or screenshots on an endpoint;
- a user approving a fingerprint without comparing it on the other device;
- traffic analysis such as observing that two IP addresses communicate;
- operating-system backups or compromise of the platform credential store.

The UI says **E2EE active** only while at least one authenticated peer is connected. Merely enabling synchronization is shown as a waiting state.

See [TECHNICAL_DETAILS.md](TECHNICAL_DETAILS.md) for the protocol and threat model and [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for platform-specific recovery steps.

Before publishing a release, complete the Linux, Windows, Android, and iPad coverage in [TESTING.md](TESTING.md). The testing branch uses GitHub Actions to produce a Windows website download and an unsigned iPadOS/iOS artifact; neither should be treated as production-ready until that matrix passes.

## Pairing

1. Put both devices on the same trusted LAN and enable synchronization.
2. Select the discovered peer, or enter its IP address manually.
3. The receiving device verifies the signed request and displays the requester's full fingerprint.
4. Compare that fingerprint with the requester device and approve only if every group matches.
5. The requester then verifies the signed response and displays the responder's fingerprint.
6. Compare it with the responder device and approve.
7. AirBoard stores the peer's public identity and starts an encrypted session.

Pairing prompts expire after two minutes. Protocol v1 trust records should be removed and paired again after upgrading.

## Development setup

Requirements:

- Flutter compatible with Dart `>=3.3.0 <4.0.0`
- Rust stable
- Platform build tools for the target OS
- Android: JDK 17, Android SDK, NDK `25.1.8937393`
- Apple targets: a complete Xcode installation and CocoaPods

Install dependencies:

```bash
flutter pub get
cargo check --manifest-path rust/Cargo.toml
```

Regenerate bridge bindings after changing an exported Rust API:

```bash
flutter_rust_bridge_codegen generate
```

The current hardening work retained existing exported signatures, so the checked-in bindings remain valid.

## Verification

Run the checks used by this repository:

```bash
flutter analyze
flutter test
cargo test --manifest-path rust/Cargo.toml
```

The Rust suite covers authenticated-encryption tampering, associated-data binding, transcript-bound key derivation, malformed fixed-length fields, replay rejection, sequence allocation, handshake transcript identity binding, and clipboard deduplication.

Platform builds:

```bash
flutter build linux --release
flutter build windows --release
flutter build macos --release
flutter build apk --debug
flutter build ipa --release --no-codesign
```

### Android release signing

Release builds are deliberately blocked unless `android/key.properties` points to a private release keystore. AirBoard no longer falls back to the Android debug signing key.

Example local file, which is ignored by Git:

```properties
storeFile=/absolute/path/to/airboard-release.jks
storePassword=replace-me
keyAlias=airboard
keyPassword=replace-me
```

Then run:

```bash
flutter build apk --release
```

Never commit the keystore or `key.properties`.

## Network ports

| Port | Scope | Purpose |
| --- | --- | --- |
| UDP `45454` | LAN | Device announcements and discovery |
| TCP `45455` | LAN | Pairing, authenticated handshake, encrypted peer traffic |
| TCP `45456` | Android loopback only | Rust-to-Kotlin incoming clipboard notification bridge |
| TCP `45457` | Android loopback only | Kotlin-to-Rust explicit clipboard-send bridge |

Only `45454` and `45455` should be allowed through a LAN firewall. Never expose the peer port directly to the public internet.

## Repository layout

- `lib/` — Flutter application, pairing UI, settings, device and clipboard views.
- `rust/src/core/` — active discovery, pairing, cryptography, session, peer, lifecycle and clipboard code.
- `android/`, `ios/`, `macos/`, `linux/`, `windows/` — native platform runners.
- `test/` and `integration_test/` — UI and bridge tests.
- `web_showcase/` — static product website; it is not a clipboard client.
- `releases/` — legacy artifacts awaiting protocol-v2 rebuilds.

## Current verification boundary

The source passes Rust checks/tests and Flutter analysis/widget tests in the current development environment. Apple compilation still requires a complete local Xcode installation. Android compilation additionally requires enough disk space to install the configured NDK. These environment requirements are documented rather than reported as successful builds when they were not executed.

## License

See [LICENSE](LICENSE).
