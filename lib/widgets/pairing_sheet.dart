import 'dart:math';
import 'package:flutter/material.dart';
import '../theme.dart';
import 'glass.dart';

/// Zero-trust pairing approval. Returns true if approved.
Future<bool> showPairingSheet(
  BuildContext context,
  String deviceName,
  String deviceId,
  String fingerprintRaw,
) async {
  // Format fingerprint: split by ':' and group into readable segments
  final cleanFingerprint = fingerprintRaw.toUpperCase();
  final parts = cleanFingerprint.split(':');
  final fingerprint = <String>[];
  for (var i = 0; i < parts.length; i += 4) {
    if (i + 4 <= parts.length) {
      fingerprint.add(parts.sublist(i, i + 4).join(":"));
    } else {
      fingerprint.add(parts.sublist(i, parts.length).join(":"));
    }
  }

  final result = await showDialog<bool>(
    context: context,
    barrierColor: Colors.black.withValues(alpha: .55),
    barrierDismissible: false,
    builder: (context) => Material(
      color: Colors.transparent,
      child: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 400),
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: GlassCard(
              radius: 24,
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(children: [
                    Container(
                      width: 42,
                      height: 42,
                      decoration: BoxDecoration(
                        color: AB.warn.withValues(alpha: .14),
                        borderRadius: BorderRadius.circular(12),
                        border:
                            Border.all(color: AB.warn.withValues(alpha: .3)),
                      ),
                      child: const Icon(Icons.warning_amber_rounded,
                          color: AB.warn, size: 22),
                    ),
                    const SizedBox(width: 12),
                    Text('Pairing Request',
                        style: AB.h1.copyWith(fontSize: 18)),
                  ]),
                  const SizedBox(height: 8),
                  Text('A device wants to pair securely with you:',
                      style: AB.sub),
                  const SizedBox(height: 16),
                  GlassCard(
                    radius: 14,
                    padding: const EdgeInsets.all(13),
                    child: Row(children: [
                      Container(
                        width: 38,
                        height: 38,
                        decoration: BoxDecoration(
                          color: AB.glassStrong,
                          borderRadius: BorderRadius.circular(11),
                          border: Border.all(color: AB.stroke),
                        ),
                        child: const Icon(Icons.devices_rounded,
                            size: 19, color: AB.accent),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Text(
                                deviceName.isEmpty
                                    ? "Unknown Peer"
                                    : deviceName,
                                style: AB.title,
                                overflow: TextOverflow.ellipsis),
                            Text(
                                'ID: ${deviceId.substring(0, min(deviceId.length, 12))}...',
                                style: AB.monoSm.copyWith(color: AB.text3)),
                          ],
                        ),
                      ),
                    ]),
                  ),
                  const SizedBox(height: 16),
                  Text('Verify this fingerprint matches on both screens',
                      style: AB.sub.copyWith(fontWeight: FontWeight.w600)),
                  const SizedBox(height: 8),
                  GlassCard(
                    radius: 14,
                    padding: const EdgeInsets.all(14),
                    child: GridView.count(
                      crossAxisCount: 2,
                      shrinkWrap: true,
                      physics: const NeverScrollableScrollPhysics(),
                      childAspectRatio: 4.5,
                      mainAxisSpacing: 6,
                      crossAxisSpacing: 18,
                      children: fingerprint
                          .map((b) => Text(b,
                              style: AB.mono.copyWith(
                                  color: AB.warn,
                                  letterSpacing: .5,
                                  fontSize: 13)))
                          .toList(),
                    ),
                  ),
                  const SizedBox(height: 14),
                  Container(
                    padding: const EdgeInsets.symmetric(
                        horizontal: 13, vertical: 11),
                    decoration: BoxDecoration(
                      color: AB.warn.withValues(alpha: .08),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: AB.warn.withValues(alpha: .25)),
                    ),
                    child: Row(children: [
                      const Icon(Icons.shield_outlined,
                          size: 15, color: AB.warn),
                      const SizedBox(width: 9),
                      Expanded(
                        child: Text(
                            'Only approve if the fingerprints are identical on both devices.',
                            style:
                                AB.sub.copyWith(color: AB.warn, fontSize: 12)),
                      ),
                    ]),
                  ),
                  const SizedBox(height: 18),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      _SheetButton(
                          label: 'Deny',
                          onTap: () => Navigator.pop(context, false),
                          ghost: true),
                      const SizedBox(width: 10),
                      _SheetButton(
                          label: 'Approve',
                          onTap: () => Navigator.pop(context, true)),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    ),
  );
  return result ?? false;
}

class _SheetButton extends StatelessWidget {
  final String label;
  final VoidCallback onTap;
  final bool ghost;
  const _SheetButton(
      {required this.label, required this.onTap, this.ghost = false});
  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(AB.rSm),
          gradient: ghost
              ? null
              : const LinearGradient(colors: [AB.ok, Color(0xFF10B981)]),
          color: ghost ? AB.glassStrong : null,
          border: ghost ? Border.all(color: AB.stroke) : null,
          boxShadow: ghost
              ? null
              : [
                  BoxShadow(
                      color: AB.ok.withValues(alpha: .4),
                      blurRadius: 18,
                      spreadRadius: -6)
                ],
        ),
        child: Text(label,
            style: AB.body.copyWith(
                color: ghost ? AB.text : const Color(0xFF062018),
                fontWeight: FontWeight.w600)),
      ),
    );
  }
}
