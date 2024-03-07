import 'package:flutter/material.dart';
import 'package:irondash_engine_context/irondash_engine_context.dart';

import '../native/api/nes.dart';
import '../native/api/texture.dart';
import 'nes_provider.dart';

class NesTextureWidget extends StatefulWidget {
  final NesEmulator emulator;

  const NesTextureWidget(this.emulator, {super.key});

  @override
  State<NesTextureWidget> createState() => _NesTextureWidgetState();
}

class _NesTextureWidgetState extends State<NesTextureWidget> {
  int? id;

  @override
  void initState() {
    _runLoop();
    super.initState();
  }

  Future<void> _runLoop() async {
    final handle = await EngineContext.instance.getEngineHandle();
    final texture = await NesTexture.create(handle: handle);
    if (texture != null) {
      setState(() {
        if (mounted) {
          id = texture.id();
        }
      });
      widget.emulator.runLoopForTexture(texture: texture);
    }
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: kNesWidth,
      height: kNesHeight,
      child: id == null ? Container() : Texture(textureId: id!),
    );
  }
}
