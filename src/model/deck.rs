//! Deck - a collection of slides that can be built, watched, and exported

use super::Slide;
use crate::{render, translations};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

// Convention over Configuration - Path constants
const SLIDES_ROOT: &str = "slides";
const OUTPUT_DIR: &str = "output";
const DECKS_CONFIG: &str = "decks.yaml";

// ============================================================================
// Deck Configuration (decks.yaml)
// ============================================================================

/// Configuration for a single deck (slides, order, sections)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeckConfig {
    /// Flat list of slide patterns
    Flat(Vec<String>),
    /// Nested structure: Section key -> Slide patterns
    Sectioned(IndexMap<String, Vec<String>>),
}

impl DeckConfig {
    pub fn load_slides(&self) -> Result<Vec<Slide>, String> {
        let patterns_with_sections: Vec<(&str, Option<String>)> = match self {
            DeckConfig::Flat(patterns) => {
                patterns.iter().map(|p| (p.as_str(), None)).collect()
            }

            DeckConfig::Sectioned(sections) => sections
                .iter()
                .flat_map(|(key, patterns)| {
                    patterns
                        .iter()
                        .map(move |p| (p.as_str(), Some(key.clone())))
                })
                .collect(),
        };

        let mut slides = Vec::new();
        for (pattern, section_key) in patterns_with_sections {
            slides.extend(Slide::load_collection(pattern, section_key)?);
        }

        Ok(slides)
    }
}

/// Collection of all deck configurations (decks.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckConfigCollection(IndexMap<String, DeckConfig>);

impl DeckConfigCollection {
    /// Load decks.yaml from project directory
    pub fn load() -> Result<Self, String> {
        let decks_path = Path::new(DECKS_CONFIG);

        if !decks_path.exists() {
            return Err("decks.yaml not found".to_string());
        }

        let content = fs::read_to_string(decks_path)
            .map_err(|e| format!("Could not read decks.yaml: {}", e))?;

        let decks: IndexMap<String, DeckConfig> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Error parsing decks.yaml: {}", e))?;

        Ok(Self(decks))
    }

    /// Get a specific deck configuration by name
    pub fn get(&self, name: &str) -> Result<DeckConfig, String> {
        self.0
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Deck '{}' not found in decks.yaml", name))
    }
}

// ============================================================================
// Deck
// ============================================================================

pub struct Deck {
    name: String,
    lang: String,
    slides: Vec<Slide>,
}

