# Using swift-scribe-rs as a Library

Complete guide to integrating swift-scribe-rs into your Rust project.

## Installation

### Option 1: From Git (Recommended for now)

Add to your `Cargo.toml`:

```toml
[dependencies]
swift-scribe-rs = { git = "https://github.com/NimbleAINinja/swift-scribe-rs" }
```

### Option 2: Path Dependency (Local Development)

If you have swift-scribe-rs checked out locally:

```toml
[dependencies]
swift-scribe-rs = { path = "../swift-scribe-rs" }
```

### Option 3: From crates.io (Future)

Once published:

```toml
[dependencies]
swift-scribe-rs = "0.1"
```

## Requirements

### 1. Install the Swift Helper

The library requires the Swift helper binary to be installed:

```bash
# From the swift-scribe-rs repository
cd /path/to/swift-scribe-rs
make helpers

# Install to user directory
mkdir -p ~/.local/bin
cp helpers/transcribe ~/.local/bin/

# Or install system-wide
sudo cp helpers/transcribe /usr/local/bin/
```

The library will look for the helper in these locations (in order):
1. `./helpers/transcribe` (for local development)
2. `~/.local/bin/transcribe` (user install)
3. `/usr/local/bin/transcribe` (system install)

### 2. macOS Requirements

- macOS 10.15+ (Catalina or later)
- macOS 26+ for SpeechAnalyzer (auto-falls back to legacy API on older versions)
- Speech recognition permissions granted

## Basic Usage

### Simple Transcription

```rust
use swift_scribe::Transcriber;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transcriber (finds helper automatically)
    let transcriber = Transcriber::new()
        .map_err(|e| format!("Failed to create transcriber: {}", e))?;
    
    // Transcribe an audio file
    let text = transcriber.transcribe_file(Path::new("audio.m4a"))?;
    
    println!("Transcription: {}", text);
    Ok(())
}
```

### With Custom Helper Path

```rust
use swift_scribe::Transcriber;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a specific helper binary location
    let transcriber = Transcriber::with_helper_path("/custom/path/transcribe")?;
    
    let text = transcriber.transcribe_file(Path::new("audio.m4a"))?;
    println!("{}", text);
    Ok(())
}
```

### Error Handling

```rust
use swift_scribe::Transcriber;
use std::path::Path;

fn transcribe_with_fallback(audio_path: &Path) -> String {
    let transcriber = match Transcriber::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Warning: {}", e);
            eprintln!("Hint: Run 'make helpers' to compile the Swift helper");
            return String::from("[Transcription unavailable]");
        }
    };
    
    transcriber.transcribe_file(audio_path)
        .unwrap_or_else(|e| {
            eprintln!("Transcription failed: {}", e);
            String::from("[Error]")
        })
}
```

### Processing Multiple Files

```rust
use swift_scribe::Transcriber;
use std::path::Path;

fn transcribe_directory(dir: &Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let transcriber = Transcriber::new()?;
    let mut results = Vec::new();
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip non-audio files
        if !path.extension().map_or(false, |ext| {
            matches!(ext.to_str(), Some("m4a" | "wav" | "mp3" | "aac"))
        }) {
            continue;
        }
        
        println!("Processing: {}", path.display());
        match transcriber.transcribe_file(&path) {
            Ok(text) => {
                results.push((path.display().to_string(), text));
                println!("OK: Success");
            }
            Err(e) => {
                eprintln!("✗ Failed: {}", e);
            }
        }
    }
    
    Ok(results)
}
```

## Advanced Usage

### Parallel Processing

```rust
use swift_scribe::Transcriber;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

fn transcribe_parallel(files: Vec<PathBuf>) -> Vec<Result<String, String>> {
    let transcriber = Arc::new(Transcriber::new().unwrap());
    
    let handles: Vec<_> = files
        .into_iter()
        .map(|path| {
            let transcriber = Arc::clone(&transcriber);
            thread::spawn(move || {
                transcriber.transcribe_file(&path)
            })
        })
        .collect();
    
    handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect()
}
```

### With Progress Tracking

```rust
use swift_scribe::Transcriber;
use std::path::Path;

fn transcribe_with_progress(files: &[&Path]) {
    let transcriber = Transcriber::new()
        .expect("Failed to create transcriber");
    
    let total = files.len();
    
    for (i, path) in files.iter().enumerate() {
        println!("[{}/{}] Processing: {}", i + 1, total, path.display());
        
        match transcriber.transcribe_file(path) {
            Ok(text) => {
                println!("Result: {} chars", text.len());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
```

### Integration with Web Framework (Axum Example)

