//! Rendering integration tests
//!
//! Tests template engine, layout loading, and slide rendering
//! Uses minimal fixtures from tests/fixtures/minimal/

use serial_test::serial;
use slides_rs::model::Slide;
use slides_rs::services::render::{
    load_layouts, render_deck_pages, render_overview, render_presenter_pages,
};
use std::collections::HashMap;
use std::path::Path;

const FIXTURES_PATH: &str = "tests/fixtures/minimal/slides";

/// Test that layouts can be loaded from a directory
#[test]
#[serial]
fn test_load_layouts() {
    let fixtures_path = Path::new(FIXTURES_PATH);
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    // Should find slides in templates/slides/
    assert!(
        layouts.contains_key("templates/slides/01-intro.html"),
        "Should contain templates/slides/01-intro.html"
    );
    assert!(
        layouts.contains_key("templates/slides/02-content.html"),
        "Should contain templates/slides/02-content.html"
    );
}

/// Test basic slide rendering with minijinja
#[test]
#[serial]
fn test_render_single_slide() {
    let fixtures_path = Path::new(FIXTURES_PATH);
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    let slides = vec![Slide {
        template: "templates/slides/01-intro.html".into(),
        section_key: None,
    }];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_deck_pages(&slides, &layouts, &translations);

    // Should generate slide-1.html
    assert!(
        pages.contains_key("slide-1.html"),
        "Should generate slide-1.html"
    );

    // Check content
    let slide1 = pages.get("slide-1.html").unwrap();
    assert!(slide1.contains("Welcome"), "Should contain slide title");
}

/// Test navigation links
#[test]
#[serial]
fn test_navigation_links() {
    let fixtures_path = Path::new(FIXTURES_PATH);
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    let slides = vec![
        Slide {
            template: "templates/slides/01-intro.html".into(),
            section_key: None,
        },
        Slide {
            template: "templates/slides/02-content.html".into(),
            section_key: None,
        },
        Slide {
            template: "templates/slides/03-end.html".into(),
            section_key: None,
        },
    ];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_deck_pages(&slides, &layouts, &translations);

    // First slide: no prev, has next
    let slide1 = pages.get("slide-1.html").unwrap();
    assert!(
        !slide1.contains("slide-0.html"),
        "First slide should not have prev link"
    );
    assert!(
        slide1.contains("slide-2.html"),
        "First slide should have next link"
    );

    // Middle slide: has prev and next
    let slide2 = pages.get("slide-2.html").unwrap();
    assert!(
        slide2.contains("slide-1.html"),
        "Middle slide should have prev link"
    );
    assert!(
        slide2.contains("slide-3.html"),
        "Middle slide should have next link"
    );

    // Last slide: has prev, no next
    let slide3 = pages.get("slide-3.html").unwrap();
    assert!(
        slide3.contains("slide-2.html"),
        "Last slide should have prev link"
    );
    assert!(
        !slide3.contains("slide-4.html"),
        "Last slide should not have next link"
    );
}

/// Test slide.current and app.total variables
#[test]
#[serial]
fn test_slide_numbers() {
    let fixtures_path = Path::new(FIXTURES_PATH);
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    let slides = vec![
        Slide {
            template: "templates/slides/01-intro.html".into(),
            section_key: None,
        },
        Slide {
            template: "templates/slides/02-content.html".into(),
            section_key: None,
        },
        Slide {
            template: "templates/slides/03-end.html".into(),
            section_key: None,
        },
    ];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_deck_pages(&slides, &layouts, &translations);

    // Check slide numbers are correct
    let slide1 = pages.get("slide-1.html").unwrap();
    assert!(slide1.contains("1 / 3"), "Slide 1 should show '1 / 3'");

    let slide2 = pages.get("slide-2.html").unwrap();
    assert!(slide2.contains("2 / 3"), "Slide 2 should show '2 / 3'");

    let slide3 = pages.get("slide-3.html").unwrap();
    assert!(slide3.contains("3 / 3"), "Slide 3 should show '3 / 3'");
}

/// Test speaker notes - minimal fixtures don't have notes, so just check templates load
#[test]
#[serial]
fn test_speaker_notes() {
    let fixtures_path = Path::new(FIXTURES_PATH);
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    // Minimal fixtures have simple templates without notes block
    let slide_content = layouts.get("templates/slides/01-intro.html").unwrap();
    assert!(slide_content.contains("<h1>"), "Slide should have content");
}

