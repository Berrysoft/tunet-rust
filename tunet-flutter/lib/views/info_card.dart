import 'package:binding/binding.dart';
import 'package:data_size/data_size.dart';
import 'package:duration/duration.dart';
import 'package:duration/locale.dart';
import 'package:flutter/material.dart';
import 'package:format/format.dart';
import '../runtime.dart';

class InfoCard extends StatelessWidget {
  const InfoCard({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.netFluxProperty,
      builder: (context, runtime) {
        final netFlux = runtime.netFlux;
        if (netFlux == null) return const LinearProgressIndicator();
        final username = netFlux.username;
        final flux = netFlux.flux.field0;
        final onlineTime = netFlux.onlineTime.field0;
        final balance = netFlux.balance.field0;

        return Card(
          child: Column(
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
          ),
        );
      },
    );
  }
}
