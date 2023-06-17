import 'dart:async';
import 'dart:math';
import 'package:flutter/material.dart';
import 'package:fluttertoast/fluttertoast.dart';
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
  FToast fToast = FToast();

  bool logBusy = false;
  NetFlux? netFlux;
  String? status;
  NetState state = NetState.Unknown;

  late StreamSubscription<bool> logBusySub;
  late StreamSubscription<String> logTextSub;
  late StreamSubscription<NetFlux> netFluxSub;
  late StreamSubscription<String> statusSub;
  late StreamSubscription<NetState> stateSub;

  @override
  void initState() {
    super.initState();

    fToast.init(context);

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

    logTextSub =
        runtime.logTextStream.listen((event) => logTextBuilder(fToast, event));
  }

  @override
  void dispose() {
    logBusySub.cancel();
    netFluxSub.cancel();
    statusSub.cancel();
    stateSub.cancel();
    logTextSub.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;

    Widget cardBody = const LinearProgressIndicator();
    Widget fluxBody = const LinearProgressIndicator();
    final netFlux = this.netFlux;
    if (netFlux != null) {
      final username = netFlux.username;
      final flux = netFlux.flux.field0;
      final onlineTime = netFlux.onlineTime.field0;
      final balance = netFlux.balance.field0;
      final status = this.status;
      final state = this.state;

      final theme = Theme.of(context);
      fluxBody = CustomPaint(
        size: Size(MediaQuery.of(context).size.width, 30.0),
        painter: FluxPainter(
          flux: flux.toDouble() / 1000000000.0,
          balance: balance,
          accent: theme.colorScheme.primary,
        ),
      );

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
            title: FutureBuilder(
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
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: fluxBody,
          ),
          Card(child: cardBody),
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
        ],
      ),
    );
  }
}

class FluxPainter extends CustomPainter {
  final double flux;
  final double balance;
  final Color accent;

  const FluxPainter({
    required this.flux,
    required this.balance,
    required this.accent,
  }) : super();

  @override
  void paint(Canvas canvas, Size size) {
    final f1 = Paint()
      ..color = accent
      ..style = PaintingStyle.fill;
    final f2 = Paint()
      ..color = accent.withOpacity(0.66)
      ..style = PaintingStyle.fill;
    final f3 = Paint()
      ..color = accent.withOpacity(0.33)
      ..style = PaintingStyle.fill;

    final totalFlux = balance + max(50.0, flux);
    final freeRatio = 50.0 / totalFlux;
    final fluxRatio = flux / totalFlux;

    final fullWidth = size.width;
    final freeWidth = freeRatio * fullWidth;
    final fluxWidth = fluxRatio * fullWidth;

    const radius = Radius.circular(8.0);

    canvas.drawRRect(RRect.fromLTRBR(0, 0, fullWidth, size.height, radius), f3);
    canvas.drawRRect(RRect.fromLTRBR(0, 0, freeWidth, size.height, radius), f2);
    canvas.drawRRect(RRect.fromLTRBR(0, 0, fluxWidth, size.height, radius), f1);
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) => true;
}

void logTextBuilder(FToast fToast, String text) {
  Widget toast = Container(
    padding: const EdgeInsets.all(8.0),
    child: Text(text),
  );
  fToast.showToast(
    child: toast,
    gravity: ToastGravity.BOTTOM,
    toastDuration: const Duration(seconds: 2),
  );
}
