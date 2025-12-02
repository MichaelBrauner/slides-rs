//! Slide - represents a single slide template

use crate::util::{get_template_extension, TEMPLATE_EXTENSIONS};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a single slide (a template file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    /// Path to the template file (relative to templates/)
    pub template: String,

    /// Optional: Section key for translation (e.g. "intro", "advanced")
    pub section_key: Option<String>,
}

impl Slide {
    /// Load a collection of slides matching a glob pattern
    /// Automatically expands .html patterns to include all template extensions
    pub fn load_collection(pattern: &str, section_key: Option<String>) -> Result<Vec<Slide>, String> {
        const TEMPLATES_DIR: &str = "slides/templates";
        let mut slides = Vec::new();
        let mut seen_base_names = std::collections::HashSet::new();

        // Expand pattern to all supported extensions if it ends with .html
        let patterns: Vec<String> = if pattern.ends_with(".html") {
            let base = pattern.trim_end_matches(".html");
            TEMPLATE_EXTENSIONS
                .iter()
                .map(|ext| format!("{}{}", base, ext))
                .collect()
        } else {
            vec![pattern.to_string()]
        };

        for pat in patterns {
            let full_pattern = Path::new(TEMPLATES_DIR).join(&pat);
            let pattern_str = full_pattern
                .to_str()
                .ok_or_else(|| format!("Invalid path: {:?}", full_pattern))?;

            let entries = match glob(pattern_str) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries {
                match entry {
                    Ok(path) => {
                        let path_str = path.to_str().unwrap_or("");
                        if let Some(ext) = get_template_extension(path_str) {
                            // Strip slides/templates/ prefix, keep relative path for layout keys
                            let relative = path
                                .strip_prefix("slides")
                                .unwrap_or(&path);

                            let template = relative
                                .to_str()
                                .ok_or_else(|| format!("Invalid path: {:?}", path))?
                                .to_string();

                            // Get base name without extension for deduplication
                            let base_name = template.trim_end_matches(ext);

                            // Avoid duplicates (e.g., if both foo.html and foo.twig exist)
                            if seen_base_names.insert(base_name.to_string()) {
                                slides.push(Slide {
                                    template,
                                    section_key: section_key.clone(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Warning: Glob error: {}", e);
                    }
                }
            }
        }

        // Sort by template name for consistent ordering
        slides.sort_by(|a, b| a.template.cmp(&b.template));

        Ok(slides)
    }
}
