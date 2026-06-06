# Implementation Plan: Interactive 3D Showcase Website (AirBoard)

This plan outlines the design and implementation of an Apple-inspired, 3D interactive showcase website for AirBoard. The website will be fully self-contained using HTML, Vanilla CSS, and JavaScript with Three.js (via CDN) for 3D interactions.

---

## Proposed Changes

We will create a new folder `web_showcase` in the project root containing:
1. `index.html`: Holds the page structure, Google Fonts, and Three.js canvas container.
2. `style.css`: Contains custom variables, grid layouts, responsive rules, glassmorphism tokens, and keyframe animations.
3. `app.js`: Houses the logic for scroll-triggered entrance animations, interactive page states, and the Three.js 3D viewport rendering a floating, interactive "AirBoard" sync hub.

---

### [NEW] [index.html](file:///home/sanal-sivakumar/Documents/clipboard/web_showcase/index.html)
- Define structure:
  - **Header/Navbar**: Glassmorphic sticky header with logo, navigation links, and a call-to-action (CTA) "Download" button.
  - **Hero Section**: Large typography ("AirBoard. Copy here. Paste there. Instantly."), text gradients, main download CTA, and background Three.js canvas.
  - **3D Interactive Section**: An interactive viewport where users can rotate and zoom a floating 3D dashboard displaying a connection grid.
  - **Features Grid**: Translucent cards showcasing:
    - *Local P2P Sync*: Zero-configuration local sync.
    - *Military-grade Security*: Cryptographic pairing keys, local-only storage, and Pause/Resume privacy button.
    - *Background Efficiency*: Android direct Rust socket loopback.
  - **Requirements**: Clear table displaying compatibility across Android, Linux, and Windows, highlighting permission requirements (Overlay / Battery Optimization).
  - **Download Section**: High-end landing cards targeting:
    - **Android (APK)**: Link pointing directly to `/build/app/outputs/flutter-apk/app-release.apk`.
    - **Linux (Zip)**: Link pointing directly to `/airboard-linux.zip`.
    - **Windows**: Tagged "Build from source / Coming soon".
  - **Footer**: Apple-style copyright and quick links.

---

### [NEW] [style.css](file:///home/sanal-sivakumar/Documents/clipboard/web_showcase/style.css)
- Implement a modern dark-mode palette:
  - Deep space dark background (`#000000` to `#0d0d11`).
  - Neon gradients (electric blue `#0071e3`, cyan `#00f2fe`, and deep violet `#7f00ff`).
- Use Google Fonts **Outfit** (for headings) and **Inter** (for body text).
- Apply custom glassmorphic cards (`background: rgba(255, 255, 255, 0.03); backdrop-filter: blur(20px); border: 1px solid rgba(255, 255, 255, 0.08)`).
- Implement slide-in/fade-in class-based animations triggered by scroll positions.

---

### [NEW] [app.js](file:///home/sanal-sivakumar/Documents/clipboard/web_showcase/app.js)
- **Three.js Scene**:
  - Initialize WebGL renderer, camera, and scene inside a background `#three-canvas`.
  - Render a glowing, semi-transparent 3D central hub (AirBoard) with floating Orbit rings and multiple floating nodes (representing devices).
  - Add interactive mouse-move listeners: moving the cursor causes the 3D scene to tilt slightly (parallax effect).
  - Add scroll listeners: scrolling down moves the camera closer or rotates the hub to show connection streams.
- **Scroll Observer**:
  - Use `IntersectionObserver` to trigger entrance animations as sections enter the viewport.

---

## Verification Plan

### Manual Verification
1. Open [index.html](file:///home/sanal-sivakumar/Documents/clipboard/web_showcase/index.html) in a standard web browser (Chrome, Firefox, or Edge).
2. Check page design for aesthetics, layout spacing, responsive scaling on mobile views, and text contrast.
3. Interact with the 3D scene: verify it rotates automatically, tilts on mouse movement, and changes orientation on page scroll.
4. Verify that clicking on "Download Android APK" and "Download Linux Bundle" triggers download of the generated releases.
