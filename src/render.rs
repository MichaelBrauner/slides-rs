//! Render module - Template rendering for slides

use crate::model::Slide;
use crate::util::get_template_extension;
use minijinja::{context, AutoEscape, Environment, Error, ErrorKind, State, Value};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Create a new MiniJinja environment with standard configuration
fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    // Disable auto-escaping - slides contain trusted HTML content
    env.set_auto_escape_callback(|_| AutoEscape::None);
    env
}

/// Convert markdown to HTML using pulldown-cmark
fn markdown_to_html(text: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(text, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Remove wrapping <p> tags for inline content (single line without block elements)
    let trimmed = html_output.trim();
    if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") && trimmed.matches("<p>").count() == 1
    {
        trimmed[3..trimmed.len() - 4].to_string()
    } else {
        html_output
    }
}

/// Load all template files recursively from a directory
pub fn load_layouts(dir: &Path) -> Result<HashMap<String, String>, String> {
    let mut layouts = HashMap::new();
    load_layouts_recursive(dir, dir, &mut layouts)?;
    Ok(layouts)
}

fn load_layouts_recursive(
    base_dir: &Path,
    current_dir: &Path,
    layouts: &mut HashMap<String, String>,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir)
        .map_err(|e| format!("Could not read directory {:?}: {}", current_dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error reading entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // Skip common non-template directories
            if matches!(
                dir_name,
                "node_modules" | "dist" | "output" | ".git" | ".idea" | "target"
            ) {
                continue;
            }
            load_layouts_recursive(base_dir, &path, layouts)?;
        } else if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if let Some(ext) = get_template_extension(filename) {
                let relative_path = path
                    .strip_prefix(base_dir)
                    .map_err(|e| format!("Error creating relative path: {}", e))?;

                let relative_str = relative_path
                    .to_str()
                    .ok_or_else(|| "Invalid path".to_string())?;

                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Could not read {:?}: {}", path, e))?;

                // Register template under its original name (e.g., "templates/layouts/base.html.twig")
                layouts.insert(relative_str.to_string(), content.clone());

                // Also register without "templates/" prefix for cleaner extends/includes
                // e.g., "templates/layouts/base.html.twig" -> "layouts/base.html.twig"
                if relative_str.starts_with("templates/") {
                    let short_name = relative_str.strip_prefix("templates/").unwrap();
                    layouts.insert(short_name.to_string(), content.clone());
                }

                // Also register under normalized .html name for backwards compatibility
                if ext != ".html" {
                    let normalized_name = relative_str.trim_end_matches(ext).to_string() + ".html";
                    layouts.insert(normalized_name.clone(), content.clone());

                    // And the short version without templates/ prefix
                    if normalized_name.starts_with("templates/") {
                        let short_normalized = normalized_name.strip_prefix("templates/").unwrap();
                        layouts.insert(short_normalized.to_string(), content);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Create a trans filter function that looks up translations
/// Automatically renders inline markdown (**bold**, *italic*, `code`)
/// Usage in templates:
///   {{ "slides.intro.title" | trans }}
///   {{ "slides.intro.title" | trans | default("Fallback") }}
///   {{ "greeting" | trans(name="World") }}
fn make_trans_filter(
    translations: Arc<HashMap<String, String>>,
) -> impl Fn(&Value, Option<Value>) -> Result<Value, Error> + Send + Sync + 'static {
    move |key: &Value, params: Option<Value>| {
        let key_str = key
            .as_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "trans key must be a string"))?;

        // Look up translation, return UNDEFINED if not found (allows chaining with | default)
        let Some(mut result) = translations.get(key_str).cloned() else {
            return Ok(Value::UNDEFINED);
        };

        // Handle parameter interpolation: {{ "hello" | trans(name="World") }}
        // Replaces {name} with the value
        if let Some(params) = params {
            if let Ok(iter) = params.try_iter() {
                for param_key in iter {
                    if let Some(param_name) = param_key.as_str() {
                        if let Ok(param_value) = params.get_item(&param_key) {
                            let placeholder = format!("{{{}}}", param_name);
                            result = result.replace(&placeholder, &param_value.to_string());
                        }
                    }
                }
            }
        }

        // Apply markdown rendering
        let html = markdown_to_html(&result);
        Ok(Value::from_safe_string(html))
    }
}

/// Standalone markdown filter for use outside translations
/// Usage: {{ some_var | markdown }}
fn markdown_filter(value: &Value) -> Result<Value, Error> {
    let text = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "markdown filter expects a string"))?;
    Ok(Value::from_safe_string(markdown_to_html(text)))
}

