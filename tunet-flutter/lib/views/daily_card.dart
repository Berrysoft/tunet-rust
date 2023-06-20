import 'package:binding/binding.dart';
import 'package:collection/collection.dart';
import 'package:data_size/data_size.dart';
import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import '../runtime.dart';

class DailyCard extends StatelessWidget {
  const DailyCard({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Binding<ManagedRuntime>(
      source: runtime,
      path: ManagedRuntime.dailyProperty,
      builder: (context, runtime) {
        final daily = runtime.daily;
        if (daily == null) {
          return const LinearProgressIndicator();
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
        return Card(
          child: SizedBox(
            height: 300,
            child: Padding(
              padding: const EdgeInsets.all(8.0),
              child: LineChart(data),
            ),
          ),
        );
      },
    );
  }
}
