import 'package:binding/binding.dart';
import 'package:flutter/material.dart';
import 'package:shimmer_animation/shimmer_animation.dart';
import '../runtime.dart';

class DetailsCard extends StatefulWidget {
  const DetailsCard({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _DetailsCardState();
}

class _DetailsCardState extends State<DetailsCard> {
  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.detailBusyProperty,
      builder: (context, runtime) {
        final detailsData = runtime.detailsData;
        return Shimmer(
          enabled: runtime.daily == null,
          child: PaginatedDataTable(
            sortColumnIndex: detailsData.sortColumnIndex,
            sortAscending: detailsData.sortAscending,
            columns: [
              DataColumn(
                label: const Text('登录时间'),
                onSort: (columnIndex, ascending) =>
                    setState(() => detailsData.sort(columnIndex, ascending)),
              ),
              DataColumn(
                label: const Text('注销时间'),
                onSort: (columnIndex, ascending) =>
                    setState(() => detailsData.sort(columnIndex, ascending)),
              ),
              DataColumn(
                label: const Text('流量'),
                onSort: (columnIndex, ascending) =>
                    setState(() => detailsData.sort(columnIndex, ascending)),
              ),
            ],
            source: detailsData,
            showCheckboxColumn: false,
          ),
        );
      },
    );
  }
}
