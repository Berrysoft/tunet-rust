.PHONY: get fmt clean run apk

get:
	flutter pub get

fmt:
	cd native && cargo fmt
	dart format .

clean:
	flutter clean
	cd native && cargo clean

run:
	flutter run

apk: get
	flutter build apk --split-per-abi

.SECONDARY:
