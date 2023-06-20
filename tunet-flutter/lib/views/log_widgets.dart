import 'package:binding/binding.dart';
import 'package:flutter/material.dart';
import '../runtime.dart';

class LogWidgets extends StatelessWidget {
  const LogWidgets({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Binding<ManagedRuntime>(
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
                      runtime.queueOnlines();
                      runtime.queueDetails();
                    },
              icon: const Icon(Icons.refresh_rounded),
            ),
          ],
        );
      },
    );
  }
}
