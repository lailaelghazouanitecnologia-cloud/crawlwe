//! Static fetch - Downloads HTML/CSS/JS via HTTP without browser
//!
//! This is faster and works in restricted environments where Chrome can't run.
//! Limitations: No JavaScript execution, no computed styles.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

use crawlwe_core::pipeline::{CssMicroparser, LibMicroparser, HtmlOptimizer, HtmlOptimizeOptions, DetectedLibrary, CssParseResult};
use crawlwe_core::export::LibraryCDN;
use crawlwe_core::js::{JsAnalyzer, LibraryRegistry, JsToCssConverter, CssModulesParser, TailwindGenerator, SmartCssExtractor};
use crawlwe_core::css::{CssAssetExtractor, CssAssetExtractionResult, ImageContext, CssFormatter, CssFormatOptions};
use crawlwe_core::assets::{AssetRegistry, SvgAsset, SvgCategory, CssStats, ImageContext as AssetImageContext};

pub async fn run(
    url: &str,
    output: &Path,
    project_toml: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe - Static Fetch (HTTP)");
    println!("==============================");
    println!("URL: {}", url);
    println!();

    let base_url = Url::parse(url)?;
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Create output structure
    let data_dir = output.join("data");
    let scripts_dir = output.join("scripts");
    let assets_dir = output.join("assets");
    let fonts_dir = assets_dir.join("fonts");
    let images_dir = assets_dir.join("images");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&scripts_dir)?;
    fs::create_dir_all(&fonts_dir)?;
    fs::create_dir_all(&images_dir)?;

    // Download HTML
    println!("Downloading HTML...");
    let html_response = client.get(url).send().await?;
    let html_raw = html_response.text().await?;
    println!("  HTML: {} bytes", html_raw.len());

    // Extract title
    let title = extract_title(&html_raw).unwrap_or_else(|| "Untitled".to_string());
    println!("  Title: {}", title);

    // Extract and download CSS
    println!("\nDownloading CSS...");
    let css_urls = extract_css_urls(&html_raw, &base_url);
    println!("  Found {} stylesheets", css_urls.len());

    let mut all_css = String::new();
    for css_url in &css_urls {
        match client.get(css_url).send().await {
            Ok(resp) => {
                match resp.text().await {
                    Ok(css) => {
                        all_css.push_str(&format!("/* Source: {} */\n", css_url));
                        all_css.push_str(&css);
                        all_css.push_str("\n\n");
                        println!("    + {}", shorten_url(css_url));
                    }
                    Err(_) => {
                        println!("    ! {} (failed to read)", shorten_url(css_url));
                    }
                }
            }
            Err(e) => {
                println!("    ! {} ({})", shorten_url(css_url), e);
            }
        }
    }

    // Extract inline styles
    let inline_css = extract_inline_styles(&html_raw);
    if !inline_css.is_empty() {
        all_css.push_str("/* Inline styles */\n");
        all_css.push_str(&inline_css);
        println!("  + inline styles: {} bytes", inline_css.len());
    }

    println!("  Total CSS: {} bytes", all_css.len());

    // Download fonts from CSS
    println!("\nDownloading fonts...");
    let (css_with_local_fonts, font_map) = download_fonts(&client, &all_css, &base_url, &fonts_dir).await;
    println!("  Downloaded {} fonts", font_map.len());

    // Download images from CSS
    println!("\nDownloading images...");
    let (css_with_local_assets, image_map) = download_css_images(&client, &css_with_local_fonts, &base_url, &images_dir).await;
    println!("  Downloaded {} images from CSS", image_map.len());

    // Download images from HTML
    let (html_with_local_images, html_image_map) = download_html_images(&client, &html_raw, &base_url, &images_dir).await;
    println!("  Downloaded {} images from HTML", html_image_map.len());

    // Download Next.js optimized images
    let (html_with_nextjs, nextjs_image_map) = download_nextjs_images(&client, &html_with_local_images, &base_url, &images_dir).await;
    println!("  Downloaded {} Next.js images", nextjs_image_map.len());

    // Download external SVGs
    let (html_with_svgs, svg_map) = download_svg_images(&client, &html_with_nextjs, &base_url, &images_dir).await;
    println!("  Downloaded {} SVG files", svg_map.len());

    // Extract large inline SVGs to files
    let (html_final, inline_svg_count) = extract_inline_svgs(&html_with_svgs, &images_dir);
    if inline_svg_count > 0 {
        println!("  Extracted {} inline SVGs", inline_svg_count);
    }

    // Extract external scripts
    println!("\nFinding scripts...");
    let script_urls = extract_script_urls(&html_raw, &base_url);
    println!("  Found {} external scripts", script_urls.len());
    for script_url in &script_urls {
        println!("    - {}", shorten_url(script_url));
    }

    // Detect libraries from script URLs
    let lib_result = LibMicroparser::parse(&script_urls, "");
    println!("  Libraries detected: {}", lib_result.libraries.len());
    for lib in &lib_result.libraries {
        println!("    - {} ({})", lib.name, lib.category);
    }

    // Parse CSS
    let css_parsed = CssMicroparser::parse(&css_with_local_assets);
    println!("\nCSS Analysis:");
    println!("  Variables: {}", css_parsed.variables.len());
    println!("  Keyframes: {}", css_parsed.keyframes.len());
    println!("  Colors: {}", css_parsed.colors.len());

    // Optimize
    println!("\nOptimizing...");
    let html_optimized = HtmlOptimizer::optimize(&html_final, &HtmlOptimizeOptions {
        remove_comments: true,
        remove_scripts: true,
        remove_styles: false,
        remove_data_attrs: false,
        remove_framework_attrs: true,
        remove_empty_attrs: true,
        minify_whitespace: false,
        format_output: true,
        preserve_structure: true,
    });

    // Don't optimize CSS to keep local asset URLs intact
    let final_css = &css_with_local_assets;

    println!("  HTML: {} -> {} bytes", html_raw.len(), html_optimized.stats.optimized_size);
    println!("  CSS: {} bytes (with local assets)", final_css.len());

    // Save files
    println!("\nSaving files...");

    // Clean HTML with embedded CSS (no base href needed - assets are local)
    let clean_html = generate_clean_html_local(&html_optimized.html, &title, final_css);
    fs::write(output.join("index.html"), &clean_html)?;
    println!("  index.html");

    // Raw HTML
    fs::write(data_dir.join("raw.html"), &html_raw)?;
    println!("  data/raw.html");

    // CSS
    fs::write(output.join("styles.css"), final_css)?;
    println!("  styles.css");

    // Metadata
    let metadata = serde_json::json!({
        "url": url,
        "domain": base_url.host_str(),
        "title": title,
        "mode": "static",
        "extracted_at": chrono::Utc::now().to_rfc3339(),
        "stats": {
            "html_bytes": html_raw.len(),
            "html_optimized_bytes": html_optimized.stats.optimized_size,
            "css_bytes": final_css.len(),
            "external_scripts": script_urls.len(),
            "external_styles": css_urls.len(),
            "fonts_downloaded": font_map.len(),
            "images_downloaded": image_map.len() + html_image_map.len() + nextjs_image_map.len() + svg_map.len(),
            "nextjs_images": nextjs_image_map.len(),
            "svgs_downloaded": svg_map.len(),
            "svgs_extracted": inline_svg_count,
        },
        "css_analysis": {
            "variables": css_parsed.variables.len(),
            "keyframes": css_parsed.keyframes.len(),
            "colors": css_parsed.colors.len(),
        },
        "detected_libs": lib_result.libraries.iter().map(|l| {
            serde_json::json!({
                "name": l.name,
                "category": l.category,
                "version": l.version,
            })
        }).collect::<Vec<_>>(),
    });
    fs::write(data_dir.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)?;
    println!("  data/metadata.json");

    // Scripts info
    fs::write(
        scripts_dir.join("external.json"),
        serde_json::to_string_pretty(&script_urls)?,
    )?;
    println!("  scripts/external.json");

    // Project.toml
    if project_toml {
        let toml_content = generate_project_toml(url, &title, &lib_result.libraries, &css_parsed);
        fs::write(output.join("project.toml"), &toml_content)?;
        println!("  project.toml");
    }

    println!("\nDone! Output: {:?}", output);
    Ok(())
}

