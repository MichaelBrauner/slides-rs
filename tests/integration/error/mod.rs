//! Tests for centralized error handling

use slides_rs::Error;
use std::path::PathBuf;

#[test]
fn test_error_display_messages() {
    // Test that error messages are user-friendly
    let err = Error::DecksNotFound;
    assert_eq!(err.to_string(), "decks.yaml not found");

    let err = Error::DeckNotFound("my-talk".to_string());
    assert_eq!(err.to_string(), "Deck 'my-talk' not found in decks.yaml");

    let err = Error::NoSlides("empty-deck".to_string());
    assert_eq!(err.to_string(), "No slides found for deck 'empty-deck'");

    let err = Error::DirNotEmpty;
    assert_eq!(
        err.to_string(),
        "Directory is not empty. 'slides init' requires an empty directory."
    );

    let err = Error::DirExists(PathBuf::from("/tmp/existing"));
    assert_eq!(err.to_string(), "Directory '/tmp/existing' already exists");

    let err = Error::ThumbnailsNotFound;
    assert_eq!(
        err.to_string(),
        "Thumbnails not found. Run 'slides build' first."
    );

    let err = Error::NoThumbnails;
    assert_eq!(err.to_string(), "No thumbnails found in output/thumbnails/");

    let err = Error::TemplateNotFound("missing.html".to_string());
    assert_eq!(err.to_string(), "Template 'missing.html' not found");

    let err = Error::TranslationNotFound(PathBuf::from("slides/translations/fr.yaml"));
    assert_eq!(
        err.to_string(),
        "Translation file not found: slides/translations/fr.yaml"
    );
}

#[test]
fn test_error_debug_format() {
    // Ensure errors can be debug-printed (required for Result unwrapping)
    let err = Error::DecksNotFound;
    let debug = format!("{:?}", err);
    assert!(debug.contains("DecksNotFound"));
}

#[test]
fn test_file_errors_include_path() {
    // IO errors should include the file path for debugging
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = Error::FileRead {
        path: PathBuf::from("/path/to/file.txt"),
        source: io_err,
    };
    let msg = err.to_string();
    assert!(msg.contains("/path/to/file.txt"));
    assert!(msg.contains("Could not read"));
}

#[test]
fn test_yaml_parse_error_includes_context() {
    let err = Error::YamlParse {
        path: PathBuf::from("config.yaml"),
        message: "invalid syntax at line 5".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("config.yaml"));
    assert!(msg.contains("invalid syntax"));
}

#[test]
fn test_template_render_error_includes_template_name() {
    let err = Error::TemplateRender {
        template: "intro.html".to_string(),
        message: "undefined variable 'title'".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("intro.html"));
    assert!(msg.contains("undefined variable"));
}
