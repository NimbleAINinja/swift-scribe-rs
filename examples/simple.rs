/// Simple example of using swift-scribe as a library
///
/// Run with: cargo run --example simple -- audio.m4a

use swift_scribe::Transcriber;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <audio-file>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run --example simple -- recording.m4a");
        std::process::exit(1);
    }
    
    let audio_path = Path::new(&args[1]);
    
    // Create transcriber
    println!("Initializing transcriber...");
    let transcriber = Transcriber::new()
        .map_err(|e| format!("Failed to initialize: {}\n\nHint: Run ./install_helper.sh first", e))?;
    
    println!("Using helper: {}", transcriber.helper_path().display());
    
    // Transcribe
    println!("Transcribing: {}", audio_path.display());
    let text = transcriber.transcribe_file(audio_path)?;
    
    // Output
    println!("\n--- Result ---");
    println!("{}", text);
    println!("\nLength: {} characters", text.len());
    
    Ok(())
}
