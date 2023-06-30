package io.github.berrysoft.tunet_flutter

import android.net.ConnectivityManager
import android.net.wifi.WifiManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val channelName = "io.github.berrysoft.tunet_flutter/status"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(
            flutterEngine.dartExecutor.binaryMessenger,
            channelName
        ).setMethodCallHandler { call, result ->
            if (call.method == "getStatus") {
                val manager = getSystemService(CONNECTIVITY_SERVICE) as ConnectivityManager
                val types =
                    manager.allNetworks.mapNotNull { network -> manager.getNetworkInfo(network)?.type }
                if (types.contains(ConnectivityManager.TYPE_WIFI)) {
                    result.success("wlan")
                } else if (types.contains(ConnectivityManager.TYPE_MOBILE)) {
                    result.success("wwan")
                } else if (types.contains(ConnectivityManager.TYPE_ETHERNET)) {
                    result.success("lan")
                } else {
                    result.success("unknown")
                }
            } else if (call.method == "getSsid") {
                val manager = applicationContext.getSystemService(WIFI_SERVICE) as WifiManager
                val ssid = manager.connectionInfo.ssid
                if (ssid == "<unknown ssid>") {
                    result.success(null)
                } else {
                    result.success(ssid.trim('\"'))
                }
            } else {
                result.notImplemented()
            }
        }
    }
}
