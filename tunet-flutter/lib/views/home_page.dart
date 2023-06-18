import 'dart:async';
import 'dart:io';
import 'dart:math';
import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:fluttertoast/fluttertoast.dart';
import 'package:format/format.dart';
import 'package:duration/duration.dart';
import 'package:duration/locale.dart';
import 'package:intl/intl.dart';
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
  String username = "";
  bool onlineBusy = false;

  List<bool> onlinesSelected = List.empty();

  late StreamSubscription<bool> logBusySub;
  late StreamSubscription<String> logTextSub;
  late StreamSubscription<NetFlux> netFluxSub;
  late StreamSubscription<String> statusSub;
  late StreamSubscription<NetState> stateSub;
  late StreamSubscription<String> usernameSub;
  late StreamSubscription<bool> onlineBusySub;

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
    final username = await runtime.username();
    final onlineBusy = await runtime.onlineBusy();
    setState(() {
      this.logBusy = logBusy;
      this.netFlux = netFlux;
      this.status = status;
      this.state = state;
      this.username = username;
      this.onlineBusy = onlineBusy;
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

    usernameSub = runtime.usernameStream
        .listen((event) => setState(() => this.username = event));
    onlineBusySub = runtime.onlineBusyStream
        .listen((event) => setState(() => this.onlineBusy = event));
  }

  @override
  void dispose() {
    logBusySub.cancel();
    netFluxSub.cancel();
    statusSub.cancel();
    stateSub.cancel();
    logTextSub.cancel();
    usernameSub.cancel();
    onlineBusySub.cancel();

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
        ],
      );
    }

    final onlines = runtime.onlinesData;
    if (onlinesSelected.length != onlines.length) {
      onlinesSelected = List.filled(onlines.length, false);
    }
    final username = this.username;
    final status = this.status;
    final state = this.state;

    return Container(
      margin: const EdgeInsets.all(8.0),
      child: SingleChildScrollView(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.all(8.0),
              child: fluxBody,
            ),
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
            Card(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  InkWell(
                    child: ListTile(
                      leading: const Icon(Icons.person_2_rounded),
                      title: username.isEmpty
                          ? const Text('设置凭据')
                          : Text(username),
                    ),
                    onTap: () => credDialogBuilder(context, runtime),
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
                        NetState.Auth6,
                      ]
                          .map((s) => PopupMenuItem(
                              value: s, child: ListTile(title: Text(s.name))))
                          .toList(),
                    ),
                  ),
                ],
              ),
            ),
            Card(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.start,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  ListTile(
                    leading: const Icon(Icons.people_alt_rounded),
                    title: const Text('管理连接'),
                    trailing: PopupMenuButton<OnlineAction>(
                      onSelected: (value) {
                        switch (value) {
                          case OnlineAction.connect:
                            ipDialogBuilder(context, runtime);
                            break;
                          case OnlineAction.drop:
                            final ips = onlines
                                .whereIndexed(
                                    (index, _) => onlinesSelected[index])
                                .map((u) => u.address)
                                .toList();
                            runtime.queueDrop(ips: ips);
                            break;
                          case OnlineAction.refresh:
                            runtime.queueOnlines();
                            break;
                        }
                      },
                      itemBuilder: (context) => const [
                        PopupMenuItem(
                          value: OnlineAction.connect,
                          child: ListTile(
                            leading: Icon(Icons.add_rounded),
                            title: Text('认证IP'),
                          ),
                        ),
                        PopupMenuItem(
                          value: OnlineAction.drop,
                          child: ListTile(
                            leading: Icon(Icons.remove_rounded),
                            title: Text('下线IP'),
                          ),
                        ),
                        PopupMenuItem(
                          value: OnlineAction.refresh,
                          child: ListTile(
                            leading: Icon(Icons.refresh_rounded),
                            title: Text('刷新'),
                          ),
                        ),
                      ],
                    ),
                  ),
                  SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    child: DataTable(
                      columns: const [
                        DataColumn(label: Text('IP地址')),
                        DataColumn(label: Text('登录时间')),
                        DataColumn(label: Text('流量')),
                        DataColumn(label: Text('MAC地址')),
                        DataColumn(label: Text('设备')),
                      ],
                      rows: onlines
                          .mapIndexed(
                            (index, element) => netUserToRow(
                              element,
                              onlinesSelected[index],
                              (selected) => setState(
                                  () => onlinesSelected[index] = selected!),
                            ),
                          )
                          .toList(),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
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

DataRow netUserToRow(
  NetUserWrap u,
  bool selected,
  void Function(bool?) onSelectedChanged,
) {
  return DataRow(
    cells: [
      DataCell(Text(InternetAddress.fromRawAddress(
        Uint8List.fromList(u.address.octets),
        type: InternetAddressType.IPv4,
      ).address)),
      DataCell(Text(DateFormat('MM-dd HH:mm').format(u.loginTime.field0))),
      DataCell(FutureBuilder(
        future: api.fluxToString(f: u.flux.field0),
        builder: (context, snap) {
          final data = snap.data;
          if (data == null) {
            return const CircularProgressIndicator();
          }
          return Text(data);
        },
      )),
      DataCell(Text(u.macAddress)),
      DataCell(Text(u.isLocal ? '本机' : '未知')),
    ],
    selected: selected,
    onSelectChanged: onSelectedChanged,
  );
}

enum OnlineAction {
  connect,
  drop,
  refresh,
}

Future<void> credDialogBuilder(BuildContext context, ManagedRuntime runtime) {
  final usernameController = TextEditingController();
  final passwordController = TextEditingController();
  return showDialog(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('设置凭据'),
      content: GestureDetector(
        behavior: HitTestBehavior.translucent,
        child: AutofillGroup(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: usernameController,
                decoration: const InputDecoration(labelText: '用户名'),
                keyboardType: TextInputType.name,
                textInputAction: TextInputAction.next,
                autofillHints: const [AutofillHints.username],
              ),
              TextField(
                controller: passwordController,
                decoration: const InputDecoration(labelText: '密码'),
                keyboardType: TextInputType.visiblePassword,
                autofillHints: const [AutofillHints.password],
                onEditingComplete: () => TextInput.finishAutofillContext(),
                obscureText: true,
              ),
            ],
          ),
        ),
        onTap: () {
          String password = passwordController.text;
          String username = usernameController.text;
          if (username.isEmpty || password.isEmpty) {
            TextInput.finishAutofillContext(shouldSave: false);
          }
        },
      ),
      actions: [
        TextButton(
          child: const Text('确定'),
          onPressed: () {
            runtime.queueCredential(
              u: usernameController.text,
              p: passwordController.text,
            );
            Navigator.of(context).pop();
          },
        ),
      ],
    ),
  );
}

Future<void> ipDialogBuilder(BuildContext context, ManagedRuntime runtime) {
  final ipController = TextEditingController();
  return showDialog(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('认证IP'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: ipController,
            decoration: const InputDecoration(labelText: 'IP地址'),
            keyboardType: TextInputType.number,
          ),
        ],
      ),
      actions: [
        TextButton(
          child: const Text('确定'),
          onPressed: () {
            runtime.queueConnect(
              ip: Ipv4AddrWrap(
                octets: U8Array4(InternetAddress(ipController.text).rawAddress),
              ),
            );
            Navigator.of(context).pop();
          },
        )
      ],
    ),
  );
}