```rust
use axum::{
    body::Bytes,
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use swift_scribe::Transcriber;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

struct AppState {
    transcriber: Transcriber,
}

async fn upload_audio(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        
        if name == "audio" {
            // Save to temp file
            let temp_path = format!("/tmp/audio_{}.m4a", uuid::Uuid::new_v4());
            let mut file = tokio::fs::File::create(&temp_path).await.unwrap();
            file.write_all(&data).await.unwrap();
            
            // Transcribe (blocking operation)
            let transcriber = &state.transcriber;
            let result = tokio::task::spawn_blocking(move || {
                transcriber.transcribe_file(Path::new(&temp_path))
            })
            .await
            .unwrap();
            
            // Clean up
            tokio::fs::remove_file(&temp_path).await.ok();
            
            match result {
                Ok(text) => return (StatusCode::OK, text),
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e),
            }
        }
    }
    
    (StatusCode::BAD_REQUEST, "No audio file provided".to_string())
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        transcriber: Transcriber::new().expect("Failed to initialize transcriber"),
    });
    
    let app = Router::new()
        .route("/transcribe", post(upload_audio))
        .with_state(state);
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## API Reference

### `Transcriber`

Main struct for transcription operations.

#### Methods

##### `new() -> Result<Self, String>`

Creates a transcriber using automatic helper discovery.

**Errors:** Returns error if helper binary not found in default locations.

##### `with_helper_path<P: AsRef<Path>>(path: P) -> Result<Self, String>`

Creates a transcriber with explicit helper path.

**Arguments:**
- `path`: Path to the transcribe helper binary

**Errors:** Returns error if specified path doesn't exist.

##### `transcribe_file(&self, path: &Path) -> Result<String, String>`

Transcribes an audio file to text.

**Arguments:**
- `path`: Path to audio file (M4A, WAV, MP3, AAC, FLAC, AIFF)

**Returns:** Transcribed text as `String`.

**Errors:** Returns error if:
- File doesn't exist
- Audio format unsupported
- Transcription fails
- Permissions not granted

##### `helper_path(&self) -> &Path`

Returns the path to the helper binary being used.

### `TranscriptionResult`

Result structure (currently minimal, prepared for future metadata).

```rust
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: Option<f32>,
}
```

## Supported Audio Formats

- M4A (recommended)
- WAV
- MP3
- AAC
- FLAC
- AIFF

## Troubleshooting

### "Helper binary not found"

**Solution 1:** Install the helper:
```bash
cd /path/to/swift-scribe-rs
make helpers
cp helpers/transcribe ~/.local/bin/
```

**Solution 2:** Use explicit path:
```rust
let transcriber = Transcriber::with_helper_path("/path/to/transcribe")?;
```

### "Failed to execute helper"

Check that the helper is executable:
```bash
chmod +x ~/.local/bin/transcribe
```

### "Speech recognizer not available"

Grant speech recognition permissions in System Settings:
- Settings → Privacy & Security → Speech Recognition

### Different Results on Different macOS Versions

This is expected - the library automatically uses:
- **macOS 26+**: SpeechAnalyzer (faster, more accurate)
- **macOS 10.15-25**: SFSpeechRecognizer (proven, reliable)

Both provide effective results but may format output slightly differently.

## Distributing Your App

### Option 1: Bundle the Helper

Include the compiled helper in your application bundle:

```rust
use swift_scribe::Transcriber;
use std::env;

fn get_bundled_transcriber() -> Result<Transcriber, String> {
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or("Failed to get executable directory")?;
    
    let helper_path = exe_dir.join("transcribe");
    Transcriber::with_helper_path(helper_path)
}
```

**In your build process:**
```bash
# Build your app
cargo build --release

# Copy helper to bundle
cp /path/to/swift-scribe-rs/helpers/transcribe target/release/
```

### Option 2: Install Helper Separately

Provide an installer script with your application:

```bash
#!/bin/bash
# install.sh

echo "Installing swift-scribe-rs transcription helper..."
curl -L https://github.com/NimbleAINinja/swift-scribe-rs/releases/download/v0.1.0/transcribe \
    -o ~/.local/bin/transcribe
chmod +x ~/.local/bin/transcribe
echo "OK: Installation complete"
```

### Option 3: Use Post-Install Hook

If distributing via Homebrew or similar:

```ruby
class YourApp < Formula
  # ...
  
  def install
    bin.install "your-app"
    bin.install "transcribe"  # Include helper
  end
end
```

## Performance Tips

1. **Reuse Transcriber**: Create one `Transcriber` instance and reuse it
2. **Batch Processing**: Process multiple files with the same instance
3. **Parallel Processing**: Safe to use from multiple threads (helper is stateless)
4. **File Formats**: M4A generally provides best results/performance

## Examples

See the `examples/` directory (if available) for:
- Simple CLI tool
- Batch processor
- Web API server
- GUI application

## Getting Help

- **Issues**: https://github.com/NimbleAINinja/swift-scribe-rs/issues
- **Discussions**: https://github.com/NimbleAINinja/swift-scribe-rs/discussions
- **Documentation**: Run `cargo doc --open` for API docs

## License

MIT - See LICENSE file for details
