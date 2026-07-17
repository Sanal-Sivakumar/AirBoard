# AirBoard Protocol-v2 Device Test Plan

Do not publish a production release until every required row in this plan passes. Use only non-sensitive sample text during testing.

## 1. Build sources

| Device | Test build source | Installation |
| --- | --- | --- |
| Linux laptop | Local source checkout | `flutter run -d linux` or `flutter build linux --release` |
| Android phone | Local source checkout over USB debugging | `flutter run -d DEVICE_ID` |
| Windows PC | GitHub Pages Windows test ZIP | Extract the complete ZIP and run the included executable |
| iPad | GitHub Actions unsigned IPA artifact | Download while signed into GitHub, then sign and sideload with a trusted Apple-ID-based tool |

All devices must use the same protocol-v2 commit. Remove old AirBoard installations and trust records before beginning.

## 2. GitHub Actions artifacts

After pushing the testing branch:

1. Open the repository's **Actions** tab.
2. Wait for **Verify Protocol v2** to pass on Ubuntu and Windows.
3. Wait for **Build Windows Test App and Deploy Pages** to pass.
4. Open the deployed website and download `AirBoard-Windows-v2-test.zip`.
5. Wait for **Build iPadOS and iOS Test IPA** to pass.
6. Open that workflow run and download the `AirBoard-iPadOS-v2-unsigned-test` artifact.

The iPad artifact is unsigned. iPadOS will not install it until it is signed for the test device. GitHub Actions does not possess or store the developer's Apple credentials in this testing workflow.

## 3. Clean installation

On every device:

- uninstall or stop every protocol-v1 AirBoard build;
- install the protocol-v2 build from the same commit;
- clear old AirBoard application data or remove all trusted devices;
- connect to the same trusted Wi-Fi LAN;
- keep TCP `45455` and UDP `45454` private to that LAN;
- confirm the displayed app starts without an exception or blank screen.

USB connects the Android phone to Flutter for installation and logs. Clipboard traffic still uses the LAN.

## 4. Pairwise coverage matrix

Test every pair in both directions. A check means both directions passed.

| Pair | Pairing | Text A to B | Text B to A | Reconnect | Remove trust |
| --- | --- | --- | --- | --- | --- |
| Linux ↔ Android | ☐ | ☐ | ☐ | ☐ | ☐ |
| Linux ↔ Windows | ☐ | ☐ | ☐ | ☐ | ☐ |
| Linux ↔ iPad | ☐ | ☐ | ☐ | ☐ | ☐ |
| Windows ↔ Android | ☐ | ☐ | ☐ | ☐ | ☐ |
| Windows ↔ iPad | ☐ | ☐ | ☐ | ☐ | ☐ |
| Android ↔ iPad | ☐ | ☐ | ☐ | ☐ | ☐ |

For every pairing:

1. Confirm both devices display a prompt.
2. Compare every group of the full fingerprint.
3. Approve on both devices only after the values match.
4. Confirm both devices change from **Waiting for peer** to **E2EE active**.
5. Reject one deliberate pairing attempt and confirm no trust entry is saved.

## 5. Clipboard cases

Run these cases in both directions for every device pair:

| Case | Expected result |
| --- | --- |
| `AirBoard hello 123` | Exact text arrives once |
| Multiple lines and emoji | Newlines and UTF-8 characters remain unchanged |
| Empty clipboard | No crash or unwanted broadcast |
| Same text copied twice | No echo storm or repeated history growth |
| Rapidly copy 20 numbered values | App remains responsive; final value converges |
| Text slightly below 512 KiB | Transfer succeeds |
| Text above 512 KiB | Transfer is rejected without disconnecting or crashing |
| Image or file copy | It is ignored because protocol v2 is text-only |

Do not use passwords, recovery codes, API keys, or personal messages as test values.

## 6. Network and lifecycle cases

For each platform:

- disable and re-enable synchronization;
- disconnect Wi-Fi for 30 seconds, reconnect, and verify session recovery;
- change the device's LAN address and verify discovery or manual-IP recovery;
- restart AirBoard and verify trusted peers reconnect without another pairing prompt;
- remove a trusted peer and confirm its next connection is rejected;
- verify no duplicate peer appears after both devices start simultaneously;
- leave the pair connected for at least 30 minutes and check heartbeat stability.

Platform-specific checks:

- **Android:** foreground copy is automatic; background operation may require notification **Sync** and **Copy** actions. Confirm notification text does not reveal clipboard contents.
- **iPad:** background the app, confirm sessions pause honestly, return to foreground, and verify reconnection. Continuous background clipboard access is not expected.
- **Windows:** allow AirBoard only on Private networks and confirm the complete extracted directory runs without missing-DLL errors.
- **Linux:** test under the actual X11 or Wayland session used day-to-day.

## 7. Security regression checks

- A fingerprint mismatch is rejected.
- An unpaired device cannot establish a session.
- A removed device cannot reconnect as trusted.
- Protocol-v1 clients cannot pair or exchange clipboard content with protocol v2.
- Lock-screen notifications do not include clipboard text.
- Logs show content lengths and state changes, not clipboard previews.
- The website labels Windows as a test build and does not expose retired v1 downloads.

## 8. Record results

For every failure, record:

```text
Commit:
Source device and OS version:
Destination device and OS version:
Network type:
Exact steps:
Expected result:
Actual result:
Relevant logs with clipboard contents removed:
```

A production release is ready only after all automated workflows pass, all six device pairs pass in both directions, background limitations match the documentation, and no security regression remains open.
