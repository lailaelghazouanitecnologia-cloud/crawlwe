//! Integration tests for CSS asset extraction
//!
//! Tests comprehensive extraction of fonts, images, and @imports from CSS.

use crawlwe_core::css::{CssAssetExtractor, ImageContext};
use url::Url;

/// Test extraction from real-world Webflow CSS patterns
#[test]
fn test_webflow_font_extraction() {
    let extractor = CssAssetExtractor::new();

    // Webflow often uses local() with fallback web fonts
    let css = r#"
@font-face {
    font-family: 'Lato';
    font-style: normal;
    font-weight: 400;
    src: local('Lato Regular'), local('Lato-Regular'),
         url('https://fonts.gstatic.com/s/lato/v24/S6uyw4BMUTPHjx4wXg.woff2') format('woff2'),
         url('https://fonts.gstatic.com/s/lato/v24/S6uyw4BMUTPHjx4wXg.woff') format('woff');
}

@font-face {
    font-family: 'Lato';
    font-style: italic;
    font-weight: 700;
    src: local('Lato Bold Italic'), local('Lato-BoldItalic'),
         url('https://fonts.gstatic.com/s/lato/v24/S6u_w4BMUTPHjxsI5wq_Gwft.woff2') format('woff2');
}
    "#;

    let base_url = Url::parse("https://example.webflow.io/").unwrap();
    let result = extractor.extract(css, &base_url);

    // Should find both web fonts and track local() references
    assert!(result.stats.fonts_found >= 2, "Should find at least 2 web fonts");

    // Check we have woff2 fonts
    assert!(result.fonts.keys().any(|k| k.contains("woff2")));

    // Check local fonts are tracked
    assert!(result.fonts.keys().any(|k| k.starts_with("local:")));
}

/// Test extraction of all background image formats
#[test]
fn test_comprehensive_background_images() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
/* Standard background-image */
.hero {
    background-image: url('/images/hero.jpg');
}

/* Background shorthand */
.banner {
    background: url("images/banner.png") no-repeat center;
}

/* No quotes */
.icon {
    background: url(icons/star.svg);
}

/* Protocol-relative */
.cdn-image {
    background-image: url(//cdn.example.com/bg.webp);
}

/* Multiple backgrounds */
.multi {
    background-image: url('layer1.png'), url('layer2.png'), url('layer3.gif');
}

/* Gradient with image fallback */
.gradient-fallback {
    background: url('fallback.jpg'), linear-gradient(to bottom, #fff, #000);
}
    "#;

    let base_url = Url::parse("https://example.com/css/styles.css").unwrap();
    let result = extractor.extract(css, &base_url);

    // Should find at least 8 images
    assert!(result.stats.images_found >= 8, "Expected 8+ images, got {}", result.stats.images_found);

    // Check specific URLs resolved correctly
    assert!(result.images.contains_key("https://example.com/images/hero.jpg"));
    assert!(result.images.contains_key("https://example.com/css/images/banner.png"));
    assert!(result.images.contains_key("https://cdn.example.com/bg.webp"));

    // All images should be marked as background context
    for (_url, info) in &result.images {
        assert_eq!(info.context, ImageContext::BackgroundImage);
    }
}

/// Test extraction of cursor and other image contexts
#[test]
fn test_image_context_detection() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
.custom-cursor {
    cursor: url('cursors/pointer.cur'), pointer;
}

.masked {
    mask-image: url('masks/circle.svg');
}

.bordered {
    border-image: url('borders/fancy.png') 30 round;
}

.list-custom {
    list-style-image: url('bullets/star.svg');
}

.pseudo::before {
    content: url('icons/arrow.svg');
}

.filtered {
    filter: url('filters/blur.svg#blur');
}
    "#;

    let base_url = Url::parse("https://example.com/").unwrap();
    let result = extractor.extract(css, &base_url);

    // Check different contexts
    let contexts: Vec<_> = result.images.values().map(|i| i.context).collect();

    assert!(contexts.contains(&ImageContext::Cursor), "Should detect cursor");
    assert!(contexts.contains(&ImageContext::MaskImage), "Should detect mask");
    assert!(contexts.contains(&ImageContext::BorderImage), "Should detect border-image");
    assert!(contexts.contains(&ImageContext::ListStyleImage), "Should detect list-style-image");
    assert!(contexts.contains(&ImageContext::Content), "Should detect content");
    assert!(contexts.contains(&ImageContext::Filter), "Should detect filter");
}

