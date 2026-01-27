//! CrawlWe - Fast web page scraper using Chrome DevTools Protocol
//!
//! Extracts fully rendered HTML + CSS + Assets from dynamic JavaScript pages.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::Page;
use clap::Parser;
use futures::StreamExt;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

#[derive(Parser)]
#[command(name = "crawlwe")]
#[command(about = "Fast web page scraper - extracts HTML + CSS + Assets from dynamic pages")]
#[command(version = "0.1.0")]
struct Args {
    /// URL to scrape
    #[arg(short, long)]
    url: String,

    /// Output directory
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Wait time in seconds for JS to load
    #[arg(short, long, default_value = "5")]
    wait: u64,

    /// Download images
    #[arg(long)]
    download_images: bool,

    /// Download fonts
    #[arg(long)]
    download_fonts: bool,

    /// Viewport width
    #[arg(long, default_value = "1440")]
    width: u32,

    /// Viewport height
    #[arg(long, default_value = "900")]
    height: u32,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    url: String,
    title: String,
    captured_at: String,
    html_size: usize,
    css_size: usize,
    scripts_count: usize,
    images_count: usize,
    fonts_count: usize,
    links_count: usize,
    libraries: Vec<DetectedLibrary>,
}

#[derive(Serialize, Deserialize, Clone)]
struct DetectedLibrary {
    name: String,
    version: Option<String>,
    category: String,
}

#[derive(Serialize, Deserialize)]
struct ProjectConfig {
    meta: ProjectMeta,
    source: ProjectSource,
    technologies: Technologies,
    dependencies: Dependencies,
}

#[derive(Serialize, Deserialize)]
struct ProjectMeta {
    name: String,
    version: String,
    generator: String,
    captured_at: String,
}

#[derive(Serialize, Deserialize)]
struct ProjectSource {
    url: String,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct Technologies {
    detected_libs: HashMap<String, LibInfo>,
}

#[derive(Serialize, Deserialize)]
struct LibInfo {
    version: Option<String>,
    category: String,
}

#[derive(Serialize, Deserialize)]
struct Dependencies {
    scripts: HashMap<String, String>,
    styles: HashMap<String, String>,
    fonts: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🕷️  CrawlWe - Web Page Scraper");
    println!("==============================");
    println!("URL: {}", args.url);
    println!();

    // Create output directories
    let output = &args.output;
    fs::create_dir_all(output)?;
    fs::create_dir_all(output.join("data"))?;
    fs::create_dir_all(output.join("scripts"))?;
    fs::create_dir_all(output.join("assets/images"))?;
    fs::create_dir_all(output.join("assets/fonts"))?;

    println!("🚀 Launching Chrome...");

    // Configure browser
    let config = BrowserConfig::builder()
        .window_size(args.width, args.height)
        .build()
        .map_err(|e| format!("Failed to build browser config: {}", e))?;

    let (mut browser, mut handler) = Browser::launch(config).await?;

