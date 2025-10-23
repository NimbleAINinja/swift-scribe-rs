# SpeechAnalyzer API Reference

Complete reference for implementing Apple's SpeechAnalyzer API (macOS 26+) in swift-scribe-rs.

## Quick Reference

### Initialization Options

#### Using Presets (Recommended)

```swift
let transcriber = SpeechTranscriber(locale: Locale.current, preset: .transcription)
```

#### Custom Configuration

```swift
let transcriber = SpeechTranscriber(
    locale: Locale.current,
    transcriptionOptions: [.etiquetteReplacements],  // Optional censoring
    reportingOptions: [.volatileResults],            // Real-time updates
    attributeOptions: [.audioTimeRange]              // Timestamp metadata
)
```

## Available Presets

| Preset | Volatile | Alternatives | Timestamps | Best For |
|--------|----------|--------------|------------|----------|
| `.transcription` | No | X | No | **Basic transcription (default)** |
| `.transcriptionWithAlternatives` | No | Yes | No | Editing with suggestions |
| `.timeIndexedTranscriptionWithAlternatives` | No | Yes |  | Editing + audio sync |
| `.progressiveTranscription` | Yes | No | X | Real-time live transcription |
| `.timeIndexedProgressiveTranscription` | Yes | No | Yes | Live + time-codes |

**Our implementation uses:** `.transcription` for simplicity and accuracy.

## Module Types

### 1. SpeechTranscriber (Primary)

Modern, Neural Engine-accelerated transcription for normal conversation.

```swift
let transcriber = SpeechTranscriber(locale: locale, preset: .transcription)
```

**When to use:** macOS 26+, general purpose transcription, best performance.

### 2. DictationTranscriber (Compatibility)

Uses same models as legacy SFSpeechRecognizer API.

```swift
let dictation = DictationTranscriber(locale: locale, preset: .transcription)
```

**When to use:** Backward compatibility with older device models, same behavior as SFSpeechRecognizer.

### 3. SpeechDetector (Voice Activity Detection)

Detects presence of speech to save power.

```swift
let detector = SpeechDetector()
let analyzer = SpeechAnalyzer(modules: [transcriber, detector])
```

**When to use:** Battery-sensitive apps, filtering silence from recordings.

## Multiple Modules

You can combine modules:

```swift
let transcriber = SpeechTranscriber(locale: locale, preset: .transcription)
let detector = SpeechDetector()
let analyzer = SpeechAnalyzer(modules: [transcriber, detector])
```

## Result Object

### Properties

```swift
for try await result in transcriber.results {
    result.text              // AttributedString - most likely transcription
    result.isFinal           // Bool - true if finalized, false if volatile
    result.alternatives      // [AttributedString] - alternative interpretations
    result.range             // CMTimeRange - audio range this covers
    result.resultsFinalizationTime  // CMTime - time up to which results are final
}
```

### Converting to Plain Text

```swift
let plainText = String(result.text.characters)
```

**Note:** There is NO `.formattedString` property like in SFSpeechRecognizer.

### Volatile vs Final Results

**Volatile (`isFinal = false`):**
- Real-time guesses for responsive UI
- Fast but less accurate
- Same audio range may be updated multiple times
- Only available with `.progressiveTranscription` presets

**Final (`isFinal = true`):**
- Best guess - will NOT be updated
- That audio range is complete
- Save to permanent transcription

## Locale Support

### Check if Locale is Supported

```swift
let supportedLocales = await SpeechTranscriber.supportedLocales
let isSupported = supportedLocales.map { $0.identifier(.bcp47) }
    .contains(locale.identifier(.bcp47))
```

### Check if Model is Installed

```swift
let installedLocales = await SpeechTranscriber.installedLocales
let isInstalled = installedLocales.map { $0.identifier(.bcp47) }
    .contains(locale.identifier(.bcp47))
```

### Auto-Download Models

Models are downloaded automatically when analyzer starts if not installed. No explicit download needed, but you can inform the user:

```swift
if !isInstalled {
    print("Downloading speech model for \(locale.identifier)...")
}
```

## Audio Input

### From File (Our Use Case)

```swift
let audioFile = try AVAudioFile(forReading: audioURL)
try await analyzer.start(inputAudioFile: audioFile, finishAfterFile: true)
```

**Important:** Use `AVAudioFile`, not raw `URL`.

