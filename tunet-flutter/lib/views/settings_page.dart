import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../runtime.dart';

class SettingsPage extends StatefulWidget {
  final ManagedRuntime runtime;

  const SettingsPage({Key? key, required this.runtime}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  String username = "";
  bool onlineBusy = false;

  late StreamSubscription<String> usernameSub;
  late StreamSubscription<bool> onlineBusySub;

  @override
  void initState() {
    super.initState();

    final runtime = widget.runtime;
    initStateAsync(runtime);
  }

  Future<void> initStateAsync(ManagedRuntime runtime) async {
    final username = await runtime.username();
    final onlineBusy = await runtime.onlineBusy();
    setState(() {
      this.username = username;
      this.onlineBusy = onlineBusy;
    });
    usernameSub = runtime.usernameStream
        .listen((event) => setState(() => this.username = event));
    onlineBusySub = runtime.onlineBusyStream
        .listen((event) => setState(() => this.onlineBusy = event));
  }

  @override
  void dispose() {
    usernameSub.cancel();
    onlineBusySub.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;
    final username = this.username;
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Card(
            child: InkWell(
              child: ListTile(
                leading: const Icon(Icons.person_2_rounded),
                title: username.isEmpty ? const Text('设置凭据') : Text(username),
              ),
              onTap: () => credDialogBuilder(context, runtime),
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
                          break;
                        case OnlineAction.drop:
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
                    rows: runtime.onlinesData,
                    showCheckboxColumn: false,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
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
    builder: (context) {
      return AlertDialog(
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
            child: const Text('取消'),
            onPressed: () => Navigator.of(context).pop(),
          ),
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
      );
    },
  );
}
