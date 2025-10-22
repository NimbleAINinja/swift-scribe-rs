import AVFoundation
import Foundation
import Speech

// Modern SpeechAnalyzer API (macOS 26+)
@available(macOS 26.0, iOS 26.0, *)
func transcribeWithSpeechAnalyzer(audioURL: URL) async throws -> String {
    let locale = Locale(identifier: "en-US")
    
    // Verify locale is supported
    let supportedLocales = await SpeechTranscriber.supportedLocales
    guard supportedLocales.map({ $0.identifier(.bcp47) }).contains(locale.identifier(.bcp47)) else {
        throw NSError(
            domain: "SpeechRecognition",
            code: 3,
            userInfo: [NSLocalizedDescriptionKey: "Locale '\(locale.identifier)' not supported for transcription"]
        )
    }
    
    // Initialize transcriber using preset for simplicity
    // Available presets:
    // - .transcription: Basic, accurate transcription (no alternatives, no time ranges)
    // - .transcriptionWithAlternatives: Includes alternative interpretations
    // - .timeIndexedTranscriptionWithAlternatives: Includes alternatives + audio time ranges
    // - .progressiveTranscription: Real-time volatile results
    // - .timeIndexedProgressiveTranscription: Real-time + time ranges
    let transcriber = SpeechTranscriber(locale: locale, preset: .transcription)
    
    // Alternative: Custom configuration for more control
    // let transcriber = SpeechTranscriber(
    //     locale: locale,
    //     transcriptionOptions: [],              // e.g., [.etiquetteReplacements] for censoring
    //     reportingOptions: [],                  // e.g., [.volatileResults] for real-time
    //     attributeOptions: []                   // e.g., [.audioTimeRange] for timestamps
    // )
    
    // Check if model is installed (optional - will auto-download if needed)
    let installedLocales = await SpeechTranscriber.installedLocales
    if !installedLocales.map({ $0.identifier(.bcp47) }).contains(locale.identifier(.bcp47)) {
        fputs("Note: Downloading speech model for \(locale.identifier)...\n", stderr)
        // Model will be downloaded automatically by the analyzer
    }
    
    // Create analyzer with the transcriber module
    // Note: Can add multiple modules like [transcriber, SpeechDetector()] for VAD
    let modules: [any SpeechModule] = [transcriber]
    let analyzer = SpeechAnalyzer(modules: modules)
    
    // Load audio file (must use AVAudioFile, not raw URL)
    let audioFile = try AVAudioFile(forReading: audioURL)
    
    // Start analysis - finishAfterFile: true means process entire file then finalize
    try await analyzer.start(inputAudioFile: audioFile, finishAfterFile: true)
    
    // Stream results and build transcription
    var fullTranscription = ""
    for try await result in transcriber.results {
        // result.text is AttributedString with the most likely transcription
        // result.isFinal: true = finalized, false = volatile (may change)
        // result.alternatives: alternative interpretations (if preset supports them)
        // result.range: CMTimeRange of audio this result covers
        
        if result.isFinal {
            // Final result - will not be updated
            fullTranscription += String(result.text.characters)
        }
        // Note: We ignore volatile results for file transcription
        // For real-time use, you'd update UI with volatile results
    }
    
    return fullTranscription
}

// Legacy API for older macOS versions using SFSpeechRecognizer
@available(macOS 10.15, *)
func transcribeWithLegacyAPI(audioURL: URL) async throws -> String {
    guard let recognizer = SFSpeechRecognizer(locale: Locale(identifier: "en-US")) else {
        throw NSError(domain: "SpeechRecognition", code: 1, userInfo: [NSLocalizedDescriptionKey: "Speech recognizer not available"])
    }
    
    guard recognizer.isAvailable else {
        throw NSError(domain: "SpeechRecognition", code: 2, userInfo: [NSLocalizedDescriptionKey: "Speech recognizer not available"])
    }
    
    let request = SFSpeechURLRecognitionRequest(url: audioURL)
    request.shouldReportPartialResults = false
    
    return try await withCheckedThrowingContinuation { continuation in
        recognizer.recognitionTask(with: request) { result, error in
            if let error = error {
                continuation.resume(throwing: error)
                return
            }
            
            if let result = result, result.isFinal {
                continuation.resume(returning: result.bestTranscription.formattedString)
            }
        }
    }
}

// Main execution
@available(macOS 10.15, *)
@MainActor
func main() async {
    guard CommandLine.arguments.count > 1 else {
        fputs("Usage: transcribe <audio-file-path>\n", stderr)
        exit(1)
    }
    
    let audioPath = CommandLine.arguments[1]
    let audioURL = URL(fileURLWithPath: audioPath)
    
    guard FileManager.default.fileExists(atPath: audioPath) else {
        fputs("Error: File not found: \(audioPath)\n", stderr)
        exit(1)
    }
    
    do {
        var transcription: String
        
        // Use SpeechAnalyzer on macOS 26+, fallback to legacy API otherwise
        if #available(macOS 26.0, *) {
            transcription = try await transcribeWithSpeechAnalyzer(audioURL: audioURL)
        } else {
            transcription = try await transcribeWithLegacyAPI(audioURL: audioURL)
        }
        
        print(transcription)
        exit(0)
    } catch {
        fputs("Error: \(error.localizedDescription)\n", stderr)
        exit(1)
    }
}

// Run the async main function
if #available(macOS 10.15, *) {
    Task {
        await main()
    }
    RunLoop.main.run()
} else {
    fputs("Error: macOS 10.15 or later required\n", stderr)
    exit(1)
}
