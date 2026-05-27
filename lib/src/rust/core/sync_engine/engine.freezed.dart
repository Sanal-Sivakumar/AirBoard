// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'engine.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$SyncEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String content) clipboardUpdated,
    required TResult Function(bool connected, String message) connectionStatus,
    required TResult Function(String message) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String content)? clipboardUpdated,
    TResult? Function(bool connected, String message)? connectionStatus,
    TResult? Function(String message)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String content)? clipboardUpdated,
    TResult Function(bool connected, String message)? connectionStatus,
    TResult Function(String message)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SyncEvent_ClipboardUpdated value)
        clipboardUpdated,
    required TResult Function(SyncEvent_ConnectionStatus value)
        connectionStatus,
    required TResult Function(SyncEvent_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult? Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult? Function(SyncEvent_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult Function(SyncEvent_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SyncEventCopyWith<$Res> {
  factory $SyncEventCopyWith(SyncEvent value, $Res Function(SyncEvent) then) =
      _$SyncEventCopyWithImpl<$Res, SyncEvent>;
}

/// @nodoc
class _$SyncEventCopyWithImpl<$Res, $Val extends SyncEvent>
    implements $SyncEventCopyWith<$Res> {
  _$SyncEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;
}

/// @nodoc
abstract class _$$SyncEvent_ClipboardUpdatedImplCopyWith<$Res> {
  factory _$$SyncEvent_ClipboardUpdatedImplCopyWith(
          _$SyncEvent_ClipboardUpdatedImpl value,
          $Res Function(_$SyncEvent_ClipboardUpdatedImpl) then) =
      __$$SyncEvent_ClipboardUpdatedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String content});
}

/// @nodoc
class __$$SyncEvent_ClipboardUpdatedImplCopyWithImpl<$Res>
    extends _$SyncEventCopyWithImpl<$Res, _$SyncEvent_ClipboardUpdatedImpl>
    implements _$$SyncEvent_ClipboardUpdatedImplCopyWith<$Res> {
  __$$SyncEvent_ClipboardUpdatedImplCopyWithImpl(
      _$SyncEvent_ClipboardUpdatedImpl _value,
      $Res Function(_$SyncEvent_ClipboardUpdatedImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? content = null,
  }) {
    return _then(_$SyncEvent_ClipboardUpdatedImpl(
      content: null == content
          ? _value.content
          : content // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SyncEvent_ClipboardUpdatedImpl extends SyncEvent_ClipboardUpdated {
  const _$SyncEvent_ClipboardUpdatedImpl({required this.content}) : super._();

  @override
  final String content;

  @override
  String toString() {
    return 'SyncEvent.clipboardUpdated(content: $content)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncEvent_ClipboardUpdatedImpl &&
            (identical(other.content, content) || other.content == content));
  }

  @override
  int get hashCode => Object.hash(runtimeType, content);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncEvent_ClipboardUpdatedImplCopyWith<_$SyncEvent_ClipboardUpdatedImpl>
      get copyWith => __$$SyncEvent_ClipboardUpdatedImplCopyWithImpl<
          _$SyncEvent_ClipboardUpdatedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String content) clipboardUpdated,
    required TResult Function(bool connected, String message) connectionStatus,
    required TResult Function(String message) error,
  }) {
    return clipboardUpdated(content);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String content)? clipboardUpdated,
    TResult? Function(bool connected, String message)? connectionStatus,
    TResult? Function(String message)? error,
  }) {
    return clipboardUpdated?.call(content);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String content)? clipboardUpdated,
    TResult Function(bool connected, String message)? connectionStatus,
    TResult Function(String message)? error,
    required TResult orElse(),
  }) {
    if (clipboardUpdated != null) {
      return clipboardUpdated(content);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SyncEvent_ClipboardUpdated value)
        clipboardUpdated,
    required TResult Function(SyncEvent_ConnectionStatus value)
        connectionStatus,
    required TResult Function(SyncEvent_Error value) error,
  }) {
    return clipboardUpdated(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult? Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult? Function(SyncEvent_Error value)? error,
  }) {
    return clipboardUpdated?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult Function(SyncEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (clipboardUpdated != null) {
      return clipboardUpdated(this);
    }
    return orElse();
  }
}

abstract class SyncEvent_ClipboardUpdated extends SyncEvent {
  const factory SyncEvent_ClipboardUpdated({required final String content}) =
      _$SyncEvent_ClipboardUpdatedImpl;
  const SyncEvent_ClipboardUpdated._() : super._();

