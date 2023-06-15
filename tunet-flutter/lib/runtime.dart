import 'dart:async';
import 'ffi.dart';
export 'ffi.dart';

class ManagedRuntime {
  final Runtime runtime;

  final StreamController<NetState> stateSink;
  final StreamController<NetFlux> netFluxSink;

  const ManagedRuntime(
      {required this.runtime,
      required this.stateSink,
      required this.netFluxSink});

  static Future<ManagedRuntime> newRuntime() async {
    final runtime = await Runtime.newRuntime(bridge: api);
    return ManagedRuntime(
        runtime: runtime,
        stateSink: StreamController(),
        netFluxSink: StreamController());
  }

  Future<void> start() async {
    await for (final msgw in runtime.start()) {
      final msg = msgw.field0;
      switch (msg) {
        case UpdateMsg.State:
          stateSink.add((await runtime.state()).field0);
          await runtime.queueFlux();
          break;
        case UpdateMsg.Flux:
          netFluxSink.add(await runtime.flux());
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
