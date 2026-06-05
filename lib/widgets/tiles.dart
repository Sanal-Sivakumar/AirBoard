import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme.dart';
import '../models.dart';
import 'glass.dart';

/// Small uppercase section header, optionally with a count chip.
class SectionTitle extends StatelessWidget {
  final String title;
  final int? count;
  const SectionTitle(this.title, {super.key, this.count});
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(2, 6, 2, 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(title.toUpperCase(), style: AB.label),
          if (count != null)
            Text('$count',
                style: AB.label.copyWith(color: AB.accent)),
        ],
      ),
    );
  }
}

/// Pill badge: Paired (green) / Pair device (amber).
class StatusBadge extends StatelessWidget {
  final bool paired;
  final VoidCallback? onPair;
  final bool compact;
  const StatusBadge(
      {super.key, required this.paired, this.onPair, this.compact = false});
  @override
  Widget build(BuildContext context) {
    if (paired) {
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 7),
        decoration: BoxDecoration(
          color: AB.ok.withOpacity(.10),
          borderRadius: BorderRadius.circular(AB.rPill),
          border: Border.all(color: AB.ok.withOpacity(.28)),
        ),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          const Icon(Icons.check_rounded, size: 14, color: AB.ok),
          if (!compact) ...[
            const SizedBox(width: 6),
            Text('Paired',
                style: AB.sub.copyWith(
                    color: AB.ok, fontWeight: FontWeight.w600, fontSize: 12)),
          ],
        ]),
      );
    }
    return GestureDetector(
      onTap: onPair,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 7),
        decoration: BoxDecoration(
          color: AB.warn.withOpacity(.10),
          borderRadius: BorderRadius.circular(AB.rPill),
          border: Border.all(color: AB.warn.withOpacity(.30)),
        ),
        child: Text(compact ? 'Pair' : 'Pair device',
            style: AB.sub.copyWith(
                color: AB.warn, fontWeight: FontWeight.w600, fontSize: 12)),
      ),
    );
  }
}

class _MiniTag extends StatelessWidget {
  final String label;
  final bool accent;
  const _MiniTag(this.label, {this.accent = true});
  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(left: 7),
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
      decoration: BoxDecoration(
        color: accent ? AB.glassHi : AB.glass,
        borderRadius: BorderRadius.circular(AB.rPill),
        border: Border.all(color: accent ? AB.strokeHi : AB.stroke),
      ),
      child: Text(label.toUpperCase(),
          style: GoogleFontsFallback.tag.copyWith(
              color: accent ? AB.accent : AB.text3)),
    );
  }
}

// tiny helper to avoid importing google_fonts here just for one style
class GoogleFontsFallback {
  static const tag = TextStyle(
      fontSize: 9.5, fontWeight: FontWeight.w700, letterSpacing: .6);
}

/// A device row (paired or discovered). Works on desktop & mobile.
class DeviceTile extends StatelessWidget {
  final Device device;
  final VoidCallback? onPair;
  final VoidCallback? onUnpair;
  final bool compact; // mobile

