//! Functional tests for `slides init` command

use serial_test::serial;
use slides_rs::model::Project;
use slides_rs::Error;
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
#[serial]
fn test_init_creates_required_files() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    Project::init().expect("init should succeed");

    assert!(
        Path::new("slides/templates").exists(),
        "Should create slides/templates directory"
    );
    assert!(
        Path::new("slides/assets").exists(),
        "Should create slides/assets directory"
    );
    assert!(Path::new(".gitignore").exists(), "Should create .gitignore");
    assert!(Path::new("decks.yaml").exists(), "Should create decks.yaml");

    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_fails_if_directory_not_empty() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    fs::write("existing.txt", "content").unwrap();

    let result = Project::init();

    assert!(result.is_err(), "Should fail in non-empty directory");
    assert!(
        matches!(result.unwrap_err(), Error::DirNotEmpty),
        "Error should be DirNotEmpty"
    );

    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_ignores_hidden_files() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    fs::create_dir_all(".git").unwrap();
    fs::write(".git/config", "").unwrap();
    fs::write(".hidden", "").unwrap();

    let result = Project::init();

    assert!(
        result.is_ok(),
        "Should succeed when only hidden files exist"
    );

    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_creates_valid_decks_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    Project::init().expect("init should succeed");

    let content = fs::read_to_string("decks.yaml").unwrap();
    assert!(content.contains("default:"), "Should contain default deck");

    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_creates_valid_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    Project::init().expect("init should succeed");

    let content = fs::read_to_string(".gitignore").unwrap();
    assert!(
        content.contains("output/"),
        "Should ignore output directory"
    );

    env::set_current_dir(original_dir).unwrap();
}
