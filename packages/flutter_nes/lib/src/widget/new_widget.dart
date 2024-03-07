import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../native/api/nes.dart';
import 'nes_keyboard.dart';
import 'nes_painter.dart';
import 'nes_provider.dart';
import 'nes_texture.dart';

enum NesRenderType { painter, texture }

class NesWidget extends StatefulWidget {
  final NesRenderType renderType;
  final NesConfig? config;
  final NesKeyboardConfig? keyboardConfig;
  final NesDataProvider dataProvider;

  const NesWidget._({
    super.key,
    this.renderType = NesRenderType.painter,
    required this.dataProvider,
    this.keyboardConfig,
    this.config,
  });

  factory NesWidget.assets({
    Key? key,
    required String assets,
    String? name,
    NesConfig? config,
    NesKeyboardConfig? keyboardConfig,
    NesRenderType renderType = NesRenderType.painter,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        keyboardConfig: keyboardConfig,
        renderType: renderType,
        dataProvider: AssetsNesDataProvider(assets: assets, name: name),
      );

  factory NesWidget.network({
    Key? key,
    required String url,
    String? name,
    NesConfig? config,
    NesKeyboardConfig? keyboardConfig,
    NesRenderType renderType = NesRenderType.painter,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        keyboardConfig: keyboardConfig,
        renderType: renderType,
        dataProvider: NetworkNesDataProvider(url: url, name: name),
      );

  factory NesWidget.file({
    Key? key,
    required String path,
    String? name,
    NesConfig? config,
    NesKeyboardConfig? keyboardConfig,
    NesRenderType renderType = NesRenderType.painter,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        keyboardConfig: keyboardConfig,
        renderType: renderType,
        dataProvider: FileNesDataProvider(path: path, name: name),
      );

  factory NesWidget.memory({
    Key? key,
    required Uint8List data,
    String? name,
    NesConfig? config,
    NesKeyboardConfig? keyboardConfig,
    NesRenderType renderType = NesRenderType.painter,
  }) =>
      NesWidget._(
        key: key,
        config: config,
        keyboardConfig: keyboardConfig,
        renderType: renderType,
        dataProvider: MemoryNesDataProvider(data: data, name: name),
      );

  @override
  State<NesWidget> createState() => _NesWidgetState();
}

class _NesWidgetState extends State<NesWidget> {
  Future<void>? _future;
  late NesKeyboardConfig keyboardConfig;
  late final NesEmulator emulator = widget.config == null
      ? NesEmulator.create()
      : NesEmulator.withConfig(config: widget.config!);

  @override
  void initState() {
    _future = loadRom();
    keyboardConfig = widget.keyboardConfig ?? NesKeyboardConfig();
    super.initState();
  }

  Future<void> loadRom() async {
    final name = widget.dataProvider.name;
    final data = await widget.dataProvider.resolveData();
    await emulator.loadRom(name: name, data: data);
  }

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: FocusNode(),
      onKeyEvent: (keyEvent) {
        final tuple = keyboardConfig.findMatchedButton(keyEvent.logicalKey);
        if (tuple != null) {
          emulator.handleButton(
            player: tuple.$1,
            button: tuple.$2,
            pressed: keyEvent is KeyDownEvent,
          );
        }
      },
      child: SizedBox(
        width: kNesWidth,
        height: kNesHeight,
        child: FutureBuilder(
            future: _future,
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.done) {
                return widget.renderType == NesRenderType.painter
                    ? NesPainterWidget(emulator)
                    : NesTextureWidget(emulator);
              }
              return const CircularProgressIndicator();
            }),
      ),
    );
  }

  @override
  void dispose() {
    emulator.stopLoop();
    super.dispose();
  }
}
