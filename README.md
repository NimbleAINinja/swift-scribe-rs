# swift-scribe-rs

A Rust CLI tool and library for fast, on-device speech-to-text transcription using Apple's Speech framework.

## Features

- **Fast Neural Engine transcription** on macOS 26+ using SpeechAnalyzer API
- **Completely on-device** processing (no cloud or internet required)
- **Live microphone transcription** with progressive real-time results  
- **System audio tap support** via stdin for capturing system/application audio
- **Automatic API selection** with fallback to SFSpeechRecognizer on older macOS versions
- **Clean Rust library API** with Swift helper integration
- **Command-line interface** and library support
- Compatible with **macOS 10.15+** (Tahoe/macOS 26+ for latest features)

## Requirements

- macOS 10.15 or later (macOS 26/Tahoe for SpeechAnalyzer API)
- Rust toolchain (latest stable)
- Xcode Command Line Tools (for swiftc compiler)
- Speech recognition permissions (requested on first run)

## Quick Start

### CLI Tool

```bash
# Build the project
make

# Transcribe an audio file
cargo run --release -- /path/to/audio.m4a

# Benchmark against Whisper API (requires API key)
export GROQ_API_KEY=gsk_your_api_key_here
./benchmark.sh audio.m4a
```

### Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
swift-scribe-rs = { git = "https://github.com/NimbleAINinja/swift-scribe-rs" }
```

Install the Swift helper binary:

```bash
./install_helper.sh
```

Example code:

```rust
use swift_scribe::Transcriber;
use std::path::Path;

let transcriber = Transcriber::new()?;
let text = transcriber.transcribe_file(Path::new("audio.m4a"))?;
println!("{}", text);
```

See `LIBRARY_USAGE.md` for complete integration documentation.

## Installation

```bash
git clone https://github.com/NimbleAINinja/swift-scribe-rs.git
cd swift-scribe-rs
make
cargo install --path .
```

## Usage

### File Transcription

```bash
# Using cargo
cargo run --release -- input.m4a

# Using the binary
./target/release/swift-scribe input.m4a

# After installation
swift-scribe input.m4a
```

### Live Microphone Transcription

```bash
# Transcribe from microphone in real-time
cargo run --release -- --mic

# Or use the library API
cargo run --example stream_mic
```

### System Audio Capture (stdin mode)

The helper can accept audio from stdin for system audio tap integration:

```bash
# Using ffmpeg to capture and pipe audio
ffmpeg -f avfoundation -i ":1" -ar 16000 -ac 1 -f s16le - | \
  ./helpers/transcribe_stream --stdin

# Or from an audio file
ffmpeg -i audio.m4a -ar 16000 -ac 1 -f s16le - | \
  ./helpers/transcribe_stream --stdin
```

**Audio Format:** 16kHz, 16-bit, mono PCM (s16le)

For integration examples, see `examples/system_audio.rs` which demonstrates the pattern for:
- Capturing system audio using ScreenCaptureKit or similar
- Resampling to the required format
- Piping to the transcription helper
- Reading real-time transcription results

**Recommended libraries for system audio:**
- [ruhear](https://github.com/aizcutei/ruhear) - Simple cross-platform audio capture
- [screencapturekit-rs](https://github.com/doom-fish/screencapturekit-rs) - macOS ScreenCaptureKit bindings
- [cidre](https://github.com/yury/cidre) - Apple frameworks for Rust

## Architecture

This project uses a hybrid Rust/Swift architecture:

```
┌─────────────────────────────────────────┐
│  Rust CLI/Library                       │
│  - Public API (Transcriber)             │
│  - Argument parsing                     │
│  - File validation                      │
└──────────────┬──────────────────────────┘
               │ Process communication
