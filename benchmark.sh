#!/bin/bash

# swift-scribe benchmark script for comparing SpeechAnalyzer vs Whisper API
# Usage: ./benchmark.sh <audio-file> [api-key]

set -e

AUDIO_FILE="$1"
API_KEY="${2:-$GROQ_API_KEY}"

if [ -z "$AUDIO_FILE" ]; then
    echo "Usage: $0 <audio-file> [api-key]"
    echo ""
    echo "Examples:"
    echo "  $0 audio.m4a"
    echo "  $0 audio.m4a gsk_your_api_key_here"
    echo "  GROQ_API_KEY=gsk_... $0 audio.m4a"
    echo ""
    echo "Get your API key from: https://console.groq.com/keys"
    exit 1
fi

if [ -z "$API_KEY" ]; then
    echo "Error: GROQ_API_KEY not set"
    echo ""
    echo "Set it via:"
    echo "  export GROQ_API_KEY=gsk_your_api_key_here"
    echo "  or"
    echo "  ./benchmark.sh audio.m4a gsk_your_api_key_here"
    echo ""
    echo "Get your API key from: https://console.groq.com/keys"
    exit 1
fi

if [ ! -f "$AUDIO_FILE" ]; then
    echo "Error: File not found: $AUDIO_FILE"
    exit 1
fi

# Build if needed
if [ ! -f "target/release/swift-scribe-bench" ]; then
    echo "Building benchmark tool..."
    cargo build --release --bin swift-scribe-bench --features="bench"
fi

# Run benchmark
./target/release/swift-scribe-bench "$AUDIO_FILE" --api-key "$API_KEY"