/// Test overview page renders when template exists
#[test]
#[serial]
fn test_render_overview_with_template() {
    let fixtures_path = Path::new("tests/fixtures/default/slides/templates");
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    let slides = vec![
        Slide {
            template: "slides/01-intro.html".into(),
            section_key: None,
        },
        Slide {
            template: "slides/02-content.html".into(),
            section_key: Some("main".into()),
        },
    ];
    let translations: HashMap<String, String> = HashMap::new();

    let result = render_overview(&layouts, &translations, &slides);

    assert!(
        result.is_some(),
        "Should render overview when template exists"
    );
    let html = result.unwrap();
    assert!(
        html.contains("slide-1.html"),
        "Overview should link to slides"
    );
}

/// Test overview returns None when template missing
#[test]
#[serial]
fn test_render_overview_without_template() {
    let layouts: HashMap<String, String> = HashMap::new(); // No templates
    let slides = vec![Slide {
        template: "test.html".into(),
        section_key: None,
    }];
    let translations: HashMap<String, String> = HashMap::new();

    let result = render_overview(&layouts, &translations, &slides);

    assert!(
        result.is_none(),
        "Should return None when overview.html template is missing"
    );
}

/// Test presenter pages render when template exists
#[test]
#[serial]
fn test_render_presenter_pages_with_template() {
    let fixtures_path = Path::new("tests/fixtures/default/slides/templates");
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    let slides = vec![
        Slide {
            template: "slides/01-intro.html".into(),
            section_key: None,
        },
        Slide {
            template: "slides/02-features.html".into(),
            section_key: None,
        },
    ];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_presenter_pages(&layouts, &translations, &slides);

    assert_eq!(pages.len(), 2, "Should render one presenter page per slide");
    assert!(
        pages.contains_key("slide-1.html"),
        "Should have presenter page for slide 1"
    );
    assert!(
        pages.contains_key("slide-2.html"),
        "Should have presenter page for slide 2"
    );

    let page1 = pages.get("slide-1.html").unwrap();
    assert!(
        page1.contains("notes") || page1.contains("Notes"),
        "Presenter page should have notes section"
    );
}

/// Test presenter returns empty map when template missing
#[test]
#[serial]
fn test_render_presenter_pages_without_template() {
    let layouts: HashMap<String, String> = HashMap::new(); // No templates
    let slides = vec![Slide {
        template: "test.html".into(),
        section_key: None,
    }];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_presenter_pages(&layouts, &translations, &slides);

    assert!(
        pages.is_empty(),
        "Should return empty map when presenter.html template is missing"
    );
}

/// Test single-slide deck edge case for presenter pages
#[test]
#[serial]
fn test_presenter_single_slide_deck() {
    let fixtures_path = Path::new("tests/fixtures/default/slides/templates");
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    // Only one slide
    let slides = vec![Slide {
        template: "slides/01-intro.html".into(),
        section_key: None,
    }];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_presenter_pages(&layouts, &translations, &slides);

    assert_eq!(pages.len(), 1, "Should render one presenter page");
    assert!(
        pages.contains_key("slide-1.html"),
        "Should have presenter page for slide 1"
    );

    let page = pages.get("slide-1.html").unwrap();

    // Single slide should be both first and last
    assert!(
        page.contains("Slide 1 / 1") || page.contains("1 / 1"),
        "Should show slide 1 of 1"
    );

    // Should NOT have navigation to other slides
    assert!(
        !page.contains("slide-2.html"),
        "Single slide should not link to slide 2"
    );
    assert!(
        !page.contains("slide-0.html"),
        "Single slide should not link to slide 0"
    );
}

/// Test asset() function generates correct relative paths for nested directories
#[test]
#[serial]
fn test_asset_paths_in_presenter_pages() {
    let fixtures_path = Path::new("tests/fixtures/default/slides/templates");
    let layouts = load_layouts(fixtures_path).expect("Should load layouts");

    let slides = vec![
        Slide {
            template: "slides/01-intro.html".into(),
            section_key: None,
        },
        Slide {
            template: "slides/02-features.html".into(),
            section_key: None,
        },
    ];
    let translations: HashMap<String, String> = HashMap::new();

    let pages = render_presenter_pages(&layouts, &translations, &slides);
    let page1 = pages.get("slide-1.html").unwrap();

    // Presenter pages are in presenter/ subdirectory, so assets should use ../
    assert!(
        page1.contains("../assets/") || page1.contains("href=\"../assets"),
        "Presenter page should reference assets with ../ prefix for correct relative path"
    );

    // Should NOT have broken paths without the prefix
    assert!(
        !page1.contains("href=\"assets/"),
        "Presenter page should not use assets/ without ../ prefix"
    );
}