### Streaming Audio (Advanced)

```swift
let (inputSequence, inputBuilder) = AsyncStream<AnalyzerInput>.makeStream()
try await analyzer.start(inputSequence: inputSequence)

// Feed audio buffers
let input = AnalyzerInput(buffer: audioBuffer)
inputBuilder.yield(input)
```

## Completion Handling

### For Files

With `finishAfterFile: true`, the analyzer automatically finalizes when the file ends:

```swift
for try await result in transcriber.results {
    // Process results
}
// Loop ends when file is complete
```

### For Streaming

Call finalization explicitly:

```swift
try await analyzer.finalizeAndFinishThroughEndOfInput()
```

## Error Handling

### Common Errors

1. **Locale not supported**
   ```swift
   guard supportedLocales.contains(locale) else {
       throw TranscriptionError.localeNotSupported
   }
   ```

2. **Model not installed**
   - Auto-downloads on first use
   - May take time on first run

3. **Audio format incompatible**
   ```swift
   let format = await SpeechAnalyzer.bestAvailableAudioFormat(compatibleWith: [transcriber])
   ```

4. **File not found**
   - Check file exists before creating AVAudioFile

## Performance Characteristics

**SpeechAnalyzer vs Competitors:**
- **55% faster** than OpenAI Whisper Large V3 Turbo
- 34-minute video transcribed in **45 seconds** (vs 1m 41s for Whisper)
- Fully on-device (no network overhead)
- Neural Engine acceleration on Apple Silicon

## Best Practices

### For File Transcription (Our Use Case)

```swift
 Use .transcription preset
 Set finishAfterFile: true
 Only process isFinal results
 Check locale support before starting
X Don't use volatile results for files
X Don't manually call finalize (automatic with finishAfterFile)
```

### For Real-Time Transcription

```swift
 Use .progressiveTranscription preset
 Update UI with volatile results (lighter opacity)
 Replace volatile with final when available
 Use SpeechDetector to filter silence
 Call finalizeAndFinishThroughEndOfInput() when done
```

### For Editing Apps

```swift
 Use .timeIndexedTranscriptionWithAlternatives
 Store result.range for audio sync
 Present result.alternatives as suggestions
 Use audioTimeRange attributes for highlighting
```

## AttributedString Attributes

When using `attributeOptions: [.audioTimeRange]`:

```swift
for run in result.text.runs {
    if let timeRange = run.audioTimeRange {
        // timeRange is CMTimeRange
        let start = timeRange.start.seconds
        let end = timeRange.end.seconds
    }
}
```

## Enhancements for swift-scribe-rs

**Currently Implemented:**

- [x] **Real-time mode** for live transcription (microphone + stdin modes)
- [x] **Progressive transcription** using `.progressiveTranscription` preset
- [x] **Volatile and final results** handling in streaming API
- [x] **Automatic fallback** to SFSpeechRecognizer on older macOS

**Future Additions:**

- [ ] **Preset selection** via CLI flag (`--preset progressive`)
- [ ] **SRT output** using `attributeOptions: [.audioTimeRange]`
- [ ] **Alternative transcriptions** for editing use case
- [ ] **Multiple languages** via `--locale` flag
- [ ] **DictationTranscriber** fallback for older hardware
- [ ] **Voice Activity Detection** with SpeechDetector
- [ ] **Confidence scores** from result attributes

## Official Resources

- [SpeechAnalyzer Docs](https://developer.apple.com/documentation/speech/speechanalyzer)
- [SpeechTranscriber Docs](https://developer.apple.com/documentation/speech/speechtranscriber)
- [WWDC25 Session 277](https://developer.apple.com/videos/play/wwdc2025/277/) (11:47-12:30 for setup)
- [Sample Code](https://developer.apple.com/documentation/Speech/bringing-advanced-speech-to-text-capabilities-to-your-app)

## Notes on Our Implementation

**File Transcription:** Uses `.transcription` preset with file input, processing only final results.

**Why:**
- Simple and reliable for batch file transcription
- Optimal accuracy without real-time overhead
- No need to handle volatile updates

**Streaming Transcription:** Uses `.progressiveTranscription` preset with microphone or stdin input.

**Why:**
- Low-latency real-time feedback
- Provides both volatile and final results
- Suitable for live transcription applications

**Future:** Add `.timeIndexedTranscription*` presets to enable word-level timestamps and SRT export.