    // Spawn handler
    let handle = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                break;
            }
        }
    });

    // Create new page
    println!("📄 Loading page...");
    let page = browser.new_page(&args.url).await?;

    // Wait for network idle
    page.wait_for_navigation().await?;

    // Additional wait for JS
    println!("⏳ Waiting {}s for JavaScript...", args.wait);
    tokio::time::sleep(Duration::from_secs(args.wait)).await;

    // Scroll to trigger lazy loading
    println!("📜 Scrolling page...");
    scroll_page(&page).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get page title
    let title = page
        .evaluate("document.title")
        .await?
        .into_value::<String>()
        .unwrap_or_else(|_| "Untitled".to_string());

    println!("📝 Title: {}", title);

    // Get rendered HTML
    println!("📥 Extracting HTML...");
    let html = page
        .evaluate("document.documentElement.outerHTML")
        .await?
        .into_value::<String>()?;

    // Save raw HTML
    fs::write(output.join("data/raw.html"), &html)?;

    // Extract and collect CSS
    println!("🎨 Extracting CSS...");
    let css = extract_all_css(&page, &args.url).await?;
    fs::write(output.join("styles.css"), &css)?;

    // Parse HTML for analysis
    let document = Html::parse_document(&html);

    // Extract scripts
    println!("📜 Extracting scripts...");
    let scripts = extract_scripts(&document, output)?;

    // Extract images
    println!("🖼️  Extracting images...");
    let images = extract_images(&document, &args.url);
    fs::write(
        output.join("data/images.json"),
        serde_json::to_string_pretty(&images)?,
    )?;

    // Extract fonts from CSS
    println!("🔤 Extracting fonts...");
    let fonts = extract_fonts(&css);
    fs::write(
        output.join("data/fonts.json"),
        serde_json::to_string_pretty(&fonts)?,
    )?;

    // Extract links
    let links = extract_links(&document, &args.url);
    fs::write(
        output.join("data/links.json"),
        serde_json::to_string_pretty(&links)?,
    )?;

    // Detect libraries
    println!("🔍 Detecting libraries...");
    let libraries = detect_libraries(&html, &css, &scripts);

    // Download assets if requested
    if args.download_images {
        println!("⬇️  Downloading images...");
        download_assets(&images, output.join("assets/images")).await?;
    }

    if args.download_fonts {
        println!("⬇️  Downloading fonts...");
        let font_urls: Vec<String> = fonts.iter().filter(|f| f.starts_with("http")).cloned().collect();
        download_assets(&font_urls, output.join("assets/fonts")).await?;
    }

    // Take screenshots
    println!("📸 Taking screenshots...");

    // Viewport screenshot
    if let Ok(screenshot) = page.screenshot(
        chromiumoxide::page::ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .build(),
    ).await {
        fs::write(output.join("screenshot.png"), screenshot)?;
    }

    // Full page screenshot
    if let Ok(screenshot) = page.screenshot(
        chromiumoxide::page::ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .build(),
    ).await {
        fs::write(output.join("screenshot_full.png"), screenshot)?;
    }

    // Clean HTML (remove scripts, add stylesheet link)
    println!("🧹 Cleaning HTML...");
    let clean_html = clean_html(&html);
    fs::write(output.join("index.html"), &clean_html)?;

    // Generate metadata
    let metadata = Metadata {
        url: args.url.clone(),
        title: title.clone(),
        captured_at: chrono::Utc::now().to_rfc3339(),
        html_size: html.len(),
        css_size: css.len(),
        scripts_count: scripts.len(),
        images_count: images.len(),
        fonts_count: fonts.len(),
        links_count: links.len(),
        libraries: libraries.clone(),
    };
    fs::write(
        output.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )?;

    // Generate project.toml
    println!("📋 Generating project.toml...");
    generate_project_toml(output, &args.url, &title, &libraries, &fonts)?;

    // Close browser
    browser.close().await?;
    handle.abort();

    // Print summary
    println!();
    println!("✅ Capture complete!");
    println!("   HTML: {} bytes", html.len());
    println!("   CSS:  {} bytes", css.len());
    println!("   Scripts: {}", scripts.len());
    println!("   Images: {}", images.len());
    println!("   Fonts: {}", fonts.len());
    println!("   Libraries: {}", libraries.len());
    for lib in &libraries {
        let ver = lib.version.as_deref().unwrap_or("?");
        println!("     - {} v{} [{}]", lib.name, ver, lib.category);
    }
    println!();
    println!("📁 Output: {}", output.display());

    Ok(())
}

async fn scroll_page(page: &Page) -> Result<(), Box<dyn std::error::Error>> {
    // Scroll down in steps to trigger lazy loading
    page.evaluate(
        r#"
        (async () => {
            const delay = ms => new Promise(r => setTimeout(r, ms));
            const height = document.body.scrollHeight;
            const step = window.innerHeight;
            for (let y = 0; y < height; y += step) {
                window.scrollTo(0, y);
                await delay(100);
            }
            window.scrollTo(0, 0);
        })()
        "#,
    )
    .await?;
    Ok(())
}

