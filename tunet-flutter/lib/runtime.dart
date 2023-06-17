import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:intl/intl.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'ffi.dart';
export 'ffi.dart';

class ManagedRuntime {
  final Runtime runtime;

  static const statusApi = MethodChannel('com.berrysoft.tunet_flutter/status');

  late StreamController<bool> logBusySink = StreamController();
  late StreamController<String> logTextSink = StreamController();
  late StreamController<NetFlux> netFluxSink = StreamController();
  late StreamController<NetState> stateSink = StreamController();
  late StreamController<String> statusSink = StreamController();
  late StreamController<DetailDailyWrap> dailySink = StreamController();
  late StreamController<String> usernameSink = StreamController();

  late Stream<bool> logBusyStream = logBusySink.stream.asBroadcastStream();
  late Stream<String> logTextStream = logTextSink.stream.asBroadcastStream();
  late Stream<NetFlux> netFluxStream = netFluxSink.stream.asBroadcastStream();
  late Stream<NetState> stateStream = stateSink.stream.asBroadcastStream();
  late Stream<String> statusStream = statusSink.stream.asBroadcastStream();
  late Stream<DetailDailyWrap> dailyStream =
      dailySink.stream.asBroadcastStream();
  late Stream<String> usernameStream = usernameSink.stream.asBroadcastStream();

  late DetailsData detailsData = DetailsData();

  ManagedRuntime({required this.runtime});

  static Future<ManagedRuntime> newRuntime() async {
    final runtime = await Runtime.newRuntime(bridge: api);
    return ManagedRuntime(runtime: runtime);
  }

  Future<void> start() async {
    NetStatusSimp sendStatus = NetStatusSimp.Unknown;
    String? ssid;
    final String? gstatus = await statusApi.invokeMethod("getStatus");
    switch (gstatus) {
      case "wwan":
        sendStatus = NetStatusSimp.Wwan;
        break;
      case "wlan":
        sendStatus = NetStatusSimp.Wlan;
        ssid = await statusApi.invokeMethod("getSsid");
        break;
      case "lan":
        sendStatus = NetStatusSimp.Lan;
        break;
    }

    final (u, p) = await loadCredential();

    final config = await RuntimeStartConfig.newRuntimeStartConfig(
      bridge: api,
      status: sendStatus,
      ssid: ssid,
      username: u,
      password: p,
    );

    await for (final msg in runtime.start(config: config)) {
      switch (msg.field0) {
        case UpdateMsg.Credential:
          await runtime.queueState();
          await runtime.queueDetails();
          usernameSink.add(await username());
          break;
        case UpdateMsg.State:
          await runtime.queueFlux();
          stateSink.add(await state());
          break;
        case UpdateMsg.Status:
          await runtime.queueState();
          statusSink.add(await status());
          break;
        case UpdateMsg.Log:
          logTextSink.add(await logText());
          break;
        case UpdateMsg.Flux:
          netFluxSink.add(await flux());
          break;
        case UpdateMsg.Details:
          detailsData.data = await details();
          final daily = await detailDaily();
          if (daily != null) {
            dailySink.add(daily);
          }
          break;
        case UpdateMsg.LogBusy:
          logBusySink.add(await logBusy());
          break;
        default:
          break;
      }
    }
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

  Future<void> queueState({NetState? s}) =>
      runtime.queueState(s: s != null ? NetStateWrap(field0: s) : null);
  Future<void> queueCredential({required String u, required String p}) async {
    await saveCredential(u, p);
    await runtime.queueCredential(u: u, p: p);
  }

  Future<void> queueLogin() => runtime.queueLogin();
  Future<void> queueLogout() => runtime.queueLogout();
  Future<void> queueFlux() => runtime.queueFlux();

  Future<bool> logBusy() => runtime.logBusy();
  Future<String> logText() => runtime.logText();
  Future<NetState> state() async => (await runtime.state()).field0;
  Future<String> status() => runtime.status();
  Future<NetFlux> flux() => runtime.flux();
  Future<List<NetDetail>> details() => runtime.details();
  Future<DetailDailyWrap?> detailDaily() => runtime.detailDaily();
  Future<String> username() => runtime.username();
}

class DetailsData extends DataTableSource {
  late List<NetDetail> data = List.empty();

  @override
  DataRow? getRow(int index) {
    if (index >= 0 && index < data.length) {
      final d = data[index];
      return DataRow(cells: [
        DataCell(Text(DateFormat('MM-dd HH:mm').format(d.loginTime.field0))),
        DataCell(Text(DateFormat('MM-dd HH:mm').format(d.logoutTime.field0))),
        DataCell(FutureBuilder(
          future: api.fluxToString(f: d.flux.field0),
          builder: (context, snap) {
            final data = snap.data;
            if (data == null) {
              return const CircularProgressIndicator();
            }
            return Text(data);
          },
        ))
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
}
