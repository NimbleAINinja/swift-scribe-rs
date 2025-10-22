.PHONY: all helpers clean build release test

all: helpers build

helpers:
	@echo "Building Swift transcription helper..."
	@swiftc -O helpers/transcribe.swift -o helpers/transcribe
	@echo "✓ Helper built successfully"

build: helpers
	cargo build

release: helpers
	cargo build --release

clean:
	cargo clean
	rm -f helpers/transcribe

test: helpers
	cargo test

.DEFAULT_GOAL := all
