FLUTTER:=flutter

.PHONY: get fmt clean run apk

get:
	$(FLUTTER) pub get

fmt:
	cd native && cargo fmt
	dart format .

clean:
	$(FLUTTER) clean
	cd native && cargo clean

run:
	$(FLUTTER) run

apk: get
	$(FLUTTER) build apk --split-per-abi

ipa: get
	$(FLUTTER) build ios --release --no-codesign

.SECONDARY:
