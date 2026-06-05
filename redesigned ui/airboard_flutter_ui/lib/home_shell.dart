import 'package:flutter/material.dart';
import 'theme.dart';
import 'widgets/glass.dart';
import 'screens/devices_screen.dart';
import 'screens/settings_screen.dart';

/// Responsive shell: desktop => glass sidebar, mobile => floating glass
/// bottom-nav. Only TWO destinations (Logs removed).
class HomeShell extends StatefulWidget {
  const HomeShell({super.key});
  @override
  State<HomeShell> createState() => _HomeShellState();
}

class _HomeShellState extends State<HomeShell> {
  int _index = 0;

  static const _dests = [
    (_NavItem(icon: Icons.devices_rounded, label: 'Devices')),
    (_NavItem(icon: Icons.settings_rounded, label: 'Settings')),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.transparent,
      body: AmbientBackground(
        child: LayoutBuilder(
          builder: (context, constraints) {
            final desktop = constraints.maxWidth >= 720;
            return desktop ? _desktop() : _mobile();
          },
        ),
      ),
    );
  }

  Widget _content({required bool compact}) {
    final screen = _index == 0
        ? DevicesScreen(compact: compact)
        : SettingsScreen(compact: compact);
    return AnimatedSwitcher(
      duration: const Duration(milliseconds: 350),
      switchInCurve: AB.ease,
      transitionBuilder: (child, anim) => FadeTransition(
        opacity: anim,
        child: SlideTransition(
          position: Tween(begin: const Offset(0, .03), end: Offset.zero)
              .animate(anim),
          child: child,
        ),
      ),
      child: KeyedSubtree(key: ValueKey(_index), child: screen),
    );
  }

  // ---------------- DESKTOP ----------------
  Widget _desktop() {
    return Row(
      children: [
        Container(
          width: 232,
          padding: const EdgeInsets.fromLTRB(14, 22, 14, 18),
          decoration: BoxDecoration(
            border: Border(right: BorderSide(color: AB.stroke)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _brand(),
              const SizedBox(height: 20),
              for (var i = 0; i < _dests.length; i++)
                _navTile(_dests[i], i),
              const Spacer(),
              _securePill(),
            ],
          ),
        ),
        Expanded(child: _content(compact: false)),
      ],
    );
  }

  Widget _brand() => Padding(
        padding: const EdgeInsets.fromLTRB(8, 4, 8, 0),
        child: Row(children: [
          Container(
            width: 34,
            height: 34,
            decoration: BoxDecoration(
              gradient: AB.accentGrad,
              borderRadius: BorderRadius.circular(10),
              boxShadow: [
                BoxShadow(color: AB.glow, blurRadius: 18, spreadRadius: -4)
              ],
            ),
            child: const Icon(Icons.content_paste_rounded,
                size: 18, color: Color(0xFF0A0B12)),
          ),
          const SizedBox(width: 11),
          Text('AirBoard', style: AB.h1.copyWith(fontSize: 16)),
        ]),
      );

  Widget _navTile(_NavItem item, int i) {
    final active = _index == i;
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: GestureDetector(
        onTap: () => setState(() => _index = i),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 220),
          curve: AB.ease,
          padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 11),
          decoration: BoxDecoration(
            color: active ? AB.glassStrong : Colors.transparent,
            borderRadius: BorderRadius.circular(AB.rSm),
            border: Border.all(
                color: active ? AB.strokeHi : Colors.transparent),
          ),
          child: Row(children: [
            Icon(item.icon,
                size: 18, color: active ? AB.accent : AB.text2),
            const SizedBox(width: 12),
            Text(item.label,
                style: AB.body
                    .copyWith(color: active ? AB.text : AB.text2)),
          ]),
        ),
      ),
    );
  }

  Widget _securePill() => GlassCard(
        radius: AB.rPill,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 11),
        child: Row(children: [
          const PulseDot(size: 8),
          const SizedBox(width: 9),
          Text('SECURE · 3 PEERS',
              style: AB.label.copyWith(color: AB.text, letterSpacing: .5)),
        ]),
      );

  // ---------------- MOBILE ----------------
  Widget _mobile() {
    return SafeArea(
      child: Column(
        children: [
          _mobileHeader(),
          Expanded(child: _content(compact: true)),
          _bottomNav(),
        ],
      ),
    );
  }

  Widget _mobileHeader() => Padding(
        padding: const EdgeInsets.fromLTRB(20, 14, 20, 6),
        child: Row(children: [
          Container(
            width: 46,
            height: 46,
            decoration: BoxDecoration(
              gradient: AB.accentGrad,
              borderRadius: BorderRadius.circular(14),
              boxShadow: [
                BoxShadow(color: AB.glow, blurRadius: 22, spreadRadius: -5)
              ],
            ),
            child: const Icon(Icons.content_paste_rounded,
                size: 24, color: Color(0xFF0A0B12)),
          ),
          const SizedBox(width: 13),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('This Phone', style: AB.h1.copyWith(fontSize: 18)),
              Text('D3:0D:CF:34…', style: AB.monoSm),
            ],
          ),
          const Spacer(),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: AB.ok.withOpacity(.10),
              borderRadius: BorderRadius.circular(AB.rPill),
              border: Border.all(color: AB.ok.withOpacity(.30)),
            ),
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              const PulseDot(size: 6),
              const SizedBox(width: 6),
              Text('E2EE',
                  style: AB.label
                      .copyWith(color: AB.ok, fontSize: 10.5, letterSpacing: .5)),
            ]),
          ),
        ]),
      );

  Widget _bottomNav() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 6, 16, 14),
      child: GlassCard(
        radius: 24,
        padding: const EdgeInsets.all(8),
        child: Row(
          children: [
            for (var i = 0; i < _dests.length; i++)
              Expanded(child: _bottomTab(_dests[i], i)),
          ],
        ),
      ),
    );
  }

  Widget _bottomTab(_NavItem item, int i) {
    final active = _index == i;
    return GestureDetector(
      onTap: () => setState(() => _index = i),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 220),
        curve: AB.ease,
        margin: const EdgeInsets.symmetric(horizontal: 3),
        padding: const EdgeInsets.symmetric(vertical: 9),
        decoration: BoxDecoration(
          color: active ? AB.glassStrong : Colors.transparent,
          borderRadius: BorderRadius.circular(16),
          border:
              Border.all(color: active ? AB.strokeHi : Colors.transparent),
        ),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          Icon(item.icon, size: 21, color: active ? AB.accent : AB.text3),
          const SizedBox(height: 4),
          Text(item.label,
              style: AB.label.copyWith(
                  fontSize: 11,
                  letterSpacing: .2,
                  color: active ? AB.text : AB.text3)),
        ]),
      ),
    );
  }
}

class _NavItem {
  final IconData icon;
  final String label;
  const _NavItem({required this.icon, required this.label});
}
