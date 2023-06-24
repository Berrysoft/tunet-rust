.PHONY: get fmt clean run apk

get:
	flutter pub get

fmt:
	cd native && cargo fmt
	dart format .

clean:
	flutter clean
	cd native && cargo clean

run: native/src/bridge_generated.rs
	RUST_LOG=warn,tunet=trace,native=trace flutter run

lib/bridge_generated.dart lib/bridge_definitions.dart ios/Runner/bridge_generated.h native/src/bridge_generated.rs &: native/src/api.rs
	flutter_rust_bridge_codegen --rust-input $^ --dart-output lib/bridge_generated.dart --c-output ios/Runner/bridge_generated.h --extra-c-output-path macos/Runner/ --dart-decl-output lib/bridge_definitions.dart

apk: get
	flutter build apk --split-per-abi

.SECONDARY:
