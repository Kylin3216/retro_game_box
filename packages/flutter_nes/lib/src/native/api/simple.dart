// This file is automatically generated, so please do not edit it.
// Generated by `flutter_rust_bridge`@ 2.0.0-dev.25.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

int sum({required int a, required int b, dynamic hint}) =>
    RustLib.instance.api.sum(a: a, b: b, hint: hint);

Future<int> sumAsync({required int a, required int b, dynamic hint}) =>
    RustLib.instance.api.sumAsync(a: a, b: b, hint: hint);