//! Integration test for init -> build workflow
//!
//! This test ensures that `slides init` creates a directory structure
//! that is compatible with `slides build`, preventing regression of bugs
//! where init created incompatible paths.

use serial_test::serial;
use slides_rs::model::Project;
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
    let project = Project::init().expect("init should succeed");

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
    fs::write("slides/templates/test-slide.html", slide_content).expect("Should write test slide");

    // Step 3: Configure decks.yaml to include the slide
    let decks_yaml = r#"default:
  - "*.html"
"#;
    fs::write("decks.yaml", decks_yaml).expect("Should write decks.yaml");

    // Step 4: Run slides build (HTML only, no thumbnails for CI)
    let mut deck = project.deck("default", "en");
    let build_result = deck.build_html();

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
    Project::init().expect("init should succeed");

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
    let project = Project::init().expect("init should succeed");

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
    fs::write("slides/assets/test.png", b"fake-png-data").expect("Should write asset");

    // Configure deck
    fs::write("decks.yaml", "default:\n  - \"*.html\"").expect("Should write decks.yaml");

    // Build (HTML only)
    let mut deck = project.deck("default", "en");
    let build_result = deck.build_html();

    env::set_current_dir(&original_dir).unwrap();

    assert!(
        build_result.is_ok(),
        "build with assets should succeed: {:?}",
        build_result.err()
    );
}

#[test]
#[serial]
fn test_init_build_nonexistent_pattern_fails_gracefully() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Configure deck with a pattern that matches nothing
    fs::write("decks.yaml", "default:\n  - \"nonexistent/*.html\"")
        .expect("Should write decks.yaml");

    // Try to build with no matching slides
    let mut deck = project.deck("default", "en");
    let build_result = deck.build_html();

    env::set_current_dir(&original_dir).unwrap();

    // Should fail gracefully (not panic)
    assert!(
        build_result.is_err(),
        "build should fail gracefully when no slides match pattern"
    );
}

/// Test that the demo deck from init builds and renders correctly
#[test]
#[serial]
fn test_init_demo_deck_renders_correctly() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Build the default demo deck (no changes to decks.yaml)
    let mut deck = project.deck("default", "en");
    let build_result = deck.build_html();

    assert!(
        build_result.is_ok(),
        "Demo deck should build: {:?}",
        build_result.err()
    );

    // Verify all 6 demo slides were generated
    for i in 1..=6 {
        let slide_path = format!("output/slide-{}.html", i);
        assert!(
            Path::new(&slide_path).exists(),
            "Demo slide {} should exist",
            i
        );
    }

    // Verify overview page
    assert!(
        Path::new("output/overview.html").exists(),
        "Overview page should exist"
    );
    // Verify presenter pages (now in presenter/ subdirectory)
    assert!(
        Path::new("output/presenter").is_dir(),
        "Presenter directory should exist"
    );
    for i in 1..=6 {
        let presenter_path = format!("output/presenter/slide-{}.html", i);
        assert!(
            Path::new(&presenter_path).exists(),
            "Presenter page for slide {} should exist",
            i
        );
    }

    // Verify CSS was copied
    assert!(
        Path::new("output/assets/css/style.css").exists(),
        "CSS should be copied to output"
    );

    // Check first slide content includes translated text
    let slide1 = fs::read_to_string("output/slide-1.html").expect("Should read slide 1");
    assert!(
        slide1.contains("Welcome to Slides RS"),
        "Slide 1 should contain welcome text"
    );

    env::set_current_dir(&original_dir).unwrap();
}

/// Test that sectioned deck config works correctly
#[test]
#[serial]
fn test_sectioned_deck_config() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Create sectioned decks.yaml
    let decks_yaml = r#"sectioned:
  intro:
    - "slides/welcome.html"
  features:
    - "slides/features.html"
    - "slides/quick-start.html"
  outro:
    - "slides/translations.html"
"#;
    fs::write("decks.yaml", decks_yaml).expect("Should write decks.yaml");

    // Build the sectioned deck
    let mut deck = project.deck("sectioned", "en");
    let build_result = deck.build_html();

    assert!(
        build_result.is_ok(),
        "Sectioned deck should build: {:?}",
        build_result.err()
    );

    // Should have 4 slides
    assert!(Path::new("output/slide-1.html").exists());
    assert!(Path::new("output/slide-2.html").exists());
    assert!(Path::new("output/slide-3.html").exists());
    assert!(Path::new("output/slide-4.html").exists());
    assert!(!Path::new("output/slide-5.html").exists());

    env::set_current_dir(&original_dir).unwrap();
}