async fn extract_all_css(page: &Page, base_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut all_css = String::new();

    // Get inline styles
    let inline_css: Vec<String> = page
        .evaluate(
            r#"
            Array.from(document.querySelectorAll('style'))
                .map(s => s.textContent)
                .filter(s => s && s.trim().length > 0)
            "#,
        )
        .await?
        .into_value()
        .unwrap_or_default();

    for (i, css) in inline_css.iter().enumerate() {
        all_css.push_str(&format!("/* Inline style {} */\n{}\n\n", i + 1, css));
    }

    // Get external stylesheet URLs
    let stylesheet_urls: Vec<String> = page
        .evaluate(
            r#"
            Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
                .map(l => l.href)
                .filter(h => h && h.length > 0)
            "#,
        )
        .await?
        .into_value()
        .unwrap_or_default();

    // Fetch external stylesheets
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    for url in stylesheet_urls {
        if let Ok(response) = client.get(&url).send().await {
            if let Ok(css) = response.text().await {
                all_css.push_str(&format!("/* Source: {} */\n{}\n\n", url, css));
            }
        }
    }

    // Get computed styles for key elements (optional, can be large)
    // Skipping for now to keep output manageable

    Ok(all_css)
}

fn extract_scripts(document: &Html, output: &PathBuf) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let script_selector = Selector::parse("script").unwrap();
    let mut scripts = Vec::new();
    let mut inline_count = 0;

    for script in document.select(&script_selector) {
        let src = script.value().attr("src");
        let script_type = script.value().attr("type").unwrap_or("text/javascript");

        if let Some(src) = src {
            scripts.push(serde_json::json!({
                "type": "external",
                "src": src,
                "script_type": script_type
            }));
        } else {
            let content = script.inner_html();
            if !content.trim().is_empty() {
                inline_count += 1;
                let filename = format!("inline_{}.js", inline_count);
                fs::write(output.join("scripts").join(&filename), &content)?;
                scripts.push(serde_json::json!({
                    "type": "inline",
                    "file": filename,
                    "script_type": script_type,
                    "size": content.len()
                }));
            }
        }
    }

    fs::write(
        output.join("scripts/scripts.json"),
        serde_json::to_string_pretty(&scripts)?,
    )?;

    Ok(scripts)
}

