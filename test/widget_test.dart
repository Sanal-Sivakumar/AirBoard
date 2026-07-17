import 'dart:ui';

import 'package:clipboard/main.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('renders the AirBoard desktop shell without layout errors',
      (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(const ClipboardSyncApp());
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('AirBoard'), findsWidgets);
    expect(find.textContaining('SECURE SYNC OFF'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('renders at a narrow phone width without overflow',
      (tester) async {
    tester.view.physicalSize = const Size(390, 844);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(const ClipboardSyncApp());
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('PAUSED'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
