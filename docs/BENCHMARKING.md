# Benchmarking Guide

Compare SpeechAnalyzer performance against Whisper API (via Groq).

## Quick Start

### 1. Get Groq API Key

Get a free API key from: https://console.groq.com/keys

### 2. Run Benchmark

```bash
# Using environment variable
export GROQ_API_KEY=gsk_your_api_key_here
./benchmark.sh audio.m4a

# Or pass directly
./benchmark.sh audio.m4a gsk_your_api_key_here
```

## Usage

### Simple Benchmark

```bash
cargo run --release --bin swift-scribe-bench -- audio.m4a --api-key $GROQ_API_KEY
```

### Multiple Runs for Averaging

```bash
cargo run --release --bin swift-scribe-bench -- audio.m4a -k $GROQ_API_KEY --runs 3
```

### JSON Output

```bash
cargo run --release --bin swift-scribe-bench -- audio.m4a -k $GROQ_API_KEY --json > results.json
```

### Different Whisper Model

```bash
# Available models: whisper-large-v3, whisper-large-v3-turbo
cargo run --release --bin swift-scribe-bench -- audio.m4a -k $GROQ_API_KEY --model whisper-large-v3
```

## Example Output

```
 Benchmarking Speech-to-Text Performance
═══════════════════════════════════════════
Audio file: test.m4a
File size:  2.34 MB
Runs:       1

   Testing local SpeechAnalyzer... 0.87s
   Testing Whisper API (whisper-large-v3-turbo)... 1.52s

 Results
═══════════════════════════════════════════

 Local SpeechAnalyzer
  Average time:  0.87s
  Output:        1234 chars

 Whisper API (whisper-large-v3-turbo)
  Average time:  1.52s
  Output:        1230 chars

 Comparison
  Speedup:       1.75x faster (local)
  Improvement:   74.7% faster with SpeechAnalyzer

Success:Both transcriptions match.
```

## JSON Output Format

```json
{
  "audio_file": "audio.m4a",
  "file_size_mb": 2.34,
  "local": {
    "duration_secs": 0.87,
    "text": "Transcription text...",
    "method": "SpeechAnalyzer"
  },
  "api": {
    "duration_secs": 1.52,
    "text": "Transcription text...",
    "model": "whisper-large-v3-turbo"
  },
  "speedup": 1.75
}
```

## Command-Line Options

```
Usage: swift-scribe-bench [OPTIONS] <FILE>

Arguments:
  <FILE>  Audio file to transcribe

Options:
  -a, --api-key <API_KEY>  Groq API key (or set GROQ_API_KEY env var)
  -j, --json               Output results as JSON
  -n, --runs <RUNS>        Number of runs for averaging (default: 1)
  -m, --model <MODEL>      Whisper model to use (default: whisper-large-v3-turbo)
  -h, --help               Print help
```

## Performance Tips

### For Accurate Benchmarks

1. **Warm up**: Run once to warm up caches before measuring
2. **Multiple runs**: Use `--runs 5` to average out variance
3. **Close apps**: Close other applications to reduce interference
4. **Same file**: Test multiple approaches on the same audio file
5. **Network**: API results include network latency - test with good connection

### Expected Results (macOS 26+)

Based on Apple's WWDC data and community testing:

| Audio Length | SpeechAnalyzer | Whisper (API) | Speedup |
|--------------|----------------|---------------|---------|
| 1 minute | ~0.5s | ~1.2s | 2.4x |
| 5 minutes | ~2.0s | ~4.5s | 2.25x |
| 34 minutes | ~45s | ~101s | 2.24x |

**Note:** Actual results vary based on:
- Audio quality and format
- macOS version (Neural Engine acceleration on 26+)
- Network latency for API
- CPU load and available resources

## Comparison Metrics

### Speed

- **SpeechAnalyzer**: On-device, Neural Engine accelerated (macOS 26+)
- **Whisper API**: Network request + cloud processing

### Privacy

- **SpeechAnalyzer**: 100% on-device, no data leaves your Mac
- **Whisper API**: Audio sent to Groq servers

### Cost

- **SpeechAnalyzer**: Free, unlimited
- **Whisper API**: Free tier available, see Groq pricing

### Accuracy

Both APIs provide high-quality transcription. Accuracy may vary slightly based on:
- Audio quality
- Accent/dialect
- Background noise
- Technical terminology

## Troubleshooting

### "Failed to run local transcriber"

Make sure the Swift helper is built:
```bash
make helpers
```

### "API request failed: 401"

Invalid API key. Check:
- Key is correct: `echo $GROQ_API_KEY`
- No extra spaces or quotes
- Key starts with `gsk_`

### "API request failed: 429"

Rate limit exceeded. Wait a moment and try again, or upgrade your Groq plan.

### Different transcription results

This is normal - different models may format output differently:
- Punctuation placement
- Capitalization
- Number formatting (e.g., "5" vs "five")

Both should capture the same spoken content.

## Batch Benchmarking

Test multiple files:

```bash
#!/bin/bash
export GROQ_API_KEY=gsk_your_key_here

for file in audio/*.m4a; do
    echo "Testing: $file"
    cargo run --release --bin swift-scribe-bench -- "$file" -k $GROQ_API_KEY --json >> results.jsonl
done
```

Analyze results:
```bash
jq '.speedup' results.jsonl | jq -s 'add/length'  # Average speedup
jq -s 'map(.speedup) | max' results.jsonl        # Best speedup
```

## Real-World Testing

For realistic benchmarks, use:
- **Varied audio sources**: podcasts, interviews, lectures
- **Different lengths**: 1min, 5min, 30min, 60min
- **Various quality**: studio vs phone recording
- **Multiple languages**: if testing international use cases

## Streaming vs File Transcription

**Note:** This benchmark tool measures file-to-text transcription speed.

**For streaming/real-time transcription:**
- Uses different API preset (`.progressiveTranscription`)
- Provides partial results as audio is received
- Latency measured differently (time-to-first-result)
- Throughput less relevant than responsiveness

To test streaming performance, use:
```bash
cargo run --example stream_mic  # Test microphone latency
```

## Contributing Benchmarks

Share your results by opening an issue with:
- macOS version
- Audio file length and size
- Average speedup over 5 runs
- File format and quality

This helps build community data on real-world performance!

## See Also

- [Apple's WWDC25 Session 277](https://developer.apple.com/videos/play/wwdc2025/277/) - Official performance claims
- [Groq API Docs](https://console.groq.com/docs/speech-text) - Whisper API reference
- [MacRumors Speed Test Article](https://www.macrumors.com/2025/06/18/apple-transcription-api-faster-than-whisper/) - 55% faster claim
