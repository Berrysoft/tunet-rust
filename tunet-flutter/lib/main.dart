import 'package:duration/locale.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:system_theme/system_theme.dart';
import 'package:format/format.dart';
import 'package:duration/duration.dart';
import 'runtime.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  if (defaultTargetPlatform == TargetPlatform.android) {
    await SystemTheme.accentColor.load();
  }

  final runtime = await ManagedRuntime.newRuntime();
  runtime.start();
  runApp(MyApp(runtime: runtime));
}

class MyApp extends StatelessWidget {
  final ManagedRuntime runtime;

  const MyApp({Key? key, required this.runtime}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '清华校园网',
      theme: ThemeData(
        brightness: PlatformDispatcher.instance.platformBrightness,
        colorSchemeSeed: SystemTheme.accentColor.accent,
        useMaterial3: true,
      ),
      home: DefaultTabController(
        length: 1,
        child: Scaffold(
          appBar: AppBar(
            title: const TabBar(
              tabs: [
                Tab(icon: Icon(Icons.home), text: '主页'),
              ],
            ),
          ),
          body: TabBarView(
            children: [
              HomePage(runtime: runtime),
            ],
          ),
        ),
      ),
    );
  }
}

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
            stream: runtime.logBusySink.stream,
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
              stream: runtime.netFluxSink.stream,
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
                            return const CircularProgressIndicator();
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
                        stream: runtime.statusSink.stream,
                        builder: (context, snap) {
                          final String? data = snap.data;
                          if (data == null) {
                            return const CircularProgressIndicator();
                          }
                          return Text(data);
                        },
                      ),
                    ),
                    ListTile(
                      leading: const Icon(Icons.pattern_rounded),
                      title: StreamBuilder<NetState>(
                        stream: runtime.stateSink.stream,
                        builder: (context, snap) {
                          final data = snap.data;
                          if (data == null) {
                            return const CircularProgressIndicator();
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
