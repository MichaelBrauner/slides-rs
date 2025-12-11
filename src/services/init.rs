//! Init Service - Creates new slides projects

use crate::error::{Error, Result};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;

static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/templates/init");

/// Create all project files in the target directory
pub fn create_project_files(target: &Path) -> Result<()> {
    copy_dir_recursive(&TEMPLATE_DIR, target)?;
    println!("   ✅ Created project files");
    Ok(())
}

fn copy_dir_recursive(dir: &Dir, target: &Path) -> Result<()> {
    for file in dir.files() {
        let file_path = target.join(file.path());
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::CreateDir {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(&file_path, file.contents()).map_err(|e| Error::FileWrite {
            path: file_path.clone(),
            source: e,
        })?;
    }

    for subdir in dir.dirs() {
        copy_dir_recursive(subdir, target)?;
    }

    Ok(())
}

/// Check if directory is empty (ignoring hidden files)
pub fn is_dir_empty(path: &Path) -> Result<bool> {
    let has_files = fs::read_dir(path)
        .map_err(|e| Error::ReadDir {
            path: path.to_path_buf(),
            source: e,
        })?
        .filter_map(|e| e.ok())
        .any(|e| !e.file_name().to_str().is_some_and(|n| n.starts_with('.')));

    Ok(!has_files)
}
