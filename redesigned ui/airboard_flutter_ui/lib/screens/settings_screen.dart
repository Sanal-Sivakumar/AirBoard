import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme.dart';
import '../widgets/glass.dart';
import '../widgets/tiles.dart';

class SettingsScreen extends StatefulWidget {
  final bool compact;
  const SettingsScreen({super.key, this.compact = false});
  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  bool _secure = true;
  final _name = TextEditingController(text: 'Linux PC');

  final _metrics = const [
    ('LAN IP Address', '192.168.10.240', true),
    ('Device ID', '637ab492-3c28-4fba-ad49-2591eb2daa40', true),
    ('Last network sync', '00:08:50', false),
    ('E2EE Fingerprint', 'ED:76:A1:EC:A6:4B:7B:61:8E…', true),
  ];

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final c = widget.compact;
    return ListView(
      padding: EdgeInsets.fromLTRB(c ? 18 : 26, c ? 6 : 24, c ? 18 : 26, 28),
      children: [
        const SectionTitle('Device configuration'),
        GlassCard(
          padding: const EdgeInsets.fromLTRB(20, 18, 20, 18),
          child: Row(children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text('LOCAL DEVICE NAME', style: AB.label),
                  const SizedBox(height: 6),
                  TextField(
                    controller: _name,
                    style: AB.h1.copyWith(fontSize: 17),
                    decoration: const InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 18),
            Column(children: [
              Text('SECURE SYNC', style: AB.label),
              const SizedBox(height: 8),
              GlassToggle(
                value: _secure,
                onChanged: (v) {
                  setState(() => _secure = v);
                  showGlassToast(context,
                      v ? 'Secure Sync enabled' : 'Secure Sync paused');
                },
              ),
            ]),
          ]),
        ),
        const SizedBox(height: 20),
        const SectionTitle('System metrics'),
        GlassCard(
          padding: EdgeInsets.zero,
          child: Column(
            children: [
              for (var i = 0; i < _metrics.length; i++)
                _metricRow(_metrics[i].$1, _metrics[i].$2, _metrics[i].$3,
                    last: i == _metrics.length - 1),
            ],
          ),
        ),
      ],
    );
  }

  Widget _metricRow(String k, String v, bool copyable, {bool last = false}) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 15),
      decoration: BoxDecoration(
        border: last
            ? null
            : Border(bottom: BorderSide(color: AB.stroke)),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(k, style: AB.body.copyWith(color: AB.text2)),
          Flexible(
            child: GestureDetector(
              onTap: copyable
                  ? () {
                      Clipboard.setData(ClipboardData(text: v));
                      showGlassToast(context, 'Copied · $k');
                    }
                  : null,
              child: Text(v,
                  textAlign: TextAlign.right,
                  overflow: TextOverflow.ellipsis,
                  style: AB.mono),
            ),
          ),
        ],
      ),
    );
  }
}
