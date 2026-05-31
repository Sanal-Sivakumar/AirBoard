import UIKit
import Flutter
import AVFoundation
import UserNotifications

class SilentAudioManager {
    static let shared = SilentAudioManager()
    private var audioPlayer: AVAudioPlayer?
    
    func start() {
        guard audioPlayer == nil else { return }
        
        let session = AVAudioSession.sharedInstance()
        do {
            try session.setCategory(.playback, options: [.mixWithOthers])
            try session.setActive(true)
        } catch {
            print("Failed to set AVAudioSession category: \(error)")
            return
        }
        
        guard let url = createSilentWav() else { return }
        
        do {
            audioPlayer = try AVAudioPlayer(contentsOf: url)
            audioPlayer?.numberOfLoops = -1 // loop infinitely
            audioPlayer?.volume = 0.01 // silent volume
            audioPlayer?.prepareToPlay()
            audioPlayer?.play()
            print("Silent audio started playing.")
        } catch {
            print("Failed to initialize AVAudioPlayer: \(error)")
        }
    }
    
    func stop() {
        audioPlayer?.stop()
        audioPlayer = nil
        
        let session = AVAudioSession.sharedInstance()
        do {
            try session.setActive(false, options: .notifyOthersOnDeactivation)
            print("Silent audio stopped.")
        } catch {
            print("Failed to deactivate AVAudioSession: \(error)")
        }
    }
    
    private func createSilentWav() -> URL? {
        let fm = FileManager.default
        let tempDir = fm.temporaryDirectory
        let fileURL = tempDir.appendingPathComponent("silence.wav")
        if fm.fileExists(atPath: fileURL.path) {
            return fileURL
        }
        
        let sampleRate: Int32 = 8000
        let numChannels: Int16 = 1
        let bitsPerSample: Int16 = 8
        let duration = 1 // second
        
        let numSamples = Int(sampleRate) * duration
        let dataSize = numSamples * Int(numChannels) * Int(bitsPerSample) / 8
        let chunkSize = 36 + dataSize
        
        var header = Data()
        header.append("RIFF".data(using: .utf8)!)
        var chunkSizeBytes = Int32(chunkSize).littleEndian
        header.append(Data(bytes: &chunkSizeBytes, count: 4))
        header.append("WAVE".data(using: .utf8)!)
        
        header.append("fmt ".data(using: .utf8)!)
        var subchunk1Size = Int32(16).littleEndian
        header.append(Data(bytes: &subchunk1Size, count: 4))
        var audioFormat = Int16(1).littleEndian
        header.append(Data(bytes: &audioFormat, count: 2))
        var channelsBytes = numChannels.littleEndian
        header.append(Data(bytes: &channelsBytes, count: 2))
        var sampleRateBytes = sampleRate.littleEndian
        header.append(Data(bytes: &sampleRateBytes, count: 4))
        var byteRate: Int32 = (sampleRate * Int32(numChannels) * Int32(bitsPerSample) / 8).littleEndian
        header.append(Data(bytes: &byteRate, count: 4))
        var blockAlign: Int16 = (numChannels * bitsPerSample / 8).littleEndian
        header.append(Data(bytes: &blockAlign, count: 2))
        var bpsBytes = bitsPerSample.littleEndian
        header.append(Data(bytes: &bpsBytes, count: 2))
        
        header.append("data".data(using: .utf8)!)
        var dataSizeBytes = Int32(dataSize).littleEndian
        header.append(Data(bytes: &dataSizeBytes, count: 4))
        
        let silenceByte: UInt8 = 128
        let silenceData = Data(repeating: silenceByte, count: dataSize)
        header.append(silenceData)
        
        do {
            try header.write(to: fileURL)
            return fileURL
        } catch {
            print("Failed to write silent WAV: \(error)")
            return nil
        }
    }
}

@main
@objc class AppDelegate: FlutterAppDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let controller : FlutterViewController = window?.rootViewController as! FlutterViewController
    let clipboardChannel = FlutterMethodChannel(name: "com.example.clipboard/clipboard",
                                              binaryMessenger: controller.binaryMessenger)
    
    // Request notification permission on app launch
    UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { granted, error in
        if let error = error {
            print("Notification authorization failed: \(error.localizedDescription)")
        } else {
            print("Notification authorization status: \(granted)")
        }
    }
    
    // Allow local notification delegate
    UNUserNotificationCenter.current().delegate = self
    
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
      } else if call.method == "startSilentAudio" {
        SilentAudioManager.shared.start()
        result(true)
      } else if call.method == "stopSilentAudio" {
        SilentAudioManager.shared.stop()
        result(true)
      } else if call.method == "showLocalNotification" {
        if let args = call.arguments as? Dictionary<String, Any>,
           let title = args["title"] as? String,
           let body = args["body"] as? String {
          let content = UNMutableNotificationContent()
          content.title = title
          content.body = body
          content.sound = .default
          
          let request = UNNotificationRequest(identifier: "airboard_clipboard_sync", content: content, trigger: nil)
          UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
              result(FlutterError(code: "NOTIFICATION_ERROR", message: error.localizedDescription, details: nil))
            } else {
              result(true)
            }
          }
        } else {
          result(FlutterError(code: "INVALID_ARGUMENTS", message: "Arguments missing", details: nil))
        }
      } else {
        result(FlutterMethodNotImplemented)
      }
    })

    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}
