import 'package:binding/binding.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:shimmer_animation/shimmer_animation.dart';
import '../runtime.dart';

class SettingsCard extends StatelessWidget {
  const SettingsCard({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Card(
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
                  return username.isEmpty ? const Text('设置凭据') : Text(username);
                },
              ),
            ),
            onTap: () => _credDialogBuilder(context, runtime),
          ),
          ListTile(
            leading: const Icon(Icons.signal_cellular_alt_rounded),
            title: Binding<ManagedRuntime>(
              source: runtime,
              path: ManagedRuntime.statusProperty,
              builder: (context, runtime) {
                final status = runtime.status;
                return Shimmer(
                  enabled: status.isEmpty,
                  child: Text(status),
                );
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
    );
  }
}

Future<void> _credDialogBuilder(BuildContext context, ManagedRuntime runtime) {
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
