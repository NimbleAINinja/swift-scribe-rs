import AVFoundation
import Foundation
import Speech

// JSON output format for streaming results
struct TranscriptionOutput: Codable {
    let text: String
    let isFinal: Bool
    let timestamp: Double
}

// Modern SpeechAnalyzer API with microphone input (macOS 26+)
@available(macOS 26.0, *)
class StreamingTranscriber {
    private let locale: Locale
    private let audioEngine: AVAudioEngine
    private var analyzer: SpeechAnalyzer?
    private var transcriber: SpeechTranscriber?
    private var inputBuilder: AsyncStream<AnalyzerInput>.Continuation?
    
    init(locale: Locale = Locale(identifier: "en-US")) {
        self.locale = locale
        self.audioEngine = AVAudioEngine()
    }
    
    func start() async throws {
        // Verify locale is supported
        let supportedLocales = await SpeechTranscriber.supportedLocales
        guard supportedLocales.map({ $0.identifier(.bcp47) }).contains(locale.identifier(.bcp47)) else {
            throw NSError(
                domain: "SpeechRecognition",
                code: 3,
                userInfo: [NSLocalizedDescriptionKey: "Locale '\(locale.identifier)' not supported"]
            )
        }
        
        // Check if model is installed
        let installedLocales = await SpeechTranscriber.installedLocales
        if !installedLocales.map({ $0.identifier(.bcp47) }).contains(locale.identifier(.bcp47)) {
            fputs("Note: Downloading speech model for \(locale.identifier)...\n", stderr)
        }
        
        // Initialize transcriber with progressive preset for real-time results
        let transcriber = SpeechTranscriber(locale: locale, preset: .progressiveTranscription)
        self.transcriber = transcriber
        
        // Create analyzer
        let modules: [any SpeechModule] = [transcriber]
        let analyzer = SpeechAnalyzer(modules: modules)
        self.analyzer = analyzer
        
        // Get best audio format for the analyzer
        guard let audioFormat = await SpeechAnalyzer.bestAvailableAudioFormat(compatibleWith: modules) else {
            throw NSError(
                domain: "AudioFormat",
                code: 4,
                userInfo: [NSLocalizedDescriptionKey: "Failed to get audio format"]
            )
        }
        
        // Set up audio engine with microphone input
        let inputNode = audioEngine.inputNode
        let inputFormat = inputNode.outputFormat(forBus: 0)
        
        // Create format converter if needed
        guard let converter = AVAudioConverter(from: inputFormat, to: audioFormat) else {
            throw NSError(
                domain: "AudioConversion",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "Failed to create audio converter"]
            )
        }
        
        // Create async stream for feeding audio to analyzer
        let (inputSequence, inputBuilder) = AsyncStream<AnalyzerInput>.makeStream()
        self.inputBuilder = inputBuilder
        
        // Install tap on microphone input
        inputNode.installTap(onBus: 0, bufferSize: 4096, format: inputFormat) { [weak self] buffer, _ in
            guard let self = self else { return }
            
            // Convert audio format if needed
            let convertedBuffer = self.convertAudioBuffer(buffer, using: converter, to: audioFormat)
            
            // Feed to analyzer
            let input = AnalyzerInput(buffer: convertedBuffer)
            self.inputBuilder?.yield(input)
        }
        
        // Start audio engine
        try audioEngine.start()
        
        // Start analyzer with streaming input
        try await analyzer.start(inputSequence: inputSequence)
        
        // Process results in parallel
        Task {
            await self.processResults()
        }
    }
    
    private func convertAudioBuffer(_ buffer: AVAudioPCMBuffer, using converter: AVAudioConverter, to format: AVAudioFormat) -> AVAudioPCMBuffer {
        // If formats match, no conversion needed
        if buffer.format == format {
            return buffer
        }
        
        // Create output buffer with converted format
        let frameCount = AVAudioFrameCount(Double(buffer.frameLength) * format.sampleRate / buffer.format.sampleRate)
        guard let convertedBuffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: frameCount) else {
            return buffer
        }
        
        var error: NSError?
        converter.convert(to: convertedBuffer, error: &error) { inNumPackets, outStatus in
            outStatus.pointee = .haveData
            return buffer
        }
        
        if error != nil {
            return buffer
        }
        
        return convertedBuffer
    }
    
    private func processResults() async {
        guard let transcriber = self.transcriber else { return }
        
        do {
            for try await result in transcriber.results {
                let output = TranscriptionOutput(
                    text: String(result.text.characters),
                    isFinal: result.isFinal,
                    timestamp: Date().timeIntervalSince1970
                )
                
                // Output as JSON to stdout
                if let jsonData = try? JSONEncoder().encode(output),
                   let jsonString = String(data: jsonData, encoding: .utf8) {
                    print(jsonString)
                    fflush(stdout)
                }
            }
        } catch {
            fputs("Error processing results: \(error.localizedDescription)\n", stderr)
        }
    }
    
    func stop() async throws {
        // Stop audio engine
        audioEngine.stop()
        audioEngine.inputNode.removeTap(onBus: 0)
        
        // Finalize analyzer
        inputBuilder?.finish()
        
        if let analyzer = self.analyzer {
            try await analyzer.finalizeAndFinishThroughEndOfInput()
        }
    }
}

