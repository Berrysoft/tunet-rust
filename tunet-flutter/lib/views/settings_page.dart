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
  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Card(
            child: ListTile(
              leading: const Icon(Icons.person_2_rounded),
              title: const Text('设置凭据'),
              onTap: () => credDialogBuilder(context, runtime),
            ),
          ),
        ],
      ),
    );
  }
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
