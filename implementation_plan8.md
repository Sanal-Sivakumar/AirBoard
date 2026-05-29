# Plan: Seamless Background Clipboard Synchronization (Android)

To enable automatic background clipboard synchronization on Android even when the app is minimized (backgrounded), we will request the **"Display over other apps" (Overlay/`SYSTEM_ALERT_WINDOW`)** permission. Under Android's Background Activity Launch (BAL) rules, holding this permission allows the foreground service to launch the translucent `ClipboardWriteActivity` automatically in the background when a clipboard update is received. This activity will briefly gain focus, write the text to the system clipboard, and finish immediately.

## Proposed Changes

### 1. Android Manifest

#### [MODIFY] [AndroidManifest.xml](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/AndroidManifest.xml)
- Declare the `SYSTEM_ALERT_WINDOW` permission:
  ```xml
  <uses-permission android:name="android.permission.SYSTEM_ALERT_WINDOW" />
  ```

---

### 2. Android Native Bridge & Service

#### [MODIFY] [MainActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/MainActivity.kt)
- Add MethodChannel handlers:
  - `checkOverlayPermission`: Returns `true` if `Settings.canDrawOverlays(this)` is true (or SDK < M), `false` otherwise.
  - `requestOverlayPermission`: Launches settings screen (`Settings.ACTION_MANAGE_OVERLAY_PERMISSION`) with package URI to prompt user to grant it.

#### [MODIFY] [ClipboardSyncService.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardSyncService.kt)
- Import `android.provider.Settings`.
- Inside `updateNotification(syncText)`:
  - Check if the overlay permission is granted: `Settings.canDrawOverlays(this)`.
  - If granted:
    - Log the start of `ClipboardWriteActivity`.
    - Launch `ClipboardWriteActivity` automatically using `startActivity()` with flags `FLAG_ACTIVITY_NEW_TASK` and a unique action (e.g. timestamp appended).
    - Update the foreground notification with the preview text, but without the "Copy" action button (since the text is written automatically).
  - If not granted:
    - Fall back to the existing behavior: update the notification and include the manual "Copy" action button.

---

### 3. Flutter Application Logic

#### [MODIFY] [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- Inside `_toggleSync(bool value)`:
  - If `value == true` and running on Android:
    - Check if overlay permission is granted by calling `checkOverlayPermission` on the `_serviceChannel`.
    - If not granted, show a user-friendly custom dialog explaining that background clipboard synchronization requires the "Display over other apps" (Overlay) permission.
    - If the user selects "Grant", call `requestOverlayPermission` to launch settings, then proceed to start sync.
- Display overlay permission status on the UI if possible, or log it to the security and sync console.

---

## Verification Plan

### Automated / Compilation Tests
- Run `flutter build apk --debug` to ensure all native Kotlin changes and AndroidManifest changes compile successfully.

### Manual Verification
1. Open the Android application in the foreground, toggle "Secure Sync" ON.
2. Verify that a dialog pops up asking for overlay permission. Tap "Grant" and toggle the permission "ON" in the system settings page for AirBoard.
3. Minimize the app.
4. From the Linux PC app, copy some text.
5. Verify on Android:
   - The notification updates automatically to say `"Synced: \"<copied-text>\""`.
   - The text is immediately copied to the Android system clipboard without requiring any tap on the notification.
   - Paste the text on the Android phone to verify it matches.
6. Verify via `adb logcat | grep -E "ClipboardSyncService|ClipboardWriteActivity"` that `ClipboardWriteActivity` was started from the service background socket thread and finished successfully.
