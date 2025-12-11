//! PDF Export integration tests
//!
//! Tests the PDF generation from thumbnails

use serial_test::serial;
use slides_rs::infrastructure::pdf;
use std::path::Path;

const THUMBNAILS_PATH: &str = "tests/fixtures/thumbnails";

/// Test that PDF can be generated from existing thumbnails
#[test]
#[serial]
fn test_generate_pdf_from_thumbnails() {
    let thumbnails_dir = Path::new(THUMBNAILS_PATH);

    // Verify fixtures exist
    assert!(
        thumbnails_dir.exists(),
        "Thumbnails fixture directory should exist"
    );
    assert!(
        thumbnails_dir.join("slide-1.png").exists(),
        "slide-1.png should exist in fixtures"
    );

    // Generate PDF from 3 thumbnail slides
    let result = pdf::generate_from_thumbnails(thumbnails_dir, 3);

    assert!(
        result.is_ok(),
        "PDF generation should succeed: {:?}",
        result.err()
    );

    let pdf_bytes = result.unwrap();

    // Check PDF header (magic bytes)
    assert!(
        pdf_bytes.starts_with(b"%PDF"),
        "Output should be a valid PDF"
    );

    // Should be a reasonable size (at least a few KB per slide)
    assert!(
        pdf_bytes.len() > 1000,
        "PDF should have substantial content"
    );
}

/// Test PDF encryption with password
#[test]
#[serial]
fn test_pdf_encryption() {
    let thumbnails_dir = Path::new(THUMBNAILS_PATH);

    let pdf_bytes =
        pdf::generate_from_thumbnails(thumbnails_dir, 1).expect("PDF generation should succeed");

    // Encrypt with password
    let password = "test-password-123";
    let result = pdf::encrypt(pdf_bytes, password, false, false);

    assert!(
        result.is_ok(),
        "PDF encryption should succeed: {:?}",
        result.err()
    );

    let (encrypted_bytes, success) = result.unwrap();
    assert!(success, "Encryption should report success");
    assert!(
        encrypted_bytes.starts_with(b"%PDF"),
        "Encrypted output should still be valid PDF"
    );
}

/// Test PDF encryption with no-print flag
#[test]
#[serial]
fn test_pdf_encryption_no_print() {
    let thumbnails_dir = Path::new(THUMBNAILS_PATH);

    let pdf_bytes =
        pdf::generate_from_thumbnails(thumbnails_dir, 1).expect("PDF generation should succeed");

    let result = pdf::encrypt(pdf_bytes, "password", true, false);

    assert!(
        result.is_ok(),
        "PDF encryption with no-print should succeed"
    );
}

/// Test PDF encryption with no-copy flag
#[test]
#[serial]
fn test_pdf_encryption_no_copy() {
    let thumbnails_dir = Path::new(THUMBNAILS_PATH);

    let pdf_bytes =
        pdf::generate_from_thumbnails(thumbnails_dir, 1).expect("PDF generation should succeed");

    let result = pdf::encrypt(pdf_bytes, "password", false, true);

    assert!(result.is_ok(), "PDF encryption with no-copy should succeed");
}

/// Test secure password generation
#[test]
fn test_generate_secure_password() {
    let password1 = pdf::generate_secure_password();
    let password2 = pdf::generate_secure_password();

    // Password should be 16 characters
    assert_eq!(password1.len(), 16, "Password should be 16 characters");

    // Passwords should be different (random)
    assert_ne!(password1, password2, "Generated passwords should be unique");

    // Password should only contain allowed characters
    for c in password1.chars() {
        assert!(
            c.is_alphanumeric() || "!@#$%".contains(c),
            "Password should only contain allowed characters"
        );
    }
}

/// Test error handling for missing thumbnails
#[test]
fn test_pdf_error_missing_thumbnail() {
    let nonexistent = Path::new("tests/fixtures/nonexistent");

    let result = pdf::generate_from_thumbnails(nonexistent, 1);

    assert!(
        result.is_err(),
        "Should fail when thumbnails directory doesn't exist"
    );
}
