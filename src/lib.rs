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
//! - 🔊 Programmatic audio input (system audio, streams, custom sources)
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
//! ### Programmatic audio input
//!
//! ```no_run
//! use swift_scribe::StreamingTranscriber;
//!
//! let mut transcriber = StreamingTranscriber::builder()
//!     .with_programmatic_input()
//!     .build()
//!     .expect("Failed to create transcriber");
//!
//! transcriber.start().expect("Failed to start transcription");
//!
//! // Feed f32 audio samples (e.g., from system audio capture)
//! loop {
//!     let audio_samples = vec![0.0; 4096]; // Your audio samples
//!     transcriber.feed_audio_f32(&audio_samples, 48000, 2)
//!         .expect("Failed to feed audio");
//!
//!     if let Some(result) = transcriber.poll_result().expect("Failed to poll") {
//!         if result.is_final {
//!             println!("Final: {}", result.text);
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
use std::io::{BufRead, BufReader, Write};
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

/// Audio input mode for streaming transcription
#[derive(Debug, Clone, Copy)]
pub enum AudioInputMode {
    /// Capture audio from the microphone
    Microphone,
    /// Accept audio programmatically via feed_audio methods
    Programmatic,
}

/// Builder for StreamingTranscriber with flexible configuration
pub struct StreamingTranscriberBuilder {
    helper_path: Option<PathBuf>,
    input_mode: AudioInputMode,
}

impl StreamingTranscriberBuilder {
    /// Creates a new builder with default settings (microphone input)
    pub fn new() -> Self {
        Self {
            helper_path: None,
            input_mode: AudioInputMode::Microphone,
        }
    }

    /// Set the input mode to microphone (default)
    pub fn with_microphone(mut self) -> Self {
        self.input_mode = AudioInputMode::Microphone;
        self
    }

    /// Set the input mode to programmatic (feed audio via API)
    pub fn with_programmatic_input(mut self) -> Self {
        self.input_mode = AudioInputMode::Programmatic;
        self
    }

    /// Set a custom path to the helper binary
    pub fn with_helper_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.helper_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Build the StreamingTranscriber
    pub fn build(self) -> Result<StreamingTranscriber, String> {
        let helper_path = if let Some(path) = self.helper_path {
            if !path.exists() {
                return Err(format!(
                    "Streaming helper binary not found at: {}",
                    path.display()
                ));
            }
            path
        } else {
            let default_paths = vec![
                PathBuf::from("./helpers/transcribe_stream"),
                dirs::home_dir()
                    .map(|h| h.join(".local/bin/transcribe_stream"))
                    .unwrap_or_default(),
                PathBuf::from("/usr/local/bin/transcribe_stream"),
            ];

            let mut found = None;
            for path in default_paths {
                if path.exists() {
                    found = Some(path);
                    break;
                }
            }

            found.ok_or_else(|| {
                "Streaming helper binary not found. Please compile with 'make helpers'.".to_string()
            })?
        };

        Ok(StreamingTranscriber {
            helper_path,
            input_mode: self.input_mode,
            process: None,
            reader: None,
            stdin: None,
        })
    }
}

impl Default for StreamingTranscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
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
    input_mode: AudioInputMode,
    process: Option<Child>,
    reader: Option<BufReader<std::process::ChildStdout>>,
    stdin: Option<std::process::ChildStdin>,
}

