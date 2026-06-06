# Implementation Plan: Clipboard History Limit & Image Sync Integration

This plan outlines the changes required to limit the local clipboard history to 10 items and to upgrade AirBoard to support copying, syncing, and pasting pictures/images across Windows, Linux, iOS, and Android.

## User Review Required

> [!IMPORTANT]
> To support images without changing the underlying cryptographic P2P transport protocols, images will be converted to Base64-encoded PNG strings (`data:image/png;base64,...`) and transmitted as text payloads. 
> 
> *   **On Windows & Linux**: The Rust clipboard monitor will watch for image copies, encode them to PNG/Base64, and set them back to the system clipboard upon reception.
> *   **On iOS/iPadOS**: The Swift runner will intercept image copies, encode them, and decode them back into `UIPasteboard.general.image` upon reception.
> *   **On Android**: The Kotlin backend will extract images from URI clips and sync them as Base64. (To avoid registering complex FileProviders, incoming synced images can be saved/previewed in the app UI).

## Proposed Changes

### 1. Project Configuration & Dependencies
#### [MODIFY] [Cargo.toml](file:///home/sanal-sivakumar/Documents/clipboard/rust/Cargo.toml)
*   Add `png = "0.17"` under target OS dependencies for Windows/Linux:
    ```toml
    [target.'cfg(any(target_os = "linux", target_os = "windows"))'.dependencies]
    arboard = "3.4.0"
    png = "0.17"
    ```

### 2. Rust Core (Linux & Windows Clipboard Interceptor)
#### [MODIFY] [desktop.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/desktop.rs)
*   Implement `rgba_to_png` and `png_to_rgba` helpers using the `png` crate.
*   Update `start_desktop_clipboard_monitor` to watch for images (`clipboard.get_image()`) if text has not changed or is not available.
*   Update `write_to_desktop_clipboard` to check if a payload starts with `"data:image/png;base64,"`, decode it, and write it as a native image using `arboard::Clipboard::set_image`.

### 3. iOS Runner (Swift Clipboard Interceptor)
#### [MODIFY] [AppDelegate.swift](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/AppDelegate.swift)
*   Update `getClipboardText` MethodChannel call handler: if `UIPasteboard.general.string` is empty but `UIPasteboard.general.image` exists, convert it to PNG data, base64-encode it, and return `"data:image/png;base64,{base64_data}"`.
*   Update `setClipboardText` MethodChannel call handler: if text starts with `"data:image/png;base64,"`, decode the base64 data and write it directly to `UIPasteboard.general.image`.

### 4. Android Runner (Kotlin Clipboard Interceptor)
#### [MODIFY] [ClipboardWriteActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardWriteActivity.kt)
*   Update clipboard reader in `read_and_send` mode: check if the primary clip item has a `uri`. If so, read its byte data from `contentResolver`, encode it as Base64, prepend `"data:image/png;base64,"`, and broadcast it.

### 5. Flutter App & UI Elements
#### [MODIFY] [models.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/models.dart)
*   Add `ClipType.image` to the `ClipType` enum.
*   Associate `ClipType.image` with `Icons.image_rounded`.

#### [MODIFY] [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
*   In `_addClipToHistory`:
    *   Change the history length check from `> 20` to `> 10`.
    *   Add a check to set the type to `ClipType.image` if the text starts with `"data:image/png;base64,"`.

#### [MODIFY] [tiles.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/widgets/tiles.dart)
*   In `ClipTile`:
    *   Check if `item.type == ClipType.image`.
    *   If so, render a thumbnail of the base64-decoded image using `Image.memory(base64Decode(...))` inside a styled, rounded `ClipRRect` block instead of the text box.

---

## Verification Plan

### Automated Tests
- Run `flutter analyze lib/` to verify Flutter code compiles successfully.
- Run `cargo check` in the `rust` folder to verify the Rust changes compile.

### Manual Verification
- Copy an image on Windows/Linux (e.g. print screen or copy from browser) and verify that it synchronizes to iPadOS and updates the system clipboards.
- Copy an image from the Photos app on iPadOS and verify that it syncs to Windows/Linux.
- Verify that the app maintains a maximum of 10 clipboard history items.
