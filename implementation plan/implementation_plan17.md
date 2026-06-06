# Implementation Plan: Android Background Auto-Sync & Label Renaming

This plan outlines the modifications needed to:
1. Automatically capture and sync clipboard changes from Android to iPad without requiring the user to click the notification button.
2. Rename the notification actions to "Sync".
3. Ensure that incoming sync events from the iPad automatically populate the Android clipboard (when overlay permission is granted) or via a manual "Sync" button click in the notification shade, while preventing infinite sync loopbacks.

## User Review Required

> [!IMPORTANT]
> - **Overlay Permission Required**: For automatic background syncing without user interaction, the Android app requires the "Display over other apps" (Overlay) permission. If not granted, the app will fallback to the notification action button.
> - **Loopback Prevention**: We are introducing a state-synchronization protocol between Dart (Flutter) and Kotlin (Android native) using a new MethodChannel call `updateLastSentText`. This ensures that when a peer updates the clipboard, Android knows not to treat that local update as a new clipboard event to send back to the peer.

## Proposed Changes

### Android Native

---

#### [MODIFY] [ClipboardSyncService.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardSyncService.kt)
- Rename the persistent action label from `"Sync to PC"` to `"Sync"`.
- Rename the incoming clip action label from `"Copy"` to `"Sync"`.
- Add a volatile `ignoreNextClipChange` static flag.
- Register an `OnPrimaryClipChangedListener` on the system `ClipboardManager` inside `onCreate`.
- On clip change events: if `ignoreNextClipChange` is true, clear the flag and ignore the change; otherwise, if overlay permission is granted, start `ClipboardWriteActivity` with action `"read_and_send"` to automatically capture and broadcast the new clipboard contents.

---

#### [MODIFY] [ClipboardWriteActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardWriteActivity.kt)
- Add a static `lastSentText` property to track the last synced text.
- In `read_and_send` mode, check if the read text matches `lastSentText`. If it does, skip sending to avoid redundant network updates. Otherwise, update `lastSentText` and notify the Dart side.
- In clipboard write mode, update `lastSentText` and set `ClipboardSyncService.ignoreNextClipChange = true` before writing to the system clipboard to prevent triggering a loopback broadcast.

---

#### [MODIFY] [MainActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/MainActivity.kt)
- Implement a method channel handler for `updateLastSentText` that updates `lastSentText` in `ClipboardWriteActivity` and sets `ignoreNextClipChange = true` in `ClipboardSyncService`.

### Flutter Dart App

---

#### [MODIFY] [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- In `_startAndroidClipboardPoller`, invoke the native `updateLastSentText` method after broadcasting a local clip update.
- In `_handleRustEvent`, invoke the native `updateLastSentText` method when successfully writing an incoming sync clipboard payload to the system clipboard.

---

## Verification Plan

### Automated Tests
- Run `flutter analyze lib/` to verify Flutter code compiles cleanly.
- Build the app on Android and run local linting.

### Manual Verification
1. Ensure the "Display over other apps" (Overlay) permission is granted on Android.
2. Copy text on Android (app minimized) and verify it automatically syncs to the iPad clipboard.
3. Copy text on iPad (app foregrounded/minimized) and verify a notification arrives on Android.
4. If overlay permission is active, confirm it writes to Android's clipboard automatically. If overlay permission is disabled, click the `"Sync"` button on the notification shade and confirm it is copied to the Android clipboard.
