//! Slide - represents a single slide template

use crate::error::Result;
use crate::util::{get_template_extension, TEMPLATE_EXTENSIONS};
use glob::glob;
use log::warn;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub template: String,
    pub section_key: Option<String>,
}

impl Slide {
    /// Load a collection of slides matching a glob pattern
    /// Automatically expands .html patterns to include all template extensions
    pub(crate) fn load_collection(
        pattern: &str,
        section_key: Option<String>,
        templates_dir: &Path,
    ) -> Result<Vec<Slide>> {
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
            // Normalize pattern separators for the current OS
            let normalized_pat = pat.replace('/', std::path::MAIN_SEPARATOR_STR);
            let full_pattern = templates_dir.join(&normalized_pat);
            let Some(pattern_str) = full_pattern.to_str() else {
                continue;
            };

            let entries = match glob(pattern_str) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries {
                match entry {
                    Ok(path) => {
                        let path_str = path.to_str().unwrap_or("");
                        let Some(ext) = get_template_extension(path_str) else {
                            continue;
                        };

                        // Strip templates_dir prefix, keep relative path for layout keys
                        let relative = path.strip_prefix(templates_dir).unwrap_or(&path);

                        // Convert to forward slashes for cross-platform compatibility
                        let Some(template) = relative.to_str().map(|s| s.replace('\\', "/")) else {
                            continue;
                        };

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
                    Err(e) => {
                        warn!("Glob error: {e}");
                    }
                }
            }
        }

        // Sort by template name for consistent ordering
        slides.sort_by(|a, b| a.template.cmp(&b.template));

        Ok(slides)
    }
}
