import 'dart:async';
import 'package:flutter/material.dart';
import 'package:fl_chart/fl_chart.dart';
import '../runtime.dart';

class DetailPage extends StatefulWidget {
  final ManagedRuntime runtime;

  const DetailPage({Key? key, required this.runtime}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _DetailPageState();
}

class _DetailPageState extends State<DetailPage> {
  DetailDailyWrap? daily;

  late StreamSubscription<DetailDailyWrap> dailySub;

  @override
  void initState() {
    super.initState();

    final runtime = widget.runtime;
    dailySub =
        runtime.dailyStream.listen((event) => setState(() => daily = event));
  }

  @override
  void dispose() {
    dailySub.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final daily = this.daily;
    Widget dailyChart = const CircularProgressIndicator();
    if (daily != null) {
      var data = LineChartData(
        lineBarsData: [
          LineChartBarData(
            spots: daily.details
                .map((p) => FlSpot(
                      p.day.toDouble(),
                      p.flux.field0.toDouble() / 1000000000.0,
                    ))
                .toList(),
          )
        ],
      );
      dailyChart = LineChart(data);
    }
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Card(child: dailyChart),
        ],
      ),
    );
  }
}
