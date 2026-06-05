import 'package:flutter/material.dart';

/// ---- UI-only models. Swap these for your real backend types. ----

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
  final String name;
  final String ip;
  final DeviceKind kind;
  bool paired;
  final bool online;
  final String subtitle; // e.g. "synced now", "discovered"
  final String? tag; // e.g. "relay hub", "idle"

  Device({
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
  String time;
  ClipItem(
      {required this.type,
      required this.text,
      required this.source,
      this.time = 'now'});
}

/// ---- Mock seed data (matches the HTML prototype) ----
class MockData {
  static List<Device> paired() => [
        Device(
            name: 'Android Phone',
            ip: '10.113.111.19',
            kind: DeviceKind.phone,
            paired: true,
            online: true,
            subtitle: '10.113.111.19 · synced now',
            tag: 'relay hub'),
        Device(
            name: 'iPad Client',
            ip: '10.113.111.136',
            kind: DeviceKind.tablet,
            paired: true,
            online: true,
            subtitle: '10.113.111.136 · synced 8s ago'),
        Device(
            name: 'MacBook Pro',
            ip: '10.113.111.74',
            kind: DeviceKind.laptop,
            paired: true,
            subtitle: '10.113.111.74 · synced 3m ago',
            tag: 'idle'),
      ];

  static List<Device> discovered() => [
        Device(
            name: 'Windows PC',
            ip: '10.19.75.156',
            kind: DeviceKind.desktop,
            subtitle: '10.19.75.156 · discovered'),
        Device(
            name: 'Galaxy Tab S9',
            ip: '10.19.75.182',
            kind: DeviceKind.tablet,
            subtitle: '10.19.75.182 · discovered'),
      ];

  static List<ClipItem> history() => [
        ClipItem(
            type: ClipType.link,
            text: 'https://github.com/airboard/p2p-mesh/pull/142',
            source: 'iPad Client',
            time: 'now'),
        ClipItem(
            type: ClipType.text,
            text: 'Verification code: 884 213 — expires in 5 min',
            source: 'Android Phone',
            time: '2m ago'),
        ClipItem(
            type: ClipType.code,
            text: 'npx airboard connect --peer 10.19.75.156',
            source: 'MacBook Pro',
            time: '9m ago'),
        ClipItem(
            type: ClipType.text,
            text: 'Remember to rotate the X25519 keypair before demo',
            source: 'Android Phone',
            time: '31m ago'),
      ];
}
