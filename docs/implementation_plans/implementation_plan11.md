# AirBoard Glassmorphism Responsive UI Redesign

This plan details the implementation of a high-fidelity glassmorphism design system in Flutter, incorporating vector-sharp SVG icons, ambient gradient backgrounds, and optimized responsive layouts for Phone, Tablet, and Desktop.

## User Review Required

> [!NOTE]
> - **Visual Tokens**: We will establish uniform glass styling (`glass`, `glass-sm`, `glass-deep`) using transparent color fills and thin white borders.
> - **Ambient Gradients**: Multiple radial gradients will be placed behind the screen content using a background Stack to create high-depth ambient glows.
> - **State Alignment**: All layout navigation models (bottom tabs, topbar segments, sidebars) will be bound to a single unified state index `_activeNavIndex`.

---

## Proposed Changes

### [Flutter UI Redesign]

#### [MODIFY] [main.dart](file:///home/sanal-sivakumar/Documents/clipboard/lib/main.dart)
- **Import Packages**: Add `package:flutter_svg/flutter_svg.dart` to support vector icon assets.
- **Icon Assets Constants**: Define a class `AppIcons` holding the exact SVG strings provided in the assets HTML.
- **Glassmorphic Design Utility Widgets**:
  - `_renderSvg(...)`: Helper method to render SVG strings with a target color and size.
  - `PulsingDot`: Stateful widget utilizing `AnimationController` to animate a pulsing network indicator.
  - `_buildCustomToggle(...)`: Animated switch replacing Flutter's default `Switch.adaptive`.
  - `_buildStatusPill(...)`: Badge widget for status indicators.
- **Responsive Layout Builders**:
  - Implement a `LayoutBuilder` inside `build()` to delegate rendering:
    - Width < 600: `_buildPhoneLayout()`
    - 600 <= Width < 900: `_buildTabletLayout()`
    - Width >= 900: `_buildDesktopLayout()`
- **Navigation State Integration**:
  - Replace the `TabController` with a single navigation state `_activeNavIndex` (0: Devices, 1: Trusted, 2: Logs, 3: Settings).
  - Synchronize tab taps, segmented items, and sidebar taps to this index.
- **Sub-panels**:
  - `_buildDevicesPane()`: Renders the Devices tab layout (Manual Connection card + Nearby Devices list).
  - `_buildTrustedPane()`: Renders the list of Trusted/paired peers.
  - `_buildLogsPane()`: Renders the scrollable console of security events.
  - `_buildSettingsPane()`: Renders the settings panels (Auto-connect, Device Name, etc.).

---

## Verification Plan

### Automated Tests
- Validate that the Dart codebase passes analysis using `flutter analyze`.

### Manual Verification
1. Run on Linux and expand/shrink the window to test responsiveness:
   - Width < 600: bottom navigation appears, titlebar collapses.
   - 600 <= Width < 900: sidebar appears, segmented control topbar visible.
   - Width >= 900: PC-specific titlebar with traffic lights, stats row cards, and full network/system sidebar sections.
2. Confirm the visual design matches the mockups:
   - Frosted borders on cards, ambient top-left/bottom-right gradients.
   - Pulse animation on E2EE indicators.
   - Customized toggle switches.
