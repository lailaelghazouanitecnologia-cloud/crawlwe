//! CrawlWe - Fast web page scraper using Chrome DevTools Protocol
//!
//! Extracts fully rendered HTML + CSS + Assets from dynamic JavaScript pages.

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::Page;
use clap::Parser;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

#[derive(Parser)]
#[command(name = "crawlwe")]
#[command(about = "Extract HTML + CSS + Assets from dynamic web pages")]
struct Args {
    /// URL to scrape
    #[arg(short, long)]
    url: String,

    /// Output directory
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Wait time in seconds for JS to load
    #[arg(short, long, default_value = "8")]
    wait: u64,

    /// Viewport width
    #[arg(long, default_value = "1440")]
    width: u32,

    /// Viewport height
    #[arg(long, default_value = "900")]
    height: u32,

    /// Download images
    #[arg(long, default_value = "true")]
    images: bool,

    /// Download fonts
    #[arg(long, default_value = "true")]
    fonts: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageData {
    url: String,
    title: String,
    html: String,
    css: String,
    scripts: Vec<ScriptInfo>,
    images: Vec<String>,
    fonts: Vec<String>,
    links: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScriptInfo {
    src: Option<String>,
    inline: bool,
    content_length: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🕷️  CrawlWe - Web Page Extractor");
    println!("================================");
    println!("URL: {}", args.url);
    println!();

    // Create output structure
    let data_dir = args.output.join("data");
    let scripts_dir = args.output.join("scripts");
    let assets_dir = args.output.join("assets");
    let images_dir = assets_dir.join("images");
    let fonts_dir = assets_dir.join("fonts");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&scripts_dir)?;
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&fonts_dir)?;

    // Launch browser
    println!("🚀 Launching Chrome...");
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .window_size(args.width, args.height)
            .build()
            .map_err(|e| format!("Failed to build browser config: {}", e))?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });

    // Create new page
    println!("📄 Opening page...");
    let page = browser.new_page(&args.url).await?;

    // Wait for page to load
    println!("⏳ Waiting {}s for JavaScript to render...", args.wait);
    tokio::time::sleep(Duration::from_secs(args.wait)).await;

    // Scroll to load lazy content
    println!("📜 Scrolling to load lazy content...");
    scroll_page(&page).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Extract all data
    println!("\n📦 Extracting content...");

    // Get rendered HTML
    let html = get_rendered_html(&page).await?;
    println!("   ✓ HTML: {} bytes", html.len());

    // Get all CSS
    let css = get_all_css(&page).await?;
    println!("   ✓ CSS: {} bytes ({} rules approx)", css.len(), css.matches('{').count());

    // Get all scripts
    let scripts = get_scripts(&page).await?;
    println!("   ✓ Scripts: {} found", scripts.len());

    // Get all images
    let images = get_images(&page).await?;
    println!("   ✓ Images: {} found", images.len());

    // Get all fonts
    let fonts = get_fonts(&page).await?;
    println!("   ✓ Fonts: {} found", fonts.len());

    // Get all links
    let links = get_links(&page).await?;
    println!("   ✓ Links: {} found", links.len());

    // Get page title
    let title = get_title(&page).await.unwrap_or_else(|_| "Untitled".to_string());
    println!("   ✓ Title: {}", title);

    // Take screenshot
    println!("\n📸 Taking screenshot...");
    let screenshot = page
        .screenshot(
            chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build(),
        )
        .await?;
    fs::write(args.output.join("screenshot.png"), &screenshot)?;

    // Take full page screenshot
    let full_screenshot = page
        .screenshot(
            chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .capture_beyond_viewport(true)
                .build(),
        )
        .await?;
    fs::write(args.output.join("screenshot_full.png"), &full_screenshot)?;
    println!("   ✓ Screenshots saved");

    // Save files
    println!("\n💾 Saving files...");

    // Save main HTML
    let clean_html = generate_clean_html(&html, &title);
    fs::write(args.output.join("index.html"), &clean_html)?;
    println!("   ✓ index.html");

    // Save raw HTML
    fs::write(data_dir.join("raw.html"), &html)?;
    println!("   ✓ data/raw.html");

    // Save CSS
    fs::write(args.output.join("styles.css"), &css)?;
    println!("   ✓ styles.css");

    // Save scripts info
    let scripts_json = serde_json::to_string_pretty(&scripts)?;
    fs::write(scripts_dir.join("scripts.json"), &scripts_json)?;
    println!("   ✓ scripts/scripts.json");

    // Extract and save inline scripts
    let inline_scripts = get_inline_scripts(&page).await?;
    for (i, script) in inline_scripts.iter().enumerate() {
        if !script.is_empty() && script.len() > 100 {
            fs::write(scripts_dir.join(format!("inline_{}.js", i)), script)?;
        }
    }
    println!("   ✓ scripts/inline_*.js ({} files)", inline_scripts.len());

    // Save images list
    fs::write(data_dir.join("images.json"), serde_json::to_string_pretty(&images)?)?;
    println!("   ✓ data/images.json");

    // Save fonts list
    fs::write(data_dir.join("fonts.json"), serde_json::to_string_pretty(&fonts)?)?;
    println!("   ✓ data/fonts.json");

    // Save links
    fs::write(data_dir.join("links.json"), serde_json::to_string_pretty(&links)?)?;
    println!("   ✓ data/links.json");

    // Save complete metadata
    let base_url = Url::parse(&args.url)?;
    let metadata = serde_json::json!({
        "url": args.url,
        "domain": base_url.host_str(),
        "title": title,
        "extracted_at": chrono_now(),
        "stats": {
            "html_bytes": html.len(),
            "css_bytes": css.len(),
            "css_rules": css.matches('{').count(),
            "scripts_count": scripts.len(),
            "images_count": images.len(),
            "fonts_count": fonts.len(),
            "links_count": links.len(),
        },
        "viewport": {
            "width": args.width,
            "height": args.height
        }
    });
    fs::write(args.output.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)?;
    println!("   ✓ metadata.json");

    // Close browser
    browser.close().await?;
    handle.abort();

    println!("\n✅ Done!");
    println!("\n📁 Output structure:");
    println!("   {:?}/", args.output);
    println!("   ├── index.html      (clean HTML)");
    println!("   ├── styles.css      (all CSS)");
    println!("   ├── screenshot.png");
    println!("   ├── screenshot_full.png");
    println!("   ├── metadata.json");
    println!("   ├── data/");
    println!("   │   ├── raw.html");
    println!("   │   ├── images.json");
    println!("   │   ├── fonts.json");
    println!("   │   └── links.json");
    println!("   ├── scripts/");
    println!("   │   ├── scripts.json");
    println!("   │   └── inline_*.js");
    println!("   └── assets/");
    println!("       ├── images/");
    println!("       └── fonts/");

    Ok(())
}