fn extract_images(document: &Html, base_url: &str) -> Vec<String> {
    let img_selector = Selector::parse("img").unwrap();
    let base = Url::parse(base_url).ok();

    let mut images: HashSet<String> = HashSet::new();

    for img in document.select(&img_selector) {
        if let Some(src) = img.value().attr("src") {
            let url = resolve_url(src, &base);
            images.insert(url);
        }
        if let Some(srcset) = img.value().attr("srcset") {
            for part in srcset.split(',') {
                if let Some(url) = part.trim().split_whitespace().next() {
                    let url = resolve_url(url, &base);
                    images.insert(url);
                }
            }
        }
    }

    // Also check for background images in style attributes
    let all_selector = Selector::parse("*").unwrap();
    let bg_regex = Regex::new(r#"url\(['"]?([^'")\s]+)['"]?\)"#).unwrap();

    for elem in document.select(&all_selector) {
        if let Some(style) = elem.value().attr("style") {
            for cap in bg_regex.captures_iter(style) {
                if let Some(url) = cap.get(1) {
                    let url = resolve_url(url.as_str(), &base);
                    if url.contains(".png") || url.contains(".jpg") || url.contains(".webp") || url.contains(".svg") {
                        images.insert(url);
                    }
                }
            }
        }
    }

    images.into_iter().collect()
}

fn extract_fonts(css: &str) -> Vec<String> {
    let mut fonts = HashSet::new();

    // Match @font-face src urls
    let font_regex = Regex::new(r#"url\(['"]?([^'")\s]+\.(woff2?|ttf|otf|eot))['"]?\)"#).unwrap();
    for cap in font_regex.captures_iter(css) {
        if let Some(url) = cap.get(1) {
            fonts.insert(url.as_str().to_string());
        }
    }

    // Match font-family names
    let family_regex = Regex::new(r#"font-family:\s*['"]?([^'";,}]+)"#).unwrap();
    for cap in family_regex.captures_iter(css) {
        if let Some(family) = cap.get(1) {
            let family = family.as_str().trim();
            if !family.is_empty()
                && !["inherit", "initial", "sans-serif", "serif", "monospace"].contains(&family)
            {
                fonts.insert(family.to_string());
            }
        }
    }

    fonts.into_iter().collect()
}

fn extract_links(document: &Html, base_url: &str) -> Vec<String> {
    let a_selector = Selector::parse("a[href]").unwrap();
    let base = Url::parse(base_url).ok();

    let mut links: HashSet<String> = HashSet::new();

    for a in document.select(&a_selector) {
        if let Some(href) = a.value().attr("href") {
            let url = resolve_url(href, &base);
            if !url.starts_with("javascript:") && !url.starts_with("#") {
                links.insert(url);
            }
        }
    }

    links.into_iter().collect()
}

fn resolve_url(url: &str, base: &Option<Url>) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    if url.starts_with("//") {
        return format!("https:{}", url);
    }
    if let Some(base) = base {
        if let Ok(resolved) = base.join(url) {
            return resolved.to_string();
        }
    }
    url.to_string()
}

fn detect_libraries(html: &str, css: &str, scripts: &[serde_json::Value]) -> Vec<DetectedLibrary> {
    let mut libraries = Vec::new();
    let combined = format!("{}\n{}", html, css);

    // Library patterns: (name, patterns, category, version_regex)
    let patterns: Vec<(&str, Vec<&str>, &str, Option<&str>)> = vec![
        // Animation
        ("gsap", vec!["gsap", "TweenMax", "TweenLite", "ScrollTrigger"], "animation", Some(r"gsap[/@](\d+\.\d+\.\d+)")),
        ("anime.js", vec!["anime.js", "animejs"], "animation", Some(r"anime[/@](\d+\.\d+\.\d+)")),
        ("framer-motion", vec!["framer-motion", "motion.div"], "animation", None),
        ("lottie", vec!["lottie", "bodymovin"], "animation", None),

        // 3D/Graphics
        ("three.js", vec!["three.js", "THREE.", "three.module"], "3d", Some(r"three[/@](\d+\.\d+\.\d+)")),
        ("babylon.js", vec!["babylonjs", "BABYLON."], "3d", None),
        ("pixi.js", vec!["pixi.js", "PIXI."], "graphics", None),
        ("p5.js", vec!["p5.js", "p5.min"], "graphics", None),

        // Scroll
        ("locomotive-scroll", vec!["locomotive-scroll", "data-scroll"], "scroll", Some(r"locomotive-scroll[/@](\d+\.\d+\.\d+)")),
        ("lenis", vec!["lenis", "@studio-freight/lenis"], "scroll", None),
        ("scrollmagic", vec!["scrollmagic", "ScrollMagic"], "scroll", None),

        // UI Frameworks
        ("react", vec!["react", "React.", "__REACT"], "ui", Some(r"react[/@](\d+\.\d+\.\d+)")),
        ("vue", vec!["vue.js", "Vue.", "__VUE"], "ui", Some(r"vue[/@](\d+\.\d+\.\d+)")),
        ("svelte", vec!["svelte", "__svelte"], "ui", None),
        ("alpine.js", vec!["alpinejs", "x-data", "Alpine."], "ui", None),
        ("next.js", vec!["_next/", "__NEXT", "next/"], "framework", None),
        ("nuxt", vec!["_nuxt/", "__NUXT"], "framework", None),

        // CSS Frameworks
        ("tailwind", vec!["tailwindcss", "tw-"], "css", None),
        ("bootstrap", vec!["bootstrap", "btn-primary"], "css", Some(r"bootstrap[/@](\d+\.\d+\.\d+)")),
        ("bulma", vec!["bulma"], "css", None),

        // Other
        ("jquery", vec!["jquery", "jQuery"], "utility", Some(r"jquery[/@](\d+\.\d+\.\d+)")),
        ("lodash", vec!["lodash"], "utility", None),
        ("moment", vec!["moment.js", "moment.min"], "utility", None),
        ("axios", vec!["axios"], "http", None),

        // CMS/Platforms
        ("shopify", vec!["shopify", "Shopify."], "platform", None),
        ("wordpress", vec!["wp-content", "wp-includes"], "platform", None),
        ("sanity", vec!["sanity.io", "@sanity"], "cms", None),
        ("contentful", vec!["contentful"], "cms", None),
    ];

    for (name, keywords, category, version_pattern) in patterns {
        let found = keywords.iter().any(|k| combined.contains(k))
            || scripts.iter().any(|s| {
                s.get("src")
                    .and_then(|v| v.as_str())
                    .map(|src| keywords.iter().any(|k| src.to_lowercase().contains(&k.to_lowercase())))
                    .unwrap_or(false)
            });

        if found {
            let version = version_pattern.and_then(|pattern| {
                Regex::new(pattern).ok().and_then(|re| {
                    re.captures(&combined)
                        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                })
            });

            libraries.push(DetectedLibrary {
                name: name.to_string(),
                version,
                category: category.to_string(),
            });
        }
    }

    libraries
}

fn clean_html(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut output = html.to_string();

    // Remove script tags
    let script_selector = Selector::parse("script").unwrap();
    for script in document.select(&script_selector) {
        let script_html = script.html();
        output = output.replace(&script_html, "");
    }

    // Add stylesheet link in head if not present
    if !output.contains("styles.css") {
        output = output.replace(
            "</head>",
            r#"  <link rel="stylesheet" href="styles.css">
</head>"#,
        );
    }

    output
}

fn generate_project_toml(
    output: &PathBuf,
    url: &str,
    title: &str,
    libraries: &[DetectedLibrary],
    fonts: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut detected_libs = HashMap::new();
    let mut script_deps = HashMap::new();

    for lib in libraries {
        detected_libs.insert(
            lib.name.clone(),
            LibInfo {
                version: lib.version.clone(),
                category: lib.category.clone(),
            },
        );

        // Add CDN URLs for known libraries
        if let Some(cdn) = get_cdn_url(&lib.name, lib.version.as_deref()) {
            script_deps.insert(lib.name.clone(), cdn);
        }
    }

    let config = ProjectConfig {
        meta: ProjectMeta {
            name: sanitize_name(title),
            version: "1.0.0".to_string(),
            generator: "crawlwe".to_string(),
            captured_at: chrono::Utc::now().to_rfc3339(),
        },
        source: ProjectSource {
            url: url.to_string(),
            title: title.to_string(),
        },
        technologies: Technologies { detected_libs },
        dependencies: Dependencies {
            scripts: script_deps,
            styles: HashMap::new(),
            fonts: fonts.iter().filter(|f| !f.starts_with("http")).cloned().collect(),
        },
    };

    let toml_string = toml::to_string_pretty(&config)?;
    fs::write(output.join("project.toml"), toml_string)?;

    Ok(())
}

fn sanitize_name(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn get_cdn_url(lib: &str, version: Option<&str>) -> Option<String> {
    let v = version.unwrap_or("latest");
    match lib {
        "gsap" => Some(format!("https://cdnjs.cloudflare.com/ajax/libs/gsap/{}/gsap.min.js", v)),
        "three.js" => Some(format!("https://cdnjs.cloudflare.com/ajax/libs/three.js/{}/three.min.js", v)),
        "anime.js" => Some(format!("https://cdnjs.cloudflare.com/ajax/libs/animejs/{}/anime.min.js", v)),
        "jquery" => Some(format!("https://cdnjs.cloudflare.com/ajax/libs/jquery/{}/jquery.min.js", v)),
        "bootstrap" => Some(format!("https://cdnjs.cloudflare.com/ajax/libs/bootstrap/{}/js/bootstrap.bundle.min.js", v)),
        "locomotive-scroll" => Some(format!("https://cdn.jsdelivr.net/npm/locomotive-scroll@{}/dist/locomotive-scroll.min.js", v)),
        _ => None,
    }
}

async fn download_assets(urls: &[String], output_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    for url in urls {
        if let Ok(parsed) = Url::parse(url) {
            let filename = parsed
                .path_segments()
                .and_then(|s| s.last())
                .unwrap_or("asset");

            if let Ok(response) = client.get(url).send().await {
                if let Ok(bytes) = response.bytes().await {
                    let path = output_dir.join(filename);
                    let _ = fs::write(path, bytes);
                }
            }
        }
    }

    Ok(())
}
