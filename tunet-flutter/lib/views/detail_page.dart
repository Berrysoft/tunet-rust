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
  bool detailBusy = false;
  DetailDailyWrap? daily;

  late StreamSubscription<bool> detailBusySub;
  late StreamSubscription<DetailDailyWrap?> dailySub;

  final GlobalKey<RefreshIndicatorState> refreshIndicatorKey = GlobalKey();

  @override
  void initState() {
    super.initState();

    final runtime = widget.runtime;
    initStateAsync(runtime);
  }

  Future<void> initStateAsync(ManagedRuntime runtime) async {
    final detailBusy = await runtime.detailBusy();
    final daily = await runtime.detailDaily();
    setState(() {
      this.detailBusy = detailBusy;
      this.daily = daily;
    });
    detailBusySub = runtime.detailBusyStream
        .listen((event) => setState(() => this.detailBusy = event));
    dailySub = runtime.dailyStream
        .listen((event) => setState(() => this.daily = event));
  }

  @override
  void dispose() {
    detailBusySub.cancel();
    dailySub.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final runtime = widget.runtime;
    final detailBusy = this.detailBusy;
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
        topTitles: AxisTitles(
          axisNameWidget:
              Text("按日统计", style: Theme.of(context).textTheme.titleLarge),
          axisNameSize: 40,
          sideTitles: const SideTitles(showTitles: false),
        ),
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
      dailyChart = Expanded(
        child: Card(
          child: Padding(
            padding: const EdgeInsets.all(8.0),
            child: LineChart(data),
          ),
        ),
      );
    }
    return Scaffold(
      body: RefreshIndicator(
        key: refreshIndicatorKey,
        onRefresh: () async {
          runtime.queueDetails();
          final iter = StreamIterator(runtime.detailBusyStream);
          while (await iter.moveNext()) {
            if (!iter.current) {
              return;
            }
          }
        },
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            dailyChart,
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
      ),
      floatingActionButton: FloatingActionButton.small(
        onPressed: detailBusy
            ? null
            : () {
                refreshIndicatorKey.currentState?.show();
              },
        child: const Icon(Icons.refresh_rounded),
      ),
      floatingActionButtonLocation: FloatingActionButtonLocation.endTop,
    );
  }
}
