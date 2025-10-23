/// Example: System Audio Transcription
///
/// This example demonstrates how to transcribe system audio in real-time,
/// useful for applications that monitor speaker output.
///
/// Architecture:
/// System Audio Capture → f32 samples → StreamingTranscriber → Transcription
///
/// In a real application, you would use a system audio capture library like:
/// - coreaudio-rs (macOS)
/// - cpal (cross-platform)
/// - screencapturekit (macOS 13+)
///
/// This example shows the integration pattern with the transcriber API.

use swift_scribe::StreamingTranscriber;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("System Audio Transcription Example");
    println!("==================================\n");

    // Create a channel for audio samples (simulating a real audio capture source)
    let (tx, rx) = channel::<Vec<f32>>();

    // Spawn a thread that simulates capturing system audio
    let audio_capture_thread = thread::spawn(move || {
        simulate_system_audio_capture(tx);
    });

    // Create transcriber for programmatic audio input
    let mut transcriber = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build()?;

    println!("Starting system audio transcription...");
    println!("(Simulating audio capture)\n");

    transcriber.start()?;

    let mut chunk_count = 0;
    let max_chunks = 10;

    // Main transcription loop
    loop {
        // Try to receive audio samples from the capture thread
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(audio_chunk) => {
                let sample_rate = 48000;
                let channels = 2;

                // Feed audio to transcriber
                transcriber.feed_audio_f32(&audio_chunk, sample_rate, channels)?;

                chunk_count += 1;
                println!("Chunk {}: Fed {} samples @ {}kHz {}ch",
                    chunk_count,
                    audio_chunk.len(),
                    sample_rate / 1000,
                    channels
                );

                // Poll for transcription results
                loop {
                    match transcriber.poll_result() {
                        Ok(Some(result)) => {
                            let label = if result.is_final { "✓ FINAL" } else { "↻ INTERIM" };
                            println!("  {} | {}", label, result.text);
                        }
                        Ok(None) => break,
                        Err(e) => {
                            eprintln!("  Error: {}", e);
                            break;
                        }
                    }
                }

                if chunk_count >= max_chunks {
                    break;
                }
            }
            Err(_) => {
                // Timeout waiting for audio
                println!("  (waiting for audio...)");
            }
        }
    }

    println!("\nCleaning up...");
    transcriber.stop()?;

    // Wait for audio capture thread to finish
    let _ = audio_capture_thread.join();

    println!("\n=== Architecture Pattern ===");
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│ Your System Audio Capture Library                   │");
    println!("│ (coreaudio, cpal, screencapturekit, etc.)          │");
    println!("│                  ↓                                   │");
    println!("│        Vec<f32> audio chunks                        │");
    println!("│          48kHz, stereo                              │");
    println!("│                  ↓                                   │");
    println!("├─────────────────────────────────────────────────────┤");
    println!("│ StreamingTranscriber::feed_audio_f32()              │");
    println!("│                  ↓                                   │");
    println!("│ • Auto-convert f32 → i16                           │");
    println!("│ • Auto-resample to 16kHz                           │");
    println!("│ • Auto-convert stereo → mono                       │");
    println!("├─────────────────────────────────────────────────────┤");
    println!("│ Swift Speech Framework (SpeechAnalyzer)            │");
    println!("│                  ↓                                   │");
    println!("│ Real-time Transcription Results                    │");
    println!("│ • Partial results (interim)                        │");
    println!("│ • Final results (with confidence)                  │");
    println!("└─────────────────────────────────────────────────────┘");

    println!("\n=== Integration Steps ===");
    println!("1. Capture system audio with your preferred library");
    println!("2. Convert to f32 samples (-1.0 to 1.0 range)");
    println!("3. Create transcriber: StreamingTranscriber::builder()");
    println!("   .with_programmatic_input()");
    println!("   .build()?");
    println!("4. Start: transcriber.start()?");
    println!("5. Feed audio: transcriber.feed_audio_f32(&samples, 48000, 2)?");
    println!("6. Poll results: transcriber.poll_result()?");
    println!("7. Process transcription results");

    println!("\nExample completed successfully!");

    Ok(())
}

fn simulate_system_audio_capture(tx: Sender<Vec<f32>>) {
    println!("[Audio Capture Thread] Starting simulation...\n");

    for _i in 0..10 {
        // Simulate capturing a chunk of system audio
        // In reality, this would come from coreaudio, cpal, or similar
        let chunk_size = 4096;
        let audio_chunk = vec![0.0f32; chunk_size];

        if let Err(_) = tx.send(audio_chunk) {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    println!("\n[Audio Capture Thread] Simulation complete");
}