/// Debug dump filter - outputs variable contents in a styled HTML block
/// Usage: {{ variable | dump }}
fn dump_filter(value: &Value) -> Result<Value, Error> {
    Ok(Value::from_safe_string(format_dump(value)))
}

/// Debug dump function - outputs variable or full context
/// Usage: {{ dump() }} or {{ dump(variable) }}
fn dump_function(state: &State, value: Option<Value>) -> Result<Value, Error> {
    match value {
        Some(v) => Ok(Value::from_safe_string(format_dump(&v))),
        None => {
            // Collect known variables from the current scope into a JSON object
            let mut context_map = serde_json::Map::new();
            let known_vars = ["app", "slide"];
            for name in known_vars {
                if let Some(val) = state.lookup(name) {
                    if let Ok(json_val) = serde_json::to_value(&val) {
                        context_map.insert(name.to_string(), json_val);
                    }
                }
            }
            let json_output = serde_json::to_string_pretty(&context_map)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(Value::from_safe_string(format!(
                r#"<pre style="background:#1e1e2e;color:#a6e3a1;padding:1rem;border-radius:8px;font-size:0.85rem;overflow-x:auto;border-left:4px solid #a6e3a1;margin:1rem 0;text-align:left;"><code>{}</code></pre>"#,
                html_escape(&json_output)
            )))
        }
    }
}

