# AirBoard - Decentralized Cross-Platform P2P Clipboard Sync

AirBoard is a free, open-source, zero-trust, serverless peer-to-peer (P2P) clipboard synchronization system. It securely shares clipboard data across Android, Linux, iPadOS, and Windows devices on your local network (LAN) in less than 50 milliseconds.

Unlike cloud-dependent alternatives that upload your sensitive data to third-party databases, AirBoard operates entirely locally. Your copy-paste streams are encrypted end-to-end and transmitted directly between your paired devices.

---

## 📥 Direct Download Links

You can download the pre-compiled beta binaries directly from the links below:
*   🤖 **Android (APK)**: [airboard-android.apk](https://Sanal-Sivakumar.github.io/AirBoard/releases/airboard-android.apk)
*   🐧 **Linux (ZIP)**: [airboard-linux.zip](https://Sanal-Sivakumar.github.io/AirBoard/releases/airboard-linux.zip)
*   ❖ **Windows (ZIP)**: [airboard-windows.zip](https://Sanal-Sivakumar.github.io/AirBoard/releases/airboard-windows.zip)
*   📱 **iOS & iPadOS (IPA)**: [AirBoard-ipadosios.ipa](https://Sanal-Sivakumar.github.io/AirBoard/releases/AirBoard-ipadosios.ipa)

---

## 📖 Deep-Dive Reference Manuals
To learn more about the engineering details of this project, check out these dedicated files:
*   **[Technical Reference Guide](TECHNICAL_DETAILS.md)** ([Local File Link](file:///home/sanal-sivakumar/Documents/clipboard/TECHNICAL_DETAILS.md)): A textbook-style guide to AirBoard's P2P networking topology, UDP broadcasts, X25519/Ed25519 cryptography, and Dart-to-Rust memory bindings.
*   **[Troubleshooting & Resolution Log](TROUBLESHOOTING.md)** ([Local File Link](file:///home/sanal-sivakumar/Documents/clipboard/TROUBLESHOOTING.md)): A chronological story detailing OS constraints (Android background limits, iOS doze execution), network isolation, and thread-panics solved during development.

---

## 🌐 Product Showcase Website

The product showcase website is located in the [web_showcase/](file:///home/sanal-sivakumar/Documents/clipboard/web_showcase) directory.

*   **Local Host Address**: [http://localhost:8080](http://localhost:8080)
*   **Launch Command**: `python3 -m http.server 8080 --directory web_showcase`

---

## 🛠️ Technology Stack
*   **Cross-Platform UI**: Flutter (Dart SDK >= 3.3.0) with an adaptive, glassmorphic layout.
*   **System Core Engine**: Rust (Edition 2021) for multi-threaded socket operations, file systems, and timers.
*   **Interoperability**: `flutter_rust_bridge` (v2.12.0) for zero-copy memory translation between Dart and Rust.
*   **End-to-End Encryption (E2EE)**: `ed25519-dalek` (device signing keys), `x25519-dalek` (Diffie-Hellman ephemeral handshakes), and `chacha20poly1305` (symmetric payload encryption).
*   **Key Storage**: System Keystore/Keyring integrations via `flutter_secure_storage`.
*   **Discovery Protocol**: UDP sockets on port `45455`.
*   **Synchronization Link**: TCP sockets on port `45457`.

---

## 🚀 Building & Running Client Builds

### Prerequisites
For Linux host environments, install the standard compilation headers:
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libx11-dev libxcb1-dev
```

Configure Rust Android targets for compilation:
```bash
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
```

Perform the bridge code generation:
```bash
flutter_rust_bridge_codegen generate
```

### 1. Linux Desktop
Run the client directly:
```bash
flutter run -d linux
```
To build the release zip:
```bash
flutter build linux --release
```

### 2. Android (Phones & Tablets)
Run the application on an active device/emulator:
```bash
flutter run -d android
```
To compile the standalone release APK:
```bash
flutter build apk --release
```
*The output binary will be generated at `build/app/outputs/flutter-apk/app-release.apk`.*

### 3. iPadOS & iOS
Due to Apple compiler constraints, building for iPadOS requires a **macOS** computer with **Xcode** installed:
```bash
flutter build ipa --release --no-codesign
```
*The output `.ipa` package will be located under `build/ios/ipa/`.*
*   To install on an iPad, refer to the **iPad Sideloading Guide** on our website or inside the [sideload instructions section](file:///home/sanal-sivakumar/Documents/clipboard/web_showcase/index.html#sideload-guide).

### 4. Windows Desktop
Building for Windows requires a **Windows** host machine with **Visual Studio (C++ Desktop development)** and the **Rust (MSVC)** toolchain installed:
```cmd
flutter build windows --release
```
*The compiled binary files will be generated inside `build/windows/x64/runner/Release/`.*

---

## 🤝 Support & Contributions
AirBoard is open-source and welcoming to community contributions:
*   **Official Repository**: [https://github.com/Sanal-Sivakumar/AirBoard](https://github.com/Sanal-Sivakumar/AirBoard)
*   **Developer Contact**: [sanalsiva2005@gmail.com](mailto:sanalsiva2005@gmail.com)