// Legacy API fallback for older macOS (SFSpeechRecognizer with microphone)
@available(macOS 10.15, *)
class LegacyStreamingTranscriber {
    private let locale: Locale
    private let audioEngine: AVAudioEngine
    private var recognizer: SFSpeechRecognizer?
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?
    
    init(locale: Locale = Locale(identifier: "en-US")) {
        self.locale = locale
        self.audioEngine = AVAudioEngine()
    }
    
    func start() throws {
        guard let recognizer = SFSpeechRecognizer(locale: locale) else {
            throw NSError(
                domain: "SpeechRecognition",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "Speech recognizer not available"]
            )
        }
        
        guard recognizer.isAvailable else {
            throw NSError(
                domain: "SpeechRecognition",
                code: 2,
                userInfo: [NSLocalizedDescriptionKey: "Speech recognizer not available"]
            )
        }
        
        self.recognizer = recognizer
        
        let request = SFSpeechAudioBufferRecognitionRequest()
        request.shouldReportPartialResults = true
        self.recognitionRequest = request
        
        recognitionTask = recognizer.recognitionTask(with: request) { result, error in
            if let error = error {
                fputs("Recognition error: \(error.localizedDescription)\n", stderr)
                return
            }
            
            if let result = result {
                let output = TranscriptionOutput(
                    text: result.bestTranscription.formattedString,
                    isFinal: result.isFinal,
                    timestamp: Date().timeIntervalSince1970
                )
                
                if let jsonData = try? JSONEncoder().encode(output),
                   let jsonString = String(data: jsonData, encoding: .utf8) {
                    print(jsonString)
                    fflush(stdout)
                }
            }
        }
        
        let inputNode = audioEngine.inputNode
        let recordingFormat = inputNode.outputFormat(forBus: 0)
        
        inputNode.installTap(onBus: 0, bufferSize: 1024, format: recordingFormat) { [weak self] buffer, _ in
            self?.recognitionRequest?.append(buffer)
        }
        
        try audioEngine.start()
    }
    
    func stop() {
        audioEngine.stop()
        audioEngine.inputNode.removeTap(onBus: 0)
        recognitionRequest?.endAudio()
        recognitionTask?.cancel()
    }
}

// Main execution
@available(macOS 10.15, *)
@MainActor
func main() async {
    fputs("Starting live microphone transcription... (Press Ctrl+C to stop)\n", stderr)
    fputs("Speak into your microphone.\n", stderr)
    
    do {
        if #available(macOS 26.0, *) {
            // Use modern SpeechAnalyzer
            let transcriber = StreamingTranscriber()
            globalTranscriber = transcriber
            try await transcriber.start()
            
            // Keep running until interrupted (sleep for ~1 year)
            while true {
                try await Task.sleep(nanoseconds: 86_400_000_000_000) // 1 day
            }
        } else {
            // Use legacy API
            let transcriber = LegacyStreamingTranscriber()
            globalTranscriber = transcriber
            try transcriber.start()
            
            // Keep running until interrupted (sleep for ~1 year)
            while true {
                try await Task.sleep(nanoseconds: 86_400_000_000_000) // 1 day
            }
        }
    } catch {
        fputs("Error: \(error.localizedDescription)\n", stderr)
        exit(1)
    }
}

// Signal handler for clean shutdown
var globalTranscriber: (any AnyObject)?

func handleSignal(_ signal: Int32) {
    fputs("\nShutting down...\n", stderr)
    exit(0)
}

// Set up signal handlers
signal(SIGINT, handleSignal)
signal(SIGTERM, handleSignal)

// Run
if #available(macOS 10.15, *) {
    Task {
        await main()
    }
    RunLoop.main.run()
} else {
    fputs("Error: macOS 10.15 or later required\n", stderr)
    exit(1)
}
