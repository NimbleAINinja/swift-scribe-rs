.PHONY: all helpers clean build release test

all: helpers build

helpers:
	@echo "Building Swift transcription helpers..."
	@swiftc -O helpers/transcribe.swift -o helpers/transcribe
	@echo "✓ File transcription helper built"
	@swiftc -O helpers/transcribe_stream.swift -o helpers/transcribe_stream
	@echo "✓ Streaming transcription helper built"

build: helpers
	cargo build

release: helpers
	cargo build --release

clean:
	cargo clean
	rm -f helpers/transcribe helpers/transcribe_stream

test: helpers
	cargo test

.DEFAULT_GOAL := all
