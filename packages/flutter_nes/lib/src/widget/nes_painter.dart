import 'dart:typed_data';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';

import '../native/api/nes.dart';
import 'nes_provider.dart';

class NesWidget extends StatefulWidget {
  final NesConfig? config;
  final NesDataProvider dataProvider;

  const NesWidget._({super.key, required this.dataProvider, this.config});

  factory NesWidget.assets({
    Key? key,
    required String assets,
    String? name,
    NesConfig? config,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        dataProvider: AssetsNesDataProvider(assets: assets, name: name),
      );

  factory NesWidget.network({
    Key? key,
    required String url,
    String? name,
    NesConfig? config,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        dataProvider: NetworkNesDataProvider(url: url, name: name),
      );

  factory NesWidget.file({
    Key? key,
    required String path,
    String? name,
    NesConfig? config,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        dataProvider: FileNesDataProvider(path: path, name: name),
      );

  factory NesWidget.memory({
    Key? key,
    required Uint8List data,
    String? name,
    NesConfig? config,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        dataProvider: MemoryNesDataProvider(data: data, name: name),
      );

  @override
  State<NesWidget> createState() => _NesWidgetState();
}

class _NesWidgetState extends State<NesWidget> {
  ui.Image? _image;
  Future<void>? _future;
  late final NesEmulator emulator = widget.config == null
      ? NesEmulator.create()
      : NesEmulator.withConfig(config: widget.config!);

  @override
  void initState() {
    _future = loadAndRun();
    super.initState();
  }

  Future<void> loadAndRun() async {
    final name = widget.dataProvider.name;
    final data = await widget.dataProvider.resolveData();
    await emulator.loadRom(name: name, data: data);
    emulator.runLoop(onData: (data) {
      ui.decodeImageFromPixels(
          data, kNesWidth.toInt(), kNesHeight.toInt(), ui.PixelFormat.rgba8888,
          (result) {
        setState(() {
          _image = result;
        });
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: kNesWidth,
      height: kNesHeight,
      child: FutureBuilder(
          future: _future,
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.done) {
              if (_image == null) {
                return Container();
              }
              return CustomPaint(
                painter: NesPainter(_image!),
                size: Size(kNesWidth.toDouble(), kNesHeight.toDouble()),
              );
            }
            return const CircularProgressIndicator();
          }),
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
