//! Deck - a collection of slides that can be built, watched, and exported

use super::Slide;
use crate::error::{Error, Result};
use crate::infrastructure::chrome;
use crate::services::{render, translations};
use crate::util;
use indexmap::IndexMap;
use log::warn;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeckConfig {
    /// Flat list of slide patterns
    Flat(Vec<String>),
    /// Nested structure: Section key -> Slide patterns
    Sectioned(IndexMap<String, Vec<String>>),
}

impl DeckConfig {
    pub fn load_slides(&self, templates_dir: &Path) -> Result<Vec<Slide>> {
        let patterns_with_sections: Vec<(&str, Option<String>)> = match self {
            DeckConfig::Flat(patterns) => patterns.iter().map(|p| (p.as_str(), None)).collect(),

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
            slides.extend(Slide::load_collection(pattern, section_key, templates_dir)?);
        }

        Ok(slides)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckConfigCollection(IndexMap<String, DeckConfig>);

impl DeckConfigCollection {
    pub fn load(decks_path: &Path) -> Result<Self> {
        if !decks_path.exists() {
            return Err(Error::DecksNotFound);
        }

        let content = fs::read_to_string(decks_path).map_err(|e| Error::FileRead {
            path: decks_path.to_path_buf(),
            source: e,
        })?;

        let decks: IndexMap<String, DeckConfig> =
            serde_yaml::from_str(&content).map_err(|e| Error::DecksParseError(e.to_string()))?;

        Ok(Self(decks))
    }

    pub fn get(&self, name: &str) -> Result<DeckConfig> {
        self.0
            .get(name)
            .cloned()
            .ok_or_else(|| Error::DeckNotFound(name.to_string()))
    }
}

pub struct Deck {
    name: String,
    lang: String,
    root: PathBuf,
    slides: Vec<Slide>,
}

impl Deck {
    pub(super) fn new(name: &str, lang: &str, root: &Path) -> Self {
        Self {
            name: name.to_string(),
            lang: lang.to_string(),
            root: root.to_path_buf(),
            slides: Vec::new(),
        }
    }

    fn decks_config(&self) -> PathBuf {
        self.root.join("decks.yaml")
    }

    fn templates_dir(&self) -> PathBuf {
        self.root.join("slides/templates")
    }

    fn translations_dir(&self) -> PathBuf {
        self.root.join("slides/translations")
    }

    fn assets_dir(&self) -> PathBuf {
        self.root.join("slides/assets")
    }

    fn output_dir(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn load(&mut self) -> Result<()> {
        self.slides = DeckConfigCollection::load(&self.decks_config())?
            .get(&self.name)?
            .load_slides(&self.templates_dir())?;

        if self.slides.is_empty() {
            return Err(Error::NoSlides(self.name.clone()));
        }

        Ok(())
    }

    pub fn build(&mut self) -> Result<()> {
        self.build_html()?;
        chrome::generate_thumbnails(&self.output_dir(), self.slides.len())?;
        println!("\n🎉 Done! Open output/slide-1.html in browser");
        Ok(())
    }

    /// Build HTML only (no thumbnails) - useful for watch mode and testing
    pub fn build_html(&mut self) -> Result<()> {
        println!("🎬 Building presentation\n");

        let output_dir = self.output_dir();

        print!("📄 Loading deck '{}'... ", self.name);
        self.load()?;
        println!("✅ {} slides", self.slides.len());

        print!("🧱 Loading templates... ");
        let layouts = render::load_layouts(&self.templates_dir())?;
        println!("✅ {} templates", layouts.len());

        let translations =
            translations::load(&self.lang, &self.translations_dir()).unwrap_or_default();

        print!("🔨 Rendering... ");
        let pages = render::render_deck_pages(&self.slides, &layouts, &translations);
        let overview = render::render_overview(&layouts, &translations, &self.slides);
        let presenter_pages = render::render_presenter_pages(&layouts, &translations, &self.slides);
        let print = render::render_print(&layouts, &translations, &self.slides);
        println!("✅ {} pages", pages.len());

        print!("💾 Writing files... ");
        fs::create_dir_all(&output_dir).map_err(|e| Error::CreateDir {
            path: output_dir.clone(),
            source: e,
        })?;

        util::write_pages(&output_dir, &pages)?;
        util::write_pages(&output_dir.join("presenter"), &presenter_pages)?;

        for (name, html) in [("overview.html", overview), ("print.html", print)] {
            if let Some(content) = html {
                let path = output_dir.join(name);
                fs::write(&path, content).map_err(|e| Error::FileWrite { path, source: e })?;
            }
        }
        println!("✅");

        let assets_src = self.assets_dir();
        if assets_src.exists() {
            print!("📦 Copying assets... ");
            util::copy_dir_recursive(&assets_src, &output_dir.join("assets"))?;
            println!("✅");
        }

        Ok(())
    }

    pub fn watch(&mut self) -> Result<()> {
        use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
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

        // Initial build with thumbnails
        if let Err(e) = self.build() {
            eprintln!("❌ Error: {}", e);
        }
        println!();
        println!("─────────────────────────────────────────────────────");
        println!();

        // Setup file watcher
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
            .map_err(|e| Error::WatcherInit(e.to_string()))?;

        let mut watch = |path: &Path, mode: RecursiveMode| -> Result<()> {
            debouncer
                .watcher()
                .watch(path, mode)
                .map_err(|e| Error::WatchPath {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                })
        };

        // Watch templates (required)
        watch(&self.templates_dir(), RecursiveMode::Recursive)?;

        // Watch optional paths if they exist
        let decks_config = self.decks_config();
        if decks_config.exists() {
            watch(&decks_config, RecursiveMode::NonRecursive)?;
        }

        let translations_dir = self.translations_dir();
        if translations_dir.exists() {
            watch(&translations_dir, RecursiveMode::Recursive)?;
        }

        println!("✅ Watching for changes...");
        println!();

        loop {
            let Ok(result) = rx.recv() else {
                warn!("Channel closed");
                break;
            };

            let Ok(events) = result else {
                warn!("Watch error");
                continue;
            };

            if events.is_empty() {
                continue;
            }

            for event in &events {
                let Some(path) = event.path.to_str() else {
                    continue;
                };
                if path.contains(".swp") || path.contains(".tmp") || path.contains("~") {
                    continue;
                }
                println!("📝 Change detected: {path}");
            }

            println!("\n🔨 Rebuilding...");
            if let Err(e) = self.build_html() {
                eprintln!("❌ Error: {e}");
            }
            println!("\n─────────────────────────────────────────────────────\n");
        }

        Ok(())
    }
}
