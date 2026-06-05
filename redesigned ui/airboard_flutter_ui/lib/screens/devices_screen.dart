import 'dart:async';
import 'package:flutter/material.dart';
import '../theme.dart';
import '../models.dart';
import '../widgets/glass.dart';
import '../widgets/tiles.dart';
import '../widgets/pairing_sheet.dart';

/// The single "Devices" destination — merges the old Discovered + Trusted +
/// clipboard history into one clean, scrollable surface.
class DevicesScreen extends StatefulWidget {
  final bool compact; // true on mobile
  const DevicesScreen({super.key, this.compact = false});
  @override
  State<DevicesScreen> createState() => _DevicesScreenState();
}

class _DevicesScreenState extends State<DevicesScreen> {
  final _paired = MockData.paired();
  final _discovered = MockData.discovered();
  final _history = MockData.history();
  final _ipCtrl = TextEditingController();
  Timer? _liveSync;

  // simulated incoming clips (delete this block when wiring real data)
  final _incoming = [
    ClipItem(type: ClipType.text, text: 'Meeting moved to 4:30 — conf room B', source: 'iPad Client'),
    ClipItem(type: ClipType.link, text: 'https://airboard.app/security/zero-trust', source: 'MacBook Pro'),
    ClipItem(type: ClipType.code, text: 'ChaCha20-Poly1305 · nonce check passed', source: 'Android Phone'),
  ];
  int _inc = 0;

  @override
  void initState() {
    super.initState();
    _liveSync = Timer.periodic(const Duration(seconds: 9), (_) {
      setState(() {
        final item = _incoming[_inc % _incoming.length];
        _inc++;
        for (final h in _history) {
          if (h.time == 'now') h.time = '1m ago';
        }
        _history.insert(0,
            ClipItem(type: item.type, text: item.text, source: item.source, time: 'now'));
        if (_history.length > 6) _history.removeLast();
      });
    });
  }

  @override
  void dispose() {
    _liveSync?.cancel();
    _ipCtrl.dispose();
    super.dispose();
  }

  Future<void> _pair(Device d) async {
    final ok = await showPairingSheet(context, d.name);
    if (ok && mounted) {
      setState(() {
        _discovered.remove(d);
        d.paired = true;
        _paired.add(d);
      });
      showGlassToast(context, '${d.name} paired · E2EE established');
    }
  }

  void _unpair(Device d) {
    setState(() => _paired.remove(d));
    showGlassToast(context, '${d.name} unpaired');
  }

  void _manualConnect() {
    final ip = _ipCtrl.text.trim();
    if (ip.isEmpty) {
      showGlassToast(context, 'Enter an IP first');
      return;
    }
    _pair(Device(name: 'Device @ $ip', ip: ip, kind: DeviceKind.desktop));
  }

  @override
  Widget build(BuildContext context) {
    final c = widget.compact;
    return ListView(
      padding: EdgeInsets.fromLTRB(c ? 18 : 26, c ? 6 : 24, c ? 18 : 26, 28),
      children: [
        if (!c) ...[_statusBar(), const SizedBox(height: 22)],
        _connectRow(),
        const SizedBox(height: 6),
        SectionTitle('Paired', count: c ? null : _paired.length),
        ..._gap(_paired
            .map((d) => DeviceTile(
                device: d, compact: c, onUnpair: () => _unpair(d)))
            .toList()),
        const SizedBox(height: 12),
        SectionTitle('Discovered nearby',
            count: c ? null : _discovered.length),
        ..._gap(_discovered
            .map((d) =>
                DeviceTile(device: d, compact: c, onPair: () => _pair(d)))
            .toList()),
        const SizedBox(height: 12),
        SectionTitle('Clipboard history',
            count: c ? null : _history.length),
        ..._gap(_history
            .map((h) => ClipTile(item: h, compact: c))
            .toList()),
      ],
    );
  }

  List<Widget> _gap(List<Widget> items) {
    final out = <Widget>[];
    for (var i = 0; i < items.length; i++) {
      out.add(items[i]);
      if (i != items.length - 1) out.add(const SizedBox(height: 10));
    }
    return out;
  }

  Widget _statusBar() {
    Widget stat(String k, String v) => Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(k.toUpperCase(), style: AB.label),
            const SizedBox(height: 2),
            Text(v,
                style: AB.title.copyWith(fontSize: 19, letterSpacing: -.2)),
          ],
        );
    Widget sep() => Container(width: 1, height: 30, color: AB.stroke);

    return GlassCard(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
      child: Row(children: [
        stat('Discovered', '${_discovered.length}'),
        const SizedBox(width: 24),
        sep(),
        const SizedBox(width: 24),
        stat('Trusted', '${_paired.length}'),
        const SizedBox(width: 24),
        sep(),
        const SizedBox(width: 24),
        stat('Last sync', '8s ago'),
        const Spacer(),
        _eePill(),
      ]),
    );
  }

  Widget _eePill() => Container(
        padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 7),
        decoration: BoxDecoration(
          color: AB.ok.withOpacity(.10),
          borderRadius: BorderRadius.circular(AB.rPill),
          border: Border.all(color: AB.ok.withOpacity(.30)),
        ),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          const PulseDot(size: 7),
          const SizedBox(width: 8),
          Text('E2EE ACTIVE',
              style: AB.label.copyWith(color: AB.ok, letterSpacing: .6)),
        ]),
      );

  Widget _connectRow() {
    return Padding(
      padding: const EdgeInsets.only(top: 12, bottom: 6),
      child: Row(children: [
        Expanded(
          child: GlassCard(
            radius: AB.rSm,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: TextField(
              controller: _ipCtrl,
              style: AB.body,
              decoration: InputDecoration(
                isDense: true,
                border: InputBorder.none,
                hintText: widget.compact
                    ? 'Enter IP…'
                    : 'Add device by IP  ·  e.g. 192.168.43.10',
                hintStyle: AB.body.copyWith(color: AB.text3),
                contentPadding: const EdgeInsets.symmetric(vertical: 14),
              ),
            ),
          ),
        ),
        const SizedBox(width: 10),
        _PrimaryButton(
          icon: Icons.link_rounded,
          label: widget.compact ? null : 'Connect',
          onTap: _manualConnect,
        ),
      ]),
    );
  }
}

class _PrimaryButton extends StatelessWidget {
  final IconData icon;
  final String? label;
  final VoidCallback onTap;
  const _PrimaryButton({required this.icon, this.label, required this.onTap});
  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: EdgeInsets.symmetric(
            horizontal: label == null ? 15 : 20, vertical: 13),
        decoration: BoxDecoration(
          gradient: AB.accentGrad,
          borderRadius: BorderRadius.circular(AB.rSm),
          boxShadow: [
            BoxShadow(color: AB.glow, blurRadius: 22, spreadRadius: -6)
          ],
        ),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Icon(icon, size: 16, color: const Color(0xFF0A0B12)),
          if (label != null) ...[
            const SizedBox(width: 8),
            Text(label!,
                style: AB.body.copyWith(
                    color: const Color(0xFF0A0B12),
                    fontWeight: FontWeight.w600)),
          ],
        ]),
      ),
    );
  }
}
