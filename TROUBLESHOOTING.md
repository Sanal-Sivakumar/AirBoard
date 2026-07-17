# AirBoard Troubleshooting

This guide applies to protocol v2 in the current source tree. Older binaries in `releases/` and legacy website downloads use an earlier protocol and must not be mixed with current builds.

The website's Windows protocol-v2 package and the GitHub Actions iPadOS/iOS artifact are testing builds. Follow [TESTING.md](TESTING.md) and use non-sensitive sample clipboard data until cross-device validation is complete.

## Start here

Run these checks from the repository root:

```bash
flutter doctor -v
flutter pub get
flutter analyze
flutter test
cargo test --manifest-path rust/Cargo.toml
```

All participating devices must be built from the same protocol-v2 revision. After upgrading from an older build, remove the old trusted-device entry and pair again.

## Devices do not appear

AirBoard discovery uses UDP port `45454`. Peer and pairing traffic uses TCP port `45455`.

Check that:

- both devices are on the same LAN or hotspot;
- synchronization is enabled on both devices;
- the operating-system firewall allows AirBoard on private/local networks;
- UDP `45454` and TCP `45455` are not blocked;
- a VPN is not routing or filtering local traffic;
- the router does not have AP/client isolation enabled.

If discovery is blocked but direct LAN traffic works, enter the peer's LAN IP manually. Manual pairing still uses TCP `45455`; ports `45456` and `45457` are Android loopback bridges and are not manual-pairing ports.

Useful checks:

```bash
# macOS/Linux: replace PEER_IP
nc -vz PEER_IP 45455

# Linux firewall examples
sudo ufw allow 45454/udp
sudo ufw allow 45455/tcp
```

Do not port-forward TCP `45455` to the public internet.

## Pairing fails or times out

Pairing requires user approval on both devices. Each prompt displays the full SHA-256 fingerprint of the other device.

- Compare every fingerprint group using the other device's screen or a separate trusted channel.
- Reject a fingerprint that differs, even by one group.
- Complete each prompt within two minutes.
- Keep both apps open during pairing, especially on iOS/iPadOS.
- Verify both devices run protocol v2.

The initial TCP connection times out after 15 seconds. Pairing responses larger than 64 KiB are rejected by the requester, while the shared server applies its 1 MiB WebSocket limit. If a previously trusted device changed or reinstalled its identity keys, remove it and pair again; AirBoard intentionally does not silently accept key rotation.

## Connected, but clipboard text does not move

AirBoard currently synchronizes UTF-8 text only. Images, files, rich text, and passwords withheld by an operating system or password manager are not supported.

Check that:

- at least one authenticated peer is shown as connected;
- the status says **E2EE active**, not **Waiting for peer** or **Paused**;
- the text is at most 512 KiB;
- synchronization is enabled;
- both devices remain on the same network after pairing;
- the clipboard manager or password manager is not marking the value as private.

If a device changed IP address, allow discovery/reconnection a few seconds or reconnect using its new IP. AirBoard exchanges last-writer state after reconnection, but equal or misleading device-clock timestamps can prevent conflict recovery. Copy the desired value again to create a new update.

## Duplicate updates or reconnect loops

Protocol v2 has packet-ID deduplication, authenticated session sequence numbers, replay rejection, and deterministic connection initiation. If duplicate updates persist:

1. Confirm every device is on protocol v2.
2. Stop all older AirBoard processes or services.
3. Disable and re-enable synchronization.
4. Remove stale trust records and pair again.

Do not run a legacy binary and a source build simultaneously on the same machine.

## Android

### Background clipboard limitations

Android 10 and newer restrict clipboard access for background apps. AirBoard does not request overlay permission and does not open transparent activities to bypass that policy.

- Keep AirBoard foregrounded for automatic read/write behavior.
- When backgrounded, tap **Sync** in the foreground-service notification to read and send the current clipboard.
- Tap **Copy** on an incoming notification to explicitly place received text on the clipboard.

Notification text does not reveal clipboard contents.

### Service is stopped

Allow notifications and exempt AirBoard from aggressive vendor battery optimization if the foreground network service is repeatedly killed. On some devices, the relevant setting is under Battery, Background usage, Auto-start, or Sleeping apps.

### Android build cannot install the NDK

The project expects NDK `25.1.8937393`. Ensure the Android SDK path is correct and enough free disk space is available:

```bash
flutter doctor -v
df -h
sdkmanager "ndk;25.1.8937393"
flutter build apk --debug
```

An incomplete NDK directory should be repaired through `sdkmanager`; do not commit machine-specific `android/local.properties` changes.

### Release signing fails

Release builds are blocked unless `android/key.properties` points to a private keystore. This is intentional; release artifacts must never use the debug signing key. Follow the signing example in `README.md`. Do not commit the keystore or credentials.

## iOS and iPadOS

iOS supports foreground-oriented clipboard synchronization. It suspends arbitrary networking and clipboard polling after the app is backgrounded.

- Accept the Local Network permission prompt.
- Keep AirBoard open while pairing or actively syncing.
- After returning to the app, allow a few seconds for discovery and reconnection.
- Enable notifications if you want privacy-preserving arrival alerts.

AirBoard does not play silent audio or claim continuous background clipboard access.

### Apple build fails before compilation

Install a complete Xcode release, select it, accept the license, and install CocoaPods:

```bash
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license accept
sudo xcodebuild -runFirstLaunch
pod --version
flutter doctor -v
```

Then retry `flutter build macos` or `flutter build ipa --no-codesign`.

## macOS

The runner includes sandbox network-client and network-server entitlements. If discovery or clipboard access still fails:

- allow AirBoard under System Settings > Network > Firewall;
- restart the app after changing firewall permissions;
- verify no other process occupies ports `45454` or `45455`;
- ensure the full Xcode toolchain is selected for builds.

## Linux

The desktop clipboard backend requires a graphical clipboard environment. If it cannot read the clipboard, verify the process has access to the active X11 or Wayland session and that required Flutter desktop packages are installed.

For Ubuntu/Debian Flutter desktop prerequisites:

```bash
sudo apt-get install clang cmake ninja-build pkg-config libgtk-3-dev liblzma-dev
```

## Windows

Allow AirBoard through Windows Defender Firewall on Private networks. A Public network profile may block discovery. Confirm that the Visual Studio Desktop development with C++ workload is installed before building.

## Security-related errors

These errors should not be bypassed:

- **Fingerprint mismatch** — reject pairing and confirm you selected the intended device.
- **Incompatible protocol** — rebuild all peers from the same revision.
- **Invalid signature / authentication failed** — remove the untrusted connection; do not retry by blindly approving prompts.
- **Replay or stale sequence** — restart synchronization only after confirming no legacy process is running.
- **Payload too large** — reduce the text below 512 KiB.

Malformed, unauthenticated, replayed, misaddressed, and oversized messages are rejected by design.

## Collecting diagnostics safely

Run:

```bash
flutter run -v
RUST_BACKTRACE=1 cargo test --manifest-path rust/Cargo.toml
```

AirBoard logs clipboard lengths and state changes rather than clipboard previews, but logs may still contain device IDs, names, IP addresses, and error context. Review them before sharing publicly. Never post private keys, `key.properties`, keystores, platform credential-store exports, or clipboard contents.

For protocol details and current limitations, see [TECHNICAL_DETAILS.md](TECHNICAL_DETAILS.md).
