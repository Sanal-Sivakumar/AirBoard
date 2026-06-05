import 'package:flutter/material.dart';
import 'theme.dart';
import 'home_shell.dart';

/// Standalone runnable demo of the AirBoard "Aurora" UI.
/// In your real app, just push `const HomeShell()` and apply `AB.theme()`.
void main() => runApp(const AirBoardApp());

class AirBoardApp extends StatelessWidget {
  const AirBoardApp({super.key});
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'AirBoard',
      debugShowCheckedModeBanner: false,
      theme: AB.theme(),
      home: const HomeShell(),
    );
  }
}
