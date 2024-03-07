import 'dart:collection';

import 'package:flutter/services.dart';
import '../native/api/nes.dart';
import '../native/api/mirrors.dart';

class NesKeyboardConfig {
  final NesKeyboardMapper playerOne;
  final NesKeyboardMapper playerTwo;
  final NesKeyboardMapper? playerThree;
  final NesKeyboardMapper? playerFour;

  NesKeyboardConfig._({
    required this.playerOne,
    required this.playerTwo,
    this.playerThree,
    this.playerFour,
  });

  factory NesKeyboardConfig({
    NesKeyboardMapper? playerOne,
    NesKeyboardMapper? playerTwo,
    NesKeyboardMapper? playerThree,
    NesKeyboardMapper? playerFour,
  }) =>
      NesKeyboardConfig._(
        playerOne: playerOne ?? NesKeyboardMapper.playerOne(),
        playerTwo: playerTwo ?? NesKeyboardMapper.playerTwo(),
        playerThree: playerThree,
        playerFour: playerFour,
      );

  (Player, NesButton)? findMatchedButton(KeyboardKey key) {
    var button = playerOne.findMatchedButton(key);
    if (button != null) return (Player.one, button);
    button = playerTwo.findMatchedButton(key);
    if (button != null) return (Player.two, button);
    button = playerThree?.findMatchedButton(key);
    if (button != null) return (Player.three, button);
    button = playerFour?.findMatchedButton(key);
    if (button != null) return (Player.four, button);
    return null;
  }
}

class NesKeyboardMapper {
  final Map<NesButton, KeyboardKey> mapper;

  NesKeyboardMapper._(this.mapper);

  factory NesKeyboardMapper.playerOne({
    KeyboardKey? start,
    KeyboardKey? select,
    KeyboardKey? turboA,
    KeyboardKey? turboB,
    KeyboardKey? a,
    KeyboardKey? b,
    KeyboardKey? up,
    KeyboardKey? down,
    KeyboardKey? left,
    KeyboardKey? right,
  }) =>
      NesKeyboardMapper._({
        NesButton.select: LogicalKeyboardKey.space,
        NesButton.start: LogicalKeyboardKey.enter,
        NesButton.turboA: LogicalKeyboardKey.keyH,
        NesButton.turboB: LogicalKeyboardKey.keyJ,
        NesButton.a: LogicalKeyboardKey.keyK,
        NesButton.b: LogicalKeyboardKey.keyL,
        NesButton.up: LogicalKeyboardKey.keyW,
        NesButton.down: LogicalKeyboardKey.keyS,
        NesButton.left: LogicalKeyboardKey.keyA,
        NesButton.right: LogicalKeyboardKey.keyD,
      });

  factory NesKeyboardMapper.playerTwo({
    KeyboardKey? start,
    KeyboardKey? select,
    KeyboardKey? turboA,
    KeyboardKey? turboB,
    KeyboardKey? a,
    KeyboardKey? b,
    KeyboardKey? up,
    KeyboardKey? down,
    KeyboardKey? left,
    KeyboardKey? right,
  }) =>
      NesKeyboardMapper._({
        NesButton.select: LogicalKeyboardKey.numpad0,
        NesButton.start: LogicalKeyboardKey.numpadEnter,
        NesButton.turboA: LogicalKeyboardKey.numpad4,
        NesButton.turboB: LogicalKeyboardKey.numpad5,
        NesButton.a: LogicalKeyboardKey.numpad7,
        NesButton.b: LogicalKeyboardKey.numpad8,
        NesButton.up: LogicalKeyboardKey.arrowUp,
        NesButton.down: LogicalKeyboardKey.arrowDown,
        NesButton.left: LogicalKeyboardKey.arrowLeft,
        NesButton.right: LogicalKeyboardKey.arrowRight,
      });

  NesButton? findMatchedButton(KeyboardKey key) {
    final entry =
        mapper.entries.where((element) => element.value == key).singleOrNull;
    return entry?.key;
  }
}
