//! End-to-end test for the complete build workflow
//!
//! Tests the full build process using fixtures from tests/fixtures/files/

use serial_test::serial;
use slides_rs::model::Project;
use std::env;
use std::fs;
use std::path::Path;

/// Path to minimal test fixtures
const FIXTURES_PATH: &str = "tests/fixtures/minimal";

/// Helper to run test in the fixtures directory
fn with_fixtures<F>(test: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let original_dir = env::current_dir().expect("Failed to get current dir");

    env::set_current_dir(FIXTURES_PATH).expect("Failed to change to fixtures dir");

    // Clean output before test
    let _ = fs::remove_dir_all("output");

    // Run test with panic recovery to ensure we restore directory
    let result = std::panic::catch_unwind(test);

    // Clean output after test
    let _ = fs::remove_dir_all("output");

    // Always restore directory
    env::set_current_dir(&original_dir).expect("Failed to restore original dir");

    // Re-panic if test failed
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
#[serial]
fn test_full_build_workflow() {
    with_fixtures(|| {
        // Build HTML only (no thumbnails for CI compatibility)
        let project = Project::current().expect("Should open project");
        let mut deck = project.deck("default", "en");
        deck.build_html().expect("build_html() should succeed");

        // Verify output directory exists
        assert!(
            Path::new("output").exists(),
            "output/ directory should exist"
        );

        // Should have slides (fixtures have 3 slides)
        assert!(
            Path::new("output/slide-1.html").exists(),
            "slide-1.html should exist"
        );
        assert!(
            Path::new("output/slide-3.html").exists(),
            "slide-3.html should exist"
        );

        // Verify HTML content
        let slide1_content =
            fs::read_to_string("output/slide-1.html").expect("Should read slide-1.html");

        assert!(
            slide1_content.contains("<!DOCTYPE html>") || slide1_content.contains("<html"),
            "Output should be valid HTML"
        );
    });
}

#[test]
#[serial]
fn test_build_generates_correct_slide_count() {
    with_fixtures(|| {
        let project = Project::current().expect("Should open project");
        let mut deck = project.deck("default", "en");
        deck.build_html().expect("build_html() should succeed");

        // Count generated slide files
        let slide_count = fs::read_dir("output")
            .expect("Should read output dir")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with("slide-") && n.ends_with(".html"))
                    .unwrap_or(false)
            })
            .count();

        // Fixtures have 3 slides
        assert_eq!(slide_count, 3, "Should generate exactly 3 slides");
    });
}

#[test]
#[serial]
fn test_build_includes_navigation() {
    with_fixtures(|| {
        let project = Project::current().expect("Should open project");
        let mut deck = project.deck("default", "en");
        deck.build_html().expect("build_html() should succeed");

        // Middle slide (slide 2) should have both prev and next
        let slide2 = fs::read_to_string("output/slide-2.html").unwrap();
        assert!(
            slide2.contains("slide-1.html"),
            "Slide 2 should link to previous"
        );
        assert!(
            slide2.contains("slide-3.html"),
            "Slide 2 should link to next"
        );
    });
}

#[test]
#[serial]
fn test_build_html_skips_thumbnails() {
    with_fixtures(|| {
        let project = Project::current().expect("Should open project");
        let mut deck = project.deck("default", "en");
        deck.build_html().expect("build_html() should succeed");

        // build_html() should NOT generate thumbnails
        assert!(
            !Path::new("output/thumbnails").exists(),
            "build_html() should skip thumbnail generation"
        );

        // But HTML should still be generated
        assert!(
            Path::new("output/slide-1.html").exists(),
            "HTML slides should still be generated"
        );
    });
}
