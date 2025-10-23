# Programmatic Audio Input API Guide

## Overview

The `swift-scribe-rs` library now supports **programmatic audio input**, enabling you to feed audio samples from any source (system audio, network streams, files, custom processing pipelines, etc.) directly to the transcriber.

### What This Enables

- **System Audio Transcription**: Monitor and transcribe speaker output
- **Network Audio Streaming**: Transcribe incoming audio from network sources
- **Custom Audio Processing**: Apply effects, mixing, or filters before transcription
- **Multi-Source Mixing**: Combine multiple audio sources
- **Integration with Audio Libraries**: Work with `cpal`, `coreaudio`, `rodio`, etc.

### Previous Workaround vs New API

**Before (Subprocess + stdin):**
```rust
let mut child = Command::new("transcribe_stream")
    .arg("--stdin")
    .stdin(Stdio::piped())
    .spawn()?;

let mut stdin = child.stdin.take().unwrap();
let bytes = convert_to_i16_le_bytes(&samples);
stdin.write_all(&bytes)?;
```

**After (Direct API):**
```rust
let mut transcriber = StreamingTranscriber::builder()
    .with_programmatic_input()
    .build()?;

transcriber.start()?;
transcriber.feed_audio_f32(&samples, 48000, 2)?;
```

---

## API Design

### Builder Pattern

Use the builder pattern to configure the transcriber:

```rust
use swift_scribe::StreamingTranscriber;

// Default: microphone input
let transcriber = StreamingTranscriber::new()?;

// Programmatic input
let transcriber = StreamingTranscriber::builder()
    .with_programmatic_input()
    .build()?;

// Custom helper path
let transcriber = StreamingTranscriber::builder()
    .with_programmatic_input()
    .with_helper_path("/custom/path/transcribe_stream")
    .build()?;
```

### Core Methods

#### `builder() -> StreamingTranscriberBuilder`
Creates a new builder for flexible configuration.

#### `with_microphone() -> StreamingTranscriberBuilder`
Configures the transcriber for microphone input (default).

#### `with_programmatic_input() -> StreamingTranscriberBuilder`
Configures the transcriber for programmatic audio input.

#### `feed_audio_i16(&mut self, samples: &[i16], sample_rate: u32, channels: u16) -> Result<(), String>`

Feeds i16 PCM audio samples to the transcriber.

**Parameters:**
- `samples`: Raw i16 PCM samples
- `sample_rate`: Sample rate in Hz (e.g., 16000, 48000)
- `channels`: Number of audio channels (1=mono, 2=stereo)

**Automatic Processing:**
- Resamples to 16kHz (optimal for Speech framework)
- Converts to mono
- Byte order: Little-endian

**Example:**
```rust
let samples = vec![0i16; 4096];
transcriber.feed_audio_i16(&samples, 48000, 2)?;
```

#### `feed_audio_f32(&mut self, samples: &[f32], sample_rate: u32, channels: u16) -> Result<(), String>`

Feeds f32 audio samples to the transcriber.

**Parameters:**
- `samples`: f32 samples in range [-1.0, 1.0]
- `sample_rate`: Sample rate in Hz
- `channels`: Number of channels

**Automatic Processing:**
- Converts f32 to i16 PCM
- Resamples to 16kHz
- Converts to mono

**Example:**
```rust
let samples = vec![0.0f32; 4096];
transcriber.feed_audio_f32(&samples, 48000, 2)?;
```

---

## Usage Examples

### Example 1: Basic Programmatic Input

```rust
use swift_scribe::StreamingTranscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transcriber
    let mut transcriber = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build()?;

    transcriber.start()?;

    // Feed audio
    let samples = vec![0.0f32; 4096];
    transcriber.feed_audio_f32(&samples, 48000, 2)?;

    // Get results
    if let Some(result) = transcriber.poll_result()? {
        println!("Transcription: {}", result.text);
    }

    transcriber.stop()?;
    Ok(())
}
```

### Example 2: System Audio Transcription