/// Format a value as a styled HTML debug block with JSON output
fn format_dump(value: &Value) -> String {
    let json_output = serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{:#?}", value));
    format!(
        r#"<pre style="background:#1e1e2e;color:#cdd6f4;padding:1rem;border-radius:8px;font-size:0.85rem;overflow-x:auto;border-left:4px solid #f38ba8;margin:1rem 0;text-align:left;"><code>{}</code></pre>"#,
        html_escape(&json_output)
    )
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Asset function - returns the path to an asset file
/// Usage: {{ asset('sqa-logo.jpeg') }} -> "assets/sqa-logo.jpeg"
/// This allows templates to reference assets without hardcoding paths
fn asset_function(filename: String) -> String {
    format!("assets/{}", filename)
}

/// Source function - returns raw content of a template file without rendering
/// Similar to Twig's source() function: https://twig.symfony.com/doc/3.x/functions/source.html
/// Usage: {{ source("code/example.js") }} -> raw contents of slides/templates/code/example.js
/// The path is relative to slides/templates/
fn make_source_function() -> impl Fn(String) -> Result<String, Error> + Send + Sync {
    move |name: String| {
        let path = format!("slides/templates/{}", name);
        if Path::new(&path).exists() {
            return fs::read_to_string(&path)
                .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Could not read {}: {}", path, e)));
        }

        Err(Error::new(
            ErrorKind::InvalidOperation,
            format!("Template not found: {}", path),
        ))
    }
}

/// Render all slides to HTML pages
pub fn render_deck_pages(
    slides: &[Slide],
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut pages = HashMap::new();
    let total = slides.len();

    // Create minijinja environment
    let mut env = create_environment();

    // Register filters
    let translations_arc = Arc::new(translations.clone());
    env.add_filter("trans", make_trans_filter(translations_arc));
    env.add_filter("markdown", markdown_filter);
    env.add_filter("dump", dump_filter);

    // Register functions
    env.add_function("dump", dump_function);
    env.add_function("asset", asset_function);
    env.add_function("source", make_source_function());

    // Add all layouts as templates
    for (name, content) in layouts {
        if let Err(e) = env.add_template_owned(name.clone(), content.clone()) {
            eprintln!("⚠️  Warning: Could not load template '{}': {}", name, e);
        }
    }

    // Also load slide templates
    for slide in slides {
        if !layouts.contains_key(&slide.template) {
            if let Ok(content) = fs::read_to_string(&slide.template) {
                if let Err(e) = env.add_template_owned(slide.template.clone(), content) {
                    eprintln!("⚠️  Warning: Could not load slide '{}': {}", slide.template, e);
                }
            }
        }
    }

    // Render each slide
    for (index, slide) in slides.iter().enumerate() {
        let current = index + 1;
        let prev = if current > 1 { Some(current - 1) } else { None };
        let next = if current < total { Some(current + 1) } else { None };

        // Build context with app and slide variables
        // Also provide uppercase aliases for backwards compatibility
        let ctx = context! {
            app => context! {
                total => total,
                first => 1,
                last => total,
            },
            slide => context! {
                current => current,
                prev => prev,
                next => next,
                isFirst => current == 1,
                isLast => current == total,
            },
            // Uppercase aliases for template compatibility
            CURRENT => current,
            TOTAL => total,
            PREV => prev,
            NEXT => next,
        };

        // Render the slide template
        match env.get_template(&slide.template) {
            Ok(tmpl) => match tmpl.render(&ctx) {
                Ok(html) => {
                    let filename = format!("slide-{}.html", current);
                    pages.insert(filename, html);
                }
                Err(e) => {
                    eprintln!("⚠️  Error rendering slide {}: {}", current, e);
                }
            },
            Err(e) => {
                eprintln!("⚠️  Template '{}' not found: {}", slide.template, e);
            }
        }
    }

    pages
}

/// Render the overview template showing all slides as thumbnails
pub fn render_overview(
    layouts: &HashMap<String, String>,
    translations: &HashMap<String, String>,
    slides: &[Slide],
) -> Option<String> {
    // Check if overview.html template exists
    if !layouts.contains_key("overview.html") {
        return None;
    }

    let total_slides = slides.len();

    // Create environment
    let mut env = create_environment();

    // Register filters
    let translations_arc = Arc::new(translations.clone());
    env.add_filter("trans", make_trans_filter(translations_arc));
    env.add_filter("markdown", markdown_filter);
    env.add_filter("dump", dump_filter);
    env.add_function("dump", dump_function);
    env.add_function("asset", asset_function);
    env.add_function("source", make_source_function());

    // Add templates
    for (name, content) in layouts {
        if let Err(e) = env.add_template_owned(name.clone(), content.clone()) {
            eprintln!("⚠️  Warning: Could not load template '{}': {}", name, e);
        }
    }

    // Build sections structure for the template
    // Group slides by section_key, preserving order
    let mut sections: Vec<minijinja::Value> = Vec::new();
    let mut current_section: Option<String> = None;
    let mut current_slides: Vec<minijinja::Value> = Vec::new();

    for (index, slide) in slides.iter().enumerate() {
        let slide_number = index + 1;
        let section_key = slide.section_key.clone();

        // Check if we need to start a new section
        if section_key != current_section {
            // Save previous section if it has slides
            if !current_slides.is_empty() {
                sections.push(context! {
                    key => current_section.clone().unwrap_or_default(),
                    slides => current_slides.clone(),
                });
            }
            current_section = section_key;
            current_slides = Vec::new();
        }

        // Add slide to current section
        current_slides.push(context! {
            number => slide_number,
            template => slide.template.clone(),
        });
    }

    // Don't forget the last section
    if !current_slides.is_empty() {
        sections.push(context! {
            key => current_section.unwrap_or_default(),
            slides => current_slides,
        });
    }

    // Build context
    let ctx = context! {
        app => context! {
            total => total_slides,
            first => 1,
            last => total_slides,
        },
        sections => sections,
    };

    // Render overview template
    match env.get_template("overview.html") {
        Ok(tmpl) => match tmpl.render(&ctx) {
            Ok(html) => Some(html),
            Err(e) => {
                eprintln!("⚠️  Error rendering overview.html: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("⚠️  overview.html template not found: {}", e);
            None
        }
    }
}

/// Render the presenter view template with embedded notes from all slides
///
/// This function:
/// 1. Extracts speaker notes from each rendered slide HTML
/// 2. Renders presenter.html template with all notes embedded as JSON
pub fn render_presenter(
    layouts: &HashMap<String, String>,
    rendered_pages: &HashMap<String, String>,
    translations: &HashMap<String, String>,
    total_slides: usize,
) -> Option<String> {
    // Check if presenter.html template exists
    if !layouts.contains_key("presenter.html") {
        return None;
    }

    // Extract notes from each rendered slide
    let mut notes: Vec<String> = Vec::with_capacity(total_slides);
    for i in 1..=total_slides {
        let filename = format!("slide-{}.html", i);
        if let Some(html) = rendered_pages.get(&filename) {
            notes.push(extract_speaker_notes(html));
        } else {
            notes.push(String::new());
        }
    }

    // Create environment and add templates
    let mut env = create_environment();

    // Register filters
    let translations_arc = Arc::new(translations.clone());
    env.add_filter("trans", make_trans_filter(translations_arc));
    env.add_filter("markdown", markdown_filter);
    env.add_filter("dump", dump_filter);
    env.add_function("dump", dump_function);
    env.add_function("asset", asset_function);
    env.add_function("source", make_source_function());

    // Add presenter template
    for (name, content) in layouts {
        if let Err(e) = env.add_template_owned(name.clone(), content.clone()) {
            eprintln!("⚠️  Warning: Could not load template '{}': {}", name, e);
        }
    }

    // Build context with notes data
    let notes_json = serde_json::to_string(&notes).unwrap_or_else(|_| "[]".to_string());
    let ctx = context! {
        app => context! {
            total => total_slides,
            first => 1,
            last => total_slides,
        },
        notes => notes_json,
    };

    // Render presenter template
    match env.get_template("presenter.html") {
        Ok(tmpl) => match tmpl.render(&ctx) {
            Ok(html) => Some(html),
            Err(e) => {
                eprintln!("⚠️  Error rendering presenter.html: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("⚠️  presenter.html template not found: {}", e);
            None
        }
    }
}

/// Render the print view template with all slides combined for PDF export
pub fn render_print(
    layouts: &HashMap<String, String>,
    rendered_pages: &HashMap<String, String>,
    translations: &HashMap<String, String>,
    total_slides: usize,
) -> Option<String> {
    // Check for print template (try multiple names)
    let template_name = if layouts.contains_key("print.html") {
        "print.html"
    } else if layouts.contains_key("views/print.html") {
        "views/print.html"
    } else if layouts.contains_key("views/print-base.html.twig") {
        "views/print-base.html.twig"
    } else if layouts.contains_key("views/print-base.html") {
        "views/print-base.html"
    } else {
        return None;
    };

    // Extract slide content from each rendered page
    let mut combined_content = String::new();
    for i in 1..=total_slides {
        let filename = format!("slide-{}.html", i);
        if let Some(html) = rendered_pages.get(&filename) {
            combined_content.push_str(&extract_slide_content(html));
            combined_content.push('\n');
        }
    }

    // Create environment and add templates
    let mut env = create_environment();

    // Register filters
    let translations_arc = Arc::new(translations.clone());
    env.add_filter("trans", make_trans_filter(translations_arc));
    env.add_filter("markdown", markdown_filter);
    env.add_filter("dump", dump_filter);
    env.add_function("dump", dump_function);
    env.add_function("asset", asset_function);
    env.add_function("source", make_source_function());

    // Add templates
    for (name, content) in layouts {
        if let Err(e) = env.add_template_owned(name.clone(), content.clone()) {
            eprintln!("⚠️  Warning: Could not load template '{}': {}", name, e);
        }
    }

    // Build context
    let ctx = context! {
        TITLE => "Presentation",
        CONTENT => combined_content,
        app => context! {
            total => total_slides,
        },
    };

    // Render print template
    match env.get_template(template_name) {
        Ok(tmpl) => match tmpl.render(&ctx) {
            Ok(html) => Some(html),
            Err(e) => {
                eprintln!("⚠️  Error rendering {}: {}", template_name, e);
                None
            }
        },
        Err(e) => {
            eprintln!("⚠️  {} template not found: {}", template_name, e);
            None
        }
    }
}

/// Extract the slide content (section.slide or main.slide) from rendered HTML
fn extract_slide_content(html: &str) -> String {
    // Try to find <section class="slide">...</section>
    if let Some(content) = extract_tag_content(html, "<section class=\"slide\"", "</section>") {
        return format!("<section class=\"slide\">{}</section>", content);
    }
    // Try to find <main class="slide">...</main>
    if let Some(content) = extract_tag_content(html, "<main class=\"slide\"", "</main>") {
        return format!("<section class=\"slide\">{}</section>", content);
    }
    // Fallback: try just <section ...>...</section>
    if let Some(content) = extract_tag_content(html, "<section", "</section>") {
        return format!("<section class=\"slide\">{}</section>", content);
    }
    String::new()
}

/// Extract content between opening tag marker and closing tag
fn extract_tag_content(html: &str, open_marker: &str, close_marker: &str) -> Option<String> {
    if let Some(start) = html.find(open_marker) {
        let after_marker = start + open_marker.len();
        if let Some(tag_close) = html[after_marker..].find('>') {
            let content_start = after_marker + tag_close + 1;
            if let Some(end) = html[content_start..].find(close_marker) {
                return Some(html[content_start..content_start + end].to_string());
            }
        }
    }
    None
}

/// Extract speaker notes content from a rendered slide HTML
fn extract_speaker_notes(html: &str) -> String {
    // Find <aside class="speaker-notes" ...>...</aside>
    // We look for the class, then find the closing > of the opening tag
    let class_marker = "<aside class=\"speaker-notes\"";
    let end_marker = "</aside>";

    if let Some(start) = html.find(class_marker) {
        // Find the closing > of the opening tag (handles additional attributes like style)
        let after_marker = start + class_marker.len();
        if let Some(tag_close) = html[after_marker..].find('>') {
            let content_start = after_marker + tag_close + 1;
            if let Some(end) = html[content_start..].find(end_marker) {
                return html[content_start..content_start + end].trim().to_string();
            }
        }
    }
    String::new()
}

/// Generate PNG thumbnail images for all slides using headless Chrome
pub fn generate_thumbnails(output_dir: &Path, total_slides: usize) -> Result<(), String> {
    use headless_chrome::{
        protocol::cdp::{Emulation, Page},
        Browser,
    };
    use std::time::Duration;

    print!("📸 Generating thumbnails... ");

    // Get absolute path
    let output_dir = output_dir
        .canonicalize()
        .map_err(|e| format!("Could not get absolute path: {}", e))?;

    // Create thumbnails directory
    let thumbnails_dir = output_dir.join("thumbnails");
    fs::create_dir_all(&thumbnails_dir)
        .map_err(|e| format!("Could not create thumbnails directory: {}", e))?;

    // Start browser
    let browser = Browser::default().map_err(|e| {
        let install_hint = get_chrome_install_hint();
        format!(
            "Could not start browser: {}\n\n\
            Chrome/Chromium is required for thumbnail generation.\n\n\
            {}",
            e, install_hint
        )
    })?;

    let tab = browser
        .new_tab()
        .map_err(|e| format!("Could not open tab: {}", e))?;

    tab.set_default_timeout(Duration::from_secs(30));

    for i in 1..=total_slides {
        let filename = format!("slide-{}.html", i);
        let file_url = format!("file://{}/{}", output_dir.display(), filename);

        // Set initial device metrics for page load (1920x1080)
        tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        })
        .ok();

        // Load page
        tab.navigate_to(&file_url)
            .map_err(|e| format!("Could not load slide {}: {}", i, e))?;

        // Wait for page load
        tab.wait_for_element("body")
            .map_err(|e| format!("Timeout loading slide {}: {}", i, e))?;

        std::thread::sleep(Duration::from_millis(300));

        // Hide navigation and reveal all hidden items before screenshot
        tab.evaluate(
            r#"
            const nav = document.querySelector('nav');
            if (nav) nav.style.display = 'none';
            document.body.classList.add('in-presenter', 'reveal-all');

            // Remove any overflow:hidden that might clip content
            document.querySelectorAll('.mermaid, [data-controller="mermaid"]').forEach(el => {
                el.style.overflow = 'visible';
                if (el.parentElement) el.parentElement.style.overflow = 'visible';
            });
        "#,
            false,
        )
        .ok();

        // Wait for layout changes to take effect
        std::thread::sleep(Duration::from_millis(200));

        // Get full document height including SVG elements
        let doc_height = tab
            .evaluate(
                r#"
                let maxHeight = Math.max(document.body.scrollHeight, document.documentElement.scrollHeight);

                // Check SVG elements (Mermaid diagrams)
                document.querySelectorAll('svg').forEach(svg => {
                    const rect = svg.getBoundingClientRect();
                    const bottom = rect.top + window.scrollY + rect.height;
                    if (bottom > maxHeight) maxHeight = bottom;
                });

                // Check all elements for their actual bottom position
                document.querySelectorAll('*').forEach(el => {
                    const rect = el.getBoundingClientRect();
                    const bottom = rect.top + window.scrollY + rect.height;
                    if (bottom > maxHeight) maxHeight = bottom;
                });

                Math.ceil(maxHeight)
                "#,
                false,
            )
            .ok()
            .and_then(|r| r.value)
            .and_then(|v| v.as_f64())
            .map(|h| h as u32)
            .unwrap_or(1080);

        // Update device metrics to match full document height
        tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: 1920,
            height: doc_height.max(1080),
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        })
        .ok();

        std::thread::sleep(Duration::from_millis(100));

        // Capture full-page screenshot
        let screenshot = tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| format!("Could not capture slide {}: {}", i, e))?;

        // Save screenshot
        let screenshot_path = thumbnails_dir.join(format!("slide-{}.png", i));
        fs::write(&screenshot_path, &screenshot)
            .map_err(|e| format!("Could not save screenshot: {}", e))?;
    }

    println!("✅ {} thumbnails", total_slides);
    Ok(())
}

/// Returns OS-specific Chrome installation instructions
fn get_chrome_install_hint() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Install Chrome or Edge:\n\
         • Download from https://www.google.com/chrome/\n\
         • Or use: winget install Google.Chrome"
    }

    #[cfg(target_os = "macos")]
    {
        "Install Chrome or Chromium:\n\
         • Download from https://www.google.com/chrome/\n\
         • Or use: brew install --cask google-chrome"
    }

    #[cfg(target_os = "linux")]
    {
        "Install Chrome or Chromium:\n\
         • Ubuntu/Debian: sudo apt install chromium-browser\n\
         • Fedora: sudo dnf install chromium\n\
         • Arch: sudo pacman -S chromium\n\
         • Or download from https://www.google.com/chrome/"
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Install Chrome or Chromium from https://www.google.com/chrome/"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let tmpl = env.template_from_str(r#"{{ "greeting" | trans }}"#).unwrap();
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
        let tmpl = env.template_from_str(r#"{{ "existing" | trans }}"#).unwrap();
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
        assert_eq!(markdown_to_html("a **bold** word"), "a <strong>bold</strong> word");
    }

    #[test]
    fn test_markdown_italic() {
        assert_eq!(markdown_to_html("*italic*"), "<em>italic</em>");
        assert_eq!(markdown_to_html("an *italic* word"), "an <em>italic</em> word");
    }

    #[test]
    fn test_markdown_code() {
        assert_eq!(markdown_to_html("`code`"), "<code>code</code>");
        assert_eq!(markdown_to_html("use `let x = 1`"), "use <code>let x = 1</code>");
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
    fn test_trans_filter_renders_markdown() {
        let mut translations = HashMap::new();
        translations.insert("point1".to_string(), "**Lehrmedium #1** für alle Kurse".to_string());
        translations.insert("point2".to_string(), "Repräsentieren unsere *Qualitätsstandards*".to_string());

        let env = create_test_env(translations);

        let tmpl = env.template_from_str(r#"{{ "point1" | trans }}"#).unwrap();
        assert_eq!(
            tmpl.render(context! {}).unwrap(),
            "<strong>Lehrmedium #1</strong> für alle Kurse"
        );

        let tmpl = env.template_from_str(r#"{{ "point2" | trans }}"#).unwrap();
        assert_eq!(
            tmpl.render(context! {}).unwrap(),
            "Repräsentieren unsere <em>Qualitätsstandards</em>"
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

        // Should contain a styled pre block
        assert!(result.contains("<pre style="));
        assert!(result.contains("</pre>"));
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

        assert!(result.contains("<pre style="));
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

    #[test]
    fn test_extract_speaker_notes() {
        let html = r#"
            <main class="slide">Content</main>
            <aside class="speaker-notes">
                <h3>Notes</h3>
                <p>Important point</p>
            </aside>
        "#;

        let notes = extract_speaker_notes(html);
        assert!(notes.contains("<h3>Notes</h3>"));
        assert!(notes.contains("<p>Important point</p>"));
    }

    #[test]
    fn test_extract_speaker_notes_empty() {
        let html = r#"<main class="slide">Content only</main>"#;
        let notes = extract_speaker_notes(html);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_extract_speaker_notes_with_style_attribute() {
        let html = r#"
            <main class="slide">Content</main>
            <aside class="speaker-notes" style="display: none;">
                <h2>Speaker Notes</h2>
                <p>This is a note with <strong>markdown</strong>.</p>
            </aside>
        "#;

        let notes = extract_speaker_notes(html);
        assert!(notes.contains("<h2>Speaker Notes</h2>"));
        assert!(notes.contains("<strong>markdown</strong>"));
    }
}
