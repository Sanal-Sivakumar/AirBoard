# Implementation Plan: Android Background Sync Reliability & Notification UX Improvements

This plan outlines changes to:
1. **Enable completely seamless background sync on Android**: Establish a direct local TCP socket loopback between Android's background service (`ClipboardWriteActivity`/Kotlin) and the Rust core Tokio runtime. This bypasses the Flutter MethodChannel entirely, ensuring clipboard changes copied on Android are synced to other devices even when the Flutter app is closed or backgrounded.
2. **Improve Android notification buttons**:
   - Replace the persistent manual `"Sync"` button (send) with a highly useful **`"Pause"` / `"Resume"`** button to temporarily suspend syncing for privacy/battery control.
   - Rename the transient incoming clip action button from `"Sync"` to **`"Copy"`**, and only show it when the "Display over other apps" (Overlay) permission is missing (as overlay permission allows writing to the clipboard automatically).
   - Set the content intent of the notification so that tapping it opens the app.

---

## User Review Required

> [!IMPORTANT]
> - **Background Sync State on Desktop**: On Windows and Linux, closing the app window exits the process. To sync in the background, you must keep the app running (minimized). If you prefer, we can implement a system tray package in a follow-up task to let the app run in the system tray when closed.
> - **Direct Local Loopback**: We will use a local TCP port `45457` on `127.0.0.1` for Kotlin-to-Rust background communication. This is completely safe as it is bounded to localhost and doesn't expose any external network ports.

---

## Proposed Changes

### Rust Core

#### [NEW] [android.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/android.rs)
- Implement `start_android_local_receiver()`: binds to `127.0.0.1:45457` and listens for local TCP connections.
- When a connection is accepted, read the raw string payload (the copied clipboard text) and call `SYNC_ENGINE.process_local_change(&content)` to broadcast the update to paired devices.

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/core/clipboard/mod.rs)
- Expose the new `android` module conditionally under `#[cfg(target_os = "android")]`.

#### [MODIFY] [api/mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/api/mod.rs)
- Conditionally spawn `start_android_local_receiver()` inside `start_sync` if `platform == "android"`.

---

### Android Native

#### [MODIFY] [ClipboardSyncService.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardSyncService.kt)
- Add a static/companion volatile `isPaused` flag to track whether clipboard synchronization is temporarily suspended.
- In `onCreate`, check `isPaused` in `addPrimaryClipChangedListener` and return early if active.
- In `startLocalServer`, if `isPaused` is true, discard incoming sync texts and do not trigger notifications.
- Define a new `ACTION_TOGGLE_PAUSE = "com.example.clipboard.TOGGLE_PAUSE"` intent action.
- Update `onStartCommand` to handle `ACTION_TOGGLE_PAUSE`, toggle `isPaused`, and call `refreshNotification()`.
- Update `createNotification` to:
  - Add a `setContentIntent` pointing to `MainActivity` so tapping the notification opens the app.
  - Set the first action button to `"Pause"` (or `"Resume"` if paused) targeting `ACTION_TOGGLE_PAUSE` via `PendingIntent.getService(...)`.
  - Add the second action button as **`"Copy"`** (instead of `"Sync"`) only when `syncText` is not null AND overlay permission is not granted AND `isPaused` is false.

#### [MODIFY] [ClipboardWriteActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardWriteActivity.kt)
- In `onWindowFocusChanged` for `read_and_send` mode:
  - Spin up a background thread to connect to `127.0.0.1:45457` and write the clipboard text directly to the Rust background receiver socket.
  - Retain the MethodChannel invocation as a non-critical fallback/UI logger helper.

---

## Verification Plan

### Automated Tests
- Run `flutter analyze` to ensure code integrity.

### Manual Verification
1. Open the app on Android and PC, make sure they are paired.
2. Minimize the Android app. Copy text on Android. Verify that it immediately syncs to the PC.
3. Close the Android app (swipe it away from recents). Copy text on Android. Verify that it still syncs to the PC because the foreground service and Rust receiver are running.
4. Copy text on PC. Verify that it writes automatically on Android (if overlay permission is active) or shows a notification with a **`[Copy]`** action button (if overlay permission is inactive).
5. Verify the persistent notification shows **`[Pause]`**. Tap it, confirm that it changes to **`[Resume]`** and the status changes to **"Sync Paused"**.
6. While paused, copy text on Android or PC, verify that no sync events are processed or written. Tap **`[Resume]`** and verify that sync resumes normally.
7. Tap the notification body itself, confirm it opens the AirBoard app.
