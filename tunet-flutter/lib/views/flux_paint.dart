import 'dart:math';
import 'package:binding/binding.dart';
import 'package:flutter/material.dart';
import '../runtime.dart';

class FluxPaint extends StatelessWidget {
  const FluxPaint({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    final size = Size(MediaQuery.of(context).size.width, 30.0);
    final accent = Theme.of(context).colorScheme.primary;
    return Padding(
      padding: const EdgeInsets.all(4.0),
      child: Binding<ManagedRuntime>(
        source: runtime,
        path: ManagedRuntime.logBusyProperty,
        builder: (context, runtime) {
          final netFlux = runtime.netFlux;
          if (netFlux == null) {
            return CustomPaint(
              size: size,
              painter: _FluxPainter(
                freeRatio: 0,
                fluxRatio: 0,
                accent: accent,
              ),
            );
          }
          final flux = netFlux.flux.field0;
          final balance = netFlux.balance.field0;

          final fluxGB = flux.toDouble() / 1000000000.0;

          final totalFlux = balance + max(50.0, fluxGB);
          final freeRatio = 50.0 / totalFlux;
          final fluxRatio = fluxGB / totalFlux;

          return TweenAnimationBuilder<double>(
            key: UniqueKey(),
            tween: Tween(begin: 0, end: 1.0),
            duration: const Duration(milliseconds: 500),
            curve: Curves.easeOut,
            builder: (context, value, child) {
              final cfree = freeRatio + (1 - freeRatio) * (1 - value);
              final cflux = fluxRatio * value;
              return CustomPaint(
                size: size,
                painter: _FluxPainter(
                  freeRatio: cfree,
                  fluxRatio: cflux,
                  accent: accent,
                ),
              );
            },
          );
        },
      ),
    );
  }
}

class _FluxPainter extends CustomPainter {
  final double freeRatio;
  final double fluxRatio;
  final Color accent;

  const _FluxPainter({
    required this.freeRatio,
    required this.fluxRatio,
    required this.accent,
  }) : super();

  @override
  void paint(Canvas canvas, Size size) {
    final f1 = Paint()
      ..color = accent
      ..style = PaintingStyle.fill;
    final f2 = Paint()
      ..color = accent.withAlpha(168)
      ..style = PaintingStyle.fill;
    final f3 = Paint()
      ..color = accent.withAlpha(84)
      ..style = PaintingStyle.fill;

    final fullWidth = size.width;
    final freeWidth = freeRatio * fullWidth;
    final fluxWidth = fluxRatio * fullWidth;

    const radius = Radius.circular(8.0);

    canvas.drawRRect(RRect.fromLTRBR(0, 0, fullWidth, size.height, radius), f3);
    canvas.drawRRect(RRect.fromLTRBR(0, 0, freeWidth, size.height, radius), f2);
    canvas.drawRRect(RRect.fromLTRBR(0, 0, fluxWidth, size.height, radius), f1);
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) => true;
}
