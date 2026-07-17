import 'package:clipboard/main.dart';
import 'package:clipboard/src/rust/api.dart' as api;
import 'package:clipboard/src/rust/frb_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(RustLib.init);

  testWidgets('boots the app and derives a canonical identity fingerprint',
      (tester) async {
    final identity = await api.registerKeys(
      signingKeyBytes: List<int>.generate(32, (index) => index + 1),
      dhKeyBytes: List<int>.generate(32, (index) => 255 - index),
    );

    expect(identity, hasLength(3));
    expect(identity[2], matches(RegExp(r'^(?:[0-9A-F]{2}:){31}[0-9A-F]{2}$')));

    await tester.pumpWidget(const ClipboardSyncApp());
    expect(find.text('AirBoard'), findsWidgets);
  });
}
