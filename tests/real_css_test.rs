//! Test CSS extraction with real captured CSS files

use crawlwe_core::css::CssAssetExtractor;
use url::Url;
use std::fs;

#[test]
fn test_adaline_css_extraction() {
    // Read the real captured CSS
    let css_path = concat!(env!("CARGO_MANIFEST_DIR"), "/captures/adaline/styles.css");
    let css = match fs::read_to_string(css_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Skipping test - adaline capture not found");
            return;
        }
    };

    println!("\n=== Adaline CSS Analysis ===");
    println!("CSS size: {} bytes ({} lines)", css.len(), css.lines().count());

    let extractor = CssAssetExtractor::new();
    let base_url = Url::parse("https://adaline.ai/").unwrap();
    let result = extractor.extract(&css, &base_url);

    println!("\n--- Extraction Results ---");
    println!("Total URLs found: {}", result.stats.total_urls_found);
    println!("Fonts found: {}", result.stats.fonts_found);
    println!("Images found: {}", result.stats.images_found);
    println!("@imports found: {}", result.stats.imports_found);
    println!("Data URIs skipped: {}", result.stats.data_uris_skipped);
    println!("Invalid URLs: {}", result.stats.invalid_urls);

    // Print font details
    if !result.fonts.is_empty() {
        println!("\n--- Fonts ({}) ---", result.fonts.len());
        for (url, info) in result.fonts.iter().take(10) {
            let family = info.font_family.as_deref().unwrap_or("unknown");
            let weight = info.font_weight.as_deref().unwrap_or("?");
            if url.starts_with("local:") {
                println!("  [local] {} ({})", url.replace("local:", ""), family);
            } else {
                println!("  {} [{}:{}]", shorten(url, 60), family, weight);
            }
        }
        if result.fonts.len() > 10 {
            println!("  ... and {} more", result.fonts.len() - 10);
        }
    }

    // Print image details
    if !result.images.is_empty() {
        println!("\n--- Images ({}) ---", result.images.len());
        for (url, info) in result.images.iter().take(10) {
            println!("  {} [{:?}]", shorten(url, 50), info.context);
        }
        if result.images.len() > 10 {
            println!("  ... and {} more", result.images.len() - 10);
        }
    }

    // Print @imports
    if !result.imports.is_empty() {
        println!("\n--- @imports ({}) ---", result.imports.len());
        for import in result.imports.iter().take(5) {
            println!("  {}", shorten(import, 70));
        }
    }

    // Print errors
    if !result.errors.is_empty() {
        println!("\n--- Errors ({}) ---", result.errors.len());
        for err in result.errors.iter().take(5) {
            println!("  ! {}", err);
        }
    }

    // Basic assertions
    assert!(result.stats.total_urls_found > 0, "Should find some URLs in the CSS");

    println!("\n=== Test Passed ===");
}

#[test]
fn test_stripe_css_extraction() {
    let css_path = concat!(env!("CARGO_MANIFEST_DIR"), "/captures/stripe-dev/styles.css");
    let css = match fs::read_to_string(css_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Skipping test - stripe capture not found");
            return;
        }
    };

    println!("\n=== Stripe CSS Analysis ===");
    println!("CSS size: {} bytes ({} lines)", css.len(), css.lines().count());

    let extractor = CssAssetExtractor::new();
    let base_url = Url::parse("https://stripe.dev/").unwrap();
    let result = extractor.extract(&css, &base_url);

    println!("\n--- Extraction Results ---");
    println!("Total URLs found: {}", result.stats.total_urls_found);
    println!("Fonts found: {}", result.stats.fonts_found);
    println!("Images found: {}", result.stats.images_found);
    println!("@imports found: {}", result.stats.imports_found);
    println!("Data URIs skipped: {}", result.stats.data_uris_skipped);

    println!("\n=== Test Passed ===");
}

fn shorten(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
