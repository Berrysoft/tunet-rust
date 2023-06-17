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

  late StreamSubscription<DetailDailyWrap?> dailySub;

  @override
  void initState() {
    super.initState();

    final runtime = widget.runtime;
    initStateAsync(runtime);
  }

  Future<void> initStateAsync(ManagedRuntime runtime) async {
    final daily = await runtime.detailDaily();
    setState(() {
      this.daily = daily;
    });
    dailySub = runtime.dailyStream
        .listen((event) => setState(() => this.daily = event));
  }

  @override
  void dispose() {
    dailySub.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;
    final daily = this.daily;
    Widget dailyChart = const Flexible(child: LinearProgressIndicator());
    if (daily != null) {
      const double gbRatio = 1000000000;
      final titles = FlTitlesData(
        leftTitles: AxisTitles(
          sideTitles: SideTitles(
            getTitlesWidget: (value, meta) => SideTitleWidget(
              axisSide: meta.axisSide,
              child: FutureBuilder(
                future: api.fluxToString(f: (value * gbRatio).toInt()),
                builder: (context, snap) {
                  final s = snap.data;
                  if (s == null) {
                    return const LinearProgressIndicator();
                  }
                  return Text(s);
                },
              ),
            ),
            showTitles: true,
            reservedSize: 70,
          ),
        ),
        bottomTitles: AxisTitles(
          sideTitles: SideTitles(
            getTitlesWidget: (value, meta) => SideTitleWidget(
              axisSide: meta.axisSide,
              child: Text('${value.toInt()}日'),
            ),
            showTitles: true,
            reservedSize: 30,
          ),
        ),
        topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
        rightTitles:
            const AxisTitles(sideTitles: SideTitles(showTitles: false)),
      );

      var data = LineChartData(
        lineBarsData: [
          LineChartBarData(
            spots: daily.details
                .map((p) => FlSpot(
                      p.day.toDouble(),
                      p.flux.field0.toDouble() / gbRatio,
                    ))
                .toList(),
          )
        ],
        titlesData: titles,
        minX: 1,
        maxX: daily.nowDay.toDouble(),
        minY: 0,
      );
      dailyChart = LineChart(data);
    }
    return Container(
      margin: const EdgeInsets.all(8.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(
            child: Card(
              child: Padding(
                padding: const EdgeInsets.all(8.0),
                child: dailyChart,
              ),
            ),
          ),
          PaginatedDataTable(
            columns: const [
              DataColumn(label: Text('登录时间')),
              DataColumn(label: Text('注销时间')),
              DataColumn(label: Text('流量')),
            ],
            source: runtime.detailsData,
            showCheckboxColumn: false,
            rowsPerPage: 6,
          ),
        ],
      ),
    );
  }
}
