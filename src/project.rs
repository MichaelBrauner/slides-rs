//! Project - functions for creating and initializing slide projects

use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;

// Convention over Configuration - Import paths
const TEMPLATES_OUTPUT_DIR: &str = "slides/templates";
const IMAGES_OUTPUT_DIR: &str = "slides/assets/import/images";

// Init templates (exported for testing)
pub const INIT_GITIGNORE_FILE: &str = include_str!("templates/init/gitignore");
pub const INIT_DECKS_FILE: &str = include_str!("templates/init/decks.yaml");

/// Initialize a new slides project in the current directory
pub fn init() -> Result<(), String> {
    println!("🚀 Initializing Slides project...\n");

    if !is_current_dir_empty()? {
        return Err(
            "Directory is not empty. 'slides init' can only be run in an empty directory."
                .to_string(),
        );
    }

    create_dir("slides/templates")?;
    println!("   ✅ slides/templates/ created");

    create_dir("slides/assets")?;
    println!("   ✅ slides/assets/ created");

    write_file(".gitignore", INIT_GITIGNORE_FILE)?;
    println!("   ✅ .gitignore created");

    write_file("decks.yaml", INIT_DECKS_FILE)?;
    println!("   ✅ decks.yaml created");

    println!("\n🎉 Project successfully initialized!");
    println!("\n📌 Next steps:");
    println!("  1. Configure decks in decks.yaml");
    println!("  2. Run: slides build");
    println!("  3. Open: output/slide-1.html");

    Ok(())
}

/// Create a new slides project in a new directory
pub fn create(name: &str, custom_path: Option<&str>) -> Result<(), String> {
    let project_dir = if let Some(p) = custom_path {
        Path::new(p).join(name)
    } else {
        Path::new(name).to_path_buf()
    };

    if project_dir.exists() {
        return Err(format!(
            "Directory '{}' already exists.",
            project_dir.display()
        ));
    }

    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Could not create directory: {}", e))?;

    println!("📁 Creating project '{}'...\n", name);

    let original_dir = std::env::current_dir()
        .map_err(|e| format!("Could not determine current directory: {}", e))?;

    std::env::set_current_dir(&project_dir)
        .map_err(|e| format!("Could not change to project directory: {}", e))?;

    let result = init();

    std::env::set_current_dir(&original_dir).ok();

    result?;

    println!("\n💡 Next steps:");
    println!("   cd {}", name);
    println!("   slides build");
    println!("   # Open output/slide-1.html in browser");

    Ok(())
}

/// Import a PPTX file into the project
pub fn import(pptx_file: &str, images_only: bool) -> Result<(), String> {
    let pptx_path = Path::new(pptx_file);
    let slides_dir = Path::new(TEMPLATES_OUTPUT_DIR);
    let images_dir = Path::new(IMAGES_OUTPUT_DIR);

    println!("📥 PPTX Import");
    println!("   File: {:?}", pptx_path);
    if images_only {
        println!("   Mode: Extract images only → {}", IMAGES_OUTPUT_DIR);
    } else {
        println!("   Slides → {}", TEMPLATES_OUTPUT_DIR);
        println!("   Images → {}", IMAGES_OUTPUT_DIR);
    }
    println!();

    let file = File::open(pptx_path)
        .map_err(|e| format!("Could not open PPTX file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Invalid PPTX file (ZIP error): {}", e))?;

    fs::create_dir_all(images_dir)
        .map_err(|e| format!("Could not create images directory: {}", e))?;

    extract_images(&mut archive, images_dir)?;

    if images_only {
        println!();
        println!("✅ Images successfully extracted to {}!", IMAGES_OUTPUT_DIR);
        return Ok(());
    }

    fs::create_dir_all(slides_dir)
        .map_err(|e| format!("Could not create output directory: {}", e))?;

    let mut slide_count = 0;

    for i in 1..200 {
        let slide_path = format!("ppt/slides/slide{}.xml", i);

        let xml_content = {
            match archive.by_name(&slide_path) {
                Ok(mut file) => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)
                        .map_err(|e| format!("Could not read slide {}: {}", i, e))?;
                    Some(content)
                }
                Err(_) => None,
            }
        };

        if let Some(xml_content) = xml_content {
            let mut slide_data = SlideData {
                slide_number: i,
                ..Default::default()
            };

            parse_slide_xml(&xml_content, &mut slide_data)?;

            let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", i);
            if let Ok(mut rels_file) = archive.by_name(&rels_path) {
                let mut rels_xml = String::new();
                rels_file.read_to_string(&mut rels_xml).ok();
                parse_image_relationships(&xml_content, &rels_xml, &mut slide_data)?;
            }

            write_slide_file(slides_dir, &slide_data, pptx_path)?;

            slide_count += 1;
            print!(".");
        } else {
            break;
        }
    }

    println!();
    println!();
    println!("✅ {} slides successfully imported!", slide_count);
    println!("   Slides: {}/", TEMPLATES_OUTPUT_DIR);
    println!("   Images: {}/", IMAGES_OUTPUT_DIR);

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_dir(path: &str) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Could not create directory '{}': {}", path, e))
}

fn write_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Could not write '{}': {}", path, e))
}

fn is_current_dir_empty() -> Result<bool, String> {
    let count = fs::read_dir(".")
        .map_err(|e| format!("Could not read directory: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            if let Some(name) = e.file_name().to_str() {
                !name.starts_with('.')
            } else {
                true
            }
        })
        .count();

    Ok(count == 0)
}

// ============================================================================
// Import Helper Functions
// ============================================================================

#[derive(Debug, Default)]
struct SlideData {
    title: Option<String>,
    content: Vec<String>,
    notes: Option<String>,
    slide_number: usize,
    images: Vec<String>,
}

