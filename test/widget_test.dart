import 'package:clipboard/main.dart';
import 'package:clipboard/screens/devices_screen.dart';
import 'package:clipboard/theme.dart';
import 'package:flutter/material.dart';
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

  testWidgets('manual connect requires a valid LAN IPv4 address',
      (tester) async {
    final controller = TextEditingController();
    var connectRequests = 0;
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        theme: AB.theme(),
        home: Scaffold(
          body: DevicesScreen(
            pairedDevices: const [],
            discoveredDevices: const [],
            clipboardHistory: const [],
            manualIpController: controller,
            isSyncEnabled: false,
            lastSyncTimestamp: 'Never',
            onPair: (_) {},
            onUnpair: (_) {},
            onManualConnect: () => connectRequests++,
          ),
        ),
      ),
    );

    expect(
      find.textContaining('Enter the LAN IP shown in AirBoard Settings'),
      findsOneWidget,
    );

    await tester.enterText(find.byType(TextField), 'not-an-ip');
    await tester.pump();
    expect(find.textContaining('Use an IPv4 address'), findsOneWidget);
    await tester.tap(find.text('Connect'));
    expect(connectRequests, 0);

    await tester.enterText(find.byType(TextField), '192.168.1.42');
    await tester.pump();
    expect(find.textContaining('Connect will enable synchronization'),
        findsOneWidget);
    await tester.tap(find.text('Connect'));
    expect(connectRequests, 1);
  });
}
