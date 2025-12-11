//! Library interface for slides-rs

pub mod error;
pub mod infrastructure;
pub mod minijinja;
pub mod model;
pub mod services;
pub mod util;

pub use error::{Error, Result};
