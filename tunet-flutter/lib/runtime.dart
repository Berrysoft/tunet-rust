import 'dart:async';
import 'package:collection/collection.dart';
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
  late StreamController<bool> detailBusySink = StreamController();
  late StreamController<DetailDailyWrap> dailySink = StreamController();
  late StreamController<String> usernameSink = StreamController();
  late StreamController<bool> onlineBusySink = StreamController();

  late Stream<bool> logBusyStream = logBusySink.stream.asBroadcastStream();
  late Stream<String> logTextStream = logTextSink.stream.asBroadcastStream();
  late Stream<NetFlux> netFluxStream = netFluxSink.stream.asBroadcastStream();
  late Stream<NetState> stateStream = stateSink.stream.asBroadcastStream();
  late Stream<String> statusStream = statusSink.stream.asBroadcastStream();
  late Stream<bool> detailBusyStream =
      detailBusySink.stream.asBroadcastStream();
  late Stream<DetailDailyWrap> dailyStream =
      dailySink.stream.asBroadcastStream();
  late Stream<String> usernameStream = usernameSink.stream.asBroadcastStream();
  late Stream<bool> onlineBusyStream =
      onlineBusySink.stream.asBroadcastStream();

  DetailsData detailsData = DetailsData();
  List<NetUserWrap> onlinesData = List.empty();

  ManagedRuntime({required this.runtime});

  static Future<ManagedRuntime> newRuntime() async {
    final runtime = await Runtime.newRuntime(bridge: api);
    return ManagedRuntime(runtime: runtime);
  }

  Future<void> start() async {
    NetStatus sendStatus = const NetStatus.unknown();
    final String? gstatus = await statusApi.invokeMethod("getStatus");
    switch (gstatus) {
      case "wwan":
        sendStatus = const NetStatus.wwan();
        break;
      case "wlan":
        sendStatus = NetStatus.wlan(await statusApi.invokeMethod("getSsid"));
        break;
      case "lan":
        sendStatus = const NetStatus.lan();
        break;
    }

    final (u, p) = await loadCredential();

    final config = RuntimeStartConfig(
      status: NetStatusWrap(field0: sendStatus),
      username: u,
      password: p,
    );

    await for (final msg in runtime.start(config: config)) {
      switch (msg.field0) {
        case UpdateMsg.Credential:
          await runtime.queueState();
          await runtime.queueDetails();
          await runtime.queueOnlines();
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
        case UpdateMsg.Online:
          onlinesData = await onlines();
          break;
        case UpdateMsg.Details:
          detailsData.setData(await details());
          final daily = await detailDaily();
          if (daily != null) {
            dailySink.add(daily);
          }
          break;
        case UpdateMsg.LogBusy:
          logBusySink.add(await logBusy());
          break;
        case UpdateMsg.OnlineBusy:
          onlineBusySink.add(await onlineBusy());
          break;
        case UpdateMsg.DetailBusy:
          detailBusySink.add(await detailBusy());
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
  Future<void> queueDetails() => runtime.queueDetails();
  Future<void> queueOnlines() => runtime.queueOnlines();
  Future<void> queueConnect({required Ipv4AddrWrap ip}) =>
      runtime.queueConnect(ip: ip);
  Future<void> queueDrop({required List<Ipv4AddrWrap> ips}) =>
      runtime.queueDrop(ips: ips);

  Future<bool> logBusy() => runtime.logBusy();
  Future<String> logText() => runtime.logText();
  Future<NetState> state() async => (await runtime.state()).field0;
  Future<String> status() => runtime.status();
  Future<NetFlux> flux() => runtime.flux();
  Future<bool> detailBusy() => runtime.detailBusy();
  Future<List<NetDetail>> details() => runtime.details();
  Future<DetailDailyWrap?> detailDaily() => runtime.detailDaily();
  Future<String> username() => runtime.username();
  Future<bool> onlineBusy() => runtime.onlineBusy();
  Future<List<NetUserWrap>> onlines() => runtime.onlines();
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
