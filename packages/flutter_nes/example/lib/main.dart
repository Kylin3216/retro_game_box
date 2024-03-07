import 'package:flutter/material.dart';
import 'package:flutter_nes/flutter_nes.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('Base Data Format Is rgba')),
        body: SingleChildScrollView(
          child: Column(
            children: [
              Text("Texture Render"),
              NesWidget.assets(
                assets: "assets/SuperMario.nes",
                config: NesConfigEx.create(
                  filter: VideoFilter.pixellate,
                ),
                renderType: NesRenderType.texture,
              ),
              Divider(),
              Text("Decode Using decodeImageFromPixels Then paint"),
              NesWidget.assets(
                assets: "assets/SuperMario.nes",
              ),
            ],
          ),
        ),
      ),
    );
  }
}
