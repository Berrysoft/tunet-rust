import 'package:binding/binding.dart';
import 'package:data_size/data_size.dart';
import 'package:duration/duration.dart';
import 'package:duration/locale.dart';
import 'package:flutter/material.dart';
import 'package:format/format.dart';
import 'package:shimmer_animation/shimmer_animation.dart';
import '../runtime.dart';

class InfoCard extends StatelessWidget {
  const InfoCard({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Card(
      child: Binding<ManagedRuntime>(
        source: runtime,
        path: ManagedRuntime.netFluxProperty,
        builder: (context, runtime) {
          final netFlux = runtime.netFlux;
          final username = netFlux?.username;
          final flux = netFlux?.flux.field0;
          final onlineTime = netFlux?.onlineTime.field0;
          final balance = netFlux?.balance.field0;

          return Column(
            mainAxisAlignment: MainAxisAlignment.start,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              ListTile(
                leading: const Icon(Icons.person_2_rounded),
                title: Shimmer(
                  enabled: netFlux == null,
                  child: Text(username ?? ""),
                ),
              ),
              ListTile(
                leading: const Icon(Icons.sync_alt_rounded),
                title: Shimmer(
                  enabled: netFlux == null,
                  child: Text(flux == null ? "" : flux.formatByteSize()),
                ),
              ),
              ListTile(
                leading: const Icon(Icons.timelapse_rounded),
                title: Shimmer(
                  enabled: netFlux == null,
                  child: Text(
                    onlineTime == null
                        ? ""
                        : prettyDuration(
                            onlineTime,
                            locale: const ChineseSimplifiedDurationLocale(),
                          ),
                  ),
                ),
              ),
              ListTile(
                leading: const Icon(Icons.account_balance_rounded),
                title: Shimmer(
                  enabled: netFlux == null,
                  child: Text(balance == null ? "" : '¥{:.2f}'.format(balance)),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}
