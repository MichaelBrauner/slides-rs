//! MiniJinja functions for template rendering

use super::filters::format_dump;
use crate::util::html_escape;
use log::warn;
use minijinja::{Error, ErrorKind, State, Value};
use std::fs;
use std::path::Path;

/// Usage: {{ dump() }} or {{ dump(variable) }}
pub fn dump_function(state: &State, value: Option<Value>) -> Result<Value, Error> {
    match value {
        Some(v) => Ok(Value::from_safe_string(format_dump(&v))),
        None => {
            let mut context_map = serde_json::Map::new();
            for name in ["app", "slide", "CURRENT", "TOTAL", "PREV", "NEXT"] {
                let Some(val) = state.lookup(name) else {
                    continue;
                };
                let Ok(json_val) = serde_json::to_value(&val) else {
                    continue;
                };
                context_map.insert(name.to_string(), json_val);
            }
            let json_output =
                serde_json::to_string_pretty(&context_map).unwrap_or_else(|_| "{}".to_string());
            Ok(Value::from_safe_string(format!(
                "<pre><code>{}</code></pre>",
                html_escape(&json_output)
            )))
        }
    }
}

/// Usage: {{ asset('logo.png') }} -> "assets/logo.png"
pub fn asset_function(state: &State, filename: String) -> String {
    if filename.is_empty() {
        warn!("asset() called with empty filename");
        return "assets/".to_string();
    }

    let depth = match state.lookup("_output_path") {
        Some(val) => val.as_str().map(|p| p.matches('/').count()).unwrap_or(0),
        None => 0,
    };

    let prefix = "../".repeat(depth);
    format!("{}assets/{}", prefix, filename)
}

/// Usage: {{ source("code/example.js") }}
pub fn make_source_function() -> impl Fn(String) -> Result<String, Error> + Send + Sync {
    move |name: String| {
        let path = format!("slides/templates/{}", name);
        if Path::new(&path).exists() {
            return fs::read_to_string(&path).map_err(|e| {
                Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Could not read {path}: {e}"),
                )
            });
        }
        Err(Error::new(
            ErrorKind::InvalidOperation,
            format!("Template not found: {path}"),
        ))
    }
}