```rust
use swift_scribe::StreamingTranscriber;
use std::sync::mpsc::{channel, Sender};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = channel::<Vec<f32>>();

    // Spawn system audio capture thread
    thread::spawn(move || {
        capture_system_audio(tx);
    });

    // Create transcriber
    let mut transcriber = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build()?;

    transcriber.start()?;

    // Main loop: receive audio, transcribe, process results
    loop {
        if let Ok(audio_chunk) = rx.recv_timeout(Duration::from_millis(100)) {
            // Feed audio (auto-converts format, resamples, converts to mono)
            transcriber.feed_audio_f32(&audio_chunk, 48000, 2)?;

            // Poll for transcription results
            while let Some(result) = transcriber.poll_result()? {
                if result.is_final {
                    println!("✓ {}", result.text);
                    // Process transcription...
                }
            }
        }
    }
}

fn capture_system_audio(tx: Sender<Vec<f32>>) {
    // Use coreaudio, cpal, or other audio capture library
    // Send f32 samples at any sample rate and channel count
}
```

### Example 3: Streaming from File

```rust
use swift_scribe::StreamingTranscriber;

fn transcribe_file_with_preprocessing(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Load audio file and decode to f32 samples
    let (audio_data, sample_rate, channels) = load_audio_file(path)?;

    let mut transcriber = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build()?;

    transcriber.start()?;

    // Feed in chunks
    let chunk_size = 4096;
    for chunk in audio_data.chunks(chunk_size) {
        transcriber.feed_audio_f32(chunk, sample_rate, channels)?;

        // Process results
        while let Some(result) = transcriber.poll_result()? {
            println!("{}", result.text);
        }
    }

    transcriber.stop()?;
    Ok(())
}
```

### Example 4: Real-Time Processing with Threading

```rust
use swift_scribe::StreamingTranscriber;
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();
    let (result_tx, result_rx) = mpsc::channel::<String>();

    // Audio capture thread
    thread::spawn(move || {
        loop {
            let audio = capture_audio_chunk();
            if audio_tx.send(audio).is_err() {
                break;
            }
        }
    });

    // Transcription thread
    thread::spawn(move || {
        let mut transcriber = StreamingTranscriber::builder()
            .with_programmatic_input()
            .build()
            .unwrap();

        transcriber.start().unwrap();

        while let Ok(audio_chunk) = audio_rx.recv() {
            let _ = transcriber.feed_audio_f32(&audio_chunk, 48000, 2);

            while let Ok(Some(result)) = transcriber.poll_result() {
                if result.is_final {
                    let _ = result_tx.send(result.text);
                }
            }
        }

        let _ = transcriber.stop();
    });

    // Result processing thread
    while let Ok(text) = result_rx.recv() {
        println!("Result: {}", text);
        // Process transcription result...
    }

    Ok(())
}

fn capture_audio_chunk() -> Vec<f32> {
    vec![0.0f32; 4096]
}
```

---

## Audio Format Details

### Input Formats

#### i16 PCM (16-bit signed integer)
- **Range**: -32768 to 32767
- **Usage**: Common in audio processing and hardware interfaces
- **Byte Order**: Little-endian

#### f32 Floating Point
- **Range**: -1.0 to 1.0 (clamped if outside range)
- **Usage**: Common in modern audio processing libraries
- **Advantage**: More intuitive for audio processing

### Automatic Conversions

The transcriber automatically handles format conversions:

| Input | Output | Process |
|-------|--------|---------|
| f32 (-1.0 to 1.0) | i16 | `(sample * 32767).clamp(-32768, 32767) as i16` |
| i16 | i16 | No conversion |
| Any rate | 16kHz | Linear interpolation resampling |
| Stereo (2ch) | Mono | Channel averaging |
| Multi-channel (N>2) | Mono | Channel averaging |

### Recommended Settings

| Parameter | Recommended | Notes |
|-----------|------------|-------|
| Sample Rate | 48000 Hz | Common for system audio; will be resampled to 16kHz |
| Channels | 2 (stereo) | Auto-converted to mono internally |
| Sample Format | f32 | More intuitive; easy to work with audio libraries |
| Chunk Size | 4096 samples | ~85ms at 48kHz; good balance between latency and buffering |

