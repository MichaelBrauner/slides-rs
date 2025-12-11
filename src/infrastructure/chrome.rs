//! Chrome module - Headless browser operations for thumbnails

use crate::error::{Error, Result};
use headless_chrome::{
    protocol::cdp::{Emulation, Page},
    Browser,
};
use std::fs;
use std::path::Path;
use std::time::Duration;

fn device_metrics(height: u32) -> Emulation::SetDeviceMetricsOverride {
    Emulation::SetDeviceMetricsOverride {
        width: 1920,
        height,
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
    }
}

pub fn generate_thumbnails(output_dir: &Path, total_slides: usize) -> Result<()> {
    print!("📸 Generating thumbnails... ");

    // Get absolute path
    let output_dir = output_dir.canonicalize().map_err(|e| Error::FileRead {
        path: output_dir.to_path_buf(),
        source: e,
    })?;

    // Create thumbnails directory
    let thumbnails_dir = output_dir.join("thumbnails");
    fs::create_dir_all(&thumbnails_dir).map_err(|e| Error::CreateDir {
        path: thumbnails_dir.clone(),
        source: e,
    })?;

    // Start browser
    let browser = Browser::default().map_err(|e| {
        let install_hint = get_chrome_install_hint();
        Error::Browser(format!(
            "Could not start browser: {e}\n\n\
            Chrome/Chromium is required for thumbnail generation.\n\n\
            {install_hint}"
        ))
    })?;

    let tab = browser
        .new_tab()
        .map_err(|e| Error::Browser(format!("Could not open tab: {e}")))?;

    tab.set_default_timeout(Duration::from_secs(30));

    for i in 1..=total_slides {
        let filename = format!("slide-{i}.html");
        let output_display = output_dir.display();
        let file_url = format!("file://{output_display}/{filename}");

        tab.call_method(device_metrics(1080)).ok();

        // Load page
        tab.navigate_to(&file_url)
            .map_err(|e| Error::Browser(format!("Could not load slide {i}: {e}")))?;

        // Wait for page load
        tab.wait_for_element("body")
            .map_err(|e| Error::Browser(format!("Timeout loading slide {i}: {e}")))?;

        std::thread::sleep(Duration::from_millis(300));

        // Capture full-page screenshot
        let screenshot = tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| Error::Browser(format!("Could not capture slide {i}: {e}")))?;

        // Save screenshot
        let screenshot_path = thumbnails_dir.join(format!("slide-{i}.png"));
        fs::write(&screenshot_path, &screenshot).map_err(|e| Error::FileWrite {
            path: screenshot_path,
            source: e,
        })?;
    }

    println!("✅ {} thumbnails", total_slides);
    Ok(())
}

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
