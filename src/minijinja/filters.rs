//! MiniJinja filters for template rendering

use crate::util::html_escape;
use minijinja::{Error, ErrorKind, Value};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::sync::Arc;

/// Usage: {{ "key" | trans }} or {{ "key" | trans(name="World") }}
pub fn make_trans_filter(
    translations: Arc<HashMap<String, String>>,
) -> impl Fn(&Value, Option<Value>) -> Result<Value, Error> + Send + Sync + 'static {
    move |key: &Value, params: Option<Value>| {
        let key_str = key
            .as_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "trans key must be a string"))?;

        let Some(result) = translations.get(key_str).cloned() else {
            return Ok(Value::UNDEFINED);
        };

        let result = params.map_or(result.clone(), |p| substitute_params(result, &p));
        Ok(Value::from(result))
    }
}

fn substitute_params(mut text: String, params: &Value) -> String {
    let Ok(iter) = params.try_iter() else {
        return text;
    };
    for key in iter {
        let Some(name) = key.as_str() else { continue };
        let Ok(value) = params.get_item(&key) else {
            continue;
        };
        text = text.replace(&format!("{{{}}}", name), &value.to_string());
    }
    text
}

/// Usage: {{ some_var | markdown }}
pub fn markdown_filter(value: &Value) -> Result<Value, Error> {
    let text = value.as_str().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidOperation,
            "markdown filter expects a string",
        )
    })?;
    Ok(Value::from_safe_string(markdown_to_html(text)))
}

/// Usage: {{ variable | dump }}
pub fn dump_filter(value: &Value) -> Result<Value, Error> {
    Ok(Value::from_safe_string(format_dump(value)))
}

pub fn markdown_to_html(text: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(text, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Remove wrapping <p> tags for inline content
    let trimmed = html_output.trim();
    if trimmed.starts_with("<p>")
        && trimmed.ends_with("</p>")
        && trimmed.matches("<p>").count() == 1
    {
        trimmed[3..trimmed.len() - 4].to_string()
    } else {
        html_output
    }
}

pub fn format_dump(value: &Value) -> String {
    let json_output =
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{:#?}", value));
    format!("<pre><code>{}</code></pre>", html_escape(&json_output))
}