  const DeviceTile({
    super.key,
    required this.device,
    this.onPair,
    this.onUnpair,
    this.compact = false,
  });

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      hoverLift: !compact,
      padding: EdgeInsets.symmetric(
          horizontal: compact ? 14 : 18, vertical: compact ? 13 : 15),
      child: Row(
        children: [
          _icon(),
          SizedBox(width: compact ? 12 : 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(children: [
                  Flexible(
                    child: Text(device.name,
                        style: AB.title
                            .copyWith(fontSize: compact ? 14.5 : 15),
                        overflow: TextOverflow.ellipsis),
                  ),
                  if (device.tag != null)
                    _MiniTag(device.tag!,
                        accent: device.tag != 'idle'),
                ]),
                const SizedBox(height: 2),
                Text(device.subtitle,
                    style: AB.sub.copyWith(fontSize: compact ? 11.5 : 12.5),
                    overflow: TextOverflow.ellipsis),
              ],
            ),
          ),
          const SizedBox(width: 8),
          StatusBadge(
              paired: device.paired, onPair: onPair, compact: compact),
          if (device.paired && onUnpair != null) ...[
            const SizedBox(width: 4),
            _IconBtn(icon: Icons.delete_outline_rounded, onTap: onUnpair!),
          ],
        ],
      ),
    );
  }

  Widget _icon() {
    final size = compact ? 40.0 : 42.0;
    return Stack(
      clipBehavior: Clip.none,
      children: [
        Container(
          width: size,
          height: size,
          decoration: BoxDecoration(
            color: AB.glassStrong,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: AB.stroke),
          ),
          child: Icon(device.kind.icon,
              size: compact ? 20 : 21, color: AB.accent),
        ),
        if (device.online)
          Positioned(
            top: -3,
            right: -3,
            child: Container(
              width: 11,
              height: 11,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: AB.ok,
                border: Border.all(color: AB.bg0, width: 2),
                boxShadow: [
                  BoxShadow(color: AB.ok.withOpacity(.8), blurRadius: 8)
                ],
              ),
            ),
          ),
      ],
    );
  }
}

class _IconBtn extends StatefulWidget {
  final IconData icon;
  final VoidCallback onTap;
  const _IconBtn({required this.icon, required this.onTap});
  @override
  State<_IconBtn> createState() => _IconBtnState();
}

class _IconBtnState extends State<_IconBtn> {
  bool _h = false;
  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 180),
          width: 34,
          height: 34,
          decoration: BoxDecoration(
            color: _h ? AB.danger.withOpacity(.12) : Colors.transparent,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(
                color: _h ? AB.danger.withOpacity(.25) : Colors.transparent),
          ),
          child: Icon(widget.icon,
              size: 17, color: _h ? AB.danger : AB.text3),
        ),
      ),
    );
  }
}

/// A clipboard-history row with a tap-to-copy action.
class ClipTile extends StatelessWidget {
  final ClipItem item;
  final bool compact;
  const ClipTile({super.key, required this.item, this.compact = false});

  @override
  Widget build(BuildContext context) {
    final isImage = item.type == ClipType.image;
    Widget contentWidget;
    if (isImage) {
      final base64Part = item.text.startsWith("data:image/png;base64,")
          ? item.text.substring(22)
          : item.text;
      contentWidget = Container(
        height: compact ? 64 : 76,
        margin: const EdgeInsets.only(top: 2, bottom: 4),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: Image.memory(
            base64Decode(base64Part),
            fit: BoxFit.cover,
            errorBuilder: (context, error, stackTrace) =>
                Text("Invalid Image Data", style: AB.mono.copyWith(fontSize: compact ? 12.5 : 13.5)),
          ),
        ),
      );
    } else {
      contentWidget = Text(
        item.text,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: AB.mono.copyWith(fontSize: compact ? 12.5 : 13.5),
      );
    }

    return GlassCard(
      hoverLift: !compact,
      padding: EdgeInsets.symmetric(
          horizontal: compact ? 12 : 16, vertical: compact ? 12 : 14),
      onTap: () {
        Clipboard.setData(ClipboardData(text: item.text));
        showGlassToast(context, 'Copied · re-synced to your devices');
      },
      child: Row(
        children: [
          Container(
            width: compact ? 34 : 36,
            height: compact ? 34 : 36,
            decoration: BoxDecoration(
              color: AB.glassStrong,
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: AB.stroke),
            ),
            child: Icon(item.type.icon, size: compact ? 16 : 17, color: AB.accent),
          ),
          SizedBox(width: compact ? 11 : 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                contentWidget,
                const SizedBox(height: 2),
                Text(
                    compact
                        ? '${item.source} · ${item.time}'
                        : 'from ${item.source} · ${item.time}',
                    style: AB.sub
                        .copyWith(color: AB.text3, fontSize: compact ? 10.5 : 11.5)),
              ],
            ),
          ),
          if (!compact)
            const Padding(
              padding: EdgeInsets.only(left: 8),
              child: Icon(Icons.copy_rounded, size: 16, color: AB.accent),
            ),
        ],
      ),
    );
  }
}
