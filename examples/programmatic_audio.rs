/// Example: Programmatic audio input
///
/// This example demonstrates how to use swift-scribe with programmatic audio input,
/// useful for transcribing system audio, network streams, or custom audio sources.
///
/// Run with:
/// cargo run --example programmatic_audio

use swift_scribe::StreamingTranscriber;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Swift Scribe - Programmatic Audio Input Example");
    println!("================================================\n");

    // Create a transcriber configured for programmatic audio input
    let mut transcriber = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build()?;

    println!("Starting transcriber with programmatic input mode...");
    transcriber.start()?;

    // Simulate receiving audio samples from a system audio capture or network stream
    // In real usage, this would be your actual audio source
    println!("Feeding audio samples...\n");

    // Generate some test audio samples (silence/zeros for demo)
    // In production, these would come from your audio source
    for chunk_num in 0..5 {
        // Generate a chunk of audio samples (4096 samples at 48kHz)
        // This would be 4096 / 48000 = ~85ms of audio
        let samples: Vec<f32> = (0..4096).map(|_| 0.0f32).collect();

        // Feed the samples to the transcriber
        // Parameters: samples, sample_rate (48kHz), channels (stereo)
        transcriber.feed_audio_f32(&samples, 48000, 2)?;

        println!("Chunk {}: Fed 4096 samples at 48kHz stereo", chunk_num + 1);

        // Poll for results (non-blocking)
        match transcriber.poll_result() {
            Ok(Some(result)) => {
                let label = if result.is_final { "FINAL" } else { "partial" };
                println!("  [{}] {}", label, result.text);
            }
            Ok(None) => {
                println!("  (no result yet)");
            }
            Err(e) => {
                println!("  Error polling: {}", e);
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nCleaning up...");
    transcriber.stop()?;

    println!("Example completed!");
    println!("\nIn a real application, you would:");
    println!("1. Set up your audio source (system audio, network stream, etc.)");
    println!("2. Receive audio chunks from that source");
    println!("3. Feed them to the transcriber with feed_audio_f32() or feed_audio_i16()");
    println!("4. Poll for results as they become available");
    println!("5. Process the transcription results");

    Ok(())
}
