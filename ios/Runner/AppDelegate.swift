import UIKit
import Flutter

@main
@objc class AppDelegate: FlutterAppDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let controller : FlutterViewController = window?.rootViewController as! FlutterViewController
    let clipboardChannel = FlutterMethodChannel(name: "com.example.clipboard/clipboard",
                                              binaryMessenger: controller.binaryMessenger)
    
    clipboardChannel.setMethodCallHandler({
      (call: FlutterMethodCall, result: @escaping FlutterResult) -> Void in
      if call.method == "getClipboardText" {
        result(UIPasteboard.general.string ?? "")
      } else if call.method == "setClipboardText" {
        if let args = call.arguments as? Dictionary<String, Any>,
           let text = args["text"] as? String {
          UIPasteboard.general.string = text
          result(true)
        } else {
          result(FlutterError(code: "INVALID_ARGUMENTS", message: "Text argument is missing", details: nil))
        }
      } else if call.method == "getChangeCount" {
        result(UIPasteboard.general.changeCount)
      } else {
        result(FlutterMethodNotImplemented)
      }
    })

    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}
