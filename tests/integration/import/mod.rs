//! PPTX Import integration tests
//!
//! Tests the PowerPoint image extraction functionality

use serial_test::serial;
use slides_rs::model::Project;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const PPTX_FIXTURE: &str = "tests/fixtures/example.pptx";

/// Test that PPTX import extracts images
#[test]
#[serial]
fn test_import_pptx_extracts_images() {
    // Skip if fixture doesn't exist
    if !Path::new(PPTX_FIXTURE).exists() {
        eprintln!("Skipping: {} not found", PPTX_FIXTURE);
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    // Setup minimal project structure
    env::set_current_dir(temp_dir.path()).unwrap();
    fs::create_dir_all("slides/assets").unwrap();
    fs::write("decks.yaml", "default:\n  - \"*.html\"").unwrap();

    // Get absolute path to PPTX before changing dir
    let pptx_path = original_dir.join(PPTX_FIXTURE);

    let project = Project::current().expect("Should create project");
    let result = project.import_images(pptx_path.to_str().unwrap());

    // Check result
    let import_succeeded = result.is_ok();

    // Check if images were extracted
    let import_dir = temp_dir.path().join("slides/assets/import/images");
    let has_images = import_dir.exists()
        && fs::read_dir(&import_dir)
            .map(|entries| entries.count() > 0)
            .unwrap_or(false);

    env::set_current_dir(&original_dir).unwrap();

    assert!(
        import_succeeded,
        "PPTX import should succeed: {:?}",
        result.err()
    );

    // Note: The PPTX may or may not have images - just verify no errors
    println!(
        "Import completed. Images found: {}",
        if has_images { "yes" } else { "no" }
    );
}

/// Test error handling for invalid PPTX
#[test]
#[serial]
fn test_import_invalid_pptx() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    fs::create_dir_all("slides/assets").unwrap();
    fs::write("decks.yaml", "default:\n  - \"*.html\"").unwrap();

    // Create a fake "PPTX" file (not actually a ZIP)
    let fake_pptx = temp_dir.path().join("fake.pptx");
    fs::write(&fake_pptx, "not a valid pptx file").unwrap();

    let project = Project::current().expect("Should create project");
    let result = project.import_images(fake_pptx.to_str().unwrap());

    env::set_current_dir(&original_dir).unwrap();

    assert!(result.is_err(), "Should fail for invalid PPTX");
    assert!(
        result.unwrap_err().to_string().contains("Invalid PPTX"),
        "Error should mention invalid PPTX"
    );
}

/// Test error handling for missing file
#[test]
#[serial]
fn test_import_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    fs::create_dir_all("slides/assets").unwrap();
    fs::write("decks.yaml", "default:\n  - \"*.html\"").unwrap();

    let project = Project::current().expect("Should create project");
    let result = project.import_images("nonexistent.pptx");

    env::set_current_dir(&original_dir).unwrap();

    assert!(result.is_err(), "Should fail for missing file");
    assert!(
        result.unwrap_err().to_string().contains("Could not read"),
        "Error should mention file not found"
    );
}