impl StreamingTranscriber {
    /// Creates a new builder for configuring a StreamingTranscriber
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    ///
    /// let transcriber = StreamingTranscriber::builder()
    ///     .with_programmatic_input()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> StreamingTranscriberBuilder {
        StreamingTranscriberBuilder::new()
    }

    /// Creates a new streaming transcriber with default settings (microphone input)
    ///
    /// This is a convenience method equivalent to `StreamingTranscriber::builder().build()`.
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
        Self::builder().build()
    }

    /// Creates a new streaming transcriber with a custom helper binary path and microphone input
    ///
    /// This is a convenience method equivalent to `StreamingTranscriber::builder().with_helper_path(path).build()`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the transcribe_stream helper binary
    ///
    /// # Errors
    ///
    /// Returns an error if the specified path does not exist.
    pub fn with_helper_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        Self::builder().with_helper_path(path).build()
    }

    /// Starts the streaming transcription
    ///
    /// - For microphone input: Launches the helper process and begins capturing from the microphone
    /// - For programmatic input: Launches the helper in stdin mode, ready to receive audio samples
    ///
    /// Call `poll_result()` to retrieve transcription results.
    /// For programmatic input, call `feed_audio_*()` methods to send audio samples.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The helper process fails to start
    /// - Permissions haven't been granted (for microphone input)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    ///
    /// // Microphone input
    /// let mut transcriber = StreamingTranscriber::new().unwrap();
    /// transcriber.start().unwrap();
    ///
    /// // Programmatic input
    /// let mut transcriber = StreamingTranscriber::builder()
    ///     .with_programmatic_input()
    ///     .build()
    ///     .unwrap();
    /// transcriber.start().unwrap();
    /// ```
    pub fn start(&mut self) -> Result<(), String> {
        let mut cmd = Command::new(&self.helper_path);
        cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());

        match self.input_mode {
            AudioInputMode::Microphone => {}
            AudioInputMode::Programmatic => {
                cmd.arg("--stdin").stdin(Stdio::piped());
            }
        }

        let mut child = cmd.spawn().map_err(|e| {
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

        if matches!(self.input_mode, AudioInputMode::Programmatic) {
            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| "Failed to capture stdin".to_string())?;
            self.stdin = Some(stdin);
        }

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

    /// Feeds i16 PCM audio samples to the transcriber
    ///
    /// Only available when using programmatic audio input mode.
    /// Audio is automatically resampled to 16kHz and converted to mono if needed.
    ///
    /// # Arguments
    ///
    /// * `samples` - Audio samples in i16 PCM format
    /// * `sample_rate` - Sample rate in Hz (e.g., 16000, 48000)
    /// * `channels` - Number of audio channels (1 for mono, 2 for stereo, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transcriber is in microphone mode (not programmatic)
    /// - Transcriber hasn't been started
    /// - Writing to the helper process fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    ///
    /// let mut transcriber = StreamingTranscriber::builder()
    ///     .with_programmatic_input()
    ///     .build()
    ///     .unwrap();
    /// transcriber.start().unwrap();
    ///
    /// let samples = vec![0i16; 4096];
    /// transcriber.feed_audio_i16(&samples, 48000, 2).unwrap();
    /// ```
    pub fn feed_audio_i16(&mut self, samples: &[i16], sample_rate: u32, channels: u16) -> Result<(), String> {
        if !matches!(self.input_mode, AudioInputMode::Programmatic) {
            return Err("feed_audio_i16 can only be used with programmatic input mode".to_string());
        }

        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| "Transcriber not started".to_string())?;

        let resampled = Self::resample_i16(samples, sample_rate, channels);
        let mono = Self::to_mono_i16(&resampled, channels);

        let bytes: Vec<u8> = mono
            .iter()
            .flat_map(|&sample| sample.to_le_bytes().to_vec())
            .collect();

        stdin
            .write_all(&bytes)
            .map_err(|e| format!("Failed to write audio to helper: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush audio: {}", e))
    }

    /// Feeds f32 audio samples to the transcriber
    ///
    /// Only available when using programmatic audio input mode.
    /// Audio is automatically converted from f32 (-1.0 to 1.0) to i16 PCM,
    /// resampled to 16kHz, and converted to mono if needed.
    ///
    /// # Arguments
    ///
    /// * `samples` - Audio samples in f32 format (range: -1.0 to 1.0)
    /// * `sample_rate` - Sample rate in Hz (e.g., 16000, 48000)
    /// * `channels` - Number of audio channels (1 for mono, 2 for stereo, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transcriber is in microphone mode (not programmatic)
    /// - Transcriber hasn't been started
    /// - Writing to the helper process fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swift_scribe::StreamingTranscriber;
    ///
    /// let mut transcriber = StreamingTranscriber::builder()
    ///     .with_programmatic_input()
    ///     .build()
    ///     .unwrap();
    /// transcriber.start().unwrap();
    ///
    /// let samples = vec![0.0f32; 4096];
    /// transcriber.feed_audio_f32(&samples, 48000, 2).unwrap();
    /// ```
    pub fn feed_audio_f32(&mut self, samples: &[f32], sample_rate: u32, channels: u16) -> Result<(), String> {
        if !matches!(self.input_mode, AudioInputMode::Programmatic) {
            return Err("feed_audio_f32 can only be used with programmatic input mode".to_string());
        }

        let i16_samples = Self::f32_to_i16(samples);
        self.feed_audio_i16(&i16_samples, sample_rate, channels)
    }

    fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * 32767.0) as i16
            })
            .collect()
    }

    fn resample_i16(samples: &[i16], from_rate: u32, _channels: u16) -> Vec<i16> {
        const TARGET_RATE: u32 = 16000;

        if from_rate == TARGET_RATE {
            return samples.to_vec();
        }

        let ratio = TARGET_RATE as f64 / from_rate as f64;
        let output_len = ((samples.len() as f64) * ratio).ceil() as usize;
        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let src_pos = (i as f64) / ratio;
            let src_idx = src_pos as usize;

            if src_idx >= samples.len() {
                break;
            }

            let frac = src_pos - src_idx as f64;

            if src_idx + 1 < samples.len() {
                let s0 = samples[src_idx] as f64;
                let s1 = samples[src_idx + 1] as f64;
                let interpolated = s0 + (s1 - s0) * frac;
                output.push(interpolated.clamp(-32768.0, 32767.0) as i16);
            } else {
                output.push(samples[src_idx]);
            }
        }

        output
    }

    fn to_mono_i16(samples: &[i16], channels: u16) -> Vec<i16> {
        if channels <= 1 {
            return samples.to_vec();
        }

        let channels = channels as usize;
        let frames = samples.len() / channels;
        let mut mono = Vec::with_capacity(frames);

        for frame_idx in 0..frames {
            let mut sum = 0i32;
            for ch in 0..channels {
                sum += samples[frame_idx * channels + ch] as i32;
            }
            let avg = (sum / channels as i32).clamp(-32768, 32767) as i16;
            mono.push(avg);
        }

        mono
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
        self.stdin = None;
        self.reader = None;

        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }

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
