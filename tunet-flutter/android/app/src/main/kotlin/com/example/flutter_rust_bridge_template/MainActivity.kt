package com.berrysoft.tunet_flutter

import android.content.Context
import android.net.ConnectivityManager
import android.net.wifi.WifiManager
import androidx.annotation.NonNull
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity: FlutterActivity() {
    private val CHANNEL = "com.berrysoft.tunet_flutter/status"

    override fun configureFlutterEngine(@NonNull flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
            MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL).setMethodCallHandler {
                call, result ->
                if (call.method == "getStatus") {
                    val manager = getSystemService(CONNECTIVITY_SERVICE) as ConnectivityManager
                    val networks = manager.getAllNetworks()
                    val types: MutableList<String> = ArrayList()
                    for (network in networks) {
                        val info = manager.getNetworkInfo(network);
                        if (info != null) {
                            when (info.getType()) {
                                ConnectivityManager.TYPE_MOBILE -> types.add("wwan")
                                ConnectivityManager.TYPE_WIFI -> types.add("wlan")
                                ConnectivityManager.TYPE_ETHERNET -> types.add("lan")
                            }
                        }
                    }
                    if (types.contains("wlan")) {
                        result.success("wlan")
                    } else if (types.contains("wwan")) {
                        result.success("wwan")
                    } else if (types.contains("lan")) { 
                        result.success("lan")
                    } else {
                        result.success("unknown")
                    }
                } else if (call.method == "getSsid") {
                    val manager = getSystemService(WIFI_SERVICE) as WifiManager
                    val ssid = manager.getConnectionInfo().getSSID()
                    result.success(ssid)
                } else {
                    result.notImplemented()
                }
        }
    }
}