/// Scroll the page to trigger lazy loading
async fn scroll_page(page: &Page) -> Result<(), Box<dyn std::error::Error>> {
    page.evaluate(
        r#"
        (async () => {
            const delay = ms => new Promise(r => setTimeout(r, ms));
            const height = document.body.scrollHeight;
            const step = window.innerHeight;

            for (let y = 0; y < height; y += step) {
                window.scrollTo(0, y);
                await delay(200);
            }
            window.scrollTo(0, 0);
        })()
        "#,
    )
    .await?;
    Ok(())
}

/// Get the fully rendered HTML from the page
async fn get_rendered_html(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let html: String = page
        .evaluate("document.documentElement.outerHTML")
        .await?
        .into_value()?;
    Ok(html)
}

/// Get all CSS from the page
async fn get_all_css(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let css: String = page
        .evaluate(
            r#"
            (function() {
                let css = '';

                // Get all stylesheets
                for (const sheet of document.styleSheets) {
                    try {
                        if (sheet.cssRules) {
                            const source = sheet.href || 'inline';
                            css += `\n/* ========================================\n`;
                            css += `   Source: ${source}\n`;
                            css += `   ======================================== */\n\n`;

                            for (const rule of sheet.cssRules) {
                                css += rule.cssText + '\n';
                            }
                        }
                    } catch (e) {
                        if (sheet.href) {
                            css += `\n/* External (CORS blocked): ${sheet.href} */\n`;
                        }
                    }
                }

                // Get computed styles for key elements
                const keyElements = document.querySelectorAll('header, nav, main, footer, section, article, .hero, .container');
                if (keyElements.length > 0) {
                    css += '\n/* ========================================\n';
                    css += '   Computed Styles for Key Elements\n';
                    css += '   ======================================== */\n\n';

                    keyElements.forEach(el => {
                        const selector = el.id ? `#${el.id}` :
                                        el.className ? `.${el.className.toString().split(' ').filter(c => c && !c.includes('_')).join('.')}` :
                                        el.tagName.toLowerCase();
                        if (selector && selector !== '.') {
                            const computed = getComputedStyle(el);
                            const important = ['display', 'position', 'width', 'max-width', 'padding', 'margin', 'background', 'color', 'font-family', 'font-size'];
                            const props = important.map(p => `  ${p}: ${computed.getPropertyValue(p)};`).join('\n');
                            css += `${selector} {\n${props}\n}\n\n`;
                        }
                    });
                }

                return css;
            })()
            "#,
        )
        .await?
        .into_value()?;
    Ok(css)
}

