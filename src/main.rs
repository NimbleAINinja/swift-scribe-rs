use swift_scribe::Transcriber;
use std::path::Path;

fn main() {
    println!("swift-scribe: Speech-to-Text Transcription Tool");
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <audio-file-path>", args[0]);
        eprintln!("\nMake sure to build the Swift helper first:");
        eprintln!("  make helpers");
        return;
    }
    
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
