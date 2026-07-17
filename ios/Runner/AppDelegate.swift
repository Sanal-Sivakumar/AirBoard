import Flutter
import UIKit
import UserNotifications

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)

    guard let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "AirBoardClipboard") else {
      return
    }
    let channel = FlutterMethodChannel(
      name: "com.example.clipboard/clipboard",
      binaryMessenger: registrar.messenger()
    )
    channel.setMethodCallHandler { call, result in
      switch call.method {
      case "getChangeCount":
        result(UIPasteboard.general.changeCount)
      case "getClipboardText":
        result(UIPasteboard.general.string)
      case "showLocalNotification":
        let arguments = call.arguments as? [String: Any]
        let content = UNMutableNotificationContent()
        content.title = arguments?["title"] as? String ?? "AirBoard"
        content.body = arguments?["body"] as? String ?? "Clipboard update received"
        content.sound = .default
        let request = UNNotificationRequest(
          identifier: "airboard_clipboard_sync",
          content: content,
          trigger: nil
        )
        UNUserNotificationCenter.current().add(request) { error in
          DispatchQueue.main.async {
            if let error = error {
              result(FlutterError(
                code: "notification_failed",
                message: error.localizedDescription,
                details: nil
              ))
            } else {
              result(nil)
            }
          }
        }
      default:
        result(FlutterMethodNotImplemented)
      }
    }

    UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
  }
}
