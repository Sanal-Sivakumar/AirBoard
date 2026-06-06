# Background Clipboard Sync via Notification Actions (Android)

This plan implements background clipboard synchronization on Android (addressing Android 10+ background read/write blocks) using a persistent notification action button and a transparent activity.

## User Review Required

> [!IMPORTANT]
> * **Android Clipboard Restrictions**: Android 10+ blocks background services from modifying the system clipboard.
> * **The Solution**: 
>   1. When the PC syncs a clip to the phone in the background, the app updates the persistent notification to display the copied text (e.g. `Synced: "hello..."`) and adds a **"Copy"** action button.
>   2. Tapping **"Copy"** launches a lightweight, transparent activity (`ClipboardWriteActivity`) that briefly takes focus, writes the text to the clipboard, displays a "Copied to clipboard!" toast, and terminates instantly.
>   3. This is 100% compliant with Android security policies and provides a seamless user experience.

---

## Proposed Changes

### Android Native

#### [NEW] [ClipboardWriteActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardWriteActivity.kt)
- Create a transparent activity that writes intent-extra text to the clipboard, displays a short Toast message, and finishes immediately.

#### [MODIFY] [ClipboardSyncService.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/ClipboardSyncService.kt)
- Update `onStartCommand` to handle an `UPDATE_NOTIFICATION` action.
- Update the notification builder to include the synced text preview and a "Copy" action button targeting `ClipboardWriteActivity`.

#### [MODIFY] [MainActivity.kt](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/kotlin/com/example/clipboard/MainActivity.kt)
- Add a `"showSyncNotification"` handler to the method channel to send update intents to `ClipboardSyncService`.

#### [MODIFY] [AndroidManifest.xml](file:///home/sanal-sivakumar/Documents/clipboard/android/app/src/main/AndroidManifest.xml)
- Declare `ClipboardWriteActivity` with a translucent theme: `android:theme="@android:style/Theme.Translucent.NoTitleBar"`.

---

### Flutter App

#### [MODIFY] [lib/main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- When a sync event is received and `_lifecycleState != "resumed"`, invoke the `"showSyncNotification"` method channel call with the synced content.

---

## Verification Plan

### Automated/Manual Tests
1. **Compilation**: Run `flutter build apk --debug` to verify the Kotlin and manifest changes compile successfully.
2. **Background Sync Check**:
   - Minimize the app on the phone.
   - Copy "Hello from PC!" on the Linux PC.
   - Verify a notification appears on the phone showing `Synced: "Hello from PC!"` with a **"Copy"** button.
   - Tap **"Copy"**, verify the "Copied to clipboard!" toast appears, and paste the text into another app (e.g. browser/messenger) to verify it is copied successfully.
