package io.github.berrysoft.tunet_flutter

import android.content.Context
import android.net.ConnectivityManager
import android.net.wifi.WifiManager
import androidx.annotation.NonNull
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result

class MainActivity : FlutterActivity() {
    private val channelName = "io.github.berrysoft.tunet_flutter/status"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        flutterEngine.plugins.add(InitPlugin())
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

class InitPlugin : FlutterPlugin, MethodCallHandler {
    companion object {
        init {
            System.loadLibrary("native")
        }
    }

    external fun init_android(ctx: Context)

    override fun onAttachedToEngine(
        @NonNull flutterPluginBinding: FlutterPlugin.FlutterPluginBinding,
    ) {
        init_android(flutterPluginBinding.applicationContext)
    }

    override fun onMethodCall(
        @NonNull call: MethodCall,
        @NonNull result: Result,
    ) {
        result.notImplemented()
    }

    override fun onDetachedFromEngine(
        @NonNull binding: FlutterPlugin.FlutterPluginBinding,
    ) {
    }
}
