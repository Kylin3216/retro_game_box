// This file is automatically generated, so please do not edit it.
// Generated by `flutter_rust_bridge`@ 2.0.0-dev.26.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'mirrors.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'texture.dart';

// Rust type: RustOpaqueMoi<flutter_rust_bridge::for_generated::rust_async::RwLock<NesEmulator>>
@sealed
class NesEmulator extends RustOpaque {
  NesEmulator.dcoDecode(List<dynamic> wire)
      : super.dcoDecode(wire, _kStaticData);

  NesEmulator.sseDecode(int ptr, int externalSizeOnNative)
      : super.sseDecode(ptr, externalSizeOnNative, _kStaticData);

  static final _kStaticData = RustArcStaticData(
    rustArcIncrementStrongCount:
        RustLib.instance.api.rust_arc_increment_strong_count_NesEmulator,
    rustArcDecrementStrongCount:
        RustLib.instance.api.rust_arc_decrement_strong_count_NesEmulator,
    rustArcDecrementStrongCountPtr:
        RustLib.instance.api.rust_arc_decrement_strong_count_NesEmulatorPtr,
  );

  static NesEmulator create({dynamic hint}) =>
      RustLib.instance.api.nesEmulatorCreate(hint: hint);

  Future<void> handleButton(
          {required Player player,
          required NesButton button,
          required bool pressed,
          dynamic hint}) =>
      RustLib.instance.api.nesEmulatorHandleButton(
        that: this,
        player: player,
        button: button,
        pressed: pressed,
      );

  Future<void> loadRom(
          {required String name, required List<int> data, dynamic hint}) =>
      RustLib.instance.api.nesEmulatorLoadRom(
        that: this,
        name: name,
        data: data,
      );

  Future<void> runLoopForCallback(
          {required FutureOr<void> Function(Uint8List) callback,
          dynamic hint}) =>
      RustLib.instance.api.nesEmulatorRunLoopForCallback(
        that: this,
        callback: callback,
      );

  Stream<Uint8List> runLoopForPainter({dynamic hint}) =>
      RustLib.instance.api.nesEmulatorRunLoopForPainter(
        that: this,
      );

  Future<void> runLoopForTexture({required NesTexture texture, dynamic hint}) =>
      RustLib.instance.api.nesEmulatorRunLoopForTexture(
        that: this,
        texture: texture,
      );

  void stopLoop({dynamic hint}) => RustLib.instance.api.nesEmulatorStopLoop(
        that: this,
      );

  static NesEmulator withConfig({required NesConfig config, dynamic hint}) =>
      RustLib.instance.api.nesEmulatorWithConfig(config: config, hint: hint);
}

enum NesButton {
  start,
  select,
  turboA,
  turboB,
  a,
  b,
  up,
  down,
  left,
  right,
}

class NesConfig {
  final VideoFilter filter;
  final NesRegion region;
  final RamState ramState;
  final FourPlayer fourPlayer;
  final bool zapper;
  final List<String> genieCodes;

  const NesConfig({
    required this.filter,
    required this.region,
    required this.ramState,
    required this.fourPlayer,
    required this.zapper,
    required this.genieCodes,
  });

  static NesConfig create(
          {required VideoFilter filter,
          required NesRegion region,
          required RamState ramState,
          required FourPlayer fourPlayer,
          required bool zapper,
          required List<String> genieCodes,
          dynamic hint}) =>
      RustLib.instance.api.nesConfigCreate(
          filter: filter,
          region: region,
          ramState: ramState,
          fourPlayer: fourPlayer,
          zapper: zapper,
          genieCodes: genieCodes,
          hint: hint);

  @override
  int get hashCode =>
      filter.hashCode ^
      region.hashCode ^
      ramState.hashCode ^
      fourPlayer.hashCode ^
      zapper.hashCode ^
      genieCodes.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is NesConfig &&
          runtimeType == other.runtimeType &&
          filter == other.filter &&
          region == other.region &&
          ramState == other.ramState &&
          fourPlayer == other.fourPlayer &&
          zapper == other.zapper &&
          genieCodes == other.genieCodes;
}
