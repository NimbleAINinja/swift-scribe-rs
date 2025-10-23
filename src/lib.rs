//! # swift-scribe
//!
//! A Rust library for on-device speech-to-text transcription using Apple's Speech framework.
//!
//! ## Features
//!
//! - 🚀 Fast Neural Engine-accelerated transcription (macOS 26+ with SpeechAnalyzer)
//! - 📱 On-device processing (no cloud, no internet required)
//! - 🔄 Automatic API selection - uses SpeechAnalyzer on macOS 26+, falls back to SFSpeechRecognizer
//! - 🎤 Live microphone transcription with real-time results
//! - ⚡ Works on macOS 10.15+
//!
//! ## Usage
//!
//! ### Basic file transcription
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
//! ### Live microphone transcription
//!
//! ```no_run
//! use swift_scribe::StreamingTranscriber;
//!
//! let mut transcriber = StreamingTranscriber::new().expect("Failed to create transcriber");
//! transcriber.start().expect("Failed to start transcription");
//!
//! loop {
//!     if let Some(result) = transcriber.poll_result().expect("Failed to poll") {
//!         if result.is_final {
//!             println!("Final: {}", result.text);
//!         } else {
//!             println!("Partial: {}", result.text);
//!         }
//!     }
//! }
//! ```
//!
//! ## Requirements
//!
//! This library requires the Swift helper binaries to be compiled and accessible.
//! See the [repository README](https://github.com/NimbleAINinja/swift-scribe-rs) for build instructions.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

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

/// Result from streaming transcription with real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResult {
    /// The transcribed text
    pub text: String,
    /// Whether this is a final result (true) or volatile/partial (false)
    #[serde(rename = "isFinal")]
    pub is_final: bool,
    /// Unix timestamp when the result was generated
    pub timestamp: f64,
}

/// Streaming transcriber for live microphone or audio stream input
///
/// Provides real-time transcription with both partial (volatile) and final results.
/// Uses progressive transcription mode for low-latency feedback.
///
/// # Examples
///
/// ```no_run
/// use swift_scribe::StreamingTranscriber;
///
/// let mut transcriber = StreamingTranscriber::new().unwrap();
/// transcriber.start().unwrap();
///
/// // Poll for results in a loop
/// while let Some(result) = transcriber.poll_result().unwrap() {
///     if result.is_final {
///         println!("Final: {}", result.text);
///     } else {
///         print!("\rPartial: {}", result.text);
///     }
/// }
/// ```
pub struct StreamingTranscriber {
    helper_path: PathBuf,
    process: Option<Child>,
    reader: Option<BufReader<std::process::ChildStdout>>,
}

impl StreamingTranscriber {
    /// Creates a new streaming transcriber with default helper path
    ///
    /// Looks for the helper binary in the following locations (in order):
    /// 1. `./helpers/transcribe_stream` (local development)
    /// 2. `~/.local/bin/transcribe_stream` (user install)
    /// 3. `/usr/local/bin/transcribe_stream` (system install)
    ///
    /// # Errors
    ///
    /// Returns an error if the helper binary cannot be found.
    pub fn new() -> Result<Self, String> {
        let default_paths = vec![
            PathBuf::from("./helpers/transcribe_stream"),
            dirs::home_dir()
                .map(|h| h.join(".local/bin/transcribe_stream"))
                .unwrap_or_default(),
            PathBuf::from("/usr/local/bin/transcribe_stream"),
        ];

        for path in default_paths {
            if path.exists() {
                return Ok(Self {
                    helper_path: path,
                    process: None,
                    reader: None,
                });
            }
        }

        Err("Streaming helper binary not found. Please compile with 'make helpers'.".to_string())
    }

    /// Creates a new streaming transcriber with a custom helper binary path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the transcribe_stream helper binary
    ///
    /// # Errors
    ///
    /// Returns an error if the specified path does not exist.
    pub fn with_helper_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!(
                "Streaming helper binary not found at: {}",
                path.display()
            ));
        }
        Ok(Self {
            helper_path: path,
            process: None,
            reader: None,
        })
    }

    /// Starts the streaming transcription
    ///
    /// Launches the helper process and begins capturing from the microphone.
    /// Call `poll_result()` to retrieve transcription results.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The helper process fails to start
    /// - Microphone permissions haven't been granted
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    ///
    /// let mut transcriber = StreamingTranscriber::new().unwrap();
    /// transcriber.start().unwrap();
    /// ```
    pub fn start(&mut self) -> Result<(), String> {
        let mut child = Command::new(&self.helper_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                format!(
                    "Failed to start streaming helper at {}: {}",
                    self.helper_path.display(),
                    e
                )
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;

        self.reader = Some(BufReader::new(stdout));
        self.process = Some(child);

        Ok(())
    }

    /// Polls for the next transcription result
    ///
    /// This is a non-blocking call that returns immediately:
    /// - `Ok(Some(result))` if a new result is available
    /// - `Ok(None)` if no result is ready yet
    /// - `Err(_)` if an error occurred
    ///
    /// Results can be partial (volatile) or final. Check `result.is_final`
    /// to determine if the transcription is complete for that segment.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(StreamingResult))` - New transcription result available
    /// - `Ok(None)` - No new result, try again later
    /// - `Err(String)` - Error occurred during polling
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut transcriber = StreamingTranscriber::new().unwrap();
    /// transcriber.start().unwrap();
    ///
    /// loop {
    ///     match transcriber.poll_result() {
    ///         Ok(Some(result)) => {
    ///             println!("[{}] {}", if result.is_final { "FINAL" } else { "partial" }, result.text);
    ///         }
    ///         Ok(None) => thread::sleep(Duration::from_millis(10)),
    ///         Err(e) => {
    ///             eprintln!("Error: {}", e);
    ///             break;
    ///         }
    ///     }
    /// }
    /// ```
    pub fn poll_result(&mut self) -> Result<Option<StreamingResult>, String> {
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| "Transcriber not started".to_string())?;

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF - process ended
                return Err("Streaming process ended".to_string());
            }
            Ok(_) => {
                let result: StreamingResult = serde_json::from_str(line.trim())
                    .map_err(|e| format!("Failed to parse result: {}", e))?;
                Ok(Some(result))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available yet
                Ok(None)
            }
            Err(e) => Err(format!("Failed to read from helper: {}", e)),
        }
    }

    /// Stops the streaming transcription and cleans up resources
    ///
    /// Terminates the helper process and releases all resources.
    /// After calling this, you must call `start()` again to resume transcription.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    ///
    /// let mut transcriber = StreamingTranscriber::new().unwrap();
    /// transcriber.start().unwrap();
    /// // ... do transcription ...
    /// transcriber.stop().unwrap();
    /// ```
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process
                .kill()
                .map_err(|e| format!("Failed to kill helper process: {}", e))?;
            process
                .wait()
                .map_err(|e| format!("Failed to wait for helper process: {}", e))?;
        }

        self.reader = None;
        Ok(())
    }

    /// Returns the path to the helper binary being used
    pub fn helper_path(&self) -> &Path {
        &self.helper_path
    }

    /// Checks if the transcription is currently running
    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

impl Drop for StreamingTranscriber {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
