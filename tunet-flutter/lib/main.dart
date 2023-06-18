import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:fluttertoast/fluttertoast.dart';
import 'package:system_theme/system_theme.dart';
import 'runtime.dart';
import 'views/home_page.dart';
import 'views/detail_page.dart';
import 'views/about_page.dart';

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
        length: 3,
        child: Scaffold(
          appBar: AppBar(
            title: const TabBar(
              tabs: [
                Tab(icon: Icon(Icons.home_rounded), text: '主页'),
                Tab(icon: Icon(Icons.auto_graph_rounded), text: '明细'),
                Tab(icon: Icon(Icons.help_outline_rounded), text: '关于'),
              ],
            ),
          ),
          body: TabBarView(
            children: [
              HomePage(runtime: runtime),
              DetailPage(runtime: runtime),
              const AboutPage(),
            ],
          ),
        ),
      ),
      builder: FToastBuilder(),
    );
  }
}