impl Deck {
    pub fn new(name: &str, lang: &str) -> Self {
        Self {
            name: name.to_string(),
            lang: lang.to_string(),
            slides: Vec::new(),
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        self.slides = DeckConfigCollection::load()?
            .get(&self.name)?
            .load_slides()?;

        if self.slides.is_empty() {
            return Err(format!("No slides found for deck '{}'", self.name));
        }

        Ok(())
    }

    pub fn build(&mut self) -> Result<(), String> {
        println!("🎬 Generate Presentation");
        println!();

        let layouts_path = Path::new(SLIDES_ROOT);
        let output_dir = Path::new(OUTPUT_DIR);

        print!("📄 Load Slides for Deck '{}'... ", self.name);
        self.load().map_err(|e| {
            format!(
                "\n   Error loading deck: {}\n   Tip: Check your decks.yaml",
                e
            )
        })?;
        println!("✅ {} slides found", self.slides.len());

        // Step 2: Load layouts (recursively)
        print!("🧱 Loading templates recursively... ");
        let layouts = render::load_layouts(layouts_path).map_err(|e| {
            format!("\n   Error loading templates: {}", e)
        })?;
        println!("✅ {} templates loaded", layouts.len());

        // Step 3: Load translations (if available)
        let translation_vars = match translations::Translations::load(&self.lang) {
            Ok(trans) => {
                println!("🌍 Translations for '{}' loaded", self.lang);
                translations::flatten_translations(&trans)
            }
            Err(_) => HashMap::new(),
        };

        // Step 4: Generate HTML pages
        print!("🔨 Generating HTML pages... ");
        let pages = render::render_deck_pages(&self.slides, &layouts, &translation_vars);
        println!("✅ {} pages", pages.len());

        // Step 4b: Generate overview (if template exists)
        let overview_html = render::render_overview(
            &layouts,
            &translation_vars,
            &self.slides,
        );
        if overview_html.is_some() {
            println!("📋 Overview generated");
        }

        // Step 4c: Generate presenter view (if template exists)
        let presenter_html = render::render_presenter(
            &layouts,
            &pages,
            &translation_vars,
            self.slides.len(),
        );
        if presenter_html.is_some() {
            println!("🎤 Presenter view generated");
        }

        // Step 4d: Generate print view (if template exists)
        let print_html = render::render_print(
            &layouts,
            &pages,
            &translation_vars,
            self.slides.len(),
        );
        if print_html.is_some() {
            println!("🖨️  Print view generated");
        }

        // Step 5: Write all pages
        print!("💾 Writing files... ");
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Could not create output directory: {}", e))?;

        for (filename, content) in &pages {
            let file_path = output_dir.join(filename);
            fs::write(&file_path, content)
                .map_err(|e| format!("Could not write {:?}: {}", file_path, e))?;
        }

        // Write overview
        if let Some(overview) = overview_html {
            let overview_path = output_dir.join("overview.html");
            fs::write(&overview_path, overview)
                .map_err(|e| format!("Could not write overview.html: {}", e))?;
        }

        // Write presenter view
        if let Some(presenter) = presenter_html {
            let presenter_path = output_dir.join("presenter.html");
            fs::write(&presenter_path, presenter)
                .map_err(|e| format!("Could not write presenter.html: {}", e))?;
        }

        // Write print view
        if let Some(print) = print_html {
            let print_path = output_dir.join("print.html");
            fs::write(&print_path, print)
                .map_err(|e| format!("Could not write print.html: {}", e))?;
        }
        println!("✅");

        // Step 5a: Generate thumbnails
        render::generate_thumbnails(output_dir, self.slides.len())?;

        // Step 6: Copy assets (if available)
        let assets_src = Path::new(SLIDES_ROOT).join("assets");
        if assets_src.exists() {
            print!("📦 Copying assets... ");
            let assets_dest = output_dir.join("assets");
            copy_dir_recursive(&assets_src, &assets_dest)?;
            println!("✅");
        }

        println!();
        println!("🎉 Presentation generated successfully!");
        println!("   Open {:?} in browser", output_dir.join("slide-1.html"));

        Ok(())
    }

    pub fn watch(&mut self) -> Result<(), String> {
        use notify_debouncer_mini::{new_debouncer, notify::*};
        use std::sync::mpsc::channel;
        use std::time::Duration;

        println!("👀 Watch mode started");
        println!();
        println!("   Watching:");
        println!("   • slides/");
        println!("   • decks.yaml");
        println!();
        println!("   Press Ctrl+C to exit");
        println!();
        println!("─────────────────────────────────────────────────────");
        println!();

        // Initial build
        println!("🔨 Initial build...");
        if let Err(e) = self.build() {
            eprintln!("❌ Error: {}", e);
        }
        println!();
        println!("─────────────────────────────────────────────────────");
        println!();

        // Setup file watcher
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
            .map_err(|e| format!("Could not initialize file watcher: {}", e))?;

        debouncer
            .watcher()
            .watch(Path::new("templates"), RecursiveMode::Recursive)
            .map_err(|e| format!("Could not watch templates/: {}", e))?;

        if Path::new("decks.yaml").exists() {
            debouncer
                .watcher()
                .watch(Path::new("decks.yaml"), RecursiveMode::NonRecursive)
                .map_err(|e| format!("Could not watch decks.yaml: {}", e))?;
        }

        if Path::new("slides/translations").exists() {
            debouncer
                .watcher()
                .watch(Path::new("slides/translations"), RecursiveMode::Recursive)
                .map_err(|e| format!("Could not watch slides/translations/: {}", e))?;
        }

        println!("✅ Watching for changes...");
        println!();

        loop {
            match rx.recv() {
                Ok(result) => match result {
                    Ok(events) => {
                        if !events.is_empty() {
                            for event in &events {
                                if let Some(path) = event.path.to_str() {
                                    if !path.contains(".swp")
                                        && !path.contains(".tmp")
                                        && !path.contains("~")
                                    {
                                        println!("📝 Change detected: {}", path);
                                    }
                                }
                            }

                            println!();
                            println!("🔨 Rebuilding...");
                            match self.build() {
                                Ok(_) => {
                                    println!();
                                    println!("─────────────────────────────────────────────────────");
                                    println!();
                                    println!("✅ Ready for more changes...");
                                    println!();
                                }
                                Err(e) => {
                                    eprintln!("❌ Error: {}", e);
                                    println!();
                                    println!("─────────────────────────────────────────────────────");
                                    println!();
                                }
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("⚠️  Watch error: {:?}", error);
                    }
                },
                Err(e) => {
                    eprintln!("⚠️  Channel error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Export the deck as PDF
    pub fn export(
        &self,
        password: Option<String>,
        no_print: bool,
        no_copy: bool,
        output_path: &str,
    ) -> Result<(), String> {
        println!("📄 PDF Export");
        println!();

        let thumbnails_dir = Path::new("output/thumbnails");
        if !thumbnails_dir.exists() {
            println!("⚠️  Thumbnails not yet generated");
            return Err(
                "Please run 'slides build' first to generate thumbnails.".to_string(),
            );
        }

        // Count thumbnails
        let mut slide_count = 0;
        while thumbnails_dir.join(format!("slide-{}.png", slide_count + 1)).exists() {
            slide_count += 1;
        }

        if slide_count == 0 {
            return Err("No thumbnails found in output/thumbnails/".to_string());
        }

        println!("📸 Found {} slide thumbnails", slide_count);

        println!("🖨️  Generating PDF from thumbnails...");
        let pdf_bytes = generate_pdf_from_thumbnails(thumbnails_dir, slide_count)?;
        println!("   ✅ PDF generated ({} KB)", pdf_bytes.len() / 1024);
        println!();

        let (final_pdf, used_password, actually_encrypted) = if let Some(pwd) = password {
            let actual_password = if pwd == "auto" {
                generate_secure_password()
            } else {
                pwd
            };

            println!("🔒 Encrypting PDF...");
            let (encrypted, was_encrypted) =
                encrypt_pdf(pdf_bytes, &actual_password, no_print, no_copy)?;
            if was_encrypted {
                println!("   ✅ PDF encrypted");
            }
            println!();
            (encrypted, Some(actual_password), was_encrypted)
        } else {
            (pdf_bytes, None, false)
        };

        fs::write(output_path, final_pdf)
            .map_err(|e| format!("Could not write PDF: {}", e))?;

        println!("🎉 PDF created successfully!");
        println!("   📁 {}", output_path);

        if let Some(pwd) = used_password {
            println!();
            if actually_encrypted {
                println!("   🔐 Password protected");
                println!("   🔑 Password: {}", pwd);
                if no_print {
                    println!("   🚫 Printing disabled");
                }
                if no_copy {
                    println!("   🚫 Copying disabled");
                }
            } else {
                println!("   ⚠️  PDF is NOT encrypted");
                println!("   🔑 Password would be: {}", pwd);
            }
        }

        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Recursively copy a directory and all its contents
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("Could not create directory {:?}: {}", dest, e))?;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Could not read directory {:?}: {}", src, e))?;

    for entry in entries.flatten() {
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                copy_dir_recursive(&src_path, &dest_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dest_path)
                    .map_err(|e| format!("Could not copy {:?}: {}", src_path, e))?;
            }
        }
    }

    Ok(())
}

// ============================================================================
// Export Helper Functions
// ============================================================================

fn find_or_download_chrome() -> Result<PathBuf, String> {
    let chrome_names = if cfg!(target_os = "windows") {
        vec!["chrome.exe", "chromium.exe"]
    } else {
        vec!["google-chrome", "chromium", "chrome", "chromium-browser"]
    };

    for name in &chrome_names {
        if let Ok(path) = which::which(name) {
            return Ok(path);
        }
    }

    let chrome_dir = dirs::home_dir()
        .ok_or_else(|| "Could not find home directory".to_string())?
        .join(".slides")
        .join("chromium");

    let chrome_bin = if cfg!(target_os = "windows") {
        chrome_dir.join("chrome-win").join("chrome.exe")
    } else if cfg!(target_os = "macos") {
        chrome_dir
            .join("chrome-mac")
            .join("Chromium.app")
            .join("Contents")
            .join("MacOS")
            .join("Chromium")
    } else {
        chrome_dir.join("chrome-linux").join("chrome")
    };

    if chrome_bin.exists() {
        return Ok(chrome_bin);
    }

    println!();
    println!("   Chrome/Chromium not found.");
    println!("   Download Chromium automatically? (~150MB)");
    print!("   [y/n]: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;

    if input.trim().to_lowercase() == "y" {
        println!();
        download_chromium(&chrome_dir)?;
        Ok(chrome_bin)
    } else {
        Err("PDF export requires Chrome/Chromium.\n   Installation:\n   • Ubuntu/Debian: sudo apt install chromium-browser\n   • macOS: brew install chromium\n   • Windows: winget install Google.Chrome".to_string())
    }
}

fn download_chromium(target_dir: &Path) -> Result<(), String> {
    println!("📦 Downloading Chromium...");
    println!();

    let (url, archive_name) = if cfg!(target_os = "linux") {
        (
            "https://storage.googleapis.com/chromium-browser-snapshots/Linux_x64/1234567/chrome-linux.zip",
            "chrome-linux.zip",
        )
    } else if cfg!(target_os = "macos") {
        (
            "https://storage.googleapis.com/chromium-browser-snapshots/Mac/1234567/chrome-mac.zip",
            "chrome-mac.zip",
        )
    } else if cfg!(target_os = "windows") {
        (
            "https://storage.googleapis.com/chromium-browser-snapshots/Win_x64/1234567/chrome-win.zip",
            "chrome-win.zip",
        )
    } else {
        return Err("Platform not supported".to_string());
    };

    println!("   URL: {}", url);
    println!("   Target: {}", target_dir.display());
    println!();

    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Download failed: {}", e))?;

    let _total_size = response.content_length().unwrap_or(0);
    let bytes = response
        .bytes()
        .map_err(|e| format!("Could not read data: {}", e))?;

    println!(
        "   ✅ Download completed ({} MB)",
        bytes.len() / 1_000_000
    );
    println!();

    println!("📂 Extracting Chromium...");
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Could not create directory: {}", e))?;

    let zip_path = target_dir.join(archive_name);
    fs::write(&zip_path, &bytes).map_err(|e| format!("Could not write ZIP: {}", e))?;

    let zip_file =
        fs::File::open(&zip_path).map_err(|e| format!("Could not open ZIP: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    archive
        .extract(target_dir)
        .map_err(|e| format!("Extraction failed: {}", e))?;

    fs::remove_file(&zip_path).ok();

    println!("   ✅ Chromium installed");
    Ok(())
}

/// Generate PDF from thumbnail images
fn generate_pdf_from_thumbnails(thumbnails_dir: &Path, slide_count: usize) -> Result<Vec<u8>, String> {
    use printpdf::*;
    use std::io::BufWriter;

    // 16:9 aspect ratio, similar to PowerPoint defaults
    let width_mm = 338.666;  // ~13.33 inches
    let height_mm = 190.5;   // ~7.5 inches

    let (doc, mut page_index, mut layer_index) = PdfDocument::new(
        "Presentation",
        Mm(width_mm),
        Mm(height_mm),
        "Slide 1",
    );

    for i in 1..=slide_count {
        let png_path = thumbnails_dir.join(format!("slide-{}.png", i));

        // Load PNG image using image crate
        let img = ::image::open(&png_path)
            .map_err(|e| format!("Could not open {}: {}", png_path.display(), e))?;

        let img_rgb = img.to_rgb8();
        let (img_width, img_height) = img_rgb.dimensions();

        // Create printpdf Image
        let image = Image::from(
            ImageXObject {
                width: Px(img_width as usize),
                height: Px(img_height as usize),
                color_space: ColorSpace::Rgb,
                bits_per_component: ColorBits::Bit8,
                interpolate: true,
                image_data: img_rgb.into_raw(),
                image_filter: None,
                clipping_bbox: None,
                smask: None,
            }
        );

        // Add page (except for first slide which already has a page)
        if i > 1 {
            let (new_page, new_layer) = doc.add_page(
                Mm(width_mm),
                Mm(height_mm),
                format!("Slide {}", i),
            );
            page_index = new_page;
            layer_index = new_layer;
        }

        let current_layer = doc.get_page(page_index).get_layer(layer_index);

        // Calculate DPI to fit image to page
        // printpdf uses 300 DPI by default for image placement
        let dpi = 300.0;
        let img_width_mm = img_width as f32 / dpi * 25.4;
        let img_height_mm = img_height as f32 / dpi * 25.4;

        // Scale to fit page
        let scale_x = width_mm / img_width_mm;
        let scale_y = height_mm / img_height_mm;

        // Add image to page, scaled to fill
        image.add_to_layer(
            current_layer,
            ImageTransform {
                translate_x: Some(Mm(0.0)),
                translate_y: Some(Mm(0.0)),
                scale_x: Some(scale_x),
                scale_y: Some(scale_y),
                ..Default::default()
            },
        );
    }

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf)
        .map_err(|e| format!("Could not generate PDF: {}", e))?;

    Ok(buf.into_inner().map_err(|e| format!("Buffer error: {}", e))?)
}

fn generate_pdf_with_chrome(chrome_path: &Path) -> Result<Vec<u8>, String> {
    use headless_chrome::types::PrintToPdfOptions;
    use headless_chrome::Browser;
    use std::env;

    let browser = Browser::new(headless_chrome::LaunchOptions {
        headless: true,
        sandbox: false,
        path: Some(chrome_path.to_path_buf()),
        ..Default::default()
    })
    .map_err(|e| format!("Could not start Chrome: {}", e))?;

    let tab = browser
        .new_tab()
        .map_err(|e| format!("Could not open tab: {}", e))?;

    let print_path = env::current_dir()
        .map_err(|e| format!("Could not determine current directory: {}", e))?
        .join("output")
        .join("print.html");

    let url = format!("file://{}", print_path.display());

    tab.navigate_to(&url)
        .map_err(|e| format!("Could not load HTML: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("Timeout while loading: {}", e))?;

    let pdf_data = tab
        .print_to_pdf(Some(PrintToPdfOptions {
            landscape: Some(false),
            print_background: Some(true),
            scale: Some(1.0),
            paper_width: Some(7.5),
            paper_height: Some(13.33),
            margin_top: Some(0.0),
            margin_bottom: Some(0.0),
            margin_left: Some(0.0),
            margin_right: Some(0.0),
            ..Default::default()
        }))
        .map_err(|e| format!("PDF generation failed: {}", e))?;

    Ok(pdf_data)
}

fn generate_secure_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789!@#$%";
    const PASSWORD_LEN: usize = 16;
    let mut rng = rand::thread_rng();

    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn encrypt_pdf(
    pdf_bytes: Vec<u8>,
    password: &str,
    no_print: bool,
    no_copy: bool,
) -> Result<(Vec<u8>, bool), String> {
    use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
    use lopdf::{Document, Object, StringFormat};
    use md5::{Digest, Md5};
    use std::io::Cursor;

    let mut document = Document::load_from(Cursor::new(&pdf_bytes))
        .map_err(|e| format!("Could not load PDF: {}", e))?;

    if !document.trailer.has(b"ID") {
        let mut hasher = Md5::new();
        hasher.update(format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        hasher.update(password.as_bytes());
        let file_id = hasher.finalize().to_vec();

        let id_array = vec![
            Object::String(file_id.clone(), StringFormat::Hexadecimal),
            Object::String(file_id, StringFormat::Hexadecimal),
        ];

        document.trailer.set("ID", Object::Array(id_array));
    }

    let mut permissions = Permissions::PRINTABLE
        | Permissions::MODIFIABLE
        | Permissions::COPYABLE
        | Permissions::ANNOTABLE
        | Permissions::FILLABLE
        | Permissions::COPYABLE_FOR_ACCESSIBILITY
        | Permissions::ASSEMBLABLE
        | Permissions::PRINTABLE_IN_HIGH_QUALITY;

    if no_print {
        permissions.remove(Permissions::PRINTABLE);
        permissions.remove(Permissions::PRINTABLE_IN_HIGH_QUALITY);
    }
    if no_copy {
        permissions.remove(Permissions::COPYABLE);
        permissions.remove(Permissions::COPYABLE_FOR_ACCESSIBILITY);
    }

    let encryption_version = EncryptionVersion::V2 {
        document: &document,
        owner_password: password,
        user_password: password,
        key_length: 128,
        permissions,
    };

    let encryption_state = EncryptionState::try_from(encryption_version)
        .map_err(|e| format!("Could not initialize encryption: {}", e))?;

    document
        .encrypt(&encryption_state)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut buffer = Vec::new();
    document
        .save_to(&mut buffer)
        .map_err(|e| format!("Could not save encrypted PDF: {}", e))?;

    Ok((buffer, true))
}
