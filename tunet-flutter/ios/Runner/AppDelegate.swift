import Flutter
import NetworkExtension
import SystemConfiguration
import UIKit

@UIApplicationMain
@objc class AppDelegate: FlutterAppDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    print("dummy_value=\(dummy_method_to_enforce_bundling())")

    let controller: FlutterViewController = window?.rootViewController as! FlutterViewController
    let statusChannel = FlutterMethodChannel(name: "io.github.berrysoft.tunet_flutter/status",
                                             binaryMessenger: controller.binaryMessenger)
    statusChannel.setMethodCallHandler {
      (call: FlutterMethodCall, result: @escaping FlutterResult) in
      if call.method == "getStatus" {
        var address = sockaddr_in()
        address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        address.sin_family = sa_family_t(AF_INET)

        let reachability = withUnsafePointer(to: &address) { pointer in
          pointer.withMemoryRebound(to: sockaddr.self, capacity: MemoryLayout<sockaddr>.size) {
            SCNetworkReachabilityCreateWithAddress(nil, $0)
          }
        }

        var flags = SCNetworkReachabilityFlags()
        SCNetworkReachabilityGetFlags(reachability!, &flags)
        let isReachable = flags.contains(.reachable)
        if !isReachable {
          result("unknown")
        } else {
          let needsConnection = flags.contains(.connectionRequired)
          let canConnectAutomatically = flags.contains(.connectionOnDemand) || flags.contains(.connectionOnTraffic)
          let canConnectWithoutUserInteraction = canConnectAutomatically && !flags.contains(.interventionRequired)
          if !needsConnection || canConnectWithoutUserInteraction {
            result("wlan")
          } else if flags.contains(.isWWAN) {
            result("wwan")
          } else {
            result("unknown")
          }
        }
      } else if call.method == "getSsid" {
        NEHotspotNetwork.fetchCurrent(completionHandler: { network in
          if let unwrappedNetwork = network {
            result(unwrappedNetwork.ssid)
          } else {
            result(nil)
          }
        })
      } else {
        result(FlutterMethodNotImplemented)
      }
    }

    GeneratedPluginRegistrant.register(with: self)
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}
