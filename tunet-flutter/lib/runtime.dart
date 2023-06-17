import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:intl/intl.dart';

import 'ffi.dart';
export 'ffi.dart';

class ManagedRuntime {
  final Runtime runtime;

  static const statusApi = MethodChannel('com.berrysoft.tunet_flutter/status');

  late StreamController<bool> logBusySink = StreamController();
  late StreamController<NetFlux> netFluxSink = StreamController();
  late StreamController<NetState> stateSink = StreamController();
  late StreamController<String> statusSink = StreamController();
  late StreamController<DetailDailyWrap> dailySink = StreamController();

  late Stream<bool> logBusyStream = logBusySink.stream.asBroadcastStream();
  late Stream<NetFlux> netFluxStream = netFluxSink.stream.asBroadcastStream();
  late Stream<NetState> stateStream = stateSink.stream.asBroadcastStream();
  late Stream<String> statusStream = statusSink.stream.asBroadcastStream();
  late Stream<DetailDailyWrap> dailyStream =
      dailySink.stream.asBroadcastStream();

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
    await runtime.initializeStatus(t: sendStatus, ssid: ssid);

    await for (final msg in runtime.start()) {
      switch (msg.field0) {
        case UpdateMsg.Credential:
          await runtime.queueState();
          await runtime.queueDetails();
          break;
        case UpdateMsg.State:
          await runtime.queueFlux();
          stateSink.add(await state());
          break;
        case UpdateMsg.Status:
          await runtime.queueState();
          statusSink.add(await status());
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

  Future<void> queueState({NetState? s}) =>
      runtime.queueState(s: s != null ? NetStateWrap(field0: s) : null);
  Future<void> queueCredential({required String u, required String p}) =>
      runtime.queueCredential(u: u, p: p);

  Future<void> queueLogin() => runtime.queueLogin();
  Future<void> queueLogout() => runtime.queueLogout();
  Future<void> queueFlux() => runtime.queueFlux();

  Future<NetState> state() async => (await runtime.state()).field0;
  Future<String> status() => runtime.status();
  Future<NetFlux> flux() => runtime.flux();
  Future<bool> logBusy() => runtime.logBusy();
  Future<List<NetDetail>> details() => runtime.details();
  Future<DetailDailyWrap?> detailDaily() => runtime.detailDaily();
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
