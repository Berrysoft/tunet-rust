import 'dart:async';
import 'ffi.dart';
export 'ffi.dart';

class ManagedRuntime {
  final Runtime runtime;

  late StreamController<bool> logBusySink;
  late StreamController<NetFlux> netFluxSink;
  late StreamController<NetState> stateSink;
  late StreamController<String> statusSink;

  ManagedRuntime({required this.runtime}) {
    logBusySink = StreamController();
    netFluxSink = StreamController();
    stateSink = StreamController();
    statusSink = StreamController();
  }

  static Future<ManagedRuntime> newRuntime() async {
    final runtime = await Runtime.newRuntime(bridge: api);
    return ManagedRuntime(runtime: runtime);
  }

  Future<void> start() async {
    await for (final msgw in runtime.start()) {
      final msg = msgw.field0;
      switch (msg) {
        case UpdateMsg.State:
          await runtime.queueFlux();
          stateSink.add((await runtime.state()).field0);
          break;
        case UpdateMsg.Status:
          await runtime.queueState(
              s: const NetStateWrap(field0: NetState.Auth4));
          statusSink.add((await runtime.status()));
          break;
        case UpdateMsg.Flux:
          netFluxSink.add(await runtime.flux());
          break;
        case UpdateMsg.LogBusy:
          logBusySink.add(await runtime.logBusy());
          break;
        default:
          break;
      }
    }
  }

  Future<void> queueLogin() => runtime.queueLogin();
  Future<void> queueLogout() => runtime.queueLogout();
  Future<void> queueFlux() => runtime.queueFlux();
}
