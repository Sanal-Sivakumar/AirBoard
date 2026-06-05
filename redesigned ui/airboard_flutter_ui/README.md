# AirBoard — Aurora Liquid Glass UI

Drop-in Flutter UI for AirBoard. **UI only** — no networking/crypto; wire your
existing backend into the screens where the mock data lives.

## Files
```
lib/
├─ main.dart                 # standalone demo entry (delete in your app)
├─ theme.dart                # AB design tokens + ThemeData (Aurora palette)
├─ models.dart               # Device / ClipItem + MockData  ← replace with your types
├─ home_shell.dart           # responsive shell: desktop sidebar / mobile bottom-nav
├─ screens/
│  ├─ devices_screen.dart    # status bar · manual connect · paired/discovered · history
│  └─ settings_screen.dart   # device name · Secure Sync · system metrics
└─ widgets/
   ├─ glass.dart             # GlassCard, AmbientBackground, PulseDot, GlassToggle, toast
   ├─ tiles.dart             # DeviceTile, ClipTile, StatusBadge, SectionTitle
   └─ pairing_sheet.dart     # zero-trust fingerprint approval dialog
```

## Use in your existing app
1. Copy `lib/theme.dart`, `lib/home_shell.dart`, `lib/screens/`, `lib/widgets/`,
   and (optionally) `lib/models.dart` into your project.
2. Add the font dependency:
   ```yaml
   dependencies:
     google_fonts: ^6.2.1
   ```
   (Or bundle Inter/JetBrains Mono as assets and swap the `GoogleFonts.*` calls in
   `theme.dart` for `TextStyle(fontFamily: ...)` — useful for offline/desktop builds.)
3. Apply the theme and show the shell:
   ```dart
   MaterialApp(theme: AB.theme(), home: const HomeShell());
   ```

## Wiring real data (where to plug in)
- **Device lists / history:** `screens/devices_screen.dart` seeds from
  `MockData`. Replace `_paired`, `_discovered`, `_history` with your state
  (Provider/Riverpod/Bloc/`ValueListenable` — your call) and delete the
  `_liveSync` timer + `_incoming` demo block.
- **Pair flow:** `_pair()` calls `showPairingSheet(...)` → returns `true` on
  approve. Call your real pairing/E2EE handshake there.
- **Unpair / connect / toggle:** `_unpair()`, `_manualConnect()`,
  Secure-Sync `onChanged` in `settings_screen.dart`.
- **Copy:** `ClipTile` already calls `Clipboard.setData`; hook your re-sync there.

## Design tokens (all in `theme.dart` → `AB`)
- bg `#06070D` · accent `#6EA8FF → #A78BFA` · ok `#34D399` · warn `#FBBF24`
- glass fill `white .055`, stroke `.10`, specular `.22`, blur `30`
- radii 26/18/12/pill · easing `Cubic(.22,1,.36,1)`

## Responsive
`HomeShell` switches at **720px**: ≥720 → desktop sidebar; <720 → mobile
header + floating glass bottom-nav. Tablet landscape uses the desktop layout.

## Run the demo
```bash
flutter pub get
flutter run            # mobile/emulator
flutter run -d macos   # or -d windows / -d linux for the desktop layout
```
