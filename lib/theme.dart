import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// AirBoard — "Aurora" Liquid Glass design tokens.
/// These map 1:1 to the CSS prototype variables.
class AB {
  // canvas
  static const bg0 = Color(0xFF06070D);

  // accents (the ONLY saturated hues)
  static const accent = Color(0xFF6EA8FF);
  static const accent2 = Color(0xFFA78BFA);
  static const accentGrad = LinearGradient(
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
    colors: [accent, accent2],
  );

  // semantic
  static const ok = Color(0xFF34D399);
  static const warn = Color(0xFFFBBF24);
  static const danger = Color(0xFFF87171);

  // text
  static const text = Color(0xFFF4F6FB);
  static Color get text2 => text.withValues(alpha: .62);
  static Color get text3 => text.withValues(alpha: .38);

  // glass
  static Color get glass => Colors.white.withValues(alpha: .055);
  static Color get glassStrong => Colors.white.withValues(alpha: .085);
  static Color get glassHi => Colors.white.withValues(alpha: .14);
  static Color get stroke => Colors.white.withValues(alpha: .10);
  static Color get strokeHi => Colors.white.withValues(alpha: .22);

  // mesh blob colors
  static const m1 = Color(0xFF3B6AD6);
  static const m2 = Color(0xFF7C3AED);
  static const m3 = Color(0xFF0EA5A5);
  static Color get glow => accent.withValues(alpha: .55);

  // shape
  static const rLg = 26.0;
  static const rMd = 18.0;
  static const rSm = 12.0;
  static const rPill = 999.0;

  // motion
  static const ease = Cubic(.22, 1, .36, 1);
  static const blur = 30.0;

  // ---- type ----
  static TextStyle get h1 => GoogleFonts.inter(
      fontSize: 22,
      fontWeight: FontWeight.w600,
      color: text,
      letterSpacing: -.3);
  static TextStyle get title =>
      GoogleFonts.inter(fontSize: 15, fontWeight: FontWeight.w600, color: text);
  static TextStyle get body =>
      GoogleFonts.inter(fontSize: 14, fontWeight: FontWeight.w500, color: text);
  static TextStyle get sub => GoogleFonts.inter(fontSize: 12.5, color: text2);
  static TextStyle get label => GoogleFonts.inter(
      fontSize: 11,
      fontWeight: FontWeight.w600,
      color: text3,
      letterSpacing: 1.4);
  static TextStyle get mono =>
      GoogleFonts.jetBrainsMono(fontSize: 12.5, color: text);
  static TextStyle get monoSm =>
      GoogleFonts.jetBrainsMono(fontSize: 11.5, color: text2);

  static ThemeData theme() {
    final base = ThemeData.dark(useMaterial3: true);
    return base.copyWith(
      scaffoldBackgroundColor: bg0,
      textTheme: GoogleFonts.interTextTheme(base.textTheme).apply(
        bodyColor: text,
        displayColor: text,
      ),
      colorScheme: base.colorScheme.copyWith(
        primary: accent,
        secondary: accent2,
        surface: bg0,
      ),
      splashFactory: InkRipple.splashFactory,
    );
  }
}
