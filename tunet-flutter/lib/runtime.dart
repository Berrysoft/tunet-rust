import 'dart:async';
import 'dart:io';
import 'package:binding/binding.dart';
import 'package:binding/src/binding_base.dart';
import 'package:flutter/services.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'ffi.dart';
export 'ffi.dart';

class ManagedRuntime extends NotifyPropertyChanged {
  final Runtime runtime;

  static const statusApi =
      MethodChannel('io.github.berrysoft.tunet_flutter/status');

  static const String logBusyProperty = "logBusy";
  bool _logBusy = false;
  bool get logBusy => _logBusy;
  set logBusy(bool v) {
    if (v != _logBusy) {
      _logBusy = v;
      propertyChanged(propertyName: logBusyProperty);
    }
  }

  static const String logTextProperty = "logText";
  String _logText = "";
  String get logText => _logText;
  set logText(String value) {
    if (_logText != value) {
      _logText = value;
      propertyChanged(propertyName: logTextProperty);
    }
  }

  static const String netFluxProperty = "netFlux";
  NetFlux? _netFlux;
  NetFlux? get netFlux => _netFlux;
  set netFlux(NetFlux? value) {
    if (_netFlux != value) {
      _netFlux = value;
      propertyChanged(propertyName: netFluxProperty);
    }
  }

  static const String stateProperty = "state";
  NetState _state = NetState.Unknown;
  NetState get state => _state;
  set state(NetState value) {
    if (_state != value) {
      _state = value;
      propertyChanged(propertyName: stateProperty);
    }
  }

  static const String statusProperty = "status";
  String _status = "";
  String get status => _status;
  set status(String value) {
    if (_status != value) {
      _status = value;
      propertyChanged(propertyName: statusProperty);
    }
  }

  static const String usernameProperty = "username";
  String _username = "";
  String get username => _username;
  set username(String value) {
    if (_username != value) {
      _username = value;
      propertyChanged(propertyName: usernameProperty);
    }
  }

  ManagedRuntime({required this.runtime});

  static Future<ManagedRuntime> newRuntime() async {
    if (Platform.isIOS || Platform.isMacOS) {
      await RustLib.init(
          externalLibrary: ExternalLibrary.process(iKnowHowToUseIt: true));
    } else {
      await RustLib.init();
    }
    final runtime = Runtime();
    return ManagedRuntime(runtime: runtime);
  }

  Future<void> start() async {
    final sendStatus = await loadStatus();
    final (u, p) = await loadCredential();
    final config = RuntimeStartConfig(
      status: sendStatus,
      username: u,
      password: p,
    );

    await for (final msg in runtime.start(config: config)) {
      switch (msg) {
        case UpdateMsgWrap_Credential(:final field0):
          queueState();
          username = field0;
        case UpdateMsgWrap_State(:final field0):
          queueFlux();
          state = field0;
        case UpdateMsgWrap_Status(:final field0):
          queueState();
          status = field0;
        case UpdateMsgWrap_Log(:final field0):
          logText = field0;
        case UpdateMsgWrap_Flux(:final field0):
          netFlux = field0;
        case UpdateMsgWrap_LogBusy(:final field0):
          logBusy = field0;
      }
    }
  }

  Future<NetStatus> loadStatus() async {
    NetStatus sendStatus = const NetStatus.unknown();
    final String? gstatus = await statusApi.invokeMethod("getStatus");
    switch (gstatus) {
      case "wwan":
        sendStatus = const NetStatus.wwan();
        break;
      case "wlan":
        String? ssid = await statusApi.invokeMethod("getSsid");
        if (ssid != null) {
          sendStatus = NetStatus.wlan(ssid);
        }
        break;
      case "lan":
        sendStatus = const NetStatus.lan();
        break;
    }
    return sendStatus;
  }

  Future<(String, String)> loadCredential() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    final username = prefs.getString('username') ?? "";

    const storage = FlutterSecureStorage();
    final password = await storage.read(key: '$username@tunet') ?? "";
    return (username, password);
  }

  Future<void> saveCredential(String u, String p) async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    await prefs.setString('username', u);

    const storage = FlutterSecureStorage();
    await storage.write(key: '$u@tunet', value: p);
  }

  void queueState({NetState? s}) => runtime.queueState(s: s);
  void queueStatus() async {
    runtime.queueStatus(s: await loadStatus());
  }

  void queueCredential({required String u, required String p}) async {
    await saveCredential(u, p);
    runtime.queueCredential(u: u, p: p);
  }

  void queueLogin() => runtime.queueLogin();
  void queueLogout() => runtime.queueLogout();
  void queueFlux() => runtime.queueFlux();
}

class PropertyChangedCallbackWrap<T extends NotifyPropertyChanged>
    extends BindingBase<T> {
  final void Function(T) callback;

  @override
  T source;

  PropertyChangedCallbackWrap({required this.source, required this.callback});

  @override
  void rebuild() {
    callback(source);
  }
}