/// Get all scripts info
async fn get_scripts(page: &Page) -> Result<Vec<ScriptInfo>, Box<dyn std::error::Error>> {
    let scripts: Vec<ScriptInfo> = page
        .evaluate(
            r#"
            Array.from(document.scripts).map(s => ({
                src: s.src || null,
                inline: !s.src,
                content_length: s.textContent?.length || 0
            }))
            "#,
        )
        .await?
        .into_value()?;
    Ok(scripts)
}

/// Get inline script contents
async fn get_inline_scripts(page: &Page) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let scripts: Vec<String> = page
        .evaluate(
            r#"
            Array.from(document.scripts)
                .filter(s => !s.src && s.textContent)
                .map(s => s.textContent)
            "#,
        )
        .await?
        .into_value()?;
    Ok(scripts)
}

/// Get all image URLs
async fn get_images(page: &Page) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let images: Vec<String> = page
        .evaluate(
            r#"
            [...new Set([
                ...Array.from(document.images).map(img => img.src),
                ...Array.from(document.querySelectorAll('[style*="background"]'))
                    .map(el => getComputedStyle(el).backgroundImage)
                    .filter(bg => bg.startsWith('url('))
                    .map(bg => bg.slice(4, -1).replace(/["']/g, '')),
                ...Array.from(document.querySelectorAll('source[srcset]'))
                    .flatMap(s => s.srcset.split(',').map(src => src.trim().split(' ')[0]))
            ])].filter(url => url && url.startsWith('http'))
            "#,
        )
        .await?
        .into_value()?;
    Ok(images)
}

/// Get all font URLs
async fn get_fonts(page: &Page) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let fonts: Vec<String> = page
        .evaluate(
            r#"
            (function() {
                const fonts = new Set();

                // From @font-face rules
                for (const sheet of document.styleSheets) {
                    try {
                        for (const rule of sheet.cssRules) {
                            if (rule instanceof CSSFontFaceRule) {
                                const src = rule.style.getPropertyValue('src');
                                const urls = src.match(/url\(['"]?([^'")\s]+)['"]?\)/g) || [];
                                urls.forEach(u => {
                                    const url = u.slice(4, -1).replace(/["']/g, '');
                                    if (url.startsWith('http')) fonts.add(url);
                                });
                            }
                        }
                    } catch (e) {}
                }

                // From link preloads
                document.querySelectorAll('link[rel="preload"][as="font"]').forEach(l => {
                    if (l.href) fonts.add(l.href);
                });

                return [...fonts];
            })()
            "#,
        )
        .await?
        .into_value()?;
    Ok(fonts)
}

/// Get all links
async fn get_links(page: &Page) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let links: Vec<String> = page
        .evaluate(
            r#"
            [...new Set(Array.from(document.links).map(a => a.href))]
                .filter(url => url && url.startsWith('http'))
            "#,
        )
        .await?
        .into_value()?;
    Ok(links)
}

/// Get page title
async fn get_title(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let title: String = page.evaluate("document.title").await?.into_value()?;
    Ok(title)
}

/// Generate clean HTML with stylesheet link
fn generate_clean_html(raw_html: &str, title: &str) -> String {
    let style_link = format!(
        r#"<meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="styles.css">"#,
        title
    );

    if raw_html.contains("<head>") {
        // Insert after <head>
        raw_html.replacen("<head>", &format!("<head>\n    {}", style_link), 1)
    } else if raw_html.contains("<HEAD>") {
        raw_html.replacen("<HEAD>", &format!("<HEAD>\n    {}", style_link), 1)
    } else {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    {}
</head>
{}
</html>"#,
            style_link, raw_html
        )
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}s since epoch", duration.as_secs())
}
