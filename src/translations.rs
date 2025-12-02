//! Translation module: Simple i18n for slides

use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Holds all translations for a locale
pub struct Translations {
    data: Value,
}

impl Translations {
    /// Load translations from a YAML file
    pub fn load(locale: &str) -> Result<Self, String> {
        let path = format!("slides/translations/{}.yaml", locale);
        let path_ref = Path::new(&path);

        if !path_ref.exists() {
            return Err(format!("Translation file '{}' not found", path));
        }

        let content = fs::read_to_string(path_ref)
            .map_err(|e| format!("Could not read {}: {}", path, e))?;

        let data: Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("YAML parse error in {}: {}", path, e))?;

        Ok(Translations { data })
    }
}

/// Flatten nested YAML translations to flat keys for the trans filter
/// Converts: { intro: { title: "Hello" } } to { "intro.title": "Hello" }
pub fn flatten_translations(translations: &Translations) -> HashMap<String, String> {
    let mut result = HashMap::new();
    flatten_value("", &translations.data, &mut result);
    result
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
