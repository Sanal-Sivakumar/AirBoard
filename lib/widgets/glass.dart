import 'dart:ui';
import 'package:flutter/material.dart';
import '../theme.dart';

/// Frosted "Liquid Glass" surface: backdrop blur + diagonal sheen +
/// specular border + soft drop shadow. The building block for every card.
class GlassCard extends StatefulWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final double radius;
  final double blur;
  final Color? fill;
  final VoidCallback? onTap;
  final bool hoverLift; // desktop micro-interaction
  final bool flash; // accent flash (e.g. on new sync / pair)

  const GlassCard({
    super.key,
    required this.child,
    this.padding,
    this.margin,
    this.radius = AB.rMd,
    this.blur = AB.blur,
    this.fill,
    this.onTap,
    this.hoverLift = false,
    this.flash = false,
  });

  @override
  State<GlassCard> createState() => _GlassCardState();
}

class _GlassCardState extends State<GlassCard> {
  bool _hover = false;

  @override
  Widget build(BuildContext context) {
    final br = BorderRadius.circular(widget.radius);
    final lifted = widget.hoverLift && _hover;

    Widget card = AnimatedContainer(
      duration: const Duration(milliseconds: 240),
      curve: AB.ease,
      margin: widget.margin,
      transform: Matrix4.translationValues(0, lifted ? -2 : 0, 0),
      decoration: BoxDecoration(
        borderRadius: br,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: .45),
            blurRadius: 44,
            spreadRadius: -18,
            offset: const Offset(0, 18),
          ),
          if (widget.flash)
            BoxShadow(color: AB.glow, blurRadius: 26, spreadRadius: -2),
        ],
        border: widget.flash ? Border.all(color: AB.accent, width: 1.4) : null,
      ),
      child: ClipRRect(
        borderRadius: br,
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: widget.blur, sigmaY: widget.blur),
          child: Container(
            padding: widget.padding ?? const EdgeInsets.all(14.0),
            decoration: BoxDecoration(
              borderRadius: br,
              // diagonal sheen from top-left into the glass fill
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                stops: const [0.0, 0.38],
                colors: [
                  Colors.white.withValues(alpha: .10),
                  widget.fill ?? (lifted ? AB.glassHi : AB.glass),
                ],
              ),
              border:
                  Border.all(color: lifted ? AB.strokeHi : AB.stroke, width: 1),
            ),
            child: widget.child,
          ),
        ),
      ),
    );

    if (widget.onTap != null || widget.hoverLift) {
      card = MouseRegion(
        cursor:
            widget.onTap != null ? SystemMouseCursors.click : MouseCursor.defer,
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: GestureDetector(onTap: widget.onTap, child: card),
      );
    }
    return card;
  }
}

/// Animated ambient mesh that lives behind the glass (the "depth").
class AmbientBackground extends StatefulWidget {
  final Widget child;
  const AmbientBackground({super.key, required this.child});
  @override
  State<AmbientBackground> createState() => _AmbientBackgroundState();
}

class _AmbientBackgroundState extends State<AmbientBackground>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c =
      AnimationController(vsync: this, duration: const Duration(seconds: 22))
        ..repeat(reverse: true);

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: AB.bg0,
      child: AnimatedBuilder(
        animation: _c,
        builder: (context, _) {
          final t = AB.ease.transform(_c.value);
          return Stack(
            children: [
              _blob(AB.m1, -160, -180, 560, .55, 30 * t, -22 * t),
              _blob(AB.m2, null, null, 520, .55, 20 * t, -16 * t,
                  right: -150, bottom: -170),
              _blob(AB.m3, 320, 360, 440, .32, 36 * t, -28 * t),
              widget.child,
            ],
          );
        },
      ),
    );
  }

  Widget _blob(Color c, double? left, double? top, double size, double op,
      double dx, double dy,
      {double? right, double? bottom}) {
    return Positioned(
      left: left == null ? null : left + dx,
      top: top == null ? null : top + dy,
      right: right == null ? null : right - dx,
      bottom: bottom == null ? null : bottom - dy,
      child: Container(
        width: size,
        height: size,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          gradient: RadialGradient(
            colors: [
              c.withValues(alpha: op),
              c.withValues(alpha: op * 0.4),
              c.withValues(alpha: 0.0),
            ],
            stops: const [0.0, 0.45, 1.0],
          ),
        ),
      ),
    );
  }
}

/// A pulsing status dot (Connected / E2EE active).
class PulseDot extends StatefulWidget {
  final Color color;
  final double size;
  const PulseDot({super.key, this.color = AB.ok, this.size = 8});
  @override
  State<PulseDot> createState() => _PulseDotState();
}

class _PulseDotState extends State<PulseDot>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c =
      AnimationController(vsync: this, duration: const Duration(seconds: 2))
        ..repeat();
  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _c,
      builder: (context, _) {
        final v = _c.value;
        return SizedBox(
          width: widget.size,
          height: widget.size,
          child: Stack(
            alignment: Alignment.center,
            clipBehavior: Clip.none,
            children: [
              Container(
                width: widget.size * (1 + v * 1.8),
                height: widget.size * (1 + v * 1.8),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: widget.color.withValues(alpha: (1 - v) * .45),
                ),
              ),
              Container(
                decoration:
                    BoxDecoration(shape: BoxShape.circle, color: widget.color),
              ),
            ],
          ),
        );
      },
    );
  }
}

/// Premium iOS-style toggle used for "Secure Sync".
class GlassToggle extends StatelessWidget {
  final bool value;
  final ValueChanged<bool> onChanged;
  const GlassToggle({super.key, required this.value, required this.onChanged});
  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => onChanged(!value),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 300),
        curve: AB.ease,
        width: 50,
        height: 30,
        padding: const EdgeInsets.all(3),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(AB.rPill),
          gradient: value
              ? const LinearGradient(colors: [AB.ok, Color(0xFF10B981)])
              : null,
          color: value ? null : AB.glassStrong,
          border: Border.all(color: value ? Colors.transparent : AB.stroke),
        ),
        child: AnimatedAlign(
          duration: const Duration(milliseconds: 300),
          curve: AB.ease,
          alignment: value ? Alignment.centerRight : Alignment.centerLeft,
          child: Container(
            width: 22,
            height: 22,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: Colors.white,
              boxShadow: [
                BoxShadow(
                    color: Colors.black.withValues(alpha: .4), blurRadius: 6)
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Lightweight glass toast (replaces the CSS toast).
void showGlassToast(BuildContext context, String message) {
  final messenger = ScaffoldMessenger.of(context);
  messenger.clearSnackBars();
  messenger.showSnackBar(
    SnackBar(
      behavior: SnackBarBehavior.floating,
      backgroundColor: Colors.transparent,
      elevation: 0,
      duration: const Duration(milliseconds: 2200),
      width: 320,
      content: GlassCard(
        radius: AB.rPill,
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_rounded, size: 17, color: AB.ok),
            const SizedBox(width: 10),
            Flexible(child: Text(message, style: AB.body)),
          ],
        ),
      ),
    ),
  );
}