fn extract_images(archive: &mut ZipArchive<File>, assets_dir: &Path) -> Result<(), String> {
    let mut image_count = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("ZIP error: {}", e))?;
        let name = file.name().to_string();

        if name.starts_with("ppt/media/") && is_image_file(&name) {
            let filename = Path::new(&name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let output_path = assets_dir.join(filename);

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("Could not read image: {}", e))?;

            fs::write(&output_path, buffer)
                .map_err(|e| format!("Could not save image: {}", e))?;

            image_count += 1;
        }
    }

    if image_count > 0 {
        println!("   📷 {} images extracted to {:?}", image_count, assets_dir);
    }

    Ok(())
}

fn is_image_file(name: &str) -> bool {
    name.ends_with(".png")
        || name.ends_with(".jpg")
        || name.ends_with(".jpeg")
        || name.ends_with(".gif")
        || name.ends_with(".svg")
}

fn parse_image_relationships(
    slide_xml: &str,
    rels_xml: &str,
    slide_data: &mut SlideData,
) -> Result<(), String> {
    use std::collections::HashMap;

    let mut relationships: HashMap<String, String> = HashMap::new();

    let mut reader = Reader::from_str(rels_xml);
    reader.trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"Relationship" => {
                let mut id = String::new();
                let mut target = String::new();

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"Id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                        b"Target" => target = String::from_utf8_lossy(&attr.value).to_string(),
                        _ => {}
                    }
                }

                if target.starts_with("../media/") {
                    let filename = target.replace("../media/", "");
                    relationships.insert(id, filename);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    let mut reader = Reader::from_str(slide_xml);
    buf.clear();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) if e.name().as_ref() == b"a:blip" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"r:embed" {
                        let rid = String::from_utf8_lossy(&attr.value).to_string();
                        if let Some(image) = relationships.get(&rid) {
                            slide_data.images.push(image.clone());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}

fn parse_slide_xml(xml: &str, slide_data: &mut SlideData) -> Result<(), String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut current_text = String::new();
    let mut paragraph_text = String::new();
    let mut in_text_element = false;
    let mut in_paragraph = false;
    let mut text_items = Vec::new();
    let mut is_title = true;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"a:p" => {
                        in_paragraph = true;
                        paragraph_text.clear();
                    }
                    b"a:t" => {
                        in_text_element = true;
                        current_text.clear();
                    }
                    _ => {}
                }
            }

            Ok(Event::Text(e)) => {
                if in_text_element {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }

            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"a:t" => {
                        if in_text_element {
                            in_text_element = false;
                            if !current_text.is_empty() {
                                if !paragraph_text.is_empty()
                                    && !current_text.starts_with(' ')
                                    && !paragraph_text.ends_with(' ')
                                {
                                    paragraph_text.push(' ');
                                }
                                paragraph_text.push_str(&current_text);
                            }
                        }
                    }
                    b"a:p" => {
                        if in_paragraph {
                            in_paragraph = false;

                            let trimmed = paragraph_text.trim();
                            if !trimmed.is_empty() {
                                if is_title && slide_data.title.is_none() {
                                    slide_data.title = Some(trimmed.to_string());
                                    is_title = false;
                                } else {
                                    text_items.push(trimmed.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!(
                    "⚠️  XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                );
                break;
            }
            _ => {}
        }

        buf.clear();
    }

    slide_data.content = text_items;

    Ok(())
}

fn write_slide_file(
    output_dir: &Path,
    slide_data: &SlideData,
    _original_file: &Path,
) -> Result<(), String> {
    let filename = if let Some(title) = &slide_data.title {
        let slug = title_to_slug(title);
        format!("imported-{:02}-{}.html", slide_data.slide_number, slug)
    } else {
        format!("imported-{:02}.html", slide_data.slide_number)
    };

    let output_path = output_dir.join(&filename);

    let title = slide_data
        .title
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Slide");

    let mut content = String::new();

    // Generate HTML template
    content.push_str("{% extends \"templates/base.html\" %}\n\n");
    content.push_str(&format!("{{% block title %}}{}{{%- endblock %}}\n\n", escape_html(title)));
    content.push_str("{% block content %}\n");
    content.push_str(&format!("<h1 class=\"slide-title\">{}</h1>\n", escape_html(title)));
    content.push_str("<div class=\"slide-content\">\n");

    if !slide_data.content.is_empty() {
        if slide_data.content.len() > 1 {
            content.push_str("    <ul>\n");
            for item in &slide_data.content {
                content.push_str(&format!("        <li>{}</li>\n", escape_html(item)));
            }
            content.push_str("    </ul>\n");
        } else {
            content.push_str(&format!("    <p>{}</p>\n", escape_html(&slide_data.content[0])));
        }
    }

    if !slide_data.images.is_empty() {
        for image in &slide_data.images {
            content.push_str(&format!("    <img src=\"{{{{ asset('import/images/{}') }}}}\" alt=\"\">\n", image));
        }
    }

    content.push_str("</div>\n");
    content.push_str("{% endblock %}\n");

    if let Some(notes) = &slide_data.notes {
        content.push_str("\n{% block notes %}\n");
        content.push_str(&format!("<p>{}</p>\n", escape_html(notes)));
        content.push_str("{% endblock %}\n");
    }

    let mut file = File::create(&output_path)
        .map_err(|e| format!("Could not create file {:?}: {}", output_path, e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Could not write file: {}", e))?;

    Ok(())
}

fn title_to_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'ä' => "ae",
            'ö' => "oe",
            'ü' => "ue",
            'ß' => "ss",
            _ if c.is_alphanumeric() => {
                let mut s = String::new();
                s.push(c);
                s.leak()
            }
            _ => "-",
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(50)
        .collect()
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
