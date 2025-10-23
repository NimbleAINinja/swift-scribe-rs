//! System audio capture and transcription example
//!
//! Demonstrates the concept of capturing system audio and feeding it to the
//! transcription helper via stdin for real-time transcription.
//!
//! The helper accepts audio in any common format and automatically resamples
//! to the optimal format for speech recognition (16kHz mono).
//!
//! For production use, consider using a library like `ruhear` that handles
//! audio capture:
//! https://github.com/aizcutei/ruhear
//!
//! Requirements:
//! - macOS 10.15+ (for transcription)
//! - Microphone/Screen Recording permissions as needed
//!
//! Usage:
//!     cargo run --example system_audio

use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("System Audio Transcription Example");
    println!("===================================");
    println!();
    println!("This example demonstrates how to pipe audio to the transcription helper.");
    println!();
    println!("The helper accepts various audio formats and automatically resamples:");
    println!("  - Any sample rate (8kHz-48kHz typical)");
    println!("  - Mono or stereo input");
    println!("  - Format: 16-bit PCM (s16le)");
    println!();
    println!("To test with actual system audio, you can use:");
    println!("  1. A system audio capture tool (e.g., BlackHole, ruhear)");
    println!("  2. ffmpeg to capture audio (helper handles resampling):");
    println!("     ffmpeg -f avfoundation -i \":1\" -f s16le - | \\");
    println!("       ./helpers/transcribe_stream --stdin --sample-rate 48000 --channels 2");
    println!();
    println!("Or use optimal format (no resampling needed):");
    println!("  ffmpeg -i audio.m4a -ar 16000 -ac 1 -f s16le - | \\");
    println!("    ./helpers/transcribe_stream --stdin");
    println!();
    println!("Or with real-time playback:");
    println!("  ffmpeg -re -i audio.m4a -ar 16000 -ac 1 -f s16le - | \\");
    println!("    ./helpers/transcribe_stream --stdin");
    println!();

    // For this example, we'll demonstrate the helper interface
    // In production, you'd integrate with screencapturekit or another audio source

    let helper_path = std::path::PathBuf::from("./helpers/transcribe_stream");
    
    if !helper_path.exists() {
        eprintln!("Error: Helper binary not found. Please run 'make helpers' first.");
        std::process::exit(1);
    }

    println!("Example: Testing stdin mode without audio...");
    println!("(In production, pipe actual audio data here)");
    println!();
    
    // Demonstrate launching the helper in stdin mode
    let mut helper_process = Command::new(&helper_path)
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let _helper_stdin = helper_process.stdin.take()
        .ok_or("Failed to get helper stdin")?;
    
    let helper_stdout = helper_process.stdout.take()
        .ok_or("Failed to get helper stdout")?;
    
    let helper_stderr = helper_process.stderr.take()
        .ok_or("Failed to get helper stderr")?;

    // Read stderr in a separate thread to show status messages
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(helper_stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                eprintln!("[helper] {}", line);
            }
        }
    });

    // Read transcription results
    let reader = std::io::BufReader::new(helper_stdout);
    
    println!("Helper is running and waiting for audio data on stdin...");
    println!();
    println!("Integration pattern for your application:");
    println!("  1. Capture system audio using ScreenCaptureKit, ruhear, or similar");
    println!("  2. Convert audio samples to 16-bit PCM (i16)");
    println!("  3. Launch helper with --sample-rate and --channels matching your source");
    println!("  4. Write PCM bytes to helper's stdin");
    println!("  5. Read JSON results from helper's stdout");
    println!();
    println!("Example code structure:");
    println!(r#"
    // Launch helper with source format (e.g., 48kHz stereo from ScreenCaptureKit)
    let mut helper = Command::new("./helpers/transcribe_stream")
        .arg("--stdin")
        .arg("--sample-rate").arg("48000")
        .arg("--channels").arg("2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    // In your audio callback:
    fn handle_audio_buffer(audio_data: &[f32]) {{
        // Convert f32 samples to i16 PCM
        let pcm_data: Vec<u8> = audio_data
            .iter()
            .flat_map(|sample| {{
                let sample_i16 = (sample * 32767.0) as i16;
                sample_i16.to_le_bytes()
            }})
            .collect();
        
        // Write to helper stdin (resampling happens automatically)
        helper_stdin.write_all(&pcm_data)?;
        helper_stdin.flush()?;
    }}
    
    // In a separate thread, read results:
    for line in BufReader::new(helper_stdout).lines() {{
        let result: StreamingResult = serde_json::from_str(&line?)?;
        println!("{{}} {{}}", 
            if result.is_final {{ "FINAL" }} else {{ "partial" }},
            result.text);
    }}
"#);
    
    // Give the helper a moment to start, then stop cleanly
    std::thread::sleep(Duration::from_secs(2));
    
    helper_process.kill()?;
    helper_process.wait()?;

    println!();
    println!("For a complete audio capture solution, consider:");
    println!("  - ruhear: https://github.com/aizcutei/ruhear");
    println!("  - screencapturekit-rs: https://github.com/doom-fish/screencapturekit-rs");
    println!("  - Core Audio directly via cidre or similar");
    println!();

    Ok(())
}