---

## Performance Considerations

### Buffering
- Small chunks (4096 samples) provide low latency
- Larger chunks (16384+) may improve throughput
- Balance based on your use case

### Resampling
- Linear interpolation is fast and efficient
- Occurs for any input rate ≠ 16kHz
- Minimal CPU overhead

### Format Conversion
- f32 → i16: Fast vectorizable operation
- Stereo → Mono: Simple averaging, negligible overhead

### Streaming Results
- `poll_result()` is non-blocking
- Returns `Ok(None)` if no result available yet
- Returns `Err(_)` if process has ended
- Check `result.is_final` to distinguish interim vs final results

---

## Architecture

### How It Works

```
Your Audio Source
      ↓
   (f32 or i16 samples)
      ↓
StreamingTranscriber::feed_audio_*()
      ↓
  Format Conversion
  • f32 → i16
  • Resample to 16kHz
  • Stereo → Mono
      ↓
helper process stdin
      ↓
transcribe_stream (--stdin mode)
      ↓
SpeechAnalyzer (macOS 26+) or
SFSpeechRecognizer (macOS 10.15+)
      ↓
  Transcription Results (JSON)
      ↓
stdout → StreamingTranscriber
      ↓
poll_result() → StreamingResult
```

### Helper Process Integration

When using programmatic input:
1. Transcriber spawns helper with `--stdin` flag
2. Helper opens stdin for reading PCM audio
3. Expected format: 16kHz, 16-bit, mono PCM
4. Helper processes audio through Apple's Speech framework
5. Results sent as JSON to stdout

The library handles all format conversions, so you only need to provide audio in f32 or i16 format.

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Streaming helper binary not found" | Helper not compiled or not in path | Run `make helpers` |
| "feed_audio_i16 can only be used with programmatic input mode" | Called on microphone mode | Use `with_programmatic_input()` |
| "Transcriber not started" | Called methods before `start()` | Call `transcriber.start()?` first |
| "Failed to write audio to helper" | Helper process crashed | Check stderr for details |
| "Streaming process ended" | Helper process terminated | Likely due to error; check logs |

---

## Testing

Run the included examples:

```bash
# Programmatic input example
cargo run --example programmatic_audio

# System audio transcription example
cargo run --example system_audio_transcription
```

Run tests:

```bash
# API tests
cargo test --test api_tests

# All tests including doc tests
cargo test
```

---

## FAQ

**Q: Can I use microphone input and programmatic input simultaneously?**
A: No. Use one transcriber per input mode. Create separate instances if needed.

**Q: What happens to audio rate mismatches?**
A: Automatic resampling to 16kHz. Linear interpolation provides good quality for most purposes.

**Q: Can I mix channels differently (e.g., take only left channel)?**
A: Not directly in the API. Pre-process audio before feeding to the transcriber.

**Q: What's the maximum audio size I should feed at once?**
A: No strict limit. Feed in chunks that match your hardware latency requirements (typically 4-16KB chunks).

**Q: Can I change sample rate between chunks?**
A: Yes! Each `feed_audio_*()` call specifies the sample rate. Mix rates freely.

**Q: Does the library perform any compression/encoding?**
A: No. Audio is kept as PCM throughout. Only resampling and channel conversion applied.

**Q: How do I monitor transcription quality?**
A: Check `result.confidence` if available. The library may add confidence scores in future versions.

---

## Integration Steps

1. Capture system audio with your preferred library
2. Convert to f32 samples (-1.0 to 1.0 range)
3. Create transcriber: StreamingTranscriber::builder()
   .with_programmatic_input()
   .build()?
4. Start: transcriber.start()?
5. Feed audio: transcriber.feed_audio_f32(&samples, 48000, 2)?
6. Poll results: transcriber.poll_result()?
7. Process transcription results

---

## Contributing

Found a bug or have a feature request? Please open an issue on GitHub:
https://github.com/NimbleAINinja/swift-scribe-rs

---

## License

MIT - See LICENSE file for details
