//! Centralized error handling for slides-rs

use std::path::PathBuf;
use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum Error {
    // IO errors
    #[error("Could not read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Could not write file '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Could not create directory '{path}': {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Could not read directory '{path}': {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("File not found: {0}")]
    NotFound(PathBuf),

    #[error("Could not get current directory: {0}")]
    CurrentDir(std::io::Error),

    // Config errors
    #[error("decks.yaml not found")]
    DecksNotFound,

    #[error("Error parsing decks.yaml: {0}")]
    DecksParseError(String),

    #[error("Deck '{0}' not found in decks.yaml")]
    DeckNotFound(String),

    #[error("No slides found for deck '{0}'")]
    NoSlides(String),

    #[error("Translation file not found: {0}")]
    TranslationNotFound(PathBuf),

    #[error("YAML parse error in '{path}': {message}")]
    YamlParse { path: PathBuf, message: String },

    // Template errors
    #[error("Template '{0}' not found")]
    TemplateNotFound(String),

    #[error("Error rendering template '{template}': {message}")]
    TemplateRender { template: String, message: String },

    // Project errors
    #[error("Directory is not empty. 'slides init' requires an empty directory.")]
    DirNotEmpty,

    #[error("Directory '{0}' already exists")]
    DirExists(PathBuf),

    // Export errors
    #[error("Thumbnails not found. Run 'slides build' first.")]
    ThumbnailsNotFound,

    #[error("No thumbnails found in output/thumbnails/")]
    NoThumbnails,

    #[error("PDF generation error: {0}")]
    PdfGeneration(String),

    #[error("PDF encryption error: {0}")]
    PdfEncryption(String),

    // Browser errors
    #[error("Browser error: {0}")]
    Browser(String),

    // Import errors
    #[error("Invalid PPTX file: {0}")]
    InvalidPptx(String),

    #[error("ZIP error: {0}")]
    ZipError(String),

    // Watch errors
    #[error("Could not initialize file watcher: {0}")]
    WatcherInit(String),

    #[error("Could not watch path '{path}': {message}")]
    WatchPath { path: PathBuf, message: String },
}

/// Result type alias for slides-rs
pub type Result<T> = std::result::Result<T, Error>;
