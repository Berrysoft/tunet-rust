import 'dart:io';
import 'dart:typed_data';

import 'package:binding/binding.dart';
import 'package:collection/collection.dart';
import 'package:data_size/data_size.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:shimmer_animation/shimmer_animation.dart';
import '../runtime.dart';

class OnlinesCard extends StatefulWidget {
  const OnlinesCard({super.key});

  @override
  State<StatefulWidget> createState() => _OnlinesCardState();
}

class _OnlinesCardState extends State<OnlinesCard> {
  List<bool> onlinesSelected = List.empty();

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Card(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          ListTile(
            leading: const Icon(Icons.people_alt_rounded),
            title: const Text('管理连接'),
            trailing: PopupMenuButton<_OnlineAction>(
              onSelected: (value) {
                switch (value) {
                  case _OnlineAction.connect:
                    _ipDialogBuilder(context, runtime);
                    break;
                  case _OnlineAction.drop:
                    final onlines = runtime.onlines;
                    if (onlines != null) {
                      final ips = onlines
                          .whereIndexed((index, _) => onlinesSelected[index])
                          .map((u) => u.address)
                          .toList();
                      runtime.queueDrop(ips: ips);
                    }
                    break;
                }
              },
              itemBuilder: (context) => const [
                PopupMenuItem(
                  value: _OnlineAction.connect,
                  child: ListTile(
                    leading: Icon(Icons.add_rounded),
                    title: Text('认证IP'),
                  ),
                ),
                PopupMenuItem(
                  value: _OnlineAction.drop,
                  child: ListTile(
                    leading: Icon(Icons.remove_rounded),
                    title: Text('下线IP'),
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
              List<DataRow> rows;
              if (onlines == null) {
                rows = [
                  DataRow(
                    cells: List.filled(
                      5,
                      DataCell(Shimmer(child: const Text('               '))),
                    ),
                    onSelectChanged: (_) {},
                  ),
                ];
              } else {
                if (onlinesSelected.length != onlines.length) {
                  onlinesSelected = List.filled(onlines.length, false);
                }
                rows = onlines
                    .mapIndexed(
                      (index, element) => _netUserToRow(
                        element,
                        onlinesSelected[index],
                        (selected) =>
                            setState(() => onlinesSelected[index] = selected!),
                      ),
                    )
                    .toList();
              }
              return LayoutBuilder(
                builder: (context, constraints) => SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: ConstrainedBox(
                    constraints: BoxConstraints(minWidth: constraints.minWidth),
                    child: DataTable(
                      columns: const [
                        DataColumn(label: Text('IP地址')),
                        DataColumn(label: Text('登录时间')),
                        DataColumn(label: Text('流量')),
                        DataColumn(label: Text('MAC地址')),
                        DataColumn(label: Text('设备')),
                      ],
                      rows: rows,
                    ),
                  ),
                ),
              );
            },
          ),
        ],
      ),
    );
  }
}

DataRow _netUserToRow(
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

enum _OnlineAction {
  connect,
  drop,
}

Future<void> _ipDialogBuilder(BuildContext context, ManagedRuntime runtime) {
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
                  octets: U8Array4(
                    InternetAddress(ipController.text).rawAddress,
                  ),
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
