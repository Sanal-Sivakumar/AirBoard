import 'package:flutter/material.dart';
import '../theme.dart';
import '../models.dart';
import '../widgets/glass.dart';
import '../widgets/tiles.dart';

class DevicesScreen extends StatelessWidget {
  final bool compact;
  final List<Device> pairedDevices;
  final List<Device> discoveredDevices;
  final List<ClipItem> clipboardHistory;
  final TextEditingController manualIpController;
  final bool isSyncEnabled;
  final String lastSyncTimestamp;
  final ValueChanged<Device> onPair;
  final ValueChanged<Device> onUnpair;
  final VoidCallback onManualConnect;

  const DevicesScreen({
    super.key,
    this.compact = false,
    required this.pairedDevices,
    required this.discoveredDevices,
    required this.clipboardHistory,
    required this.manualIpController,
    required this.isSyncEnabled,
    required this.lastSyncTimestamp,
    required this.onPair,
    required this.onUnpair,
    required this.onManualConnect,
  });

  @override
  Widget build(BuildContext context) {
    final c = compact;
    return ListView(
      padding: EdgeInsets.fromLTRB(c ? 18 : 26, c ? 6 : 24, c ? 18 : 26, 28),
      children: [
        if (!c) ...[_statusBar(context), const SizedBox(height: 22)],
        _connectRow(),
        const SizedBox(height: 6),
        SectionTitle('Paired', count: c ? null : pairedDevices.length),
        if (pairedDevices.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 4.0),
            child: Text(
              "No paired devices yet. Nearby devices will show up below.",
              style: AB.sub.copyWith(color: AB.text3, fontSize: 13),
            ),
          )
        else
          ..._gap(pairedDevices
              .map((d) => DeviceTile(
                  device: d, compact: c, onUnpair: () => onUnpair(d)))
              .toList()),
        const SizedBox(height: 12),
        SectionTitle('Discovered nearby',
            count: c ? null : discoveredDevices.length),
        if (discoveredDevices.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 4.0),
            child: Text(
              "Searching for nearby AirBoard devices on local Wi-Fi...",
              style: AB.sub.copyWith(color: AB.text3, fontSize: 13),
            ),
          )
        else
          ..._gap(discoveredDevices
              .map((d) =>
                  DeviceTile(device: d, compact: c, onPair: () => onPair(d)))
              .toList()),
        const SizedBox(height: 12),
        SectionTitle('Clipboard history',
            count: c ? null : clipboardHistory.length),
        if (clipboardHistory.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8.0, horizontal: 4.0),
            child: Text(
              "Your clipboard history is empty. Copy some text to synchronize!",
              style: AB.sub.copyWith(color: AB.text3, fontSize: 13),
            ),
          )
        else
          ..._gap(clipboardHistory
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

  Widget _statusBar(BuildContext context) {
    final connectedCount =
        pairedDevices.where((device) => device.online).length;
    Widget stat(String k, String v) => Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(k.toUpperCase(), style: AB.label),
            const SizedBox(height: 2),
            Text(v, style: AB.title.copyWith(fontSize: 19, letterSpacing: -.2)),
          ],
        );
    return GlassCard(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
      child: Wrap(
          spacing: 24,
          runSpacing: 14,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            stat('Discovered', '${discoveredDevices.length}'),
            stat('Trusted', '${pairedDevices.length}'),
            stat('Connected', '$connectedCount'),
            stat('Last sync', lastSyncTimestamp),
            _eePill(connectedCount),
          ]),
    );
  }

  Widget _eePill(int connectedCount) {
    final secure = connectedCount > 0;
    final color = secure
        ? AB.ok
        : isSyncEnabled
            ? AB.accent
            : AB.text3;
    final label = secure
        ? 'E2EE ACTIVE'
        : isSyncEnabled
            ? 'WAITING FOR PEER'
            : 'SYNC PAUSED';
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 7),
      decoration: BoxDecoration(
        color: color.withValues(alpha: .10),
        borderRadius: BorderRadius.circular(AB.rPill),
        border: Border.all(color: color.withValues(alpha: .30)),
      ),
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        PulseDot(size: 7, color: color),
        const SizedBox(width: 8),
        Text(label, style: AB.label.copyWith(color: color, letterSpacing: .6)),
      ]),
    );
  }

  Widget _connectRow() {
    return Padding(
      padding: const EdgeInsets.only(top: 12, bottom: 6),
      child: Row(children: [
        Expanded(
          child: GlassCard(
            radius: AB.rSm,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: TextField(
              controller: manualIpController,
              style: AB.body,
              decoration: InputDecoration(
                isDense: true,
                border: InputBorder.none,
                hintText: compact
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
          label: compact ? null : 'Connect',
          onTap: onManualConnect,
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
