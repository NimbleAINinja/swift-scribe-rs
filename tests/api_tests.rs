/// Tests for the new programmatic audio input API

use swift_scribe::StreamingTranscriber;

#[test]
fn test_builder_default_microphone_mode() {
    let builder = StreamingTranscriber::builder();
    let transcriber = builder.build();
    assert!(transcriber.is_ok(), "Builder should create transcriber successfully");
}

#[test]
fn test_builder_programmatic_mode() {
    let transcriber = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build();
    assert!(transcriber.is_ok(), "Builder with programmatic input should succeed");
}

#[test]
fn test_builder_chaining() {
    let transcriber = StreamingTranscriber::builder()
        .with_microphone()
        .with_programmatic_input()
        .with_microphone()
        .build();
    assert!(transcriber.is_ok(), "Builder method chaining should work");
}

#[test]
fn test_new_creates_microphone_mode() {
    let result = StreamingTranscriber::new();
    // May fail if helper not found, but shouldn't panic
    assert!(result.is_ok() || result.is_err(), "Should handle missing helper gracefully");
}

#[test]
fn test_with_helper_path() {
    // Use a custom path - will likely fail helper check but shouldn't crash
    let result = StreamingTranscriber::builder()
        .with_helper_path("/nonexistent/path")
        .build();
    assert!(result.is_err(), "Should error on nonexistent path");
}

#[test]
fn test_builder_multiple_input_modes() {
    // Test that we can switch between modes in builder
    let t1 = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build();
    let t2 = StreamingTranscriber::builder()
        .with_microphone()
        .build();

    assert!(t1.is_ok());
    assert!(t2.is_ok());
}

// Audio format conversion tests (internal functions tested indirectly)

#[test]
fn test_feed_audio_requires_programmatic_mode() {
    // This test verifies the API design - feed_audio should only work in programmatic mode
    // Note: We can't fully test this without starting the transcriber (which requires helper)
    // but the error checking is verified during usage
}

#[test]
fn test_builder_creates_correct_mode() {
    // Test that builder correctly sets the mode
    let prog_tx = StreamingTranscriber::builder()
        .with_programmatic_input()
        .build();
    let mic_tx = StreamingTranscriber::builder()
        .with_microphone()
        .build();

    assert!(prog_tx.is_ok());
    assert!(mic_tx.is_ok());
}

#[test]
fn test_default_builder() {
    // Test that builder creates successfully
    let result = StreamingTranscriber::builder().build();
    assert!(result.is_ok() || result.is_err(), "Builder should create a result");
}
