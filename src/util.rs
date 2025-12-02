//! Utility functions shared across modules

/// Supported template extensions (for IDE support)
/// Supports Symfony-style compound extensions like .html.twig
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

/// Check if filename ends with any supported template extension
pub fn get_template_extension(filename: &str) -> Option<&'static str> {
    for ext in TEMPLATE_EXTENSIONS {
        if filename.ends_with(ext) {
            return Some(ext);
        }
    }
    None
}
