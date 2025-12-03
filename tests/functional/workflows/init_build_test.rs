//! Integration test for init -> build workflow
//!
//! This test ensures that `slides init` creates a directory structure
//! that is compatible with `slides build`, preventing regression of bugs
//! where init created incompatible paths.

use serial_test::serial;
use slides_rs::model::Deck;
use slides_rs::project;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
#[serial]
fn test_init_then_build_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    // Step 1: Run slides init
    env::set_current_dir(temp_dir.path()).unwrap();
    project::init().expect("init should succeed");

    // Step 2: Create a minimal slide
    let slide_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Slide</title>
</head>
<body>
    <h1>Hello World</h1>
    <p>This is a test slide created after init</p>
</body>
</html>
"#;
    fs::write("slides/templates/test-slide.html", slide_content)
        .expect("Should write test slide");

    // Step 3: Configure decks.yaml to include the slide
    let decks_yaml = r#"default:
  - "*.html"
"#;
    fs::write("decks.yaml", decks_yaml).expect("Should write decks.yaml");

    // Step 4: Run slides build
    let mut deck = Deck::new("default", "en");
    let build_result = deck.build_fast();

    // Cleanup
    env::set_current_dir(&original_dir).unwrap();

    // Assert build succeeded
    assert!(
        build_result.is_ok(),
        "build should succeed after init: {:?}",
        build_result.err()
    );
}

#[test]
#[serial]
fn test_init_creates_structure_compatible_with_build() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    project::init().expect("init should succeed");

    // Verify the structure matches what build expects
    assert!(
        Path::new("slides/templates").exists(),
        "slides/templates/ must exist (required by build)"
    );
    assert!(
        Path::new("slides/assets").exists(),
        "slides/assets/ should exist for asset pipeline"
    );
    assert!(
        Path::new("decks.yaml").exists(),
        "decks.yaml must exist (required by build)"
    );

    env::set_current_dir(&original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_build_with_assets() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    project::init().expect("init should succeed");

    // Create a slide with asset reference
    let slide_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Slide with Assets</title>
</head>
<body>
    <h1>Slide with Image</h1>
    <img src="test.png" alt="Test">
</body>
</html>
"#;
    fs::write("slides/templates/slide-with-assets.html", slide_content)
        .expect("Should write slide");

    // Create a dummy asset
    fs::write("slides/assets/test.png", b"fake-png-data")
        .expect("Should write asset");

    // Configure deck
    fs::write("decks.yaml", "default:\n  - \"*.html\"")
        .expect("Should write decks.yaml");

    // Build
    let mut deck = Deck::new("default", "en");
    let build_result = deck.build_fast();

    env::set_current_dir(&original_dir).unwrap();

    assert!(
        build_result.is_ok(),
        "build with assets should succeed: {:?}",
        build_result.err()
    );
}

#[test]
#[serial]
fn test_init_build_empty_templates_fails_gracefully() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    project::init().expect("init should succeed");

    // Configure deck but don't create any slides
    fs::write("decks.yaml", "default:\n  - \"*.html\"")
        .expect("Should write decks.yaml");

    // Try to build with no slides
    let mut deck = Deck::new("default", "en");
    let build_result = deck.build_fast();

    env::set_current_dir(&original_dir).unwrap();

    // Should fail gracefully (not panic)
    assert!(
        build_result.is_err(),
        "build should fail gracefully when no slides exist"
    );
}