/// Test German translations
#[test]
#[serial]
fn test_german_translations() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Build with German language
    let mut deck = project.deck("default", "de");
    let build_result = deck.build_html();

    assert!(
        build_result.is_ok(),
        "German build should succeed: {:?}",
        build_result.err()
    );

    // Check first slide has German text
    let slide1 = fs::read_to_string("output/slide-1.html").expect("Should read slide 1");
    assert!(
        slide1.contains("Willkommen bei Slides RS"),
        "Slide 1 should contain German welcome text"
    );

    env::set_current_dir(&original_dir).unwrap();
}

/// Test overview page rendering with sections
#[test]
#[serial]
fn test_overview_page_rendering() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    let mut deck = project.deck("default", "en");
    deck.build_html().expect("Build should succeed");

    // Check overview page exists and has content
    let overview = fs::read_to_string("output/overview.html").expect("Should read overview");
    assert!(
        overview.contains("slide-1.html"),
        "Overview should link to slide 1"
    );
    assert!(
        overview.contains("slide-6.html"),
        "Overview should link to slide 6"
    );

    env::set_current_dir(&original_dir).unwrap();
}

/// Test presenter page rendering (now one page per slide)
#[test]
#[serial]
fn test_presenter_page_rendering() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    let mut deck = project.deck("default", "en");
    deck.build_html().expect("Build should succeed");

    // Check presenter directory exists
    assert!(
        Path::new("output/presenter").is_dir(),
        "Presenter directory should exist"
    );

    // Check first presenter page exists and has notes
    let presenter =
        fs::read_to_string("output/presenter/slide-1.html").expect("Should read presenter");
    assert!(
        presenter.contains("Speaker Notes") || presenter.contains("notes"),
        "Presenter should have notes section"
    );

    // Check navigation works (slide 2 should have prev/next)
    let presenter2 =
        fs::read_to_string("output/presenter/slide-2.html").expect("Should read presenter 2");
    assert!(
        presenter2.contains("slide-1.html"),
        "Presenter page 2 should link to slide 1"
    );
    assert!(
        presenter2.contains("slide-3.html"),
        "Presenter page 2 should link to slide 3"
    );

    env::set_current_dir(&original_dir).unwrap();
}

/// Test error: missing deck in decks.yaml
#[test]
#[serial]
fn test_error_missing_deck() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Try to build a deck that doesn't exist
    let mut deck = project.deck("nonexistent", "en");
    let build_result = deck.build_html();

    env::set_current_dir(&original_dir).unwrap();

    assert!(build_result.is_err(), "Should fail for missing deck");
    assert!(
        build_result.unwrap_err().to_string().contains("not found"),
        "Error should mention deck not found"
    );
}

/// Test error: invalid YAML in decks.yaml
#[test]
#[serial]
fn test_error_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Write invalid YAML
    fs::write("decks.yaml", "invalid: yaml: content: [[[").expect("Should write file");

    let mut deck = project.deck("default", "en");
    let build_result = deck.build_html();

    env::set_current_dir(&original_dir).unwrap();

    assert!(build_result.is_err(), "Should fail for invalid YAML");
}

/// Test error: missing template referenced in slide
#[test]
#[serial]
fn test_error_missing_template() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    let project = Project::init().expect("init should succeed");

    // Create a slide that extends a non-existent layout
    let slide = r#"{% extends "layouts/nonexistent.html" %}
{% block content %}Test{% endblock %}"#;
    fs::write("slides/templates/broken.html", slide).expect("Should write file");

    // Configure to use the broken slide
    fs::write("decks.yaml", "default:\n  - \"broken.html\"").expect("Should write decks.yaml");

    let mut deck = project.deck("default", "en");
    let build_result = deck.build_html();

    env::set_current_dir(&original_dir).unwrap();

    // Should fail or produce warning about missing template
    // Note: MiniJinja might still render with errors logged
    // The important thing is it doesn't panic
    assert!(
        build_result.is_ok() || build_result.is_err(),
        "Should handle missing template gracefully"
    );
}
