use swift_scribe::{StreamingTranscriber, Transcriber};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

fn main() {
    println!("swift-scribe: Speech-to-Text Transcription Tool");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    // Check for --mic flag for live microphone transcription
    if args[1] == "--mic" || args[1] == "-m" {
        run_microphone_mode();
    } else {
        run_file_mode(&args);
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage:");
    eprintln!("  {} <audio-file-path>  - Transcribe an audio file", program_name);
    eprintln!("  {} --mic              - Live microphone transcription", program_name);
    eprintln!();
    eprintln!("Make sure to build the Swift helpers first:");
    eprintln!("  make helpers");
}

fn run_file_mode(args: &[String]) {
    let transcriber = match Transcriber::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error initializing transcriber: {}", e);
            return;
        }
    };

    let audio_path = Path::new(&args[1]);
    if !audio_path.exists() {
        eprintln!("Error: File not found: {}", audio_path.display());
        return;
    }

    println!("Transcribing: {}", audio_path.display());
    println!("This may take a moment...\n");

    match transcriber.transcribe_file(audio_path) {
        Ok(text) => {
            println!("--- Transcription ---");
            println!("{}", text);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn run_microphone_mode() {
    println!("\n🎤 Live Microphone Transcription Mode");
    println!("=====================================");
    println!("Starting microphone capture...");
    println!("Speak into your microphone. Press Ctrl+C to stop.\n");

    let mut transcriber = match StreamingTranscriber::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error initializing streaming transcriber: {}", e);
            eprintln!("\nMake sure you've built the streaming helper:");
            eprintln!("  make helpers");
            return;
        }
    };

    if let Err(e) = transcriber.start() {
        eprintln!("Error starting transcription: {}", e);
        return;
    }

    println!("✓ Microphone active - listening...\n");

    loop {
        match transcriber.poll_result() {
            Ok(Some(result)) => {
                if result.is_final {
                    // Only print final results - cleaner output
                    println!("{}", result.text);
                    io::stdout().flush().unwrap();
                }
                // Skip partial results to avoid display issues with line wrapping
            }
            Ok(None) => {
                // No data yet, sleep briefly to avoid busy-waiting
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }

    println!("\nShutting down...");
    if let Err(e) = transcriber.stop() {
        eprintln!("Error stopping transcription: {}", e);
    }
}
