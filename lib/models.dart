import 'package:flutter/material.dart';

enum DeviceKind { phone, tablet, laptop, desktop }

extension DeviceKindIcon on DeviceKind {
  IconData get icon => switch (this) {
        DeviceKind.phone => Icons.smartphone_rounded,
        DeviceKind.tablet => Icons.tablet_mac_rounded,
        DeviceKind.laptop => Icons.laptop_mac_rounded,
        DeviceKind.desktop => Icons.desktop_windows_rounded,
      };
}

class Device {
  final String id;
  final String name;
  final String ip;
  final DeviceKind kind;
  bool paired;
  final bool online;
  final String subtitle;
  final String? tag;

  Device({
    required this.id,
    required this.name,
    required this.ip,
    required this.kind,
    this.paired = false,
    this.online = false,
    this.subtitle = '',
    this.tag,
  });
}

enum ClipType { text, link, code }

extension ClipTypeIcon on ClipType {
  IconData get icon => switch (this) {
        ClipType.text => Icons.notes_rounded,
        ClipType.link => Icons.link_rounded,
        ClipType.code => Icons.code_rounded,
      };
}

class ClipItem {
  final ClipType type;
  final String text;
  final String source;
  final DateTime timestamp;

  ClipItem({
    required this.type,
    required this.text,
    required this.source,
    required this.timestamp,
  });

  String get time {
    final diff = DateTime.now().difference(timestamp);
    if (diff.inSeconds < 60) return "now";
    if (diff.inMinutes < 60) return "${diff.inMinutes}m ago";
    if (diff.inHours < 24) return "${diff.inHours}h ago";
    return "${diff.inDays}d ago";
  }
}
