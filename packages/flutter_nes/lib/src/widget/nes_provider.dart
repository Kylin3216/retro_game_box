import 'dart:io';

import 'package:dio/dio.dart';
import 'package:flutter/services.dart';
import 'package:flutter_nes/flutter_nes.dart';

const kNesWidth = 256.0;
const kNesHeight = 240.0;

extension NesConfigEx on NesConfig {
  static NesConfig create({
    VideoFilter filter = VideoFilter.ntsc,
    NesRegion region = NesRegion.ntsc,
    RamState ramState = RamState.allZeros,
    FourPlayer fourPlayer = FourPlayer.disabled,
    bool zapper = false,
    List<String> genieCodes = const [],
  }) {
    return NesConfig.create(
      filter: filter,
      region: region,
      ramState: ramState,
      fourPlayer: fourPlayer,
      zapper: zapper,
      genieCodes: genieCodes,
    );
  }
}

abstract class NesDataProvider {
  final String name;

  NesDataProvider(this.name);

  Future<Uint8List> resolveData();
}

class AssetsNesDataProvider extends NesDataProvider {
  final String assets;

  AssetsNesDataProvider({required this.assets, String? name})
      : super(name ?? assets);

  @override
  Future<Uint8List> resolveData() async {
    final data = await rootBundle.load(assets);
    return data.buffer.asUint8List();
  }
}

class NetworkNesDataProvider extends NesDataProvider {
  final String url;

  NetworkNesDataProvider({required this.url, String? name})
      : super(name ?? url.split("/").last);

  @override
  Future<Uint8List> resolveData() async {
    final res = await Dio().get(url);
    return res.data;
  }
}

class FileNesDataProvider extends NesDataProvider {
  final String path;

  FileNesDataProvider({required this.path, String? name})
      : super(name ?? path.split("/").last);

  @override
  Future<Uint8List> resolveData() {
    final file = File(path);
    return file.readAsBytes();
  }
}

class MemoryNesDataProvider extends NesDataProvider {
  final Uint8List data;

  MemoryNesDataProvider({required this.data, String? name})
      : super(name ?? "MemoryNes");

  @override
  Future<Uint8List> resolveData() async {
    return data;
  }
}
