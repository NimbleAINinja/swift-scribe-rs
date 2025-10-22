# Development Notes

## Project Goal

Create a Rust CLI tool and library for speech-to-text transcription using Apple's Speech framework, with support for the new SpeechAnalyzer API (macOS 26+).

## Implementation Approach

### Chosen: Swift Helper + Rust Wrapper

We implemented a hybrid architecture:
- **Rust**: CLI, argument parsing, file validation, user interface
- **Swift Helper**: Compiled Swift binary that handles Speech framework APIs
- **Communication**: Simple process execution with stdin/stdout

```
Rust CLI → executes → Swift Helper → returns → transcription text
```

### Why This Approach?

1. **Simplicity**: No complex FFI bindings needed
2. **Maintainability**: Swift code can be updated independently  
3. **Works Today**: Uses proven SFSpeechRecognizer API (macOS 10.15+)
4. **Future-Ready**: Easy to add SpeechAnalyzer support later

## Alternative Approaches Evaluated

### 1. swift-bridge

**Pros:**
- Type-safe FFI
- Bidirectional Rust ↔ Swift calls
- Automatic code generation

**Cons:**
- Complex build system integration required
- Needs Swift Package Manager or xcodeproj setup
- Difficult to compile Swift code in Cargo build script
- More moving parts to maintain

**Verdict**: Too complex for this use case.

### 2. objc2 with Manual Bindings

**Pros:**
- Direct access to Objective-C APIs
- Fine-grained control
- No external dependencies

**Cons:**
- SpeechAnalyzer not yet in objc2-speech bindings
- Requires implementing Objective-C blocks in Rust
- Lots of unsafe code
- Async callback handling is tricky

**Verdict**: Too much boilerplate for async APIs.

### 3. Direct C FFI

**Pros:**
- Maximum control
- No framework dependencies

**Cons:**
- Extremely verbose
- Manual memory management
- Callback hell with async Swift
- Not worth the effort

**Verdict**: Overkill and error-prone.

## Current Status

 **Working**:
- Rust CLI compiles and runs
- Swift helper compiles with swiftc
-  **SpeechAnalyzer API implementation (macOS 26+)** with automatic fallback
- SFSpeechRecognizer API (macOS 10.15+) as fallback
- Runtime version detection and automatic API selection
- Handles audio file transcription
- Error handling and user feedback
- Makefile for build orchestration

⏳ **Future Work**:
- Support multiple languages/locales (via CLI flag)
- JSON output format with metadata
- Word-level timestamps
- Confidence scores
- Streaming audio input
- Asset management for model downloads

## Building

```bash
# Build everything
make

# Just Swift helper
make helpers

# Just Rust
cargo build

# Release build
make release

# Clean
make clean
```

## File Structure

```
swift-scribe-rs/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library API
│   └── bench.rs             # Benchmarking tool
├── helpers/
│   ├── transcribe.swift     # Swift source
│   └── transcribe           # Compiled Swift binary (gitignored)
├── examples/
│   ├── simple.rs            # Basic usage example
│   └── batch.rs             # Batch processing example
├── Cargo.toml               # Dependencies
├── Makefile                 # Build orchestration
├── install_helper.sh        # Helper installation
└── README.md                # User documentation
```

## SpeechAnalyzer Implementation ( Complete)

Successfully implemented by studying working Swift code from the `yap` CLI tool. Key learnings:

### Correct API Usage

```swift
// Initialize transcriber with options
let transcriber = SpeechTranscriber(
    locale: locale,
    transcriptionOptions: [],  // Can add .etiquetteReplacements for censoring
    reportingOptions: [],
    attributeOptions: []       // Can add .audioTimeRange for timestamps
)

// Create analyzer with modules
let modules: [any SpeechModule] = [transcriber]
let analyzer = SpeechAnalyzer(modules: modules)

// Use AVAudioFile, not URL directly
let audioFile = try AVAudioFile(forReading: audioURL)

// Start analysis (not addAudioFile!)
try await analyzer.start(inputAudioFile: audioFile, finishAfterFile: true)

// Stream results
for try await result in transcriber.results {
    // result.text is AttributedString
    transcription += String(result.text.characters)
}
```

### What We Learned

1. **Don't use URLs directly** - SpeechAnalyzer needs `AVAudioFile`
2. **Use `analyzer.start()`** - not `addAudioFile()` + `endAudio()`
3. **Results are AttributedString** - Convert with `String(result.text.characters)`
4. **Options are separate** - transcriptionOptions, reportingOptions, attributeOptions
5. **Modules are protocols** - Use `[any SpeechModule]` type

### Automatic Fallback

The helper detects macOS version at runtime:
```swift
if #available(macOS 26.0, *) {
    transcription = try await transcribeWithSpeechAnalyzer(audioURL: audioURL)
} else {
    transcription = try await transcribeWithLegacyAPI(audioURL: audioURL)
}
```

No changes needed in Rust code - it's API-agnostic! 

## Lessons Learned

1. **FFI Complexity**: Rust ↔ Swift FFI is harder than it looks, especially with async
2. **Process Communication**: Sometimes the simplest solution is best
3. **Build Systems**: Mixing Rust and Swift builds is tricky
4. **API Maturity**: New APIs (SpeechAnalyzer) lack tooling and examples
5. **Pragmatism Wins**: Working code > elegant architecture

## References

- [WWDC25: SpeechAnalyzer](https://developer.apple.com/videos/play/wwdc2025/277/)
- [swift-bridge GitHub](https://github.com/chinedufn/swift-bridge)
- [objc2 Documentation](https://docs.rs/objc2)
- [Speech Framework Docs](https://developer.apple.com/documentation/Speech)
