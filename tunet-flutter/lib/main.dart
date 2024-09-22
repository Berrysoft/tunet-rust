import 'dart:io';
import 'package:binding/binding.dart';
import 'package:dynamic_color/dynamic_color.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:fluttertoast/fluttertoast.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:tunet/views/about_card.dart';
import 'package:tunet/views/main_app_bar.dart';
import 'views/daily_card.dart';
import 'views/details_card.dart';
import 'views/flux_paint.dart';
import 'views/info_card.dart';
import 'views/onlines_card.dart';
import 'views/settings_card.dart';
import 'runtime.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final accent = await getAccentColor();
  await Permission.location.request();

  final runtime = await ManagedRuntime.newRuntime();
  runtime.start();
  runApp(MyApp(runtime: runtime, accent: accent));
}

Future<Color?> getAccentColor() async {
  if (Platform.isAndroid) {
    return (await DynamicColorPlugin.getCorePalette())
        ?.toColorScheme(
            brightness: PlatformDispatcher.instance.platformBrightness)
        .primary;
  } else {
    return await DynamicColorPlugin.getAccentColor();
  }
}

class MyApp extends StatelessWidget {
  final ManagedRuntime runtime;
  final Color? accent;

  const MyApp({super.key, required this.runtime, required this.accent});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '清华校园网',
      theme: ThemeData(
        colorSchemeSeed: accent,
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        brightness: Brightness.dark,
        colorSchemeSeed: accent,
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: BindingProvider(
        child: BindingSource<ManagedRuntime>(
          instance: runtime,
          child: const MainPage(),
        ),
      ),
      builder: FToastBuilder(),
    );
  }
}

class MainPage extends StatefulWidget {
  const MainPage({super.key});

  @override
  State<StatefulWidget> createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  FToast fToast = FToast();
  late PropertyChangedCallbackWrap<ManagedRuntime> logTextCallback;

  @override
  void initState() {
    super.initState();

    fToast.init(context);

    final runtime = BindingSource.of<ManagedRuntime>(context);
    logTextCallback = PropertyChangedCallbackWrap<ManagedRuntime>(
      source: runtime,
      callback: (runtime) {
        final logText = runtime.logText;
        _logTextBuilder(fToast, logText);
      },
    );

    final provider = BindingProvider.of(context);
    provider.add(ManagedRuntime.logTextProperty, logTextCallback);
  }

  @override
  void dispose() {
    final provider = BindingProvider.of(context);
    provider.remove(ManagedRuntime.logTextProperty, logTextCallback);

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      appBar: MainAppBar(),
      body: SingleChildScrollView(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            FluxPaint(),
            InfoCard(),
            SettingsCard(),
            OnlinesCard(),
            DailyCard(),
            DetailsCard(),
            AboutCard(),
          ],
        ),
      ),
    );
  }
}

void _logTextBuilder(FToast fToast, String text) {
  Widget toast = Container(
    padding: const EdgeInsets.all(8.0),
    child: Text(text),
  );
  fToast.showToast(
    child: toast,
    gravity: ToastGravity.BOTTOM,
    toastDuration: const Duration(seconds: 1),
  );
}