  String get content;
  @JsonKey(ignore: true)
  _$$SyncEvent_ClipboardUpdatedImplCopyWith<_$SyncEvent_ClipboardUpdatedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SyncEvent_ConnectionStatusImplCopyWith<$Res> {
  factory _$$SyncEvent_ConnectionStatusImplCopyWith(
          _$SyncEvent_ConnectionStatusImpl value,
          $Res Function(_$SyncEvent_ConnectionStatusImpl) then) =
      __$$SyncEvent_ConnectionStatusImplCopyWithImpl<$Res>;
  @useResult
  $Res call({bool connected, String message});
}

/// @nodoc
class __$$SyncEvent_ConnectionStatusImplCopyWithImpl<$Res>
    extends _$SyncEventCopyWithImpl<$Res, _$SyncEvent_ConnectionStatusImpl>
    implements _$$SyncEvent_ConnectionStatusImplCopyWith<$Res> {
  __$$SyncEvent_ConnectionStatusImplCopyWithImpl(
      _$SyncEvent_ConnectionStatusImpl _value,
      $Res Function(_$SyncEvent_ConnectionStatusImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? connected = null,
    Object? message = null,
  }) {
    return _then(_$SyncEvent_ConnectionStatusImpl(
      connected: null == connected
          ? _value.connected
          : connected // ignore: cast_nullable_to_non_nullable
              as bool,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SyncEvent_ConnectionStatusImpl extends SyncEvent_ConnectionStatus {
  const _$SyncEvent_ConnectionStatusImpl(
      {required this.connected, required this.message})
      : super._();

  @override
  final bool connected;
  @override
  final String message;

  @override
  String toString() {
    return 'SyncEvent.connectionStatus(connected: $connected, message: $message)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncEvent_ConnectionStatusImpl &&
            (identical(other.connected, connected) ||
                other.connected == connected) &&
            (identical(other.message, message) || other.message == message));
  }

  @override
  int get hashCode => Object.hash(runtimeType, connected, message);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncEvent_ConnectionStatusImplCopyWith<_$SyncEvent_ConnectionStatusImpl>
      get copyWith => __$$SyncEvent_ConnectionStatusImplCopyWithImpl<
          _$SyncEvent_ConnectionStatusImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String content) clipboardUpdated,
    required TResult Function(bool connected, String message) connectionStatus,
    required TResult Function(String message) error,
  }) {
    return connectionStatus(connected, message);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String content)? clipboardUpdated,
    TResult? Function(bool connected, String message)? connectionStatus,
    TResult? Function(String message)? error,
  }) {
    return connectionStatus?.call(connected, message);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String content)? clipboardUpdated,
    TResult Function(bool connected, String message)? connectionStatus,
    TResult Function(String message)? error,
    required TResult orElse(),
  }) {
    if (connectionStatus != null) {
      return connectionStatus(connected, message);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SyncEvent_ClipboardUpdated value)
        clipboardUpdated,
    required TResult Function(SyncEvent_ConnectionStatus value)
        connectionStatus,
    required TResult Function(SyncEvent_Error value) error,
  }) {
    return connectionStatus(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult? Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult? Function(SyncEvent_Error value)? error,
  }) {
    return connectionStatus?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult Function(SyncEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (connectionStatus != null) {
      return connectionStatus(this);
    }
    return orElse();
  }
}

abstract class SyncEvent_ConnectionStatus extends SyncEvent {
  const factory SyncEvent_ConnectionStatus(
      {required final bool connected,
      required final String message}) = _$SyncEvent_ConnectionStatusImpl;
  const SyncEvent_ConnectionStatus._() : super._();

  bool get connected;
  String get message;
  @JsonKey(ignore: true)
  _$$SyncEvent_ConnectionStatusImplCopyWith<_$SyncEvent_ConnectionStatusImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$SyncEvent_ErrorImplCopyWith<$Res> {
  factory _$$SyncEvent_ErrorImplCopyWith(_$SyncEvent_ErrorImpl value,
          $Res Function(_$SyncEvent_ErrorImpl) then) =
      __$$SyncEvent_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String message});
}

/// @nodoc
class __$$SyncEvent_ErrorImplCopyWithImpl<$Res>
    extends _$SyncEventCopyWithImpl<$Res, _$SyncEvent_ErrorImpl>
    implements _$$SyncEvent_ErrorImplCopyWith<$Res> {
  __$$SyncEvent_ErrorImplCopyWithImpl(
      _$SyncEvent_ErrorImpl _value, $Res Function(_$SyncEvent_ErrorImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? message = null,
  }) {
    return _then(_$SyncEvent_ErrorImpl(
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$SyncEvent_ErrorImpl extends SyncEvent_Error {
  const _$SyncEvent_ErrorImpl({required this.message}) : super._();

  @override
  final String message;

  @override
  String toString() {
    return 'SyncEvent.error(message: $message)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SyncEvent_ErrorImpl &&
            (identical(other.message, message) || other.message == message));
  }

  @override
  int get hashCode => Object.hash(runtimeType, message);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$SyncEvent_ErrorImplCopyWith<_$SyncEvent_ErrorImpl> get copyWith =>
      __$$SyncEvent_ErrorImplCopyWithImpl<_$SyncEvent_ErrorImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String content) clipboardUpdated,
    required TResult Function(bool connected, String message) connectionStatus,
    required TResult Function(String message) error,
  }) {
    return error(message);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String content)? clipboardUpdated,
    TResult? Function(bool connected, String message)? connectionStatus,
    TResult? Function(String message)? error,
  }) {
    return error?.call(message);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String content)? clipboardUpdated,
    TResult Function(bool connected, String message)? connectionStatus,
    TResult Function(String message)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(message);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SyncEvent_ClipboardUpdated value)
        clipboardUpdated,
    required TResult Function(SyncEvent_ConnectionStatus value)
        connectionStatus,
    required TResult Function(SyncEvent_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult? Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult? Function(SyncEvent_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SyncEvent_ClipboardUpdated value)? clipboardUpdated,
    TResult Function(SyncEvent_ConnectionStatus value)? connectionStatus,
    TResult Function(SyncEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class SyncEvent_Error extends SyncEvent {
  const factory SyncEvent_Error({required final String message}) =
      _$SyncEvent_ErrorImpl;
  const SyncEvent_Error._() : super._();

  String get message;
  @JsonKey(ignore: true)
  _$$SyncEvent_ErrorImplCopyWith<_$SyncEvent_ErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
