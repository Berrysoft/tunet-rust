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
  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          StreamBuilder<bool>(
            stream: runtime.logBusyStream,
            builder: (context, snap) {
              final busy = snap.data ?? false;
              return Row(
                mainAxisAlignment: MainAxisAlignment.spaceAround,
                children: [
                  IconButton.filled(
                    onPressed: busy
                        ? null
                        : () {
                            runtime.queueLogin();
                          },
                    icon: const Icon(Icons.login_rounded),
                  ),
                  IconButton.filled(
                    onPressed: busy
                        ? null
                        : () {
                            runtime.queueLogout();
                          },
                    icon: const Icon(Icons.logout_rounded),
                  ),
                  IconButton.filled(
                    onPressed: busy
                        ? null
                        : () {
                            runtime.queueFlux();
                          },
                    icon: const Icon(Icons.refresh_rounded),
                  ),
                ],
              );
            },
          ),
          Card(
            child: StreamBuilder<NetFlux>(
              stream: runtime.netFluxStream,
              builder: (context, snap) {
                final data = snap.data;
                if (data == null) {
                  return const CircularProgressIndicator();
                }
                final username = data.username;
                final flux = data.flux.field0;
                final onlineTime = data.onlineTime.field0;
                final balance = data.balance.field0;
                return Column(
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
                      title: StreamBuilder<String>(
                        stream: runtime.statusStream,
                        builder: (context, snap) {
                          final String? data = snap.data;
                          if (data == null) {
                            return const LinearProgressIndicator();
                          }
                          return Text(data);
                        },
                      ),
                    ),
                    ListTile(
                      leading: const Icon(Icons.pattern_rounded),
                      title: StreamBuilder<NetState>(
                        stream: runtime.stateStream,
                        builder: (context, snap) {
                          final data = snap.data;
                          if (data == null) {
                            return const LinearProgressIndicator();
                          }
                          return Text(data.name);
                        },
                      ),
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
              },
            ),
          ),
        ],
      ),
    );
  }
}
