//! Fetch command - Capture a single web page

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::Page;
use futures::StreamExt;
use std::fs;
use std::path::Path;
use std::time::Duration;
use url::Url;

use crawlwe_core::pipeline::{
    CssMicroparser, LibMicroparser, CssParseResult,
    CssOptimizer, HtmlOptimizer, CssOptimizeOptions, HtmlOptimizeOptions,
    DetectedLibrary, LibrarySource,
};
use crawlwe_core::export::LibraryCDN;

pub async fn run(
    url: &str,
    output: &Path,
    wait: u64,
    width: u32,
    height: u32,
    screenshot: bool,
    detect_libs: bool,
    project_toml: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe - Web Page Extractor");
    println!("============================");
    println!("URL: {}", url);
    println!();

    // Create output structure
    let data_dir = output.join("data");
    let scripts_dir = output.join("scripts");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&scripts_dir)?;

    // Launch browser
    println!("Launching browser...");
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .window_size(width, height)
            .build()
            .map_err(|e| format!("Failed to build browser config: {}", e))?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });

    // Create new page
    println!("Opening page...");
    let page = browser.new_page(url).await?;

    // Wait for page to load
    println!("Waiting {}s for JavaScript...", wait);
    tokio::time::sleep(Duration::from_secs(wait)).await;

    // Scroll to load lazy content
    println!("Scrolling for lazy content...");
    scroll_page(&page).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Extract all data
    println!("\nExtracting content...");

    // Get rendered HTML
    let html_raw = get_rendered_html(&page).await?;
    println!("  HTML: {} bytes", html_raw.len());

    // Get all CSS
    let css_raw = get_all_css(&page).await?;
    println!("  CSS: {} bytes", css_raw.len());

    // Get inline JS for analysis
    let js_inline = get_inline_js(&page).await?;

    // Get external resources
    let external_scripts = get_external_scripts(&page).await?;
    let external_styles = get_external_styles(&page).await?;
    println!("  External scripts: {}", external_scripts.len());
    println!("  External styles: {}", external_styles.len());

    // Get page title
    let title = get_title(&page).await.unwrap_or_else(|_| "Untitled".to_string());
    println!("  Title: {}", title);

    // Detect libraries
    let mut detected_libs = Vec::new();
    if detect_libs {
        let lib_result = LibMicroparser::parse(&external_scripts, &js_inline);
        detected_libs = lib_result.libraries;
        println!("  Libraries: {} detected", detected_libs.len());
        for lib in &detected_libs {
            println!("    - {} ({})", lib.name, lib.category);
        }
    }

    // Parse CSS for analysis
    let css_parsed = CssMicroparser::parse(&css_raw);
    println!("  CSS variables: {}", css_parsed.variables.len());
    println!("  CSS keyframes: {}", css_parsed.keyframes.len());

    // Take screenshot
    if screenshot {
        println!("\nTaking screenshot...");
        let screenshot_data = page
            .screenshot(
                chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .capture_beyond_viewport(true)
                    .build(),
            )
            .await?;
        fs::write(output.join("screenshot.png"), &screenshot_data)?;
        println!("  screenshot.png saved");
    }

    // Optimize content
    println!("\nOptimizing...");

    let html_optimized = HtmlOptimizer::optimize(&html_raw, &HtmlOptimizeOptions {
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
    println!("  HTML: {} -> {} bytes", html_raw.len(), html_optimized.stats.optimized_size);

    let css_optimized = CssOptimizer::optimize(&css_raw, &CssOptimizeOptions::default());
    println!("  CSS: {} -> {} bytes", css_raw.len(), css_optimized.stats.optimized_size);

    // Save files
    println!("\nSaving files...");

    // Save clean HTML
    let clean_html = generate_clean_html(&html_optimized.html, &title);
    fs::write(output.join("index.html"), &clean_html)?;
    println!("  index.html");

    // Save raw HTML
    fs::write(data_dir.join("raw.html"), &html_raw)?;
    println!("  data/raw.html");

    // Save CSS
    fs::write(output.join("styles.css"), &css_optimized.css)?;
    println!("  styles.css");

    // Save metadata
    let base_url = Url::parse(url)?;
    let metadata = serde_json::json!({
        "url": url,
        "domain": base_url.host_str(),
        "title": title,
        "extracted_at": chrono::Utc::now().to_rfc3339(),
        "stats": {
            "html_bytes": html_raw.len(),
            "html_optimized_bytes": html_optimized.stats.optimized_size,
            "css_bytes": css_raw.len(),
            "css_optimized_bytes": css_optimized.stats.optimized_size,
            "scripts_removed": html_optimized.stats.scripts_removed,
            "external_scripts": external_scripts.len(),
            "external_styles": external_styles.len(),
        },
        "css_analysis": {
            "variables": css_parsed.variables.len(),
            "keyframes": css_parsed.keyframes.len(),
            "colors": css_parsed.colors.len(),
            "fonts": css_parsed.fonts,
        },
        "detected_libs": detected_libs.iter().map(|l| {
            serde_json::json!({
                "name": l.name,
                "category": l.category,
                "version": l.version,
            })
        }).collect::<Vec<_>>(),
        "viewport": {
            "width": width,
            "height": height
        }
    });
    fs::write(data_dir.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)?;
    println!("  data/metadata.json");

    // Generate project.toml
    if project_toml {
        let toml_content = generate_project_toml(url, &title, &detected_libs, &css_parsed);
        fs::write(output.join("project.toml"), &toml_content)?;
        println!("  project.toml");
    }

    // Save scripts info
    fs::write(
        scripts_dir.join("external.json"),
        serde_json::to_string_pretty(&external_scripts)?,
    )?;
    println!("  scripts/external.json");

    // Close browser
    browser.close().await?;
    handle.abort();

    println!("\nDone! Output: {:?}", output);

    Ok(())
}

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

async fn get_rendered_html(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let html: String = page
        .evaluate("document.documentElement.outerHTML")
        .await?
        .into_value()?;
    Ok(html)
}

async fn get_all_css(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let css: String = page
        .evaluate(
            r#"
            (function() {
                let css = '';
                for (const sheet of document.styleSheets) {
                    try {
                        if (sheet.cssRules) {
                            const source = sheet.href || 'inline';
                            css += `/* Source: ${source} */\n`;
                            for (const rule of sheet.cssRules) {
                                css += rule.cssText + '\n';
                            }
                            css += '\n';
                        }
                    } catch (e) {
                        if (sheet.href) {
                            css += `/* External (CORS): ${sheet.href} */\n`;
                        }
                    }
                }
                return css;
            })()
            "#,
        )
        .await?
        .into_value()?;
    Ok(css)
}

async fn get_inline_js(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let js: String = page
        .evaluate(
            r#"
            Array.from(document.scripts)
                .filter(s => !s.src && s.textContent)
                .map(s => s.textContent)
                .join('\n\n')
            "#,
        )
        .await?
        .into_value()?;
    Ok(js)
}

async fn get_external_scripts(page: &Page) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let scripts: Vec<String> = page
        .evaluate(
            r#"
            Array.from(document.scripts)
                .filter(s => s.src)
                .map(s => s.src)
            "#,
        )
        .await?
        .into_value()?;
    Ok(scripts)
}

