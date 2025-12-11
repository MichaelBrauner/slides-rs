//! Render module - Template rendering for slides

use crate::error::{Error, Result};
use crate::minijinja::filters::markdown_to_html;
use crate::minijinja::setup_environment;
use crate::model::Slide;
use crate::util::get_template_extension;
use itertools::Itertools;
use log::warn;
use minijinja::{context, Environment, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct SlideNav {
    current: usize,
    prev: Option<usize>,
    next: Option<usize>,
    is_first: bool,
    is_last: bool,
    total: usize,
}

impl SlideNav {
    fn new(index: usize, total: usize) -> Self {
        let current = index + 1;
        Self {
            current,
            prev: (current > 1).then(|| current - 1),
            next: (current < total).then(|| current + 1),
            is_first: current == 1,
            is_last: current == total,
            total,
        }
    }

    fn to_context(self) -> Value {
        context! {
            current => self.current,
            prev => self.prev,
            next => self.next,
            isFirst => self.is_first,
            isLast => self.is_last,
        }
    }

    fn app_context(self) -> Value {
        context! { total => self.total, first => 1, last => self.total }
    }
}

pub fn load_layouts(dir: &Path) -> Result<HashMap<String, String>> {
    let mut layouts = HashMap::new();
    load_layouts_recursive(dir, dir, &mut layouts)?;
    Ok(layouts)
}

fn load_layouts_recursive(
    base_dir: &Path,
    current_dir: &Path,
    layouts: &mut HashMap<String, String>,
) -> Result<()> {
    let entries = fs::read_dir(current_dir).map_err(|e| Error::ReadDir {
        path: current_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            load_layouts_recursive(base_dir, &path, layouts)?;
            continue;
        }

        let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(ext) = get_template_extension(filename) else {
            continue;
        };

        let Ok(relative_path) = path.strip_prefix(base_dir) else {
            continue;
        };
        let Some(relative_str) = relative_path.to_str() else {
            continue;
        };

        // Normalize to forward slashes for cross-platform compatibility
        let normalized_path = relative_str.replace('\\', "/");

        let content = fs::read_to_string(&path).map_err(|e| Error::FileRead {
            path: path.clone(),
            source: e,
        })?;

        register_template(layouts, &normalized_path, ext, content);
    }

    Ok(())
}

/// Register template under original name and normalized .html name
fn register_template(
    layouts: &mut HashMap<String, String>,
    name: &str,
    ext: &str,
    content: String,
) {
    layouts.insert(name.to_string(), content.clone());

    // Also register as .html for templates with other extensions (.twig, .jinja2)
    if ext != ".html" {
        let normalized = name.trim_end_matches(ext).to_string() + ".html";
        layouts.insert(normalized, content);
    }
}

fn try_render(env: &Environment, template: &str, ctx: &Value) -> Option<String> {
    let tmpl = env.get_template(template).ok()?;
    tmpl.render(ctx)
        .map_err(|e| warn!("Error rendering {}: {}", template, e))
        .ok()
}

fn extract_slide_notes(env: &Environment, template: &str, current: usize, total: usize) -> String {
    let ctx = context! {
        app => context! { total => total, first => 1, last => total },
        slide => context! { current => current },
    };

    let Ok(tmpl) = env.get_template(template) else {
        warn!("Template '{}' not found for speaker notes", template);
        return String::new();
    };

    let Ok(mut state) = tmpl.eval_to_state(&ctx) else {
        warn!(
            "Failed to evaluate template '{}' for slide {}",
            template, current
        );
        return String::new();
    };

    state
        .render_block("notes")
        .ok()
        .map(|raw| markdown_to_html(&raw))
        .unwrap_or_default()
}

pub fn render_deck_pages(
    slides: &[Slide],
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut pages = HashMap::new();
    let total = slides.len();
    let env = setup_environment(layouts, translations);

    for (index, slide) in slides.iter().enumerate() {
        let nav = SlideNav::new(index, total);

        let ctx = context! {
            app => nav.app_context(),
            slide => nav.to_context(),
            // Legacy uppercase variables for backwards compatibility
            CURRENT => nav.current,
            TOTAL => nav.total,
            PREV => nav.prev,
            NEXT => nav.next,
        };

        let Ok(tmpl) = env.get_template(&slide.template) else {
            warn!("Template '{}' not found", slide.template);
            continue;
        };

        match tmpl.render(&ctx) {
            Ok(html) => {
                pages.insert(format!("slide-{}.html", nav.current), html);
            }
            Err(e) => warn!("Error rendering slide {}: {}", nav.current, e),
        }
    }

    pages
}

pub fn render_overview(
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
    slides: &[Slide],
) -> Option<String> {
    if !layouts.contains_key("overview.html") {
        warn!("No overview.html template found - skipping overview page");
        return None;
    }

    let total_slides = slides.len();
    let env = setup_environment(layouts, translations);

    let sections: Vec<Value> = slides
        .iter()
        .enumerate()
        .chunk_by(|(_, slide)| slide.section_key.clone())
        .into_iter()
        .map(|(key, group)| {
            let slides: Vec<Value> = group
                .map(|(index, slide)| {
                    context! {
                        number => index + 1,
                        template => slide.template.clone(),
                    }
                })
                .collect();
            context! {
                key => key.unwrap_or_default(),
                slides => slides,
            }
        })
        .collect();

    // Build context
    let ctx = context! {
        app => context! {
            total => total_slides,
            first => 1,
            last => total_slides,
        },
        sections => sections,
    };

    try_render(&env, "overview.html", &ctx)
}

pub fn render_presenter_pages(
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
    slides: &[Slide],
) -> HashMap<String, String> {
    let mut pages = HashMap::new();

    if !layouts.contains_key("presenter.html") {
        warn!("No presenter.html template found - skipping presenter pages");
        return pages;
    }

    let total = slides.len();
    let env = setup_environment(layouts, translations);

    for (index, slide) in slides.iter().enumerate() {
        let nav = SlideNav::new(index, total);
        let notes = extract_slide_notes(&env, &slide.template, nav.current, total);
        let output_path = format!("presenter/slide-{}.html", nav.current);

        let ctx = context! {
            app => nav.app_context(),
            slide => nav.to_context(),
            notes => notes,
            _output_path => output_path,
        };

        match try_render(&env, "presenter.html", &ctx) {
            Some(html) => {
                pages.insert(format!("slide-{}.html", nav.current), html);
            }
            None => warn!("Error rendering presenter page for slide {}", nav.current),
        }
    }

    pages
}

pub fn render_print(
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
    slides: &[Slide],
) -> Option<String> {
    if !layouts.contains_key("print.html") {
        return None;
    }

    let total = slides.len();
    let env = setup_environment(layouts, translations);

    let slides_ctx: Vec<Value> = slides
        .iter()
        .enumerate()
        .map(|(index, slide)| {
            context! {
                number => index + 1,
                template => slide.template.clone(),
            }
        })
        .collect();

    let ctx = context! {
        app => context! {
            total => total,
            first => 1,
            last => total,
        },
        slides => slides_ctx,
    };

    try_render(&env, "print.html", &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minijinja::filters::markdown_to_html;
    use crate::minijinja::{
        asset_function, dump_filter, dump_function, make_trans_filter, markdown_filter,
    };
    use std::sync::Arc;

    fn create_test_env(translations: HashMap<String, String>) -> Environment<'static> {
        let mut env = Environment::new();
        env.add_filter("trans", make_trans_filter(Arc::new(translations)));
        env.add_filter("markdown", markdown_filter);
        env
    }

    #[test]
    fn test_trans_filter_basic() {
        let mut translations = HashMap::new();
        translations.insert("greeting".to_string(), "Hello".to_string());
        translations.insert("slides.intro.title".to_string(), "Welcome".to_string());

        let env = create_test_env(translations);

        // Test simple lookup
        let tmpl = env
            .template_from_str(r#"{{ "greeting" | trans }}"#)
            .unwrap();
        assert_eq!(tmpl.render(context! {}).unwrap(), "Hello");

        // Test dotted key
        let tmpl = env
            .template_from_str(r#"{{ "slides.intro.title" | trans }}"#)
            .unwrap();

        assert_eq!(tmpl.render(context! {}).unwrap(), "Welcome");
    }

    #[test]
    fn test_trans_filter_missing_key_returns_undefined() {
        let env = create_test_env(HashMap::new());

        // Missing key with default fallback
        let tmpl = env
            .template_from_str(r#"{{ "unknown.key" | trans | default("Fallback") }}"#)
            .unwrap();

        assert_eq!(tmpl.render(context! {}).unwrap(), "Fallback");
    }

    #[test]
    fn test_trans_filter_with_params() {
        let mut translations = HashMap::new();
        translations.insert("greeting".to_string(), "Hello, {name}!".to_string());
        translations.insert(
            "welcome".to_string(),
            "{name} welcome to {place}".to_string(),
        );

        let env = create_test_env(translations);

        // Test with single parameter
        let tmpl = env
            .template_from_str(r#"{{ "greeting" | trans(name="World") }}"#)
            .unwrap();
        assert_eq!(tmpl.render(context! {}).unwrap(), "Hello, World!");

        // Test with multiple parameters
        let tmpl = env
            .template_from_str(r#"{{ "welcome" | trans(name="Michael", place="Rust") }}"#)
            .unwrap();
        assert_eq!(tmpl.render(context! {}).unwrap(), "Michael welcome to Rust");
    }

    #[test]
    fn test_trans_filter_chaining_with_default() {
        let mut translations = HashMap::new();
        translations.insert("existing".to_string(), "Found".to_string());

        let env = create_test_env(translations);

        // Test existing key
        let tmpl = env
            .template_from_str(r#"{{ "existing" | trans }}"#)
            .unwrap();
        assert_eq!(tmpl.render(context! {}).unwrap(), "Found");

        // Test missing key with default
        let tmpl = env
            .template_from_str(r#"{{ "missing" | trans | default("Fallback") }}"#)
            .unwrap();
        assert_eq!(tmpl.render(context! {}).unwrap(), "Fallback");

        // Test existing key ignores default
        let tmpl = env
            .template_from_str(r#"{{ "existing" | trans | default("Fallback") }}"#)
            .unwrap();
        assert_eq!(tmpl.render(context! {}).unwrap(), "Found");
    }

    #[test]
    fn test_markdown_bold() {
        assert_eq!(markdown_to_html("**bold**"), "<strong>bold</strong>");
        assert_eq!(
            markdown_to_html("a **bold** word"),
            "a <strong>bold</strong> word"
        );
    }

    #[test]
    fn test_markdown_italic() {
        assert_eq!(markdown_to_html("*italic*"), "<em>italic</em>");
        assert_eq!(
            markdown_to_html("an *italic* word"),
            "an <em>italic</em> word"
        );
    }

    #[test]
    fn test_markdown_code() {
        assert_eq!(markdown_to_html("`code`"), "<code>code</code>");
        assert_eq!(
            markdown_to_html("use `let x = 1`"),
            "use <code>let x = 1</code>"
        );
    }

    #[test]
    fn test_markdown_mixed() {
        assert_eq!(
            markdown_to_html("**bold** and *italic* and `code`"),
            "<strong>bold</strong> and <em>italic</em> and <code>code</code>"
        );
    }

    #[test]
    fn test_markdown_strikethrough() {
        assert_eq!(markdown_to_html("~~deleted~~"), "<del>deleted</del>");
    }

    #[test]
    fn test_markdown_multiline() {
        let input = "# Heading\n\nSome **bold** text.\n\n- item 1\n- item 2";
        let output = markdown_to_html(input);
        assert!(output.contains("<h1>Heading</h1>"));
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<li>item 1</li>"));
    }

    #[test]
    fn test_trans_filter_returns_raw_text() {
        let mut translations = HashMap::new();
        translations.insert(
            "point1".to_string(),
            "**Lehrmedium #1** für alle Kurse".to_string(),
        );

        let env = create_test_env(translations);

        // trans alone returns raw text (no markdown rendering)
        let tmpl = env.template_from_str(r#"{{ "point1" | trans }}"#).unwrap();
        assert_eq!(
            tmpl.render(context! {}).unwrap(),
            "**Lehrmedium #1** für alle Kurse"
        );

        // chain with markdown filter if rendering is needed
        let tmpl = env
            .template_from_str(r#"{{ "point1" | trans | markdown }}"#)
            .unwrap();
        assert_eq!(
            tmpl.render(context! {}).unwrap(),
            "<strong>Lehrmedium #1</strong> für alle Kurse"
        );
    }

    #[test]
    fn test_markdown_filter_standalone() {
        let env = create_test_env(HashMap::new());

        let tmpl = env.template_from_str(r#"{{ text | markdown }}"#).unwrap();
        let result = tmpl.render(context! { text => "**bold** text" }).unwrap();
        assert_eq!(result, "<strong>bold</strong> text");
    }

    #[test]
    fn test_dump_filter() {
        let mut env = Environment::new();
        env.add_filter("dump", dump_filter);

        let tmpl = env.template_from_str(r#"{{ data | dump }}"#).unwrap();
        let result = tmpl
            .render(context! { data => context! { name => "test", count => 42 } })
            .unwrap();

        // Should contain a pre/code block
        assert!(result.contains("<pre><code>"));
        assert!(result.contains("</code></pre>"));
        // Should contain the debug output
        assert!(result.contains("name"));
        assert!(result.contains("test"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_dump_function_with_argument() {
        let mut env = Environment::new();
        env.add_function("dump", dump_function);
        env.add_function("asset", asset_function);

        let tmpl = env.template_from_str(r#"{{ dump(data) }}"#).unwrap();
        let result = tmpl
            .render(context! { data => context! { foo => "bar" } })
            .unwrap();

        assert!(result.contains("<pre><code>"));
        assert!(result.contains("foo"));
        assert!(result.contains("bar"));
    }

    #[test]
    fn test_dump_function_without_argument() {
        let mut env = Environment::new();
        env.add_function("dump", dump_function);
        env.add_function("asset", asset_function);

        let tmpl = env.template_from_str(r#"{{ dump() }}"#).unwrap();
        let result = tmpl
            .render(context! { app => context! { total => 5 }, slide => context! { current => 1 } })
            .unwrap();

        // Should show all available variables as JSON (quotes are HTML-escaped)
        assert!(result.contains("&quot;app&quot;"));
        assert!(result.contains("&quot;slide&quot;"));
        assert!(result.contains("&quot;total&quot;: 5"));
        assert!(result.contains("&quot;current&quot;: 1"));
    }
}
