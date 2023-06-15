import 'package:duration/locale.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:system_theme/system_theme.dart';
import 'package:material_color_generator/material_color_generator.dart';
import 'package:format/format.dart';
import 'package:duration/duration.dart';
import 'runtime.dart';

void main() async {
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
        primarySwatch:
            generateMaterialColor(color: SystemTheme.accentColor.accent),
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
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              StreamBuilder<NetState>(
                stream: runtime.stateSink.stream,
                builder: (context, snap) {
                  final data = snap.data;
                  if (data == null) {
                    return const CircularProgressIndicator();
                  }

                  return DropdownButton<NetState>(
                    items: [NetState.Net, NetState.Auth4, NetState.Auth6]
                        .map((NetState s) {
                      return DropdownMenuItem(value: s, child: Text(s.name));
                    }).toList(),
                    value: data,
                    onChanged: (v) {},
                  );
                },
              ),
              Expanded(
                child: ElevatedButton(
                  onPressed: () {
                    runtime.queueLogin();
                  },
                  child: const Text('登录'),
                ),
              ),
              Expanded(
                child: ElevatedButton(
                  onPressed: () {
                    runtime.queueLogout();
                  },
                  child: const Text('注销'),
                ),
              ),
              Expanded(
                child: ElevatedButton(
                  onPressed: () {
                    runtime.queueFlux();
                  },
                  child: const Text('刷新'),
                ),
              ),
            ],
          ),
          StreamBuilder<NetFlux>(
            stream: runtime.netFluxSink.stream,
            builder: (context, snap) {
              final style = Theme.of(context).textTheme.bodyLarge;

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
                  Text('用户：$username', style: style),
                  FutureBuilder<String>(
                    future: api.fluxToString(f: flux),
                    builder: (context, snap) {
                      final s = snap.data;
                      String text = '流量：';
                      if (s != null) {
                        text += s;
                      }
                      return Text(text, style: style);
                    },
                  ),
                  Text(
                      '时长：{}'.format(prettyDuration(onlineTime,
                          locale: const ChineseSimplifiedDurationLocale())),
                      style: style),
                  Text('余额：¥{:.2f}'.format(balance), style: style),
                ],
              );
            },
          ),
        ],
      ),
    );
  }
}