async fn get_external_styles(page: &Page) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let styles: Vec<String> = page
        .evaluate(
            r#"
            Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
                .filter(l => l.href)
                .map(l => l.href)
            "#,
        )
        .await?
        .into_value()?;
    Ok(styles)
}

async fn get_title(page: &Page) -> Result<String, Box<dyn std::error::Error>> {
    let title: String = page.evaluate("document.title").await?.into_value()?;
    Ok(title)
}

fn generate_clean_html(html: &str, title: &str) -> String {
    // Add proper head with stylesheet link
    let head = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="styles.css">
</head>"#,
        title
    );

    // Extract body content
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

    // Meta
    toml.push_str("[meta]\n");
    toml.push_str(&format!("name = \"{}\"\n", sanitize_name(title)));
    toml.push_str("version = \"1.0.0\"\n");
    toml.push_str("generator = \"crawlwe\"\n");
    toml.push_str(&format!("captured_at = \"{}\"\n", chrono::Utc::now().to_rfc3339()));
    toml.push('\n');

    // Source
    toml.push_str("[source]\n");
    toml.push_str(&format!("url = \"{}\"\n", url));
    toml.push_str(&format!("title = \"{}\"\n", title.replace('"', "\\\"")));
    toml.push('\n');

    // Technologies
    toml.push_str("[technologies]\n");

    let animation_libs: Vec<_> = libs.iter().filter(|l| l.category == "animation").collect();
    let graphics_libs: Vec<_> = libs.iter().filter(|l| l.category == "3d" || l.category == "2d").collect();
    let ui_libs: Vec<_> = libs.iter().filter(|l| l.category == "ui").collect();
    let css_libs: Vec<_> = libs.iter().filter(|l| l.category == "css").collect();

    if !css_libs.is_empty() {
        toml.push_str(&format!("css_framework = \"{}\"\n", css_libs[0].name));
    }
    if !ui_libs.is_empty() {
        toml.push_str(&format!("ui_framework = \"{}\"\n", ui_libs[0].name));
    }
    if !animation_libs.is_empty() {
        let names: Vec<_> = animation_libs.iter().map(|l| format!("\"{}\"", l.name)).collect();
        toml.push_str(&format!("animation_libs = [{}]\n", names.join(", ")));
    }
    if !graphics_libs.is_empty() {
        let names: Vec<_> = graphics_libs.iter().map(|l| format!("\"{}\"", l.name)).collect();
        toml.push_str(&format!("graphics_libs = [{}]\n", names.join(", ")));
    }
    toml.push('\n');

    // Dependencies
    toml.push_str("[dependencies]\n\n");

    toml.push_str("[dependencies.scripts]\n");
    for lib in libs {
        let cdn_url = match &lib.source {
            LibrarySource::Url(url) => Some(url.clone()),
            _ => LibraryCDN::get_js(&lib.name, lib.version.as_deref()),
        };
        if let Some(url) = cdn_url {
            if let Some(version) = &lib.version {
                toml.push_str(&format!("{} = {{ version = \"{}\", cdn = \"{}\" }}\n",
                    lib.name.replace('-', "_").replace('.', "_"), version, url));
            } else {
                toml.push_str(&format!("{} = \"{}\"\n",
                    lib.name.replace('-', "_").replace('.', "_"), url));
            }
        }
    }
    toml.push('\n');

    // Fonts
    if !css.fonts.is_empty() {
        toml.push_str("[dependencies.fonts]\n");
        for font in &css.fonts {
            let url = LibraryCDN::get_font_url(font, &["400".to_string(), "700".to_string()]);
            toml.push_str(&format!("\"{}\" = {{ weights = [\"400\", \"700\"], url = \"{}\" }}\n",
                font, url));
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