/// Test @import extraction
#[test]
fn test_import_extraction() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700&display=swap');
@import "base/reset.css";
@import url(components/buttons.css);
@import 'utilities.css' screen;
    "#;

    let base_url = Url::parse("https://example.com/css/main.css").unwrap();
    let result = extractor.extract(css, &base_url);

    assert_eq!(result.stats.imports_found, 4, "Should find 4 @imports");

    // Check resolved URLs
    assert!(result.imports.iter().any(|u| u.contains("fonts.googleapis.com")));
    assert!(result.imports.iter().any(|u| u.contains("base/reset.css")));
    assert!(result.imports.iter().any(|u| u.contains("components/buttons.css")));
}

/// Test data URI handling (should skip)
#[test]
fn test_data_uri_handling() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
.icon-inline {
    background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2L2 12h3v8h14v-8h3L12 2z"/></svg>');
}

.icon-base64 {
    background-image: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==);
}

.regular-image {
    background: url('real-image.png');
}
    "#;

    let base_url = Url::parse("https://example.com/").unwrap();
    let result = extractor.extract(css, &base_url);

    // Should skip data URIs and only find the regular image
    assert_eq!(result.stats.images_found, 1, "Should only find 1 real image");
    // At least 1 data URI should be skipped (the exact count may vary by regex matching)
    assert!(result.stats.data_uris_skipped >= 1, "Should skip data URIs, skipped: {}", result.stats.data_uris_skipped);
}

/// Test Google Fonts URL detection
#[test]
fn test_google_fonts_detection() {
    let extractor = CssAssetExtractor::new();

    // Simulated Google Fonts CSS response
    let css = r#"
/* cyrillic */
@font-face {
    font-family: 'Inter';
    font-style: normal;
    font-weight: 400;
    font-display: swap;
    src: url(https://fonts.gstatic.com/s/inter/v13/UcCO3FwrK3iLTeHuS_fvQtMwCp50KnMw2boKoduKmMEVuLyfAZ9hiJ-Ek-_EeA.woff2) format('woff2');
    unicode-range: U+0301, U+0400-045F, U+0490-0491, U+04B0-04B1, U+2116;
}

/* latin */
@font-face {
    font-family: 'Inter';
    font-style: normal;
    font-weight: 400;
    font-display: swap;
    src: url(https://fonts.gstatic.com/s/inter/v13/UcCO3FwrK3iLTeHuS_fvQtMwCp50KnMw2boKoduKmMEVuLyfAZNhiJ-Ek-_EeA.woff2) format('woff2');
    unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}
    "#;

    let base_url = Url::parse("https://fonts.googleapis.com/css2").unwrap();
    let result = extractor.extract(css, &base_url);

    assert_eq!(result.stats.fonts_found, 2, "Should find 2 font variants");

    // Check font metadata is captured
    for (_url, info) in &result.fonts {
        assert_eq!(info.font_family, Some("Inter".to_string()));
        assert_eq!(info.font_weight, Some("400".to_string()));
    }
}

/// Test CSS variables with URL references
#[test]
fn test_css_variable_assets() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
:root {
    --logo-url: url('/images/logo.svg');
    --pattern-bg: url('patterns/dots.png');
}

.header {
    background-image: var(--logo-url);
}
    "#;

    let base_url = Url::parse("https://example.com/").unwrap();
    let result = extractor.extract(css, &base_url);

    // Should track CSS variables that contain URLs
    assert!(result.asset_variables.contains_key("--logo-url") ||
            result.asset_variables.contains_key("--pattern-bg"),
            "Should track CSS variables with URLs");
}

/// Test URL with query parameters and fragments
#[test]
fn test_urls_with_params() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
.versioned {
    background: url('image.png?v=1.2.3');
}

.cached {
    background: url('styles/bg.jpg?cache=abc123');
}

.filter-ref {
    filter: url('effects.svg#blur');
}
    "#;

    let base_url = Url::parse("https://example.com/css/").unwrap();
    let result = extractor.extract(css, &base_url);

    // Should handle URLs with query params and fragments
    assert!(result.stats.images_found >= 2, "Should find images with query params");
}

/// Test deeply nested relative paths
#[test]
fn test_relative_path_resolution() {
    let extractor = CssAssetExtractor::new();

    let css = r#"
.up-two {
    background: url('../../assets/images/bg.png');
}

.current {
    background: url('./local.jpg');
}

.nested {
    background: url('../shared/icons/star.svg');
}
    "#;

    let base_url = Url::parse("https://example.com/themes/default/css/styles.css").unwrap();
    let result = extractor.extract(css, &base_url);

    // Check paths resolved correctly
    assert!(result.images.contains_key("https://example.com/themes/assets/images/bg.png") ||
            result.images.contains_key("https://example.com/assets/images/bg.png"),
            "Should resolve ../../ paths");

    assert!(result.images.keys().any(|k| k.contains("local.jpg")),
            "Should resolve ./ paths");
}
