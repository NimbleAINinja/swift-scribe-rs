use clap::Parser;
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "swift-scribe-bench")]
#[command(about = "Benchmark SpeechAnalyzer vs Whisper API", long_about = None)]
struct Args {
    /// Audio file to transcribe
    #[arg(value_name = "FILE")]
    audio_file: PathBuf,

    /// Groq API key (or set GROQ_API_KEY env var)
    #[arg(short = 'k', long)]
    api_key: Option<String>,

    /// Output results as JSON
    #[arg(short, long)]
    json: bool,

    /// Number of runs for averaging (default: 1)
    #[arg(short = 'n', long, default_value = "1")]
    runs: usize,

    /// Whisper model to use
    #[arg(short, long, default_value = "whisper-large-v3-turbo")]
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WhisperResponse {
    text: String,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Serialize, Debug)]
struct BenchmarkResult {
    audio_file: String,
    file_size_mb: f64,
    local: LocalResult,
    api: ApiResult,
    speedup: f64,
}

#[derive(Serialize, Debug)]
struct LocalResult {
    duration_secs: f64,
    text: String,
    method: String,
}

#[derive(Serialize, Debug)]
struct ApiResult {
    duration_secs: f64,
    text: String,
    model: String,
}

fn main() {
    let args = Args::parse();

    // Get API key from args or environment
    let api_key = args.api_key
        .or_else(|| std::env::var("GROQ_API_KEY").ok())
        .expect("GROQ_API_KEY not provided. Use --api-key or set GROQ_API_KEY env var");

    if !args.audio_file.exists() {
        eprintln!("Error: File not found: {}", args.audio_file.display());
        std::process::exit(1);
    }

    // Get file size
    let file_size_mb = std::fs::metadata(&args.audio_file)
        .map(|m| m.len() as f64 / 1_000_000.0)
        .unwrap_or(0.0);

    if !args.json {
        println!("\n🔬 Benchmarking Speech-to-Text Performance");
        println!("═══════════════════════════════════════════");
        println!("Audio file: {}", args.audio_file.display());
        println!("File size:  {:.2} MB", file_size_mb);
        println!("Runs:       {}", args.runs);
        println!();
    }

    // Run benchmarks
    let mut local_times = Vec::new();
    let mut api_times = Vec::new();
    let mut local_text = String::new();
    let mut api_text = String::new();

    for run in 1..=args.runs {
        if !args.json && args.runs > 1 {
            println!("Run {}/{}...", run, args.runs);
        }

        // Benchmark local SpeechAnalyzer
        if !args.json {
            print!("  ⚡ Testing local SpeechAnalyzer... ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        
        let start = Instant::now();
        local_text = run_local_transcription(&args.audio_file);
        let local_duration = start.elapsed().as_secs_f64();
        local_times.push(local_duration);

        if !args.json {
            println!("{:.2}s", local_duration);
        }

        // Benchmark Whisper API
        if !args.json {
            print!("  🌐 Testing Whisper API ({})... ", args.model);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        let start = Instant::now();
        api_text = run_whisper_api(&args.audio_file, &api_key, &args.model);
        let api_duration = start.elapsed().as_secs_f64();
        api_times.push(api_duration);

        if !args.json {
            println!("{:.2}s", api_duration);
        }
    }

    // Calculate averages
    let avg_local = local_times.iter().sum::<f64>() / local_times.len() as f64;
    let avg_api = api_times.iter().sum::<f64>() / api_times.len() as f64;
    let speedup = avg_api / avg_local;

    let result = BenchmarkResult {
        audio_file: args.audio_file.display().to_string(),
        file_size_mb,
        local: LocalResult {
            duration_secs: avg_local,
            text: local_text.clone(),
            method: "SpeechAnalyzer".to_string(),
        },
        api: ApiResult {
            duration_secs: avg_api,
            text: api_text.clone(),
            model: args.model.clone(),
        },
        speedup,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        print_results(&result, &local_times, &api_times);
    }
}

fn run_local_transcription(audio_file: &PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("./helpers/transcribe")
        .arg(audio_file)
        .output()
        .expect("Failed to run local transcriber");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Local transcription failed: {}", stderr);
        return String::from("[ERROR]");
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_whisper_api(audio_file: &PathBuf, api_key: &str, model: &str) -> String {
    let client = reqwest::blocking::Client::new();

    let form = multipart::Form::new()
        .text("model", model.to_string())
        .text("temperature", "0")
        .text("response_format", "json")
        .file("file", audio_file)
        .expect("Failed to read audio file");

    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .expect("Failed to send request");

    if !response.status().is_success() {
        eprintln!("API request failed: {}", response.status());
        eprintln!("Response: {}", response.text().unwrap_or_default());
        return String::from("[ERROR]");
    }

    let text = response.text().expect("Failed to read response");
    let whisper: WhisperResponse = serde_json::from_str(&text).expect("Failed to parse response");
    whisper.text
}

fn print_results(result: &BenchmarkResult, local_times: &[f64], api_times: &[f64]) {
    println!("\n📊 Results");
    println!("═══════════════════════════════════════════");
    
    println!("\n⚡ Local SpeechAnalyzer");
    println!("  Average time:  {:.2}s", result.local.duration_secs);
    if local_times.len() > 1 {
        let min = local_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = local_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        println!("  Min/Max:       {:.2}s / {:.2}s", min, max);
    }
    println!("  Output:        {} chars", result.local.text.len());

    println!("\n🌐 Whisper API ({})", result.api.model);
    println!("  Average time:  {:.2}s", result.api.duration_secs);
    if api_times.len() > 1 {
        let min = api_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = api_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        println!("  Min/Max:       {:.2}s / {:.2}s", min, max);
    }
    println!("  Output:        {} chars", result.api.text.len());

    println!("\n🏆 Comparison");
    println!("  Speedup:       {:.2}x faster (local)", result.speedup);
    
    let percentage = ((result.speedup - 1.0) * 100.0).abs();
    if result.speedup > 1.0 {
        println!("  Improvement:   {:.1}% faster with SpeechAnalyzer", percentage);
    } else {
        println!("  Improvement:   {:.1}% faster with Whisper API", percentage);
    }

    // Show text comparison if they differ
    if result.local.text.trim() != result.api.text.trim() {
        println!("\n📝 Transcription Comparison");
        println!("  Note: Outputs differ in length/content");
        println!("\n  Local (first 200 chars):");
        println!("  {}", &result.local.text.chars().take(200).collect::<String>());
        println!("\n  API (first 200 chars):");
        println!("  {}", &result.api.text.chars().take(200).collect::<String>());
    } else {
        println!("\n✓ Both transcriptions match!");
    }

    println!();
}
