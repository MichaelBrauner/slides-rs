//! PDF module - PDF generation and encryption for slide exports

use crate::error::{Error, Result};
use md5::{Digest, Md5};
use printpdf::{ColorBits, ColorSpace, Image, ImageTransform, ImageXObject, Mm, PdfDocument, Px};
use rand::Rng;
use std::io::{BufWriter, Cursor};
use std::path::Path;

pub fn generate_from_thumbnails(thumbnails_dir: &Path, slide_count: usize) -> Result<Vec<u8>> {
    // 16:9 aspect ratio, similar to PowerPoint defaults
    let width_mm = 338.666; // ~13.33 inches
    let height_mm = 190.5; // ~7.5 inches

    let (doc, mut page_index, mut layer_index) =
        PdfDocument::new("Presentation", Mm(width_mm), Mm(height_mm), "Slide 1");

    for i in 1..=slide_count {
        let png_path = thumbnails_dir.join(format!("slide-{i}.png"));

        // Load PNG image using image crate
        let img = ::image::open(&png_path).map_err(|e| {
            Error::PdfGeneration(format!("Could not open {}: {e}", png_path.display()))
        })?;

        let img_rgb = img.to_rgb8();
        let (img_width, img_height) = img_rgb.dimensions();

        // Create printpdf Image
        let image = Image::from(ImageXObject {
            width: Px(img_width as usize),
            height: Px(img_height as usize),
            color_space: ColorSpace::Rgb,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: img_rgb.into_raw(),
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        });

        // Add page (except for first slide which already has a page)
        if i > 1 {
            let (new_page, new_layer) =
                doc.add_page(Mm(width_mm), Mm(height_mm), format!("Slide {i}"));
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
        .map_err(|e| Error::PdfGeneration(format!("Could not generate PDF: {e}")))?;

    buf.into_inner()
        .map_err(|e| Error::PdfGeneration(format!("Buffer error: {e}")))
}

pub fn generate_secure_password() -> String {
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

pub fn encrypt(
    pdf_bytes: Vec<u8>,
    password: &str,
    no_print: bool,
    no_copy: bool,
) -> Result<(Vec<u8>, bool)> {
    use lopdf::{Document, EncryptionState, EncryptionVersion, Object, Permissions, StringFormat};

    let mut document = Document::load_from(Cursor::new(&pdf_bytes))
        .map_err(|e| Error::PdfEncryption(format!("Could not load PDF: {e}")))?;

    if !document.trailer.has(b"ID") {
        let mut hasher = Md5::new();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        hasher.update(nanos.to_string().as_bytes());
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
        .map_err(|e| Error::PdfEncryption(format!("Could not initialize encryption: {e}")))?;

    document
        .encrypt(&encryption_state)
        .map_err(|e| Error::PdfEncryption(format!("Encryption failed: {e}")))?;

    let mut buffer = Vec::new();
    document
        .save_to(&mut buffer)
        .map_err(|e| Error::PdfEncryption(format!("Could not save encrypted PDF: {e}")))?;

    Ok((buffer, true))
}
