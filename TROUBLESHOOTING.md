# AirBoard: Troubleshooting & Resolution Log

This document archives the major engineering challenges, system bugs, OS restrictions, and network constraints encountered during the development of AirBoard, alongside the exact solutions implemented.

---

## 1. Android Background Clipboard Restrictions (Android 10+)

### Symptom
When the application is minimized or swiped away from the screen, copy-paste synchronization fails to capture local clipboard actions or write incoming network updates. The application throws a security exception or returns empty clipboard buffers.

### Root Cause
Beginning with Android 10 (API 29), Google introduced strict clipboard data privacy policies. Applications can no longer access the system `ClipboardManager` (neither read nor write) while running in the background. Only the active Input Method Editor (the system keyboard) or the currently focused foreground activity can read/write to the clipboard.

### Resolution
We resolved this by designing a native-to-core hybrid bridge architecture combining a Kotlin Foreground Service, an transparent Activity, and a local loopback network channel:
1. **Foreground Service**: We implemented `ClipboardSyncService` in Kotlin. It runs as a persistent service with a foreground notification type of `dataSync` (fully compatible with Android 14 requirements).
2. **OnPrimaryClipChangedListener**: We registered a native clipboard listener inside the foreground service.
3. **Display Over Other Apps (Overlay)**: We request the overlay permission (`SYSTEM_ALERT_WINDOW`).
4. **Local Loopback socket (`127.0.0.1:45457`)**: The background Rust runtime listens on a local loopback TCP port. 
   * **For Reading**: When the primary clip listener in Kotlin fires, if it is a genuine user copy, it spawns an asynchronous thread that opens a TCP connection to `127.0.0.1:45457` and writes the clipboard text directly to Rust.
   * **For Writing**: When Rust receives a sync payload, it connects to Kotlin over the loopback socket. Kotlin opens an invisible, transparent activity (`ClipboardWriteActivity`) which momentarily grabs window focus, writes the payload to the native system clipboard, and immediately finishes itself.

---

## 2. Infinite Clipboard Feedback Loops (Broadcast Storms)

### Symptom
Copying a text string on Device A successfully sends it to Device B. However, once B writes it, B's local clipboard listener flags this write as a new copy event, sends it back to A, and triggers an infinite back-and-forth loop that crashes the sockets or freezes the devices.

### Root Cause
Operating system clipboard listeners (such as Android's `OnPrimaryClipChangedListener` or Linux X11 monitors) are stateless. They trigger whenever clipboard content changes but do not native distinguish between a manual *user copy action* and a programmatic *pasteboard write action*.

### Resolution
We implemented a two-tiered loop-prevention protocol at the Rust and platform levels:
1. **Cryptographic Deduplication (Rust)**: We maintain the SHA-256 hashes of the last written and last sent payloads. If an incoming payload's hash matches the last written hash, it is discarded.
2. **Ignore Flags & State Sync Channels (Platform)**:
   * We added a static `ignoreNextClipChange` boolean flag in `ClipboardSyncService.kt` and a `lastSentText` tracker in `ClipboardWriteActivity.kt`.
   * We created a Flutter-to-Kotlin `MethodChannel` called `updateLastSentText`.
   * When Rust successfully writes an incoming clipboard sync event, Flutter sends a call over the `updateLastSentText` method channel. This sets `ignoreNextClipChange = true` in the foreground service.
   * When the native primary clipboard listener triggers, it checks the flag. If it is `true`, it clears the flag and ignores the event, successfully breaking the feedback loop.

---

## 3. Enterprise Wi-Fi Network Isolation (Blocked UDP)

### Symptom
Devices connected to the same Wi-Fi router (especially in university libraries, corporate offices, or public cafes) cannot discover each other in the "Discovered Devices" tab.

### Root Cause
Public and corporate routers frequently implement **Client Isolation** or block **UDP Broadcast/Multicast** packets on the local subnet. This is done to prevent malicious local attacks and reduce network traffic. As a result, our UDP broadcasting discovery protocol on port `45455` cannot route packets between peers.

### Resolution
We implemented a manual unicast connection fallback:
1. **Network Info Exporter**: The Settings page displays the active local LAN IP address of the device (retrieved via network interface scans in Rust).
2. **Direct TCP Connection**: Users can manually type the peer's local IP address into the input field under the "Devices" tab.
3. **Bypassing UDP**: Entering the IP address bypasses UDP discovery and attempts to directly establish a TCP stream on port `45457`. Since TCP unicast routing is rarely blocked on local networks, this succeeds in initiating the secure handshakes.

---

## 4. Background Thread Expiry & Doze Mode

### Symptom
The synchronization stops working after the device (especially Android or iPadOS) remains inactive or screen-locked for several minutes.

### Root Cause
1. **Android Doze Mode / Battery Optimization**: The operating system turns off network antennas, suspends CPU threads, and drops multicast lock states to extend battery life.
2. **iOS Background Execution Limits**: Apple suspends background application threads within 10-30 seconds of the user swiping the application away.

### Resolution
1. **On Android**:
   * We acquire a `WifiMulticastLock` inside `ClipboardSyncService` using the Android `WifiManager`. This forces the device's network hardware to keep processing incoming UDP discovery packets during sleep states.
   * We require users to disable "Battery Optimization" for AirBoard in system settings to keep the background threads active.
2. **On iOS & iPadOS**:
   * We run a silent audio loop utility using the Apple `AVFoundation` framework. Playing a silent audio stream keeps the application's background thread priority active.
   * If the thread is suspended, we fallback to local system notifications. The user receives a banner alert when clipboard data is synchronized; tapping the notification launches the application and immediately flushes the memory sync buffer onto the system clipboard.