fn shorten_url(url: &str) -> String {
    if url.len() > 60 {
        format!("{}...", &url[..57])
    } else {
        url.to_string()
    }
}

fn extract_title(html: &str) -> Option<String> {
    let re = Regex::new(r"<title[^>]*>([^<]+)</title>").ok()?;
    re.captures(html).map(|c| c[1].trim().to_string())
}

fn extract_css_urls(html: &str, base_url: &Url) -> Vec<String> {
    let mut urls = Vec::new();
    let re = Regex::new(r#"<link[^>]+rel=["']stylesheet["'][^>]+href=["']([^"']+)["']|<link[^>]+href=["']([^"']+)["'][^>]+rel=["']stylesheet["']"#).unwrap();

    for cap in re.captures_iter(html) {
        let href = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());
        if let Some(href) = href {
            if let Ok(full_url) = base_url.join(href) {
                urls.push(full_url.to_string());
            }
        }
    }
    urls
}

fn extract_inline_styles(html: &str) -> String {
    let mut styles = String::new();
    let re = Regex::new(r"<style[^>]*>([\s\S]*?)</style>").unwrap();

    for cap in re.captures_iter(html) {
        styles.push_str(&cap[1]);
        styles.push('\n');
    }
    styles
}

fn extract_script_urls(html: &str, base_url: &Url) -> Vec<String> {
    let mut urls = Vec::new();
    let re = Regex::new(r#"<script[^>]+src=["']([^"']+)["']"#).unwrap();

    for cap in re.captures_iter(html) {
        let src = &cap[1];
        if let Ok(full_url) = base_url.join(src) {
            urls.push(full_url.to_string());
        }
    }
    urls
}

fn generate_project_toml(
    url: &str,
    title: &str,
    libs: &[DetectedLibrary],
    css: &CssParseResult,
) -> String {
    let mut toml = String::new();

    toml.push_str("[meta]\n");
    toml.push_str(&format!("name = \"{}\"\n", sanitize_name(title)));
    toml.push_str("version = \"1.0.0\"\n");
    toml.push_str("generator = \"crawlwe\"\n");
    toml.push_str("mode = \"static\"\n");
    toml.push_str(&format!("captured_at = \"{}\"\n", chrono::Utc::now().to_rfc3339()));
    toml.push('\n');

    toml.push_str("[source]\n");
    toml.push_str(&format!("url = \"{}\"\n", url));
    toml.push_str(&format!("title = \"{}\"\n", title.replace('"', "\\\"")));
    toml.push('\n');

    toml.push_str("[technologies]\n");
    for lib in libs {
        if lib.category == "css" {
            toml.push_str(&format!("css_framework = \"{}\"\n", lib.name));
            break;
        }
    }
    for lib in libs {
        if lib.category == "ui" {
            toml.push_str(&format!("ui_framework = \"{}\"\n", lib.name));
            break;
        }
    }

    let animation_libs: Vec<_> = libs.iter().filter(|l| l.category == "animation").collect();
    if !animation_libs.is_empty() {
        let names: Vec<_> = animation_libs.iter().map(|l| format!("\"{}\"", l.name)).collect();
        toml.push_str(&format!("animation_libs = [{}]\n", names.join(", ")));
    }
    toml.push('\n');

    toml.push_str("[dependencies.scripts]\n");
    for lib in libs {
        if let Some(cdn_url) = LibraryCDN::get_js(&lib.name, lib.version.as_deref()) {
            let key = lib.name.replace('-', "_").replace('.', "_").to_lowercase();
            if let Some(version) = &lib.version {
                toml.push_str(&format!("{} = {{ version = \"{}\", cdn = \"{}\" }}\n", key, version, cdn_url));
            } else {
                toml.push_str(&format!("{} = \"{}\"\n", key, cdn_url));
            }
        }
    }
    toml.push('\n');

    if !css.fonts.is_empty() {
        toml.push_str("[dependencies.fonts]\n");
        for font in &css.fonts {
            let font_url = LibraryCDN::get_font_url(font, &["400".to_string(), "700".to_string()]);
            toml.push_str(&format!("\"{}\" = {{ weights = [\"400\", \"700\"], url = \"{}\" }}\n", font, font_url));
        }
    }

    toml
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn generate_clean_html_local(html: &str, title: &str, css: &str) -> String {
    // No base href - all assets are local
    let head = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
{}
    </style>
</head>"#,
        title, css
    );

    if let Some(body_start) = html.find("<body") {
        if let Some(body_end) = html.rfind("</body>") {
            let body = &html[body_start..body_end + 7];
            return format!("{}\n{}\n</html>", head, body);
        }
    }

    format!("{}\n<body>\n{}\n</body>\n</html>", head, html)
}

async fn download_fonts(
    client: &reqwest::Client,
    css: &str,
    base_url: &Url,
    fonts_dir: &Path,
) -> (String, HashMap<String, String>) {
    let mut result_css = css.to_string();
    let mut downloaded: HashMap<String, String> = HashMap::new();

    // Match url() in @font-face blocks
    let re = Regex::new(r#"url\(["']?([^)"']+\.(?:woff2?|ttf|otf|eot))["']?\)"#).unwrap();

    for cap in re.captures_iter(css) {
        let font_path = &cap[1];

        // Skip already processed or data URLs
        if font_path.starts_with("data:") || font_path.starts_with("assets/") {
            continue;
        }

        // Resolve full URL
        let font_url = if font_path.starts_with("http") {
            font_path.to_string()
        } else {
            match base_url.join(font_path) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            }
        };

        // Skip if already downloaded
        if downloaded.contains_key(&font_url) {
            continue;
        }

        // Download font
        match client.get(&font_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        // Generate local filename
                        let filename = font_path.split('/').last().unwrap_or("font.woff2");
                        let local_path = fonts_dir.join(filename);

                        if fs::write(&local_path, &bytes).is_ok() {
                            let relative_path = format!("assets/fonts/{}", filename);
                            downloaded.insert(font_url.clone(), relative_path.clone());
                            println!("    + {}", filename);

                            // Replace in CSS
                            result_css = result_css.replace(font_path, &relative_path);
                        }
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    (result_css, downloaded)
}

async fn download_css_images(
    client: &reqwest::Client,
    css: &str,
    base_url: &Url,
    images_dir: &Path,
) -> (String, HashMap<String, String>) {
    let mut result_css = css.to_string();
    let mut downloaded: HashMap<String, String> = HashMap::new();

    // Match url() for images (not fonts)
    let re = Regex::new(r#"url\(["']?([^)"']+\.(?:png|jpg|jpeg|gif|svg|webp|ico))["']?\)"#).unwrap();

    for cap in re.captures_iter(css) {
        let img_path = &cap[1];

        if img_path.starts_with("data:") || img_path.starts_with("assets/") {
            continue;
        }

        let img_url = if img_path.starts_with("http") {
            img_path.to_string()
        } else {
            match base_url.join(img_path) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            }
        };

        if downloaded.contains_key(&img_url) {
            continue;
        }

        match client.get(&img_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        let filename = img_path.split('/').last().unwrap_or("image.png");
                        let local_path = images_dir.join(filename);

                        if fs::write(&local_path, &bytes).is_ok() {
                            let relative_path = format!("assets/images/{}", filename);
                            downloaded.insert(img_url.clone(), relative_path.clone());
                            println!("    + {} ({} bytes)", filename, bytes.len());

                            result_css = result_css.replace(img_path, &relative_path);
                        }
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    (result_css, downloaded)
}

async fn download_html_images(
    client: &reqwest::Client,
    html: &str,
    base_url: &Url,
    images_dir: &Path,
) -> (String, HashMap<String, String>) {
    let mut result_html = html.to_string();
    let mut downloaded: HashMap<String, String> = HashMap::new();

    // Match src="" for images
    let re = Regex::new(r#"src=["']([^"']+\.(?:png|jpg|jpeg|gif|svg|webp|ico))["']"#).unwrap();

    for cap in re.captures_iter(html) {
        let img_path = &cap[1];

        if img_path.starts_with("data:") || img_path.starts_with("assets/") {
            continue;
        }

        let img_url = if img_path.starts_with("http") {
            img_path.to_string()
        } else {
            match base_url.join(img_path) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            }
        };

        if downloaded.contains_key(&img_url) {
            continue;
        }

        match client.get(&img_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        let filename = img_path.split('/').last().unwrap_or("image.png");
                        let local_path = images_dir.join(filename);

                        if fs::write(&local_path, &bytes).is_ok() {
                            let relative_path = format!("assets/images/{}", filename);
                            downloaded.insert(img_url.clone(), relative_path.clone());
                            println!("    + {} ({} bytes)", filename, bytes.len());

                            // Replace src in HTML
                            let old_src = format!("src=\"{}\"", img_path);
                            let new_src = format!("src=\"{}\"", relative_path);
                            result_html = result_html.replace(&old_src, &new_src);

                            let old_src = format!("src='{}'", img_path);
                            let new_src = format!("src='{}'", relative_path);
                            result_html = result_html.replace(&old_src, &new_src);
                        }
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    (result_html, downloaded)
}

async fn download_nextjs_images(
    client: &reqwest::Client,
    html: &str,
    base_url: &Url,
    images_dir: &Path,
) -> (String, HashMap<String, String>) {
    let mut result_html = html.to_string();
    let mut downloaded: HashMap<String, String> = HashMap::new();

    // Match Next.js image URLs: /_next/image?url=...
    // Handle both regular & and HTML-encoded &amp;
    let re = Regex::new(r#"["']([^"']*/_next/image\?[^"'\s]+)["']"#).unwrap();

    for cap in re.captures_iter(html) {
        let img_url_raw = &cap[1];

        // Skip srcset entries with pixel density (e.g., "1x", "2x")
        if img_url_raw.ends_with("x") && img_url_raw.chars().rev().nth(1).map_or(false, |c| c.is_ascii_digit()) {
            continue;
        }

        // Decode HTML entities in URL
        let img_url_decoded = img_url_raw.replace("&amp;", "&");

        // Skip if already downloaded (check both encoded and decoded)
        if downloaded.contains_key(img_url_raw) || downloaded.contains_key(&img_url_decoded) {
            continue;
        }

        // Build full URL
        let full_url = if img_url_decoded.starts_with("http") {
            img_url_decoded.clone()
        } else {
            match base_url.join(&img_url_decoded) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            }
        };

        // Extract original filename from url parameter
        let filename = extract_nextjs_filename(&img_url_decoded);

        match client.get(&full_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        let local_path = images_dir.join(&filename);

                        if fs::write(&local_path, &bytes).is_ok() {
                            let relative_path = format!("assets/images/{}", filename);
                            downloaded.insert(img_url_raw.to_string(), relative_path.clone());
                            println!("    + {} ({} bytes) [Next.js]", filename, bytes.len());

                            // Replace in HTML (both encoded and decoded versions)
                            result_html = result_html.replace(img_url_raw, &relative_path);
                        }
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    (result_html, downloaded)
}

fn extract_nextjs_filename(url: &str) -> String {
    // Try to extract from url= parameter
    if let Some(start) = url.find("url=") {
        let rest = &url[start + 4..];
        let encoded_url = rest.split('&').next().unwrap_or("");

        // URL decode
        if let Ok(decoded) = urlencoding::decode(encoded_url) {
            let path = decoded.to_string();
            if let Some(filename) = path.split('/').last() {
                // Remove query params from filename
                let clean_name = filename.split('?').next().unwrap_or(filename);
                if !clean_name.is_empty() {
                    return clean_name.to_string();
                }
            }
        }
    }

    // Fallback: generate name from hash
    format!("nextjs-{}.webp", &url.len())
}

async fn download_svg_images(
    client: &reqwest::Client,
    html: &str,
    base_url: &Url,
    images_dir: &Path,
) -> (String, HashMap<String, String>) {
    let mut result_html = html.to_string();
    let mut downloaded: HashMap<String, String> = HashMap::new();

    // Match <img src="*.svg">
    let re = Regex::new(r#"<img[^>]+src=["']([^"']+\.svg)["']"#).unwrap();

    for cap in re.captures_iter(html) {
        let svg_path = &cap[1];

        if svg_path.starts_with("data:") || svg_path.starts_with("assets/") {
            continue;
        }

        let svg_url = if svg_path.starts_with("http") {
            svg_path.to_string()
        } else {
            match base_url.join(svg_path) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            }
        };

        if downloaded.contains_key(&svg_url) {
            continue;
        }

        match client.get(&svg_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(svg_content) => {
                        let filename = svg_path.split('/').last().unwrap_or("image.svg");
                        let local_path = images_dir.join(filename);

                        if fs::write(&local_path, &svg_content).is_ok() {
                            let relative_path = format!("assets/images/{}", filename);
                            downloaded.insert(svg_url.clone(), relative_path.clone());
                            println!("    + {} ({} bytes) [SVG]", filename, svg_content.len());

                            // Replace src in HTML
                            result_html = result_html.replace(svg_path, &relative_path);
                        }
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    (result_html, downloaded)
}

fn extract_inline_svgs(html: &str, images_dir: &Path) -> (String, usize) {
    let mut result_html = html.to_string();
    let mut count = 0;

    // Match inline <svg>...</svg> tags (simplified - matches most common cases)
    let re = Regex::new(r#"<svg[^>]*>[\s\S]*?</svg>"#).unwrap();

    for (i, cap) in re.find_iter(html).enumerate() {
        let svg_content = cap.as_str();

        // Skip very small SVGs (likely icons that are fine inline)
        if svg_content.len() < 500 {
            continue;
        }

        let filename = format!("inline-svg-{}.svg", i);
        let local_path = images_dir.join(&filename);

        // Add XML declaration if not present
        let full_svg = if svg_content.starts_with("<?xml") {
            svg_content.to_string()
        } else {
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", svg_content)
        };

        if fs::write(&local_path, &full_svg).is_ok() {
            let relative_path = format!("assets/images/{}", filename);

            // Replace inline SVG with <img> tag
            let img_tag = format!(
                r#"<img src="{}" alt="SVG graphic" class="inline-svg">"#,
                relative_path
            );
            result_html = result_html.replacen(svg_content, &img_tag, 1);
            count += 1;
            println!("    + {} ({} bytes) [inline]", filename, svg_content.len());
        }
    }

    (result_html, count)
}

// ============================================================================
// COMPREHENSIVE CSS ASSET EXTRACTION (NEW!)
// ============================================================================

/// Download all CSS assets using the comprehensive CssAssetExtractor
async fn download_css_assets_comprehensive(
    client: &reqwest::Client,
    css: &str,
    base_url: &Url,
    fonts_dir: &Path,
    images_dir: &Path,
) -> (String, CssAssetExtractionResult, HashMap<String, String>, HashMap<String, String>) {
    let extractor = CssAssetExtractor::new();
    let mut extraction = extractor.extract(css, base_url);

    let mut result_css = css.to_string();
    let mut font_url_map: HashMap<String, String> = HashMap::new();
    let mut image_url_map: HashMap<String, String> = HashMap::new();

    println!("   Asset extraction analysis:");
    println!("     Total URLs found: {}", extraction.stats.total_urls_found);
    println!("     Fonts: {} (+ {} local)",
        extraction.stats.fonts_found,
        extraction.fonts.iter().filter(|(k, _)| k.starts_with("local:")).count()
    );
    println!("     Images: {}", extraction.stats.images_found);
    println!("     @imports: {}", extraction.stats.imports_found);
    println!("     Data URIs skipped: {}", extraction.stats.data_uris_skipped);

    if extraction.stats.invalid_urls > 0 {
        println!("     Invalid URLs: {}", extraction.stats.invalid_urls);
    }

    // Download fonts
    println!("\n   Downloading fonts:");
    for (url, font_info) in &mut extraction.fonts {
        // Skip local() references
        if url.starts_with("local:") {
            continue;
        }

        // Skip data URIs (shouldn't happen but be safe)
        if url.starts_with("data:") {
            continue;
        }

        match client.get(url.as_str()).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        // Generate filename from URL
                        let filename = generate_asset_filename(url, "font", &font_info.format);
                        let local_path = fonts_dir.join(&filename);

                        if fs::write(&local_path, &bytes).is_ok() {
                            let relative_path = format!("assets/fonts/{}", filename);
                            font_url_map.insert(url.clone(), relative_path.clone());
                            font_info.local_path = Some(relative_path.clone());

                            // Show font metadata if available
                            let meta = if let Some(family) = &font_info.font_family {
                                format!(" [{}]", family)
                            } else {
                                String::new()
                            };
                            println!("     + {} ({} bytes){}", filename, bytes.len(), meta);

                            // Replace URL in CSS
                            result_css = result_css.replace(url, &relative_path);
                        }
                    }
                    Err(e) => {
                        extraction.errors.push(format!("Failed to read font bytes: {} - {}", url, e));
                    }
                }
            }
            Ok(resp) => {
                extraction.errors.push(format!("Font HTTP {}: {}", resp.status(), url));
            }
            Err(e) => {
                extraction.errors.push(format!("Font download failed: {} - {}", url, e));
            }
        }
    }

    // Download images
    println!("\n   Downloading images:");
    for (url, image_info) in &mut extraction.images {
        // Skip data URIs
        if url.starts_with("data:") {
            continue;
        }

        // Skip already downloaded
        if image_url_map.contains_key(url) {
            continue;
        }

        match client.get(url.as_str()).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        let filename = generate_asset_filename(url, "image", &None);
                        let local_path = images_dir.join(&filename);

                        if fs::write(&local_path, &bytes).is_ok() {
                            let relative_path = format!("assets/images/{}", filename);
                            image_url_map.insert(url.clone(), relative_path.clone());
                            image_info.local_path = Some(relative_path.clone());

                            // Show context
                            let context = match image_info.context {
                                ImageContext::BackgroundImage => " [background]",
                                ImageContext::BorderImage => " [border]",
                                ImageContext::MaskImage => " [mask]",
                                ImageContext::Cursor => " [cursor]",
                                ImageContext::Content => " [content]",
                                ImageContext::Filter => " [filter]",
                                _ => "",
                            };
                            println!("     + {} ({} bytes){}", filename, bytes.len(), context);

                            // Replace URL in CSS
                            result_css = result_css.replace(url, &relative_path);
                        }
                    }
                    Err(e) => {
                        extraction.errors.push(format!("Failed to read image bytes: {} - {}", url, e));
                    }
                }
            }
            Ok(resp) => {
                extraction.errors.push(format!("Image HTTP {}: {}", resp.status(), url));
            }
            Err(e) => {
                extraction.errors.push(format!("Image download failed: {} - {}", url, e));
            }
        }
    }

    // Report errors
    if !extraction.errors.is_empty() {
        println!("\n   Asset errors ({}):", extraction.errors.len());
        for err in extraction.errors.iter().take(10) {
            println!("     ! {}", err);
        }
        if extraction.errors.len() > 10 {
            println!("     ... and {} more", extraction.errors.len() - 10);
        }
    }

    (result_css, extraction, font_url_map, image_url_map)
}

/// Generate a clean filename for an asset URL
fn generate_asset_filename(url: &str, asset_type: &str, format_hint: &Option<String>) -> String {
    // Try to extract filename from URL
    if let Some(path) = url.split('?').next() {
        if let Some(filename) = path.split('/').last() {
            // Check if it's a valid filename with extension
            if filename.contains('.') && filename.len() < 100 {
                // Sanitize filename (remove special chars)
                let clean: String = filename
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
                    .collect();
                if !clean.is_empty() {
                    return clean;
                }
            }
        }
    }

    // Generate hash-based filename
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    // Determine extension
    let ext = if let Some(fmt) = format_hint {
        match fmt.as_str() {
            "woff2" => "woff2",
            "woff" => "woff",
            "truetype" => "ttf",
            "opentype" => "otf",
            "embedded-opentype" => "eot",
            _ => if asset_type == "font" { "woff2" } else { "png" }
        }
    } else {
        // Try to detect from URL
        let lower = url.to_lowercase();
        if lower.contains(".woff2") { "woff2" }
        else if lower.contains(".woff") { "woff" }
        else if lower.contains(".ttf") { "ttf" }
        else if lower.contains(".otf") { "otf" }
        else if lower.contains(".eot") { "eot" }
        else if lower.contains(".svg") { "svg" }
        else if lower.contains(".png") { "png" }
        else if lower.contains(".jpg") || lower.contains(".jpeg") { "jpg" }
        else if lower.contains(".gif") { "gif" }
        else if lower.contains(".webp") { "webp" }
        else if lower.contains(".avif") { "avif" }
        else if asset_type == "font" { "woff2" }
        else { "png" }
    };

    format!("{}-{:016x}.{}", asset_type, hash, ext)
}

// ============================================================================
// HYBRID MODE FUNCTIONS
// ============================================================================

/// Resolve @import statements in CSS and fetch the imported stylesheets
async fn resolve_css_imports(
    client: &reqwest::Client,
    css: &str,
    base_url: &Url,
    depth: usize,
) -> String {
    if depth > 5 {
        return css.to_string(); // Prevent infinite recursion
    }

    let mut result_css = css.to_string();

    // Match @import url("...") or @import "..."
    let re = Regex::new(r#"@import\s+(?:url\()?["']([^"']+)["']\)?[^;]*;"#).unwrap();

    let mut imports_to_resolve: Vec<(String, String)> = Vec::new();

    for cap in re.captures_iter(css) {
        let import_path = &cap[1];
        let full_match = cap.get(0).unwrap().as_str();

        // Skip data URLs
        if import_path.starts_with("data:") {
            continue;
        }

        let import_url = if import_path.starts_with("http") {
            import_path.to_string()
        } else {
            match base_url.join(import_path) {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            }
        };

        imports_to_resolve.push((full_match.to_string(), import_url));
    }

    for (import_statement, import_url) in imports_to_resolve {
        match client.get(&import_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(imported_css) = resp.text().await {
                    println!("    + @import: {}", shorten_url(&import_url));

                    // Recursively resolve imports in the imported CSS
                    let import_base = Url::parse(&import_url).unwrap_or(base_url.clone());
                    let resolved_import = Box::pin(resolve_css_imports(
                        client,
                        &imported_css,
                        &import_base,
                        depth + 1,
                    )).await;

                    // Replace @import with actual CSS content
                    result_css = result_css.replace(
                        &import_statement,
                        &format!("/* Imported from: {} */\n{}\n", import_url, resolved_import),
                    );
                }
            }
            _ => {
                println!("    ! @import failed: {}", shorten_url(&import_url));
            }
        }
    }

    result_css
}

/// Discover and download ALL Next.js CSS chunks (not just those in link tags)
async fn discover_nextjs_css(
    client: &reqwest::Client,
    html: &str,
    base_url: &Url,
) -> Vec<(String, String)> {
    let mut css_chunks: Vec<(String, String)> = Vec::new();
    let mut found_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Pattern 1: CSS files referenced in HTML (including preload, prefetch)
    let link_re = Regex::new(r#"href=["']([^"']*/_next/static/css/[^"']+\.css)["']"#).unwrap();
    for cap in link_re.captures_iter(html) {
        let css_path = &cap[1];
        if let Ok(full_url) = base_url.join(css_path) {
            found_urls.insert(full_url.to_string());
        }
    }

    // Pattern 2: CSS files in script chunks (Next.js loads CSS dynamically)
    let chunk_re = Regex::new(r#"["']([^"']*/_next/static/css/[a-f0-9]+\.css)["']"#).unwrap();
    for cap in chunk_re.captures_iter(html) {
        let css_path = &cap[1];
        if let Ok(full_url) = base_url.join(css_path) {
            found_urls.insert(full_url.to_string());
        }
    }

    // Pattern 3: Build manifest references
    let manifest_re = Regex::new(r#"static/css/([a-f0-9]+\.css)"#).unwrap();
    for cap in manifest_re.captures_iter(html) {
        let css_file = &cap[1];
        let css_path = format!("/_next/static/css/{}", css_file);
        if let Ok(full_url) = base_url.join(&css_path) {
            found_urls.insert(full_url.to_string());
        }
    }

    // Download all discovered CSS files
    for css_url in found_urls {
        match client.get(&css_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(css_content) = resp.text().await {
                    css_chunks.push((css_url, css_content));
                }
            }
            _ => {}
        }
    }

    css_chunks
}

/// Extract CSS patterns from JavaScript bundles (CSS-in-JS at build time)
async fn extract_css_from_js(
    client: &reqwest::Client,
    script_urls: &[String],
) -> String {
    let mut extracted_css = String::new();

    // Patterns that indicate CSS content in JS
    let css_patterns = [
        // Styled-components / Emotion compiled CSS
        Regex::new(r#"\.css\(["'`]([^"'`]{50,})["'`]"#).unwrap(),
        // CSS module injections
        Regex::new(r#"\.push\(\[module\.id,\s*["'`]([^"'`]{50,})["'`]"#).unwrap(),
        // Style injections
        Regex::new(r#"insertRule\(["'`]([^"'`]{30,})["'`]"#).unwrap(),
        // CSS template literals (common in CSS-in-JS)
        Regex::new(r#"css`([^`]{50,})`"#).unwrap(),
        // Direct style object with CSS properties
        Regex::new(r#"styles?:\s*["'`]([^"'`]*\{[^}]+\}[^"'`]*)["'`]"#).unwrap(),
    ];

    // Only check main/app chunks (not tiny chunks)
    let relevant_scripts: Vec<_> = script_urls
        .iter()
        .filter(|url| {
            url.contains("main") ||
            url.contains("app") ||
            url.contains("pages") ||
            url.contains("layout")
        })
        .take(10) // Limit to avoid too many requests
        .collect();

    for script_url in relevant_scripts {
        match client.get(script_url.as_str()).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(js_content) = resp.text().await {
                    // Skip if too small to have meaningful CSS
                    if js_content.len() < 1000 {
                        continue;
                    }

                    for pattern in &css_patterns {
                        for cap in pattern.captures_iter(&js_content) {
                            let css = &cap[1];
                            // Basic validation that it looks like CSS
                            if css.contains('{') && css.contains('}') &&
                               (css.contains(':') || css.contains(';')) {
                                extracted_css.push_str(&format!("/* From JS: {} */\n", shorten_url(script_url)));
                                extracted_css.push_str(css);
                                extracted_css.push_str("\n\n");
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    extracted_css
}

/// Extract CSS class definitions from Tailwind/utility class patterns in JS
fn extract_tailwind_from_html(html: &str) -> Vec<String> {
    let mut classes: Vec<String> = Vec::new();

    // Match class="" or className="" attributes
    let re = Regex::new(r#"class(?:Name)?=["']([^"']+)["']"#).unwrap();

    for cap in re.captures_iter(html) {
        let class_list = &cap[1];
        for class in class_list.split_whitespace() {
            if !classes.contains(&class.to_string()) {
                classes.push(class.to_string());
            }
        }
    }

    classes
}

/// Main hybrid fetch function
pub async fn run_hybrid(
    url: &str,
    output: &Path,
    project_toml: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe - Hybrid Fetch");
    println!("======================");
    println!("URL: {}", url);
    println!("Mode: Static HTTP + CSS Discovery + JS Extraction");
    println!();

    let base_url = Url::parse(url)?;
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Create output structure
    let data_dir = output.join("data");
    let scripts_dir = output.join("scripts");
    let assets_dir = output.join("assets");
    let fonts_dir = assets_dir.join("fonts");
    let images_dir = assets_dir.join("images");
    let svgs_dir = assets_dir.join("svgs");
    let icons_dir = svgs_dir.join("icons");
    let logos_dir = svgs_dir.join("logos");
    let illustrations_dir = svgs_dir.join("illustrations");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&scripts_dir)?;
    fs::create_dir_all(&fonts_dir)?;
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&icons_dir)?;
    fs::create_dir_all(&logos_dir)?;
    fs::create_dir_all(&illustrations_dir)?;

    // Initialize Asset Registry
    let mut asset_registry = AssetRegistry::new();

    // Download HTML
    println!("1. Downloading HTML...");
    let html_response = client.get(url).send().await?;
    let html_raw = html_response.text().await?;
    println!("   HTML: {} bytes", html_raw.len());

    let title = extract_title(&html_raw).unwrap_or_else(|| "Untitled".to_string());
    println!("   Title: {}", title);

    // Extract all classes used in HTML (for Tailwind detection)
    let html_classes = extract_tailwind_from_html(&html_raw);
    println!("   Classes found: {}", html_classes.len());

    // Phase 2: CSS Discovery
    println!("\n2. CSS Discovery...");

    // 2a. Standard CSS from link tags
    let css_urls = extract_css_urls(&html_raw, &base_url);
    println!("   Link stylesheets: {}", css_urls.len());

    let mut all_css = String::new();
    for css_url in &css_urls {
        match client.get(css_url).send().await {
            Ok(resp) => {
                if let Ok(css) = resp.text().await {
                    all_css.push_str(&format!("/* Source: {} */\n", css_url));
                    all_css.push_str(&css);
                    all_css.push_str("\n\n");
                    println!("    + {}", shorten_url(css_url));
                }
            }
            Err(e) => println!("    ! {} ({})", shorten_url(css_url), e),
        }
    }

    // 2b. Inline styles
    let inline_css = extract_inline_styles(&html_raw);
    if !inline_css.is_empty() {
        all_css.push_str("/* Inline styles */\n");
        all_css.push_str(&inline_css);
        println!("    + inline styles: {} bytes", inline_css.len());
    }

    // 2c. Resolve @import statements
    println!("\n   Resolving @imports...");
    all_css = resolve_css_imports(&client, &all_css, &base_url, 0).await;

    // 2d. Discover additional Next.js CSS chunks
    println!("\n   Discovering Next.js CSS chunks...");
    let nextjs_css_chunks = discover_nextjs_css(&client, &html_raw, &base_url).await;
    println!("   Found {} additional CSS chunks", nextjs_css_chunks.len());

    for (chunk_url, chunk_css) in &nextjs_css_chunks {
        // Avoid duplicates
        if !all_css.contains(chunk_css) {
            all_css.push_str(&format!("\n/* Next.js chunk: {} */\n", shorten_url(chunk_url)));
            all_css.push_str(chunk_css);
            all_css.push_str("\n");
            println!("    + {}", shorten_url(chunk_url));
        }
    }

    // Phase 3: Extract CSS from JS bundles
    println!("\n3. Extracting CSS from JS bundles...");
    let script_urls = extract_script_urls(&html_raw, &base_url);
    let js_css = extract_css_from_js(&client, &script_urls).await;
    if !js_css.is_empty() {
        all_css.push_str("\n/* === CSS extracted from JavaScript === */\n");
        all_css.push_str(&js_css);
        println!("   Extracted: {} bytes from JS", js_css.len());
    } else {
        println!("   No CSS-in-JS patterns found");
    }

    println!("\n   Total CSS: {} bytes", all_css.len());

    // Phase 4: Download assets using comprehensive extractor
    println!("\n4. Comprehensive Asset Extraction...");

    // Use the new CssAssetExtractor for comprehensive extraction
    let (css_with_local_assets, extraction_result, font_map, css_image_map) =
        download_css_assets_comprehensive(&client, &all_css, &base_url, &fonts_dir, &images_dir).await;

    println!("\n   Summary:");
    println!("     Fonts downloaded: {}", font_map.len());
    println!("     CSS images downloaded: {}", css_image_map.len());

    // Images from HTML
    println!("   HTML Images:");
    let (html_with_local_images, html_image_map) = download_html_images(&client, &html_raw, &base_url, &images_dir).await;
    println!("   Downloaded {} images from HTML", html_image_map.len());

    // Next.js images
    println!("   Next.js Images:");
    let (html_with_nextjs, nextjs_image_map) = download_nextjs_images(&client, &html_with_local_images, &base_url, &images_dir).await;
    println!("   Downloaded {} Next.js images", nextjs_image_map.len());

    // SVGs - external files
    println!("   SVGs (external):");
    let (html_with_svgs, svg_map) = download_svg_images(&client, &html_with_nextjs, &base_url, &images_dir).await;
    println!("   Downloaded {} SVG files", svg_map.len());

    // SVGs - comprehensive extraction with registry
    println!("\n   SVG Registry Analysis:");
    asset_registry.extract_svgs(&html_with_svgs);
    println!("     Total SVGs found: {}", asset_registry.stats.total_svgs_found);
    println!("     Unique SVGs: {}", asset_registry.stats.unique_svgs);
    println!("     Duplicates removed: {}", asset_registry.stats.duplicate_svgs);
    println!("     Icons: {}", asset_registry.stats.icons_found);
    println!("     Logos: {}", asset_registry.stats.logos_found);
    println!("     Illustrations: {}", asset_registry.stats.illustrations_found);

    // Extract SVGs to categorized directories
    let html_final = extract_svgs_with_registry(&mut asset_registry, &html_with_svgs, &svgs_dir)?;
    let extractable_count = asset_registry.get_extractable_svgs().len();
    if extractable_count > 0 {
        println!("     Extracted {} SVGs to files", extractable_count);
    }

    // Phase 5: Library detection
    println!("\n5. Analyzing...");
    let lib_result = LibMicroparser::parse(&script_urls, "");
    println!("   Libraries: {}", lib_result.libraries.len());
    for lib in &lib_result.libraries {
        println!("    - {} ({})", lib.name, lib.category);
    }

    let css_parsed = CssMicroparser::parse(&css_with_local_assets);
    println!("   CSS Variables: {}", css_parsed.variables.len());
    println!("   Keyframes: {}", css_parsed.keyframes.len());

    // Phase 6: Format CSS and optimize
    println!("\n6. Formatting CSS...");

    let formatter = CssFormatter::with_options(CssFormatOptions {
        indent_size: 2,
        blank_lines_between_rules: true,
        add_section_comments: true,
        group_properties: true,
        ..Default::default()
    });
    let format_result = formatter.format(&css_with_local_assets);
    println!("   Rules formatted: {}", format_result.stats.rules_formatted);
    println!("   Keyframes: {}", format_result.stats.keyframes_formatted);
    println!("   Media queries: {}", format_result.stats.media_queries_formatted);
    println!("   Font faces: {}", format_result.stats.font_faces_formatted);
    println!("   Variables: {}", format_result.stats.variables_found);

    let final_css = format_result.css;

    println!("\n7. Optimizing & saving...");

    let html_optimized = HtmlOptimizer::optimize(&html_final, &HtmlOptimizeOptions {
        remove_comments: true,
        remove_scripts: true,
        remove_styles: false,
        remove_data_attrs: false,
        remove_framework_attrs: true,
        remove_empty_attrs: true,
        minify_whitespace: false,
        format_output: true,
        preserve_structure: true,
    });

    println!("   HTML: {} -> {} bytes", html_raw.len(), html_optimized.stats.optimized_size);
    println!("   CSS: {} bytes (formatted)", final_css.len());

    // Save files
    let clean_html = generate_clean_html_local(&html_optimized.html, &title, &final_css);
    fs::write(output.join("index.html"), &clean_html)?;
    println!("   + index.html");

    fs::write(data_dir.join("raw.html"), &html_raw)?;
    println!("   + data/raw.html");

    fs::write(output.join("styles.css"), &final_css)?;
    println!("   + styles.css");

    // Register fonts and images in the asset registry
    for (url, font_info) in &extraction_result.fonts {
        if !url.starts_with("local:") {
            asset_registry.register_font(
                font_info.font_family.as_deref().unwrap_or("Unknown"),
                font_info.font_weight.as_deref(),
                font_info.font_style.as_deref(),
                url,
                font_info.local_path.as_deref(),
                None,
            );
        }
    }

    for (url, _) in &html_image_map {
        asset_registry.register_image(url, None, AssetImageContext::Content, None);
    }

    // Metadata (including comprehensive extraction details)
    let total_images = css_image_map.len() + html_image_map.len() + nextjs_image_map.len() + svg_map.len();
    let extractable_svgs = asset_registry.get_extractable_svgs().len();
    let metadata = serde_json::json!({
        "url": url,
        "domain": base_url.host_str(),
        "title": title,
        "mode": "hybrid",
        "extracted_at": chrono::Utc::now().to_rfc3339(),
        "stats": {
            "html_bytes": html_raw.len(),
            "html_optimized_bytes": html_optimized.stats.optimized_size,
            "css_bytes": final_css.len(),
            "css_sources": css_urls.len() + nextjs_css_chunks.len(),
            "js_css_extracted": !js_css.is_empty(),
            "external_scripts": script_urls.len(),
            "fonts_downloaded": font_map.len(),
            "images_downloaded": total_images,
            "nextjs_images": nextjs_image_map.len(),
            "svgs_downloaded": svg_map.len(),
            "svgs_extracted": extractable_svgs,
            "html_classes": html_classes.len(),
        },
        "asset_registry": {
            "total_svgs": asset_registry.stats.total_svgs_found,
            "unique_svgs": asset_registry.stats.unique_svgs,
            "duplicate_svgs": asset_registry.stats.duplicate_svgs,
            "icons": asset_registry.stats.icons_found,
            "logos": asset_registry.stats.logos_found,
            "illustrations": asset_registry.stats.illustrations_found,
            "fonts": asset_registry.stats.fonts_found,
            "images": asset_registry.stats.images_found,
        },
        "asset_extraction": {
            "total_urls_found": extraction_result.stats.total_urls_found,
            "fonts_found": extraction_result.stats.fonts_found,
            "images_found": extraction_result.stats.images_found,
            "imports_found": extraction_result.stats.imports_found,
            "data_uris_skipped": extraction_result.stats.data_uris_skipped,
            "invalid_urls": extraction_result.stats.invalid_urls,
            "local_fonts": extraction_result.fonts.iter().filter(|(k, _)| k.starts_with("local:")).count(),
            "errors": extraction_result.errors.len(),
        },
        "css_analysis": {
            "variables": css_parsed.variables.len(),
            "keyframes": css_parsed.keyframes.len(),
            "colors": css_parsed.colors.len(),
        },
        "detected_libs": lib_result.libraries.iter().map(|l| {
            serde_json::json!({
                "name": l.name,
                "category": l.category,
                "version": l.version,
            })
        }).collect::<Vec<_>>(),
    });
    fs::write(data_dir.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)?;
    println!("   + data/metadata.json");

    fs::write(scripts_dir.join("external.json"), serde_json::to_string_pretty(&script_urls)?)?;
    println!("   + scripts/external.json");

    // Save HTML classes for reference
    fs::write(data_dir.join("classes.json"), serde_json::to_string_pretty(&html_classes)?)?;
    println!("   + data/classes.json");

    if project_toml {
        // Generate TOML with full asset registry
        let libraries: Vec<(String, String)> = lib_result.libraries
            .iter()
            .map(|l| (l.name.clone(), l.category.clone()))
            .collect();
        let css_stats = CssStats {
            variables: css_parsed.variables.len(),
            keyframes: css_parsed.keyframes.len(),
            colors: css_parsed.colors.len(),
        };
        let toml_content = asset_registry.generate_toml(url, &title, "hybrid", &libraries, &css_stats);
        fs::write(output.join("project.toml"), &toml_content)?;
        println!("   + project.toml (with asset registry)");
    }

    // Phase 6b: Analyze JS with library parsers (NEW!)
    println!("\n6b. Library-specific JS analysis...");
    let library_css = analyze_js_with_library_parsers(&client, &script_urls, &html_classes).await;
    if !library_css.is_empty() {
        // Append library-generated CSS to the file
        let mut final_css_with_libs = fs::read_to_string(output.join("styles.css"))?;
        final_css_with_libs.push_str("\n\n/* === Library-Generated CSS === */\n");
        final_css_with_libs.push_str(&library_css);
        fs::write(output.join("styles.css"), &final_css_with_libs)?;
        println!("   + Added {} bytes of library CSS", library_css.len());
    }

    println!("\n✓ Done! Output: {:?}", output);
    Ok(())
}

/// Download and analyze JavaScript files with library-specific parsers
async fn analyze_js_with_library_parsers(
    client: &reqwest::Client,
    script_urls: &[String],
    html_classes: &[String],
) -> String {
    let mut all_js_code = String::new();
    let mut generated_css = String::new();

    // Download relevant JS files (skip tiny chunks and third-party analytics)
    let relevant_scripts: Vec<_> = script_urls
        .iter()
        .filter(|url| {
            !url.contains("analytics") &&
            !url.contains("gtag") &&
            !url.contains("gtm") &&
            !url.contains("seline") &&
            !url.contains("hotjar") &&
            !url.contains("tracking") &&
            (url.contains("app") ||
             url.contains("main") ||
             url.contains("chunk") ||
             url.contains("bundle") ||
             url.contains("webflow") ||
             url.ends_with(".js"))
        })
        .take(15) // Limit to avoid too many requests
        .collect();

    println!("   Analyzing {} JS files...", relevant_scripts.len());

    for script_url in relevant_scripts {
        match client.get(script_url.as_str()).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(js_content) = resp.text().await {
                    // Skip tiny files
                    if js_content.len() > 500 {
                        all_js_code.push_str(&format!("\n// Source: {}\n", script_url));
                        all_js_code.push_str(&js_content);
                        all_js_code.push('\n');
                    }
                }
            }
            _ => {}
        }
    }

    if all_js_code.is_empty() {
        println!("   No JS code downloaded");
        return String::new();
    }

    println!("   Downloaded {} bytes of JS", all_js_code.len());

    // Analyze with JsAnalyzer
    let mut analysis = JsAnalyzer::analyze(&all_js_code);

    // Add HTML classes to the analysis
    for class in html_classes {
        if !analysis.classes.contains(class) {
            analysis.classes.push(class.clone());
        }
    }

    // Use library registry to detect and analyze
    let registry = LibraryRegistry::new();
    let detected = registry.detect_libraries(&analysis);

    if !detected.is_empty() {
        println!("   Libraries detected by parsers:");
        for lib in &detected {
            println!("    - {} ({})", lib.name, lib.category.as_str());
        }
    }

    // NEW: Use JsToCssConverter for comprehensive JS to CSS conversion
    println!("   Converting JS styles to CSS...");
    let mut converter = JsToCssConverter::new();
    let conversion_result = converter.convert(&all_js_code);

    // Report conversion stats
    if conversion_result.stats.styled_components_found > 0 {
        println!("     Styled-components: {}", conversion_result.stats.styled_components_found);
    }
    if conversion_result.stats.emotion_blocks_found > 0 {
        println!("     Emotion/css blocks: {}", conversion_result.stats.emotion_blocks_found);
    }
    if conversion_result.stats.gsap_animations_found > 0 {
        println!("     GSAP animations: {}", conversion_result.stats.gsap_animations_found);
    }
    if conversion_result.stats.framer_animations_found > 0 {
        println!("     Framer Motion: {}", conversion_result.stats.framer_animations_found);
    }
    if conversion_result.stats.keyframes_generated > 0 {
        println!("     Keyframes generated: {}", conversion_result.stats.keyframes_generated);
    }
    if conversion_result.stats.inline_styles_converted > 0 {
        println!("     Inline styles converted: {}", conversion_result.stats.inline_styles_converted);
    }

    // Add converted CSS
    if !conversion_result.css.is_empty() {
        generated_css.push_str("\n/* === JS to CSS Conversion === */\n");
        generated_css.push_str(&conversion_result.css);
        generated_css.push_str("\n");
    }

    // NEW: CSS Modules Parser - detect hashed class patterns
    println!("   Parsing CSS Modules patterns...");
    let css_modules_parser = CssModulesParser::new();
    let css_modules_result = css_modules_parser.extract_css_from_js(&all_js_code);
    if !css_modules_result.components.is_empty() {
        println!("     CSS Modules components: {}", css_modules_result.components.len());
        println!("     Hashed classes mapped: {}", css_modules_result.class_map.len());

        // Generate import map for reference
        let modules_import_map = css_modules_parser.generate_import_map(&css_modules_result);
        if !modules_import_map.is_empty() {
            generated_css.push_str("\n/* === CSS Modules Import Map === */\n/*\n");
            generated_css.push_str(&modules_import_map);
            generated_css.push_str("\n*/\n");
        }
    }

    // NEW: Tailwind Generator - generate CSS for Tailwind classes
    println!("   Generating Tailwind CSS...");
    let tailwind_gen = TailwindGenerator::new();
    let tailwind_result = tailwind_gen.generate(html_classes);
    if tailwind_result.classes_generated > 0 {
        println!("     Tailwind classes: {}/{} generated", tailwind_result.classes_generated, tailwind_result.classes_processed);
        if !tailwind_result.unknown_classes.is_empty() {
            println!("     Unknown classes: {} (not Tailwind)", tailwind_result.unknown_classes.len());
        }

        if !tailwind_result.css.is_empty() {
            generated_css.push_str("\n/* === Tailwind CSS (generated) === */\n");
            generated_css.push_str(&tailwind_result.css);
            generated_css.push_str("\n");
        }
    }

    // NEW: Smart CSS Extractor - extract CSS patterns from JS
    println!("   Smart CSS extraction...");
    let smart_extractor = SmartCssExtractor::new();

    // Extract CSS from JS code
    let smart_result = smart_extractor.extract_from_js(&all_js_code);
    if !smart_result.class_rules.is_empty() || !smart_result.keyframes.is_empty() {
        println!("     Smart-extracted rules: {}", smart_result.class_rules.len());
        println!("     Smart-extracted keyframes: {}", smart_result.keyframes.len());

        let smart_css = smart_extractor.generate_clean_css(&smart_result);
        if !smart_css.is_empty() {
            generated_css.push_str("\n/* === Smart CSS Extraction === */\n");
            generated_css.push_str(&smart_css);
            generated_css.push_str("\n");
        }
    }

    // Run library-specific analysis
    let lib_analysis = registry.analyze(&all_js_code, &analysis);

    // Generate CSS from library analysis
    if !lib_analysis.animations.is_empty() {
        println!("   Animations extracted: {}", lib_analysis.animations.len());
        generated_css.push_str(&lib_analysis.to_css());
    }

    if !lib_analysis.generated_css.is_empty() {
        for css_block in &lib_analysis.generated_css {
            if !generated_css.contains(css_block) {
                generated_css.push_str(css_block);
                generated_css.push_str("\n\n");
            }
        }
    }

    // Print warnings
    for warning in &lib_analysis.warnings {
        println!("   ⚠ {}", warning);
    }

    generated_css
}

/// Extract SVGs using the asset registry with deduplication and categorization
fn extract_svgs_with_registry(
    registry: &mut AssetRegistry,
    html: &str,
    svgs_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut result_html = html.to_string();

    // Get SVGs that should be extracted
    let extractable = registry.get_extractable_svgs();

    if extractable.is_empty() {
        return Ok(result_html);
    }

    // Create category directories
    let icons_dir = svgs_dir.join("icons");
    let logos_dir = svgs_dir.join("logos");
    let illustrations_dir = svgs_dir.join("illustrations");

    fs::create_dir_all(&icons_dir)?;
    fs::create_dir_all(&logos_dir)?;
    fs::create_dir_all(&illustrations_dir)?;

    // Collect SVG IDs to extract
    let svg_ids: Vec<String> = extractable.iter().map(|s| s.id.clone()).collect();

    for svg_id in svg_ids {
        // Get mutable reference to update local_path
        if let Some(svg) = registry.svgs.get_mut(&svg_id) {
            // Determine output directory based on category
            let (category_dir, category_name) = match svg.category {
                SvgCategory::Icon => (&icons_dir, "icons"),
                SvgCategory::Logo => (&logos_dir, "logos"),
                SvgCategory::Illustration => (&illustrations_dir, "illustrations"),
                _ => (&icons_dir, "icons"), // Default to icons
            };

            // Write SVG file
            let file_path = category_dir.join(&svg.filename);

            // Add XML declaration if not present
            let svg_content = if svg.content.starts_with("<?xml") {
                svg.content.clone()
            } else {
                format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", svg.content)
            };

            if fs::write(&file_path, &svg_content).is_ok() {
                let relative_path = format!("assets/svgs/{}/{}", category_name, svg.filename);
                svg.local_path = Some(relative_path.clone());

                // Replace inline SVG with <img> tag in HTML
                let classes_str = if svg.classes.is_empty() {
                    format!("svg-{}", svg.category.as_str())
                } else {
                    svg.classes.join(" ")
                };

                let img_tag = format!(
                    r#"<img src="{}" alt="{}" class="{}">"#,
                    relative_path,
                    svg.category.as_str(),
                    classes_str
                );

                result_html = result_html.replacen(&svg.content, &img_tag, 1);
            }
        }
    }

    Ok(result_html)
}
