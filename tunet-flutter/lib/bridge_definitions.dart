// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.78.0.
// ignore_for_file: non_constant_identifier_names, unused_element, duplicate_ignore, directives_ordering, curly_braces_in_flow_control_structures, unnecessary_lambdas, slash_for_doc_comments, prefer_const_literals_to_create_immutables, implicit_dynamic_list_literal, duplicate_import, unused_import, unnecessary_import, prefer_single_quotes, prefer_const_constructors, use_super_parameters, always_use_package_imports, annotate_overrides, invalid_use_of_protected_member, constant_identifier_names, invalid_use_of_internal_member, prefer_is_empty, unnecessary_const

import 'dart:convert';
import 'dart:async';
import 'package:meta/meta.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:uuid/uuid.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
import 'package:collection/collection.dart';

part 'bridge_definitions.freezed.dart';

abstract class Native {
  Future<Runtime> newStaticMethodRuntime({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kNewStaticMethodRuntimeConstMeta;

  Stream<UpdateMsgWrap> startMethodRuntime(
      {required Runtime that,
      required RuntimeStartConfig config,
      dynamic hint});

  FlutterRustBridgeTaskConstMeta get kStartMethodRuntimeConstMeta;

  Future<NetStatus> currentStatusMethodRuntime(
      {required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kCurrentStatusMethodRuntimeConstMeta;

  Future<(String, String)> loadCredentialMethodRuntime(
      {required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kLoadCredentialMethodRuntimeConstMeta;

  Future<void> saveCredentialMethodRuntime(
      {required Runtime that,
      required String u,
      required String p,
      dynamic hint});

  FlutterRustBridgeTaskConstMeta get kSaveCredentialMethodRuntimeConstMeta;

  Future<void> queueCredentialMethodRuntime(
      {required Runtime that,
      required String u,
      required String p,
      dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueCredentialMethodRuntimeConstMeta;

  Future<void> queueLoginMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueLoginMethodRuntimeConstMeta;

  Future<void> queueLogoutMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueLogoutMethodRuntimeConstMeta;

  Future<void> queueFluxMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueFluxMethodRuntimeConstMeta;

  Future<void> queueStateMethodRuntime(
      {required Runtime that, NetStateWrap? s, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueStateMethodRuntimeConstMeta;

  Future<void> queueDetailsMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueDetailsMethodRuntimeConstMeta;

  Future<void> queueOnlinesMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueOnlinesMethodRuntimeConstMeta;

  Future<void> queueConnectMethodRuntime(
      {required Runtime that, required Ipv4AddrWrap ip, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueConnectMethodRuntimeConstMeta;

  Future<void> queueDropMethodRuntime(
      {required Runtime that, required List<Ipv4AddrWrap> ips, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kQueueDropMethodRuntimeConstMeta;

  Future<bool> logBusyMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kLogBusyMethodRuntimeConstMeta;

  Future<String> logTextMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kLogTextMethodRuntimeConstMeta;

  Future<NetFlux> fluxMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kFluxMethodRuntimeConstMeta;

  Future<NetStateWrap> stateMethodRuntime(
      {required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kStateMethodRuntimeConstMeta;

  Future<String> statusMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kStatusMethodRuntimeConstMeta;

  Future<bool> detailBusyMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kDetailBusyMethodRuntimeConstMeta;

  Future<List<NetDetail>> detailsMethodRuntime(
      {required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kDetailsMethodRuntimeConstMeta;

  Future<DetailDailyWrap?> detailDailyMethodRuntime(
      {required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kDetailDailyMethodRuntimeConstMeta;

  Future<String> usernameMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kUsernameMethodRuntimeConstMeta;

  Future<bool> onlineBusyMethodRuntime({required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kOnlineBusyMethodRuntimeConstMeta;

  Future<List<NetUserWrap>> onlinesMethodRuntime(
      {required Runtime that, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kOnlinesMethodRuntimeConstMeta;

  DropFnType get dropOpaqueMutexModel;
  ShareFnType get shareOpaqueMutexModel;
  OpaqueTypeFinalizer get MutexModelFinalizer;

  DropFnType get dropOpaqueMutexOptionHandle;
  ShareFnType get shareOpaqueMutexOptionHandle;
  OpaqueTypeFinalizer get MutexOptionHandleFinalizer;

  DropFnType get dropOpaqueMutexOptionMpscReceiverAction;
  ShareFnType get shareOpaqueMutexOptionMpscReceiverAction;
  OpaqueTypeFinalizer get MutexOptionMpscReceiverActionFinalizer;
}

@sealed
class MutexModel extends FrbOpaque {
  final Native bridge;
  MutexModel.fromRaw(int ptr, int size, this.bridge) : super.unsafe(ptr, size);
  @override
  DropFnType get dropFn => bridge.dropOpaqueMutexModel;

  @override
  ShareFnType get shareFn => bridge.shareOpaqueMutexModel;

  @override
  OpaqueTypeFinalizer get staticFinalizer => bridge.MutexModelFinalizer;
}

@sealed
class MutexOptionHandle extends FrbOpaque {
  final Native bridge;
  MutexOptionHandle.fromRaw(int ptr, int size, this.bridge)
      : super.unsafe(ptr, size);
  @override
  DropFnType get dropFn => bridge.dropOpaqueMutexOptionHandle;

  @override
  ShareFnType get shareFn => bridge.shareOpaqueMutexOptionHandle;

  @override
  OpaqueTypeFinalizer get staticFinalizer => bridge.MutexOptionHandleFinalizer;
}

@sealed
class MutexOptionMpscReceiverAction extends FrbOpaque {
  final Native bridge;
  MutexOptionMpscReceiverAction.fromRaw(int ptr, int size, this.bridge)
      : super.unsafe(ptr, size);
  @override
  DropFnType get dropFn => bridge.dropOpaqueMutexOptionMpscReceiverAction;

  @override
  ShareFnType get shareFn => bridge.shareOpaqueMutexOptionMpscReceiverAction;

  @override
  OpaqueTypeFinalizer get staticFinalizer =>
      bridge.MutexOptionMpscReceiverActionFinalizer;
}

class Balance {
  final double field0;

  const Balance({
    required this.field0,
  });
}

class DetailDailyPoint {
  final int day;
  final Flux flux;

  const DetailDailyPoint({
    required this.day,
    required this.flux,
  });
}

class DetailDailyWrap {
  final List<DetailDailyPoint> details;
  final int nowMonth;
  final int nowDay;
  final Flux maxFlux;

  const DetailDailyWrap({
    required this.details,
    required this.nowMonth,
    required this.nowDay,
    required this.maxFlux,
  });
}

class Flux {
  final int field0;

  const Flux({
    required this.field0,
  });
}

class Ipv4AddrWrap {
  final U8Array4 octets;

  const Ipv4AddrWrap({
    required this.octets,
  });
}

class NetDateTime {
  final DateTime field0;

  const NetDateTime({
    required this.field0,
  });
}

class NetDetail {
  final NetDateTime loginTime;
  final NetDateTime logoutTime;
  final Flux flux;

  const NetDetail({
    required this.loginTime,
    required this.logoutTime,
    required this.flux,
  });
}

class NetFlux {
  final String username;
  final Flux flux;
  final NewDuration onlineTime;
  final Balance balance;

  const NetFlux({
    required this.username,
    required this.flux,
    required this.onlineTime,
    required this.balance,
  });
}

enum NetState {
  Unknown,
  Net,
  Auth4,
  Auth6,
}

class NetStateWrap {
  final NetState field0;

  const NetStateWrap({
    required this.field0,
  });
}

@freezed
sealed class NetStatus with _$NetStatus {
  const factory NetStatus.unknown() = NetStatus_Unknown;
  const factory NetStatus.wwan() = NetStatus_Wwan;
  const factory NetStatus.wlan(
    String field0,
  ) = NetStatus_Wlan;
  const factory NetStatus.lan() = NetStatus_Lan;
}

class NetUserWrap {
  final Ipv4AddrWrap address;
  final NetDateTime loginTime;
  final String macAddress;
  final Flux flux;
  final bool isLocal;

  const NetUserWrap({
    required this.address,
    required this.loginTime,
    required this.macAddress,
    required this.flux,
    required this.isLocal,
  });
}

class NewDuration {
  final Duration field0;

  const NewDuration({
    required this.field0,
  });
}

class Runtime {
  final Native bridge;
  final MutexOptionMpscReceiverAction rx;
  final MutexModel model;
  final MutexOptionHandle handle;

  const Runtime({
    required this.bridge,
    required this.rx,
    required this.model,
    required this.handle,
  });

  static Future<Runtime> newRuntime({required Native bridge, dynamic hint}) =>
      bridge.newStaticMethodRuntime(hint: hint);

  Stream<UpdateMsgWrap> start(
          {required RuntimeStartConfig config, dynamic hint}) =>
      bridge.startMethodRuntime(
        that: this,
        config: config,
      );

  Future<NetStatus> currentStatus({dynamic hint}) =>
      bridge.currentStatusMethodRuntime(
        that: this,
      );

  Future<(String, String)> loadCredential({dynamic hint}) =>
      bridge.loadCredentialMethodRuntime(
        that: this,
      );

  Future<void> saveCredential(
          {required String u, required String p, dynamic hint}) =>
      bridge.saveCredentialMethodRuntime(
        that: this,
        u: u,
        p: p,
      );

  Future<void> queueCredential(
          {required String u, required String p, dynamic hint}) =>
      bridge.queueCredentialMethodRuntime(
        that: this,
        u: u,
        p: p,
      );

  Future<void> queueLogin({dynamic hint}) => bridge.queueLoginMethodRuntime(
        that: this,
      );

  Future<void> queueLogout({dynamic hint}) => bridge.queueLogoutMethodRuntime(
        that: this,
      );

  Future<void> queueFlux({dynamic hint}) => bridge.queueFluxMethodRuntime(
        that: this,
      );

  Future<void> queueState({NetStateWrap? s, dynamic hint}) =>
      bridge.queueStateMethodRuntime(
        that: this,
        s: s,
      );

  Future<void> queueDetails({dynamic hint}) => bridge.queueDetailsMethodRuntime(
        that: this,
      );

  Future<void> queueOnlines({dynamic hint}) => bridge.queueOnlinesMethodRuntime(
        that: this,
      );

  Future<void> queueConnect({required Ipv4AddrWrap ip, dynamic hint}) =>
      bridge.queueConnectMethodRuntime(
        that: this,
        ip: ip,
      );

  Future<void> queueDrop({required List<Ipv4AddrWrap> ips, dynamic hint}) =>
      bridge.queueDropMethodRuntime(
        that: this,
        ips: ips,
      );

  Future<bool> logBusy({dynamic hint}) => bridge.logBusyMethodRuntime(
        that: this,
      );

  Future<String> logText({dynamic hint}) => bridge.logTextMethodRuntime(
        that: this,
      );

  Future<NetFlux> flux({dynamic hint}) => bridge.fluxMethodRuntime(
        that: this,
      );

  Future<NetStateWrap> state({dynamic hint}) => bridge.stateMethodRuntime(
        that: this,
      );

  Future<String> status({dynamic hint}) => bridge.statusMethodRuntime(
        that: this,
      );

  Future<bool> detailBusy({dynamic hint}) => bridge.detailBusyMethodRuntime(
        that: this,
      );

  Future<List<NetDetail>> details({dynamic hint}) =>
      bridge.detailsMethodRuntime(
        that: this,
      );

  Future<DetailDailyWrap?> detailDaily({dynamic hint}) =>
      bridge.detailDailyMethodRuntime(
        that: this,
      );

  Future<String> username({dynamic hint}) => bridge.usernameMethodRuntime(
        that: this,
      );

  Future<bool> onlineBusy({dynamic hint}) => bridge.onlineBusyMethodRuntime(
        that: this,
      );

  Future<List<NetUserWrap>> onlines({dynamic hint}) =>
      bridge.onlinesMethodRuntime(
        that: this,
      );
}

class RuntimeStartConfig {
  final NetStatus status;
  final String username;
  final String password;

  const RuntimeStartConfig({
    required this.status,
    required this.username,
    required this.password,
  });
}

class U8Array4 extends NonGrowableListView<int> {
  static const arraySize = 4;
  U8Array4(Uint8List inner)
      : assert(inner.length == arraySize),
        super(inner);
  U8Array4.unchecked(Uint8List inner) : super(inner);
  U8Array4.init() : super(Uint8List(arraySize));
}

enum UpdateMsg {
  Credential,
  State,
  Status,
  Log,
  Flux,
  Online,
  Details,
  LogBusy,
  OnlineBusy,
  DetailBusy,
}

class UpdateMsgWrap {
  final UpdateMsg field0;

  const UpdateMsgWrap({
    required this.field0,
  });
}
