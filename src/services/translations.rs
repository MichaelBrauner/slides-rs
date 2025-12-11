//! Translation module: Simple i18n for slides

use crate::error::{Error, Result};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Load and flatten translations: { intro: { title: "Hello" } } -> { "intro.title": "Hello" }
pub fn load(locale: &str, translations_dir: &Path) -> Result<HashMap<String, String>> {
    let path = translations_dir.join(format!("{}.yaml", locale));

    if !path.exists() {
        return Err(Error::TranslationNotFound(path));
    }

    let content = fs::read_to_string(&path).map_err(|e| Error::FileRead {
        path: path.clone(),
        source: e,
    })?;

    let data: Value = serde_yaml::from_str(&content).map_err(|e| Error::YamlParse {
        path: path.clone(),
        message: e.to_string(),
    })?;

    let mut result = HashMap::new();
    flatten_value("", &data, &mut result);
    Ok(result)
}

fn flatten_value(prefix: &str, value: &Value, result: &mut HashMap<String, String>) {
    match value {
        Value::Mapping(map) => {
            for (key, val) in map {
                if let Value::String(key_str) = key {
                    let new_prefix = if prefix.is_empty() {
                        key_str.clone()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };
                    flatten_value(&new_prefix, val, result);
                }
            }
        }
        Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        Value::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        Value::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        _ => {}
    }
}
