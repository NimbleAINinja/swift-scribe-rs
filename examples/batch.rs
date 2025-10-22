/// Batch processing example - transcribe all audio files in a directory
///
/// Run with: cargo run --example batch -- /path/to/audio/files

use swift_scribe::Transcriber;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run --example batch -- ~/Music/Podcasts");
        std::process::exit(1);
    }
    
    let dir_path = Path::new(&args[1]);
    
    if !dir_path.is_dir() {
        eprintln!("Error: Not a directory: {}", dir_path.display());
        std::process::exit(1);
    }
    
    // Initialize transcriber once
    let transcriber = Transcriber::new()
        .map_err(|e| format!("Failed to initialize: {}\n\nHint: Run ./install_helper.sh first", e))?;
    
    // Find all audio files
    let audio_extensions = ["m4a", "wav", "mp3", "aac", "flac", "aiff"];
    let mut audio_files = Vec::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(ext) = path.extension() {
            if audio_extensions.contains(&ext.to_str().unwrap_or("")) {
                audio_files.push(path);
            }
        }
    }
    
    if audio_files.is_empty() {
        println!("No audio files found in {}", dir_path.display());
        return Ok(());
    }
    
    println!("Found {} audio files\n", audio_files.len());
    
    // Process each file
    let mut results = Vec::new();
    
    for (i, path) in audio_files.iter().enumerate() {
        println!("[{}/{}] Processing: {}", i + 1, audio_files.len(), path.file_name().unwrap().to_str().unwrap());
        
        match transcriber.transcribe_file(path) {
            Ok(text) => {
                println!("  ✓ Success: {} chars\n", text.len());
                results.push((path.clone(), text));
            }
            Err(e) => {
                eprintln!("  ✗ Failed: {}\n", e);
            }
        }
    }
    
    // Summary
    println!("\n=== Summary ===");
    println!("Processed: {} / {}", results.len(), audio_files.len());
    
    if results.is_empty() {
        println!("No successful transcriptions");
        return Ok(());
    }
    
    // Save to file
    let output_path = dir_path.join("transcriptions.txt");
    let mut output = String::new();
    
    for (path, text) in &results {
        output.push_str(&format!("\n=== {} ===\n", path.file_name().unwrap().to_str().unwrap()));
        output.push_str(text);
        output.push_str("\n");
    }
    
    fs::write(&output_path, output)?;
    println!("\nSaved to: {}", output_path.display());
    
    Ok(())
}
