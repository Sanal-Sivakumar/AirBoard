# Implementation Plan: iPadOS Background Clipboard Synchronization

This plan implements background execution support for iPadOS (iOS) to enable seamless, real-time clipboard synchronization even when the app is minimized (running in the background).

## Background Context & Constraint Management

On iOS/iPadOS, background processes are strictly limited. Apps running socket connections (like our WebSocket clients) are automatically suspended or terminated 10–30 seconds after being backgrounded. To bypass this, we will implement the **Background Audio** mode. By playing an infinite, silent audio loop, iOS treats our process as an active audio playback task, keeping the process active and the WebSocket client/discovery socket connections alive indefinitely.

Because `UIPasteboard` is restricted from being read or written directly in the background, we will:
1. **Send Updates**: When a clipboard change is received from the PC/Android device while the iPad app is minimized, we will buffer the update in memory and show a local notification (`"Synced from PC: <text>"`).
2. **Apply Updates**: As soon as the user opens the app (either normally or by tapping the notification), we will write the buffered text to the system clipboard instantly.
3. **Receive Updates**: Since the WebSocket connection is kept alive in the background by the silent audio, the iPad receives the PC's copied text instantly, ensuring a zero-delay sync upon returning to the app (no reconnect or discovery delay).

---

## Proposed Changes

### iOS Native Config & Code

#### [MODIFY] [Info.plist](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/Info.plist)
- Declare the `UIBackgroundModes` key containing `audio` to inform the OS that the app requires background audio capabilities.

```xml
	<key>UIBackgroundModes</key>
	<array>
		<string>audio</string>
	</array>
```

#### [MODIFY] [AppDelegate.swift](file:///home/sanal-sivakumar/Documents/clipboard/ios/Runner/AppDelegate.swift)
- Import `AVFoundation` and `UserNotifications`.
- Request local notification permissions on app startup.
- Implement `SilentAudioManager` to generate a lightweight, silent 1-second WAV file in the temporary directory dynamically, load it into an `AVAudioPlayer` with looping enabled, and configure `AVAudioSession` to mix with other audio sources.
- Add MethodChannel handlers:
  - `startSilentAudio`: Invokes `SilentAudioManager.shared.start()`.
  - `stopSilentAudio`: Invokes `SilentAudioManager.shared.stop()`.
  - `showLocalNotification`: Triggers a native system notification with a title and body.

---

### Flutter Application Logic

#### [MODIFY] [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)

- **Lifecycle Suspension Bypass**:
  - Update `didChangeAppLifecycleState` to prevent calling `api.handleAppBackground()` on iOS when `_isSyncEnabled` is active. This keeps the Rust background networking thread active.
  
- **Background Notification Routing**:
  - Update `_handleRustEvent` so that when a clipboard update is received on iOS, if the app is backgrounded (`_lifecycleState != "Resumed"`), it buffers the text in `_pendingClipboardWrite` and triggers the native `'showLocalNotification'` method channel call.

- **Audio Control Hook**:
  - Update `_toggleSync` so that when enabling sync on iOS, we call `_clipboardChannel.invokeMethod('startSilentAudio')`. When disabling sync, we call `_clipboardChannel.invokeMethod('stopSilentAudio')` and trigger `api.handleAppBackground()`.

---

## Verification Plan

### Automated/Compilation Tests
- Verify compilation of Flutter/iOS changes (static checks).
- Run `flutter analyze` to ensure no Dart syntax errors.

### Manual Verification
1. Open the AirBoard application on iPad, toggle sync **ON**.
2. Verify that the system prompts for local notification permission and approve it.
3. Minimize the app on the iPad.
4. From the Linux PC, copy some text.
5. Verify on iPad:
   - A local notification appears showing `"Synced from PC: <text>"`.
   - Tap the notification (which opens the app).
   - Verify that the text is immediately written to the iPad clipboard, and pasting it into another app matches the text.
6. Toggle sync **OFF** on the iPad, verify the background audio player stops.
