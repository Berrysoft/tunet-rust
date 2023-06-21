import 'dart:math';
import 'package:binding/binding.dart';
import 'package:flutter/material.dart';
import '../runtime.dart';

class FluxPaint extends StatelessWidget {
  const FluxPaint({super.key});

  @override
  Widget build(BuildContext context) {
    final runtime = BindingSource.of<ManagedRuntime>(context);
    return Padding(
      padding: const EdgeInsets.all(4.0),
      child: Binding<ManagedRuntime>(
        source: runtime,
        path: ManagedRuntime.netFluxProperty,
        builder: (context, runtime) {
          final netFlux = runtime.netFlux;
          if (netFlux == null) return const LinearProgressIndicator();
          final flux = netFlux.flux.field0;
          final balance = netFlux.balance.field0;

          final fluxGB = flux.toDouble() / 1000000000.0;

          final costBalance = max(0.0, fluxGB - 50.0);

          return TweenAnimationBuilder<double>(
            tween: Tween(begin: 0, end: 1.0),
            duration: const Duration(milliseconds: 500),
            curve: Curves.easeOut,
            builder: (context, value, child) {
              final cflux = fluxGB * value;
              final cbalance = balance + costBalance * (1.0 - value);
              return CustomPaint(
                size: Size(MediaQuery.of(context).size.width, 30.0),
                painter: _FluxPainter(
                  flux: cflux,
                  balance: cbalance,
                  accent: Theme.of(context).colorScheme.primary,
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
  final double flux;
  final double balance;
  final Color accent;

  const _FluxPainter({
    required this.flux,
    required this.balance,
    required this.accent,
  }) : super();

  @override
  void paint(Canvas canvas, Size size) {
    final f1 = Paint()
      ..color = accent
      ..style = PaintingStyle.fill;
    final f2 = Paint()
      ..color = accent.withOpacity(0.66)
      ..style = PaintingStyle.fill;
    final f3 = Paint()
      ..color = accent.withOpacity(0.33)
      ..style = PaintingStyle.fill;

    final totalFlux = balance + max(50.0, flux);
    final freeRatio = 50.0 / totalFlux;
    final fluxRatio = flux / totalFlux;

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
