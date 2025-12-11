//! Project - the root aggregate representing a slides project

use super::deck::Deck;
use crate::error::{Error, Result};
use crate::infrastructure::pdf;
use crate::services::init;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const IMPORT_IMAGES_DIR: &str = "slides/assets/import/images";
const THUMBNAILS_DIR: &str = "output/thumbnails";

/// A slides project with a root directory
#[derive(Debug)]
pub struct Project {
    root: PathBuf,
}

impl Project {
    /// Open the current directory as a project
    pub fn current() -> Result<Self> {
        let root = std::env::current_dir().map_err(Error::CurrentDir)?;
        Ok(Self { root })
    }

    /// Initialize a new project in the current directory
    pub fn init() -> Result<Self> {
        let root = std::env::current_dir().map_err(Error::CurrentDir)?;

        if !init::is_dir_empty(&root)? {
            return Err(Error::DirNotEmpty);
        }

        println!("🚀 Initializing slides project...\n");

        init::create_project_files(&root)?;

        println!("\n🎉 Project ready! Run 'slides build' to see your presentation.");

        Ok(Self { root })
    }

    /// Create a new project in a new directory
    pub fn create(name: &str, path: Option<&str>) -> Result<Self> {
        let project_dir = path
            .map(|p| Path::new(p).join(name))
            .unwrap_or_else(|| Path::new(name).to_path_buf());

        if project_dir.exists() {
            return Err(Error::DirExists(project_dir));
        }

        fs::create_dir_all(&project_dir).map_err(|e| Error::CreateDir {
            path: project_dir.clone(),
            source: e,
        })?;

        println!("📁 Creating project '{name}'...\n");
        println!("🚀 Initializing slides project...\n");

        init::create_project_files(&project_dir)?;

        println!("\n🎉 Project ready! Run 'slides build' to see your presentation.");
        println!("\n💡 Run: cd {name} && slides build");

        Ok(Self { root: project_dir })
    }

    /// Get a deck by name and language
    pub fn deck(&self, name: &str, lang: &str) -> Deck {
        Deck::new(name, lang, &self.root)
    }

    /// Export the presentation as PDF
    pub fn export_pdf(
        &self,
        password: Option<String>,
        no_print: bool,
        no_copy: bool,
        output_path: &str,
    ) -> Result<()> {
        println!("📄 PDF Export\n");

        let thumbnails_dir = self.root.join(THUMBNAILS_DIR);
        if !thumbnails_dir.exists() {
            return Err(Error::ThumbnailsNotFound);
        }

        let slide_count = (1..)
            .take_while(|i| thumbnails_dir.join(format!("slide-{i}.png")).exists())
            .count();

        if slide_count == 0 {
            return Err(Error::NoThumbnails);
        }

        println!("📸 {slide_count} thumbnails found");
        let pdf_bytes = pdf::generate_from_thumbnails(&thumbnails_dir, slide_count)?;
        println!("   ✅ PDF generated ({} KB)\n", pdf_bytes.len() / 1024);

        let final_pdf = match &password {
            Some(pwd) => {
                let pwd = if pwd == "auto" {
                    pdf::generate_secure_password()
                } else {
                    pwd.clone()
                };
                println!("🔒 Encrypting...");
                let (encrypted, ok) = pdf::encrypt(pdf_bytes, &pwd, no_print, no_copy)?;
                if ok {
                    println!("   ✅ Encrypted (password: {pwd})");
                }
                encrypted
            }
            None => pdf_bytes,
        };

        let output = self.root.join(output_path);
        fs::write(&output, final_pdf).map_err(|e| Error::FileWrite {
            path: output,
            source: e,
        })?;
        println!("\n🎉 PDF saved to {output_path}");

        Ok(())
    }

    /// Import images from a PowerPoint file
    pub fn import_images(&self, pptx_file: &str) -> Result<()> {
        println!("📥 Extracting images from {pptx_file}");

        let pptx_path = PathBuf::from(pptx_file);
        let file = File::open(&pptx_path).map_err(|e| Error::FileRead {
            path: pptx_path.clone(),
            source: e,
        })?;
        let mut archive = ZipArchive::new(file).map_err(|e| Error::InvalidPptx(e.to_string()))?;

        let import_dir = self.root.join(IMPORT_IMAGES_DIR);
        fs::create_dir_all(&import_dir).map_err(|e| Error::CreateDir {
            path: import_dir.clone(),
            source: e,
        })?;

        let count = extract_pptx_images(&mut archive, &import_dir)?;

        if count > 0 {
            println!("   ✅ {count} images → {IMPORT_IMAGES_DIR}");
        } else {
            println!("   No images found");
        }

        Ok(())
    }
}

fn extract_pptx_images(archive: &mut ZipArchive<File>, output_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| Error::ZipError(e.to_string()))?;
        let path = file.name().to_string();

        if !path.starts_with("ppt/media/") {
            continue;
        }

        let Some(ext) = Path::new(&path).extension().and_then(|e| e.to_str()) else {
            continue;
        };

        if !matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp") {
            continue;
        }

        let filename = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image");

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| Error::ZipError(e.to_string()))?;
        let output_path = output_dir.join(filename);
        fs::write(&output_path, buffer).map_err(|e| Error::FileWrite {
            path: output_path,
            source: e,
        })?;

        count += 1;
    }

    Ok(count)
}
