import 'dart:async';
import 'dart:io';
import 'package:binding/binding.dart';
import 'package:binding/src/binding_base.dart';
import 'package:collection/collection.dart';
import 'package:data_size/data_size.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:intl/intl.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'ffi.dart';
export 'ffi.dart';

class ManagedRuntime extends NotifyPropertyChanged {
  final Runtime runtime;

  static const statusApi = MethodChannel('com.berrysoft.tunet_flutter/status');

  static const String logBusyProperty = "logBusy";
  bool _logBusy = false;
  bool get logBusy => _logBusy;
  set logBusy(bool v) {
    if (v != _logBusy) {
      _logBusy = v;
      propertyChanged(propertyName: logBusyProperty);
    }
  }

  static const String detailBusyProperty = "detailBusy";
  bool _detailBusy = false;
  bool get detailBusy => _detailBusy;
  set detailBusy(bool value) {
    if (_detailBusy != value) {
      _detailBusy = value;
      propertyChanged(propertyName: detailBusyProperty);
    }
  }

  static const String onlineBusyProperty = "onlineBusy";
  bool _onlineBusy = false;
  bool get onlineBusy => _onlineBusy;
  set onlineBusy(bool value) {
    if (_onlineBusy != value) {
      _onlineBusy = value;
      propertyChanged(propertyName: onlineBusyProperty);
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

  static const String dailyProperty = "daily";
  DetailDailyWrap? _daily;
  DetailDailyWrap? get daily => _daily;
  set daily(DetailDailyWrap? value) {
    if (_daily != value) {
      _daily = value;
      propertyChanged(propertyName: dailyProperty);
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

  static const String onlinesProperty = "onlines";
  List<NetUserWrap>? _onlines = List.empty();
  List<NetUserWrap>? get onlines => _onlines;
  set onlines(List<NetUserWrap>? value) {
    if (_onlines != value) {
      _onlines = value;
      propertyChanged(propertyName: onlinesProperty);
    }
  }

  DetailsData detailsData = DetailsData();

  ManagedRuntime({required this.runtime});

  static Future<ManagedRuntime> newRuntime() async {
    final runtime = await Runtime.newRuntime(bridge: api);
    return ManagedRuntime(runtime: runtime);
  }

  Future<void> start() async {
    NetStatus sendStatus = await currentStatus();

    final (u, p) = await loadCredential();

    final config = RuntimeStartConfig(
      status: sendStatus,
      username: u,
      password: p,
    );

    await for (final msg in runtime.start(config: config)) {
      await msg.when<Future<void>>(
        credential: (username) async {
          await runtime.queueState();
          await runtime.queueDetails();
          await runtime.queueOnlines();
          this.username = username;
        },
        state: (state) async {
          await runtime.queueFlux();
          this.state = state;
        },
        status: (status) async {
          await runtime.queueState();
          this.status = status;
        },
        log: (logText) async {
          this.logText = logText;
        },
        flux: (netFlux) async {
          this.netFlux = null;
          this.netFlux = netFlux;
        },
        online: (onlines) async {
          this.onlines = onlines;
        },
        details: (details, daily) async {
          detailsData.setData(details);
          this.daily = daily;
        },
        logBusy: (logBusy) async {
          this.logBusy = logBusy;
        },
        onlineBusy: (onlineBusy) async {
          this.onlineBusy = onlineBusy;
        },
        detailBusy: (detailBusy) async {
          this.detailBusy = detailBusy;
        },
      );
    }
  }

  Future<NetStatus> currentStatus() async {
    if (Platform.isAndroid || Platform.isIOS) {
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
    } else {
      return await runtime.currentStatus();
    }
  }

  Future<(String, String)> loadCredential() async {
    if (Platform.isAndroid || Platform.isIOS) {
      final SharedPreferences prefs = await SharedPreferences.getInstance();
      final username = prefs.getString('username') ?? "";

      const storage = FlutterSecureStorage();
      final password = await storage.read(key: '$username@tunet') ?? "";
      return (username, password);
    } else {
      return await runtime.loadCredential();
    }
  }

  Future<void> saveCredential(String u, String p) async {
    if (Platform.isAndroid || Platform.isIOS) {
      final SharedPreferences prefs = await SharedPreferences.getInstance();
      await prefs.setString('username', u);

      const storage = FlutterSecureStorage();
      await storage.write(key: '$u@tunet', value: p);
    } else {
      await runtime.saveCredential(u: u, p: p);
    }
  }

  Future<void> queueState({NetState? s}) => runtime.queueState(s: s);
  Future<void> queueCredential({required String u, required String p}) async {
    await saveCredential(u, p);
    await runtime.queueCredential(u: u, p: p);
  }

  Future<void> queueLogin() => runtime.queueLogin();
  Future<void> queueLogout() => runtime.queueLogout();
  Future<void> queueFlux() async {
    netFlux = null;
    await runtime.queueFlux();
  }

  Future<void> queueDetails() async {
    detailsData.setData(List.empty());
    daily = null;
    await runtime.queueDetails();
  }

  Future<void> queueOnlines() async {
    onlines = null;
    await runtime.queueOnlines();
  }

  Future<void> queueConnect({required Ipv4AddrWrap ip}) =>
      runtime.queueConnect(ip: ip);
  Future<void> queueDrop({required List<Ipv4AddrWrap> ips}) =>
      runtime.queueDrop(ips: ips);
}

class DetailsData extends DataTableSource {
  List<NetDetail> data = List.empty();

  void setData(List<NetDetail> data) {
    this.data = data;
    sortColumnIndex = null;
    sortAscending = true;
    notifyListeners();
  }

  @override
  DataRow? getRow(int index) {
    if (index >= 0 && index < data.length) {
      final d = data[index];
      return DataRow(cells: [
        DataCell(Text(DateFormat('MM-dd HH:mm').format(d.loginTime.field0))),
        DataCell(Text(DateFormat('MM-dd HH:mm').format(d.logoutTime.field0))),
        DataCell(Text(d.flux.field0.formatByteSize())),
      ]);
    }
    return null;
  }

  @override
  bool get isRowCountApproximate => false;

  @override
  int get rowCount => data.length;

  @override
  int get selectedRowCount => 0;

  int? sortColumnIndex;
  bool sortAscending = true;

  void sort(int columnIndex, bool ascending) {
    sortColumnIndex = columnIndex;
    sortAscending = ascending;
    switch (columnIndex) {
      case 0:
        data.sortBy((d) => d.loginTime.field0);
        break;
      case 1:
        data.sortBy((d) => d.logoutTime.field0);
        break;
      case 2:
        data.sortBy<num>((d) => d.flux.field0);
        break;
    }
    if (!ascending) {
      reverse(data);
    }
    notifyListeners();
  }
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
