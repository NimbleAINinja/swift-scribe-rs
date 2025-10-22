//! # swift-scribe
//!
//! A Rust library for on-device speech-to-text transcription using Apple's Speech framework.
//!
//! ## Features
//!
//! - 🚀 Fast Neural Engine-accelerated transcription (macOS 26+ with SpeechAnalyzer)
//! - 📱 On-device processing (no cloud, no internet required)
//! - 🔄 Automatic API selection - uses SpeechAnalyzer on macOS 26+, falls back to SFSpeechRecognizer
//! - ⚡ Works on macOS 10.15+
//!
//! ## Usage
//!
//! ### Basic transcription
//!
//! ```no_run
//! use swift_scribe::Transcriber;
//! use std::path::Path;
//!
//! let transcriber = Transcriber::new().expect("Failed to create transcriber");
//! let text = transcriber.transcribe_file(Path::new("audio.m4a"))
//!     .expect("Transcription failed");
//! println!("Transcription: {}", text);
//! ```
//!
//! ### With custom helper path
//!
//! ```no_run
//! use swift_scribe::Transcriber;
//! use std::path::Path;
//!
//! let transcriber = Transcriber::with_helper_path("/usr/local/bin/transcribe")
//!     .expect("Failed to create transcriber");
//! let text = transcriber.transcribe_file(Path::new("audio.m4a"))
//!     .expect("Transcription failed");
//! println!("Transcription: {}", text);
//! ```
//!
//! ## Requirements
//!
//! This library requires the Swift helper binary to be compiled and accessible.
//! See the [repository README](https://github.com/NimbleAINinja/swift-scribe-rs) for build instructions.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of a transcription operation with optional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// The transcribed text
    pub text: String,
    /// Optional confidence score (0.0-1.0)
    pub confidence: Option<f32>,
}

/// Main transcriber interface for speech-to-text conversion
///
/// # Examples
///
/// ```no_run
/// use swift_scribe::Transcriber;
/// use std::path::Path;
///
/// let transcriber = Transcriber::new().unwrap();
/// let result = transcriber.transcribe_file(Path::new("audio.m4a")).unwrap();
/// println!("Transcription: {}", result);
/// ```
pub struct Transcriber {
    helper_path: PathBuf,
}

impl Transcriber {
    /// Creates a new transcriber with default helper path
    ///
    /// Looks for the helper binary in the following locations (in order):
    /// 1. `./helpers/transcribe` (local development)
    /// 2. `~/.local/bin/transcribe` (user install)
    /// 3. `/usr/local/bin/transcribe` (system install)
    ///
    /// # Errors
    ///
    /// Returns an error if the helper binary cannot be found in any of the default locations.
    pub fn new() -> Result<Self, String> {
        let default_paths = vec![
            PathBuf::from("./helpers/transcribe"),
            dirs::home_dir()
                .map(|h| h.join(".local/bin/transcribe"))
                .unwrap_or_default(),
            PathBuf::from("/usr/local/bin/transcribe"),
        ];

        for path in default_paths {
            if path.exists() {
                return Ok(Self { helper_path: path });
            }
        }

        Err(
            "Helper binary not found. Please compile with 'make helpers' or install system-wide."
                .to_string(),
        )
    }

    /// Creates a new transcriber with a custom helper binary path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the transcribe helper binary
    ///
    /// # Errors
    ///
    /// Returns an error if the specified path does not exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::Transcriber;
    ///
    /// let transcriber = Transcriber::with_helper_path("/custom/path/transcribe").unwrap();
    /// ```
    pub fn with_helper_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Helper binary not found at: {}", path.display()));
        }
        Ok(Self { helper_path: path })
    }

    /// Transcribes an audio file to text
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file (supports M4A, WAV, MP3, AAC, FLAC, AIFF)
    ///
    /// # Returns
    ///
    /// The transcribed text as a `String`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The audio format is unsupported
    /// - The transcription fails
    /// - Speech recognition permissions haven't been granted
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::Transcriber;
    /// use std::path::Path;
    ///
    /// let transcriber = Transcriber::new().unwrap();
    /// match transcriber.transcribe_file(Path::new("recording.m4a")) {
    ///     Ok(text) => println!("Transcription: {}", text),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn transcribe_file(&self, path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Err(format!("Audio file not found: {}", path.display()));
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| "Invalid UTF-8 path".to_string())?;

        let output = Command::new(&self.helper_path)
            .arg(path_str)
            .output()
            .map_err(|e| {
                format!(
                    "Failed to execute helper at {}: {}",
                    self.helper_path.display(),
                    e
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Transcription failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    /// Returns the path to the helper binary being used
    pub fn helper_path(&self) -> &Path {
        &self.helper_path
    }
}

impl Default for Transcriber {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
