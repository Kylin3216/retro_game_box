import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../native/api/nes.dart';
import 'nes_provider.dart';

class NesPainterWidget extends StatefulWidget {
  final NesEmulator emulator;

  const NesPainterWidget(this.emulator, {super.key});

  @override
  State<NesPainterWidget> createState() => _NesPainterWidgetState();
}

class _NesPainterWidgetState extends State<NesPainterWidget> {
  ui.Image? _image;

  @override
  void initState() {
    _runLoop();
    super.initState();
  }

  Future<void> _runLoop() async {
    widget.emulator.runLoopForPainter().listen((data) {
      ui.decodeImageFromPixels(
          data, kNesWidth.toInt(), kNesHeight.toInt(), ui.PixelFormat.rgba8888,
          (result) {
        if (mounted) {
          setState(() {
            _image = result;
          });
        }
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: kNesWidth,
      height: kNesHeight,
      child: _image == null
          ? Container()
          : CustomPaint(
              painter: NesPainter(_image!),
              size: Size(kNesWidth.toDouble(), kNesHeight.toDouble()),
            ),
    );
  }
}

class NesPainter extends CustomPainter {
  ui.Image image;

  NesPainter(this.image);

  final Paint _paint = Paint();

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawImage(image, Offset.zero, _paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return (oldDelegate as NesPainter).image != image;
  }
}
