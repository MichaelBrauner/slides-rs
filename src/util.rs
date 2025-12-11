//! Utility functions shared across modules

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// IMPORTANT: Longer extensions must come first for correct matching!
pub const TEMPLATE_EXTENSIONS: &[&str] = &[
    ".html.twig",
    ".html.jinja2",
    ".html.jinja",
    ".jinja2",
    ".jinja",
    ".twig",
    ".html",
];

pub fn get_template_extension(filename: &str) -> Option<&'static str> {
    TEMPLATE_EXTENSIONS
        .iter()
        .find(|ext| filename.ends_with(*ext))
        .copied()
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|e| Error::CreateDir {
        path: dest.to_path_buf(),
        source: e,
    })?;

    let entries = fs::read_dir(src).map_err(|e| Error::ReadDir {
        path: src.to_path_buf(),
        source: e,
    })?;

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                copy_dir_recursive(&src_path, &dest_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dest_path).map_err(|e| Error::FileWrite {
                    path: dest_path.clone(),
                    source: e,
                })?;
            }
        }
    }

    Ok(())
}

pub fn write_pages(dir: &Path, pages: &HashMap<String, String>) -> Result<()> {
    if pages.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(dir).map_err(|e| Error::CreateDir {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for (name, html) in pages {
        let path = dir.join(name);
        fs::write(&path, html).map_err(|e| Error::FileWrite { path, source: e })?;
    }

    Ok(())
}