┌──────────────▼──────────────────────────┐
│  Swift Helper (helpers/transcribe)      │
│  - SpeechAnalyzer API (macOS 26+)       │
│  - SFSpeechRecognizer (fallback)        │
│  - Async/await handling                 │
└─────────────────────────────────────────┘
```

### Design Rationale

The helper binary approach was chosen over alternatives for several reasons:

- **Simplicity**: No complex FFI bindings required
- **Maintainability**: Swift code can be updated independently
- **Performance**: Minimal subprocess overhead
- **Flexibility**: Easy to extend with new Speech framework features

Alternative approaches considered:
- `swift-bridge`: Complex build configuration and Swift Package Manager integration
- `objc2`: Manual Objective-C block implementations for async callbacks
- Direct FFI: Excessive boilerplate for async Swift APIs

## Project Structure

```
swift-scribe-rs/
├── src/
│   ├── main.rs              # CLI application
│   ├── lib.rs               # Library API
│   └── bench.rs             # Benchmarking tool
├── helpers/
│   └── transcribe.swift     # Swift helper implementation
├── examples/
│   ├── simple.rs            # Basic usage
│   └── batch.rs             # Batch processing
├── benchmark.sh             # Benchmarking script
├── install_helper.sh        # Helper installation
├── Makefile                 # Build configuration
└── Cargo.toml               # Package manifest
```

## Supported Audio Formats

- M4A (recommended)
- WAV
- MP3
- AAC
- FLAC
- AIFF

## API Implementation

### macOS 26+ (SpeechAnalyzer)

Modern API with Neural Engine acceleration:

```swift
let transcriber = SpeechTranscriber(
    locale: locale,
    transcriptionOptions: [],
    reportingOptions: [],
    attributeOptions: []
)
let analyzer = SpeechAnalyzer(modules: [transcriber])
let audioFile = try AVAudioFile(forReading: audioURL)
try await analyzer.start(inputAudioFile: audioFile, finishAfterFile: true)

for try await result in transcriber.results {
    if result.isFinal {
        transcription += String(result.text.characters)
    }
}
```

### macOS 10.15-25 (SFSpeechRecognizer)

Legacy API for compatibility:

```swift
let recognizer = SFSpeechRecognizer(locale: locale)
let request = SFSpeechURLRecognitionRequest(url: audioURL)
recognizer.recognitionTask(with: request) { result, error in
    // Handle transcription result
}
```

## Build Commands

```bash
# Full build
make

# Swift helper only
make helpers

# Rust components only
cargo build

# Release build
make release

# Clean build artifacts
make clean

# Run tests
make test
```

## Performance

### macOS 26+ (SpeechAnalyzer)

- Neural Engine hardware acceleration
- 55% faster than OpenAI Whisper (Apple WWDC benchmarks)
- Processes 34-minute video in 45 seconds (vs 101 seconds for Whisper)
- Real-time streaming support
- Completely on-device processing

### macOS 10.15-25 (SFSpeechRecognizer)

- On-device processing with no network requirements
- Supports all standard audio formats
- Proven API with broad compatibility

### Benchmarking

Compare against Whisper API:

```bash
export GROQ_API_KEY=gsk_your_api_key_here
./benchmark.sh audio.m4a
```

Or use the benchmark tool directly:

```bash
cargo run --release --bin swift-scribe-bench -- audio.m4a -k $GROQ_API_KEY
```

Detailed benchmarking documentation available in `BENCHMARKING.md`.

## Troubleshooting

### Helper Binary Not Found

```bash
make helpers
# Or manually
swiftc -O helpers/transcribe.swift -o helpers/transcribe
```

### Speech Recognition Permission Denied

Grant permissions in System Settings:
- Settings > Privacy & Security > Speech Recognition

### Unsupported Audio Format

Verify the file format is supported. Use `ffprobe` to check:

```bash
ffprobe audio_file.ext
```

## Future Enhancements

Potential features based on SpeechAnalyzer capabilities:

- Preset selection for different use cases
- Multi-language support with locale selection
- Real-time transcription mode
- SRT subtitle export with timestamps
- Alternative transcription suggestions
- Microphone input streaming
- Voice activity detection
- DictationTranscriber fallback for older hardware
- Confidence scores in output
- JSON export with metadata

See `SPEECHANALYZER_API_REFERENCE.md` for API capabilities.

## Contributing

Contributions are welcome. Please submit issues for bugs or feature requests, and pull requests for improvements.

## License

MIT

## References

- [Apple SpeechAnalyzer Documentation](https://developer.apple.com/documentation/speech/speechanalyzer)
- [Speech Framework Overview](https://developer.apple.com/documentation/Speech)
- [WWDC25 Session 277: SpeechAnalyzer](https://developer.apple.com/videos/play/wwdc2025/277/)
