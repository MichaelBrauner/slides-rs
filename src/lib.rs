// Library interface for slides-rs
// This allows integration tests to use the core functionality

pub mod model;
pub mod project;
pub mod render;
pub mod translations;
pub mod util;

// Re-export util functions at crate root for convenience
pub use util::{get_template_extension, TEMPLATE_EXTENSIONS};
