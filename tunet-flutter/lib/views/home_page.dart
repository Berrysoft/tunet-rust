import 'dart:async';
import 'dart:io';
import 'dart:math';
import 'package:binding/binding.dart';
import 'package:collection/collection.dart';
import 'package:data_size/data_size.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:fluttertoast/fluttertoast.dart';
import 'package:format/format.dart';
import 'package:duration/duration.dart';
import 'package:duration/locale.dart';
import 'package:intl/intl.dart';
import '../runtime.dart';

class HomePage extends StatefulWidget {
  const HomePage({Key? key}) : super(key: key);

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  FToast fToast = FToast();

  List<bool> onlinesSelected = List.empty();

  @override
  void initState() {
    super.initState();

    fToast.init(context);
  }

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);

    Widget fluxBody = Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.netFluxProperty,
      builder: (context, runtime) {
        final netFlux = runtime.netFlux;
        if (netFlux == null) return const LinearProgressIndicator();
        final flux = netFlux.flux.field0;
        final balance = netFlux.balance.field0;

        return CustomPaint(
          size: Size(MediaQuery.of(context).size.width, 30.0),
          painter: FluxPainter(
            flux: flux.toDouble() / 1000000000.0,
            balance: balance,
            accent: Theme.of(context).colorScheme.primary,
          ),
        );
      },
    );

    Widget cardBody = Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.netFluxProperty,
      builder: (context, runtime) {
        final netFlux = runtime.netFlux;
        if (netFlux == null) return const LinearProgressIndicator();
        final username = netFlux.username;
        final flux = netFlux.flux.field0;
        final onlineTime = netFlux.onlineTime.field0;
        final balance = netFlux.balance.field0;

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
              title: Text(flux.formatByteSize()),
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
      },
    );

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
            Binding<ManagedRuntime>(
              source: runtime,
              path: ManagedRuntime.logBusyProperty,
              builder: (context, runtime) {
                final logBusy = runtime.logBusy;
                return Row(
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
                );
              },
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
                      title: Binding<ManagedRuntime>(
                        source: runtime,
                        path: ManagedRuntime.usernameProperty,
                        builder: (context, runtime) {
                          final username = runtime.username;
                          return username.isEmpty
                              ? const Text('设置凭据')
                              : Text(username);
                        },
                      ),
                    ),
                    onTap: () => credDialogBuilder(context, runtime),
                  ),
                  ListTile(
                    leading: const Icon(Icons.signal_cellular_alt_rounded),
                    title: Binding<ManagedRuntime>(
                      source: runtime,
                      path: ManagedRuntime.statusProperty,
                      builder: (context, runtime) {
                        final status = runtime.status;
                        return status.isEmpty
                            ? const LinearProgressIndicator()
                            : Text(status);
                      },
                    ),
                  ),
                  ListTile(
                    leading: const Icon(Icons.pattern_rounded),
                    title: Binding<ManagedRuntime>(
                      source: runtime,
                      path: ManagedRuntime.stateProperty,
                      builder: (context, runtime) => Text(runtime.state.name),
                    ),
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
                            final onlines = runtime.onlines;
                            if (onlines != null) {
                              final ips = onlines
                                  .whereIndexed(
                                      (index, _) => onlinesSelected[index])
                                  .map((u) => u.address)
                                  .toList();
                              runtime.queueDrop(ips: ips);
                            }
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
                  Binding<ManagedRuntime>(
                    source: runtime,
                    path: ManagedRuntime.onlinesProperty,
                    builder: (context, runtime) {
                      final onlines = runtime.onlines;
                      if (onlines == null) {
                        return const Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [CircularProgressIndicator()],
                        );
                      }
                      if (onlinesSelected.length != onlines.length) {
                        onlinesSelected = List.filled(onlines.length, false);
                      }
                      return SingleChildScrollView(
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
                      );
                    },
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
      DataCell(Text(u.flux.field0.formatByteSize())),
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
  final formKey = GlobalKey<FormState>();
  final usernameController = TextEditingController();
  final passwordController = TextEditingController();
  return showDialog(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('设置凭据'),
      content: GestureDetector(
        behavior: HitTestBehavior.translucent,
        child: AutofillGroup(
          child: Form(
            key: formKey,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextFormField(
                  controller: usernameController,
                  decoration: const InputDecoration(labelText: '用户名'),
                  keyboardType: TextInputType.name,
                  textInputAction: TextInputAction.next,
                  autofillHints: const [AutofillHints.username],
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return '用户名不能为空';
                    }
                    return null;
                  },
                ),
                TextFormField(
                  controller: passwordController,
                  decoration: const InputDecoration(labelText: '密码'),
                  keyboardType: TextInputType.visiblePassword,
                  textInputAction: TextInputAction.done,
                  autofillHints: const [AutofillHints.password],
                  obscureText: true,
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return '密码不能为空';
                    }
                    return null;
                  },
                ),
              ],
            ),
            onChanged: () => formKey.currentState!.validate(),
          ),
        ),
        onTap: () {
          if (!formKey.currentState!.validate()) {
            TextInput.finishAutofillContext(shouldSave: false);
          }
        },
      ),
      actions: [
        TextButton(
          child: const Text('确定'),
          onPressed: () {
            final username = usernameController.text;
            final password = passwordController.text;
            if (formKey.currentState!.validate()) {
              TextInput.finishAutofillContext();
              runtime.queueCredential(u: username, p: password);
              Navigator.of(context).pop();
            }
          },
        ),
      ],
    ),
  );
}

Future<void> ipDialogBuilder(BuildContext context, ManagedRuntime runtime) {
  final formKey = GlobalKey<FormState>();
  final ipController = TextEditingController();
  return showDialog(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('认证IP'),
      content: Form(
        key: formKey,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextFormField(
              controller: ipController,
              decoration: const InputDecoration(labelText: 'IP地址'),
              keyboardType: TextInputType.number,
              validator: (value) {
                if (value != null) {
                  final ip = InternetAddress.tryParse(value);
                  if (ip != null) {
                    if (ip.type == InternetAddressType.IPv4) {
                      return null;
                    } else {
                      return '不支持的地址类型';
                    }
                  }
                }
                return '文本无效';
              },
            ),
          ],
        ),
        onChanged: () => formKey.currentState!.validate(),
      ),
      actions: [
        TextButton(
          child: const Text('确定'),
          onPressed: () {
            if (formKey.currentState!.validate()) {
              runtime.queueConnect(
                ip: Ipv4AddrWrap(
                  octets:
                      U8Array4(InternetAddress(ipController.text).rawAddress),
                ),
              );
              Navigator.of(context).pop();
            }
          },
        )
      ],
    ),
  );
}
