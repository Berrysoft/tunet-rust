import 'package:binding/binding.dart';
import 'package:collection/collection.dart';
import 'package:data_size/data_size.dart';
import 'package:flutter/material.dart';
import 'package:fl_chart/fl_chart.dart';
import '../runtime.dart';

class DetailPage extends StatefulWidget {
  const DetailPage({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _DetailPageState();
}

class _DetailPageState extends State<DetailPage> {
  @override
  Widget build(BuildContext context) {
    const Widget loadingWidget = Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [CircularProgressIndicator()],
    );
    final runtime = BindingSource.of<ManagedRuntime>(context);
    Widget dailyChart = Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.dailyProperty,
      builder: (context, runtime) {
        final daily = runtime.daily;
        if (daily == null) {
          return loadingWidget;
        }
        final titles = FlTitlesData(
          leftTitles: AxisTitles(
            sideTitles: SideTitles(
              getTitlesWidget: (value, meta) => SideTitleWidget(
                axisSide: meta.axisSide,
                child: Text(value.toInt().formatByteSize()),
              ),
              showTitles: true,
              reservedSize: 80,
            ),
          ),
          bottomTitles: AxisTitles(
            sideTitles: SideTitles(
              getTitlesWidget: (value, meta) => SideTitleWidget(
                axisSide: meta.axisSide,
                child: Text(value.toInt().toString()),
              ),
              showTitles: true,
              reservedSize: 30,
            ),
          ),
          topTitles:
              const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          rightTitles:
              const AxisTitles(sideTitles: SideTitles(showTitles: false)),
        );

        var touch = LineTouchData(
          touchTooltipData: LineTouchTooltipData(
            getTooltipItems: (touchedSpots) {
              final items = defaultLineTooltipItem(touchedSpots);
              return IterableZip<dynamic>([touchedSpots, items]).map((list) {
                LineBarSpot spot = list[0];
                LineTooltipItem item = list[1];
                return LineTooltipItem(
                  spot.y.toInt().formatByteSize(),
                  item.textStyle,
                );
              }).toList();
            },
          ),
        );

        var data = LineChartData(
          lineBarsData: [
            LineChartBarData(
              spots: daily.details
                  .map((p) => FlSpot(
                        p.day.toDouble(),
                        p.flux.field0.toDouble(),
                      ))
                  .toList(),
            )
          ],
          titlesData: titles,
          lineTouchData: touch,
          minX: 1,
          maxX: daily.nowDay.toDouble(),
          minY: 0,
        );
        return Expanded(
          child: Card(
            child: Padding(
              padding: const EdgeInsets.all(8.0),
              child: LineChart(data),
            ),
          ),
        );
      },
    );
    return Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.detailBusyProperty,
      builder: (context, runtime) {
        final detailBusy = runtime.detailBusy;
        final detailsData = runtime.detailsData;
        Widget tableWidget = loadingWidget;
        if (runtime.daily != null) {
          tableWidget = PaginatedDataTable(
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
            rowsPerPage: 6,
          );
        }
        return Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Card(
              child: InkWell(
                onTap: detailBusy ? null : () => runtime.queueDetails(),
                child: const ListTile(
                  leading: Icon(Icons.refresh_rounded),
                  title: Text('刷新'),
                ),
              ),
            ),
            dailyChart,
            tableWidget
          ],
        );
      },
    );
  }
}
