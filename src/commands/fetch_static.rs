//! Static fetch - Downloads HTML/CSS/JS via HTTP without browser
//!
//! This is faster and works in restricted environments where Chrome can't run.
//! Limitations: No JavaScript execution, no computed styles.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

use crawlwe_core::pipeline::{CssMicroparser, LibMicroparser, CssOptimizer, HtmlOptimizer, CssOptimizeOptions, HtmlOptimizeOptions, DetectedLibrary, CssParseResult};
use crawlwe_core::export::LibraryCDN;

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
    let html_optimized = HtmlOptimizer::optimize(&html_with_local_images, &HtmlOptimizeOptions {
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
            "images_downloaded": image_map.len() + html_image_map.len(),
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

fn generate_clean_html(html: &str, title: &str, base_url: &str, css: &str) -> String {
    // Extract base URL (without path) for assets
    let base_href = if let Ok(url) = Url::parse(base_url) {
        format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""))
    } else {
        base_url.to_string()
    };

    // Embed CSS inline so base href doesn't affect it
    let head = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <base href="{}/">
    <title>{}</title>
    <style>
{}
    </style>
</head>"#,
        base_href, title, css
    );

    if let Some(body_start) = html.find("<body") {
        if let Some(body_end) = html.rfind("</body>") {
            let body = &html[body_start..body_end + 7];
            return format!("{}\n{}\n</html>", head, body);
        }
    }

    format!("{}\n<body>\n{}\n</body>\n</html>", head, html)
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
