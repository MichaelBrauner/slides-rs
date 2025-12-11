//! MiniJinja template extensions (filters and functions)

pub(crate) mod filters;
mod functions;

pub use filters::{dump_filter, make_trans_filter, markdown_filter};
pub use functions::{asset_function, dump_function, make_source_function};

use minijinja::{AutoEscape, Environment};
use std::collections::HashMap;
use std::sync::Arc;

fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.set_auto_escape_callback(|_| AutoEscape::None);
    env
}

pub fn setup_environment(
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
) -> Environment<'static> {
    let mut env = create_environment();

    let translations_arc = Arc::new(translations.clone());
    env.add_filter("trans", make_trans_filter(translations_arc));
    env.add_filter("markdown", markdown_filter);
    env.add_filter("dump", dump_filter);

    env.add_function("dump", dump_function);
    env.add_function("asset", asset_function);
    env.add_function("source", make_source_function());

    for (name, content) in layouts {
        if let Err(e) = env.add_template_owned(name.clone(), content.clone()) {
            eprintln!("⚠️  Warning: Could not load template '{}': {}", name, e);
        }
    }

    env
}
