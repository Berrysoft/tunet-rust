import 'package:duration/locale.dart';
import 'package:flutter/material.dart';
import 'package:format/format.dart';
import 'package:duration/duration.dart';
import 'runtime.dart';

void main() async {
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
        primarySwatch: Colors.blue,
      ),
      home: MyHomePage(runtime: runtime),
    );
  }
}

class MyHomePage extends StatefulWidget {
  final ManagedRuntime runtime;

  const MyHomePage({Key? key, required this.runtime}) : super(key: key);

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;
    return Scaffold(
      appBar: AppBar(
        title: const Text('清华校园网'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
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
                  mainAxisAlignment: MainAxisAlignment.start,
                  children: <Widget>[
                    Text('用户：$username', style: style),
                    FutureBuilder<String>(
                      future: api.fluxToString(f: flux),
                      builder: (context, snap) {
                        final s = snap.data;
                        if (s == null) {
                          return Text('流量：', style: style);
                        } else {
                          return Text('流量：{}'.format(s), style: style);
                        }
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
            )
          ],
        ),
      ),
    );
  }
}
