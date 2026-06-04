# Implementation Plan: Windows Platform Support Configuration

This plan details the steps required to configure and support Windows desktop in the AirBoard Flutter app and the underlying Rust core.

## User Review Required

> [!NOTE]
> We will configure the project to support Windows in exactly the same way as Linux. The Rust core will handle clipboard monitoring and updates natively on Windows via the `arboard` crate, which is fully cross-platform.
>
> Since the build environment is currently Linux, we will generate the Windows platform files, update all dependencies, and configure target-conditional compilation. This ensures that the workspace will build cleanly on Windows.

---

## Proposed Changes

### [Flutter Application]

#### [NEW] [Windows Runner Directories](file:///home/sanal-sivakumar/Documents/clipboard/windows)
- Generate the native Windows runner folder and cmake configuration files using `flutter create --platforms=windows .`

#### [MODIFY] [pubspec.yaml](file:///home/sanal-sivakumar/Documents/clipboard/pubspec.yaml)
- Add `windows: true` to the `flutter_launcher_icons` configuration to build launcher icons for Windows.

#### [MODIFY] [lib/main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- Update [initState](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart#L159-L184) to set the default device name to `"Windows PC"` when running on Windows.
- Update storage directory path logic to use `Platform.environment['APPDATA']` as the local config root on Windows.
- Update dynamic platform string checking to pass `"windows"` to the Rust sync initializer.

---

### [Rust Core Engine]

#### [MODIFY] [Cargo.toml](file:///home/sanal-sivakumar/Documents/clipboard/rust/Cargo.toml)
- Update target-conditional dependency for `arboard` to include both Linux and Windows: `target.'cfg(any(target_os = "linux", target_os = "windows"))'`.

#### [NEW] [desktop.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/desktop.rs)
- Move and rename `rust/src/core/clipboard/linux.rs` to [desktop.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/desktop.rs), renaming methods to `start_desktop_clipboard_monitor` and `write_to_desktop_clipboard` since they are fully cross-platform under `arboard`.

#### [DELETE] [linux.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/linux.rs)
- Remove the Linux-specific file, now replaced by the generic `desktop.rs`.

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/mod.rs)
- Declare the `desktop` module conditionally under `cfg(any(target_os = "linux", target_os = "windows"))`.

#### [MODIFY] [api/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/api/mod.rs)
- Import `start_desktop_clipboard_monitor` conditionally and spawn it if the platform is `"linux"` or `"windows"`.

#### [MODIFY] [peer_manager/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/peer_manager/mod.rs)
- Import `write_to_desktop_clipboard` conditionally and call it to update the native clipboard when receiving updates.

---

## Verification Plan

### Automated Tests
- Run `cargo check` to verify the Rust library compiles correctly on the host Linux OS.
- Run `flutter analyze` to ensure the Dart frontend code compiles without syntax or type errors.
