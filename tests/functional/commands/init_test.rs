//! Functional tests for `slides init` command

use serial_test::serial;
use slides_rs::project;
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
    project::init().expect("init should succeed");

    assert!(Path::new("templates").exists(), "Should create templates directory");
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

    let result = project::init();

    assert!(result.is_err(), "Should fail in non-empty directory");
    assert_eq!(
        result.unwrap_err(),
        "Directory is not empty. 'slides init' can only be run in an empty directory."
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

    let result = project::init();

    assert!(result.is_ok(), "Should succeed when only hidden files exist");

    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_creates_valid_decks_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    project::init().expect("init should succeed");

    let content = fs::read_to_string("decks.yaml").unwrap();
    assert_eq!(content, project::INIT_DECKS_FILE);

    env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_init_creates_valid_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().unwrap();

    env::set_current_dir(temp_dir.path()).unwrap();
    project::init().expect("init should succeed");

    let content = fs::read_to_string(".gitignore").unwrap();
    assert_eq!(content, project::INIT_GITIGNORE_FILE);

    env::set_current_dir(original_dir).unwrap();
}
