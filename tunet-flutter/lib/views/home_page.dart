import 'dart:async';
import 'package:flutter/material.dart';
import 'package:format/format.dart';
import 'package:duration/duration.dart';
import 'package:duration/locale.dart';
import '../runtime.dart';

class HomePage extends StatefulWidget {
  final ManagedRuntime runtime;

  const HomePage({Key? key, required this.runtime}) : super(key: key);

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  late bool logBusy = false;
  late NetFlux? netFlux;
  late String? status;
  late NetState state = NetState.Unknown;

  late StreamSubscription<bool> logBusySub;
  late StreamSubscription<NetFlux> netFluxSub;
  late StreamSubscription<String> statusSub;
  late StreamSubscription<NetState> stateSub;

  @override
  void initState() {
    super.initState();

    final runtime = widget.runtime;
    initStateAsync(runtime);
  }

  Future<void> initStateAsync(ManagedRuntime runtime) async {
    final logBusy = await runtime.logBusy();
    final netFlux = await runtime.flux();
    final status = await runtime.status();
    final state = await runtime.state();
    setState(() {
      this.logBusy = logBusy;
      this.netFlux = netFlux;
      this.status = status;
      this.state = state;
    });
    listenState(runtime);
  }

  void listenState(ManagedRuntime runtime) {
    logBusySub = runtime.logBusyStream
        .listen((event) => setState(() => logBusy = event));
    netFluxSub = runtime.netFluxStream
        .listen((event) => setState(() => netFlux = event));
    statusSub =
        runtime.statusStream.listen((event) => setState(() => status = event));
    stateSub =
        runtime.stateStream.listen((event) => setState(() => state = event));
  }

  @override
  void dispose() {
    logBusySub.cancel();
    netFluxSub.cancel();
    statusSub.cancel();
    stateSub.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;

    Widget cardBody = const CircularProgressIndicator();
    final netFlux = this.netFlux;
    if (netFlux != null) {
      final username = netFlux.username;
      final flux = netFlux.flux.field0;
      final onlineTime = netFlux.onlineTime.field0;
      final balance = netFlux.balance.field0;
      final status = this.status;
      final state = this.state;
      cardBody = Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ListTile(
            leading: const Icon(Icons.person_2_rounded),
            title: Text(username),
          ),
          ListTile(
            leading: const Icon(Icons.sync_alt_rounded),
            title: FutureBuilder<String>(
              future: api.fluxToString(f: flux),
              builder: (context, snap) {
                final s = snap.data;
                if (s == null) {
                  return const LinearProgressIndicator();
                }
                return Text(s);
              },
            ),
          ),
          ListTile(
            leading: const Icon(Icons.timelapse_rounded),
            title: Text(
              prettyDuration(
                onlineTime,
                locale: const ChineseSimplifiedDurationLocale(),
              ),
            ),
          ),
          ListTile(
            leading: const Icon(Icons.account_balance_rounded),
            title: Text('¥{:.2f}'.format(balance)),
          ),
          ListTile(
              leading: const Icon(Icons.signal_cellular_alt_rounded),
              title: status == null
                  ? const LinearProgressIndicator()
                  : Text(status)),
          ListTile(
            leading: const Icon(Icons.pattern_rounded),
            title: Text(state.name),
            trailing: PopupMenuButton<NetState>(
              onSelected: (value) {
                runtime.queueState(s: value);
              },
              itemBuilder: (context) => [
                NetState.Net,
                NetState.Auth4,
                NetState.Auth6
              ]
                  .map((NetState s) =>
                      PopupMenuItem(value: s, child: Text(s.name)))
                  .toList(),
            ),
          ),
        ],
      );
    }
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              IconButton.filled(
                onPressed: logBusy
                    ? null
                    : () {
                        runtime.queueLogin();
                      },
                icon: const Icon(Icons.login_rounded),
              ),
              IconButton.filled(
                onPressed: logBusy
                    ? null
                    : () {
                        runtime.queueLogout();
                      },
                icon: const Icon(Icons.logout_rounded),
              ),
              IconButton.filled(
                onPressed: logBusy
                    ? null
                    : () {
                        runtime.queueFlux();
                      },
                icon: const Icon(Icons.refresh_rounded),
              ),
            ],
          ),
          Card(child: cardBody),
        ],
      ),
    );
  }
}
