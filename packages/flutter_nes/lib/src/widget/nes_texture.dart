import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:irondash_engine_context/irondash_engine_context.dart';

import '../native/api/nes.dart';
import '../native/api/texture.dart';
import 'nes_provider.dart';

class NesTextureWidget extends StatefulWidget {
  final NesConfig? config;
  final NesDataProvider dataProvider;

  const NesTextureWidget._({
    super.key,
    required this.dataProvider,
    this.config,
  });

  factory NesTextureWidget.assets({
    Key? key,
    required String assets,
    String? name,
    NesConfig? config,
  }) =>
      NesTextureWidget._(
        key: key,
        config: config,
        dataProvider: AssetsNesDataProvider(assets: assets, name: name),
      );

  factory NesTextureWidget.network({
    Key? key,
    required String url,
    String? name,
    NesConfig? config,
  }) =>
      NesTextureWidget._(
        key: key,
        config: config,
        dataProvider: NetworkNesDataProvider(url: url, name: name),
      );

  factory NesTextureWidget.file({
    Key? key,
    required String path,
    String? name,
    NesConfig? config,
  }) =>
      NesTextureWidget._(
        key: key,
        config: config,
        dataProvider: FileNesDataProvider(path: path, name: name),
      );

  factory NesTextureWidget.memory({
    Key? key,
    required Uint8List data,
    String? name,
    NesConfig? config,
  }) =>
      NesTextureWidget._(
        key: key,
        config: config,
        dataProvider: MemoryNesDataProvider(data: data, name: name),
      );

  @override
  State<NesTextureWidget> createState() => _NesTextureWidgetState();
}

class _NesTextureWidgetState extends State<NesTextureWidget> {
  Future<int>? _future;
  NesTexture? texture;
  late final NesEmulator emulator = widget.config == null
      ? NesEmulator.create()
      : NesEmulator.withConfig(config: widget.config!);

  @override
  void initState() {
    _future = loadAndRun();
    super.initState();
  }

  Future<int> loadAndRun() async {
    final handle = await EngineContext.instance.getEngineHandle();
    texture = await NesTexture.create(handle: handle);
    final id = texture?.id();
    if (id == null) return 0;
    final name = widget.dataProvider.name;
    final data = await widget.dataProvider.resolveData();
    await emulator.loadRom(name: name, data: data);
    emulator.runLoopForTexture(texture: texture!);
    return id;
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: kNesWidth,
      height: kNesHeight,
      child: FutureBuilder<int>(
          future: _future,
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.done) {
              if (snapshot.hasError) {
                return Text("${snapshot.error}");
              }
              return Texture(textureId: snapshot.data!);
            }
            return const CircularProgressIndicator();
          }),
    );
  }
}
