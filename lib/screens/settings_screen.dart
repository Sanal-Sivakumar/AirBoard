import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme.dart';
import '../widgets/glass.dart';
import '../widgets/tiles.dart';

class SettingsScreen extends StatelessWidget {
  final bool compact;
  final TextEditingController nameController;
  final bool isSyncEnabled;
  final ValueChanged<bool> onSyncToggle;
  final String localIp;
  final String deviceId;
  final String lastSyncTimestamp;
  final String myFingerprint;
  final List<String> logs;
  final VoidCallback onClearLogs;

  const SettingsScreen({
    super.key,
    this.compact = false,
    required this.nameController,
    required this.isSyncEnabled,
    required this.onSyncToggle,
    required this.localIp,
    required this.deviceId,
    required this.lastSyncTimestamp,
    required this.myFingerprint,
    required this.logs,
    required this.onClearLogs,
  });

  @override
  Widget build(BuildContext context) {
    final c = compact;
    final metrics = [
      ('LAN IP Address', localIp, true),
      ('Device ID', deviceId, true),
      ('Last network sync', lastSyncTimestamp, false),
      ('E2EE Fingerprint', myFingerprint, true),
    ];

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
                    controller: nameController,
                    enabled:
                        !isSyncEnabled, // Prevent changing name while engine is active
                    style: AB.h1.copyWith(fontSize: 17),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: 'Enter device name...',
                      hintStyle: TextStyle(color: AB.text3),
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
                value: isSyncEnabled,
                onChanged: (v) {
                  onSyncToggle(v);
                  showGlassToast(context,
                      v ? 'Initializing Secure Sync...' : 'Secure Sync paused');
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
              for (var i = 0; i < metrics.length; i++)
                _metricRow(context, metrics[i].$1, metrics[i].$2, metrics[i].$3,
                    last: i == metrics.length - 1),
            ],
          ),
        ),
        const SizedBox(height: 20),
        _CollapsibleLogConsole(
          logs: logs,
          onClear: onClearLogs,
        ),
      ],
    );
  }

  Widget _metricRow(BuildContext context, String k, String v, bool copyable,
      {bool last = false}) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 15),
      decoration: BoxDecoration(
        border: last ? null : Border(bottom: BorderSide(color: AB.stroke)),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(k, style: AB.body.copyWith(color: AB.text2)),
          const SizedBox(width: 12),
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

class _CollapsibleLogConsole extends StatefulWidget {
  final List<String> logs;
  final VoidCallback onClear;

  const _CollapsibleLogConsole({
    required this.logs,
    required this.onClear,
  });

  @override
  State<_CollapsibleLogConsole> createState() => _CollapsibleLogConsoleState();
}

class _CollapsibleLogConsoleState extends State<_CollapsibleLogConsole> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        GestureDetector(
          onTap: () {
            setState(() {
              _expanded = !_expanded;
            });
          },
          child: Padding(
            padding: const EdgeInsets.fromLTRB(2, 6, 2, 12),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('SECURITY LOGS', style: AB.label),
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (_expanded)
                      GestureDetector(
                        onTap: widget.onClear,
                        child: Text(
                          'CLEAR',
                          style:
                              AB.label.copyWith(color: AB.danger, fontSize: 10),
                        ),
                      ),
                    const SizedBox(width: 12),
                    Icon(
                      _expanded
                          ? Icons.keyboard_arrow_up_rounded
                          : Icons.keyboard_arrow_down_rounded,
                      size: 16,
                      color: AB.text3,
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
        AnimatedSize(
          duration: const Duration(milliseconds: 300),
          curve: AB.ease,
          child: _expanded
              ? Container(
                  height: 200,
                  margin: const EdgeInsets.only(bottom: 12),
                  child: GlassCard(
                    padding: const EdgeInsets.all(12),
                    radius: AB.rSm,
                    child: widget.logs.isEmpty
                        ? const Center(
                            child: Text(
                              "Empty console logs",
                              style: TextStyle(color: AB.m1, fontSize: 11),
                            ),
                          )
                        : ListView.builder(
                            itemCount: widget.logs.length,
                            itemBuilder: (context, index) {
                              return Padding(
                                padding:
                                    const EdgeInsets.symmetric(vertical: 2.0),
                                child: Text(
                                  widget.logs[index],
                                  style: AB.monoSm.copyWith(
                                    fontSize: 11,
                                    color: AB.accent,
                                  ),
                                ),
                              );
                            },
                          ),
                  ),
                )
              : const SizedBox.shrink(),
        ),
      ],
    );
  }
}
