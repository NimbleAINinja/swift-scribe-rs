/// Example: Live microphone transcription
///
/// Demonstrates how to use the StreamingTranscriber API for real-time
/// speech-to-text from microphone input.

use swift_scribe::StreamingTranscriber;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!("🎤 Microphone Streaming Example");
    println!("================================\n");

    // Create streaming transcriber
    let mut transcriber = match StreamingTranscriber::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nMake sure to build the helpers first:");
            eprintln!("  make helpers");
            return;
        }
    };

    // Start transcription
    println!("Starting microphone capture...");
    if let Err(e) = transcriber.start() {
        eprintln!("Failed to start: {}", e);
        return;
    }

    println!("✓ Listening... (Press Ctrl+C to stop)\n");

    let mut partial_active = false;
    let mut final_transcription = Vec::new();

    // Poll for results
    loop {
        match transcriber.poll_result() {
            Ok(Some(result)) => {
                if result.is_final {
                    // Move to new line if partial was active
                    if partial_active {
                        println!();
                        partial_active = false;
                    }

                    // Print and save final result
                    println!("[FINAL] {}", result.text);
                    final_transcription.push(result.text);
                    io::stdout().flush().unwrap();
                } else {
                    // Display partial in-place with carriage return
                    print!("\r\x1B[K[partial] {}", result.text);
                    io::stdout().flush().unwrap();
                    partial_active = true;
                }
            }
            Ok(None) => {
                // No data available, sleep briefly
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                if partial_active {
                    println!();
                }
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }

    // Cleanup
    if partial_active {
        println!();
    }
    println!("\n\nFull transcription:");
    println!("==================");
    for (i, text) in final_transcription.iter().enumerate() {
        println!("{}. {}", i + 1, text);
    }

    if let Err(e) = transcriber.stop() {
        eprintln!("Error stopping: {}", e);
    }
}
