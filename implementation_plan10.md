# Hotspot Discovery & Pairing Improvements

This implementation plan outlines the steps to resolve issues with hotspot-based discovery, introduce manual IP pairing, clean up the pairing dialog design, and detail the technical background of iOS clipboard sync and notification behavior.

## Technical Details & User Review Required

### iOS system popup and background restrictions
> [!IMPORTANT]
> - **System Overlays**: iOS strictly sandboxes third-party apps and does not allow them to display custom popup overlays on top of other apps (unlike Android's `SYSTEM_ALERT_WINDOW` permission). Only iOS system dialogs can do this.
> - **Clipboard Background Access**: Background processes are blocked by iOS from writing to `UIPasteboard.general`. The app must be brought to the foreground to commit a clipboard write.
> - **Heads-Up Notifications**: The notification banner serves as the heads-up display. To sync, the user taps the banner, which opens the app in the foreground and copies the text.

### Auto-replacing (non-stacking) notifications
> [!TIP]
> Currently, every sync notification uses a new random UUID, causing them to stack up in the iOS Notification Center. 
> We will change this to use a constant identifier (`"airboard_clipboard_sync"`). This ensures that **only the latest copied message is displayed**, and any previous sync notification is automatically replaced and cleared.

---

## Proposed Changes

### [iOS Platform integration]

#### [MODIFY] [AppDelegate.swift](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/AppDelegate.swift)
- Change the `UNNotificationRequest` identifier from `UUID().uuidString` to the static string `"airboard_clipboard_sync"`.
- This automatically updates/cancels the old notification banner when a new sync event arrives.

### [Rust Core API]

#### [MODIFY] [mod.rs](file:///home/sanal-sivakumar/Documents/clipboard/rust/src/api/mod.rs)
- Add `initiate_pairing_to_ip(ip_or_addr: String)` function.
- Parse `ip` and optional `port` (defaulting to `45455`).
- Spawn a runtime task executing `initiate_pairing_flow` with a dummy peer ID (e.g., `"manual_connection"`).

### [Android Platform integration]

#### [MODIFY] [AndroidManifest.xml](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/AndroidManifest.xml)
- Add `android.permission.CHANGE_WIFI_MULTICAST_STATE` to permissions list.

#### [MODIFY] [ClipboardSyncService.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardSyncService.kt)
- Define `multicastLock: WifiManager.MulticastLock? = null`.
- In `onCreate`, acquire the lock using `wifiManager.createMulticastLock("AirBoard:MulticastLock")`.
- In `onDestroy`, release the lock to prevent battery/resource leaks.

### [Flutter Frontend]

#### [MODIFY] [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- Declare `final _manualIpController = TextEditingController()` inside `_SyncHomeScreenState`.
- Dispose of `_manualIpController` inside `dispose()`.
- Restyle the **Deny** button in `_showPairingRequestDialog`:
  - Change foreground color from `Colors.redAccent` to `Colors.white60` (or `Color(0xFF94A3B8)`).
  - Change border from `Colors.redAccent.withOpacity(0.5)` to `Color(0xFF475569)`.
- Update `_buildDiscoveredTab()` layout:
  - Add a **Manual Connection** section card at the top.
  - Include a styled text field for IP input and a "Connect" button that invokes `api.initiatePairingToIp`.

---

## Verification Plan

### Automated Tests
- Run `flutter_rust_bridge_codegen generate` to generate the new bindings.
- Validate that the app compiles on Linux via `flutter run -d linux`.

### Manual Verification
1. Run on Linux and verify the UI loads.
2. Check the "Devices" tab; the "Manual Connection" entry card should be displayed.
3. Check the "Pairing Request" UI: the "Deny" button must be a calm slate outline instead of red.
4. Verify notifications on iOS: copy multiple times from Linux, and check that notifications do not stack up on the iPad (only the latest message notification should remain visible).
