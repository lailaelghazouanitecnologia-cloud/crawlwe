//! Fetch command - Capture a web page using headless Chrome

use headless_chrome::{Browser, LaunchOptions};
use std::fs;
use std::path::Path;
use std::time::Duration;
use url::Url;

use crawlwe_core::pipeline::{
    CssMicroparser, LibMicroparser, CssParseResult,
    CssOptimizer, HtmlOptimizer, CssOptimizeOptions, HtmlOptimizeOptions,
    DetectedLibrary,
};
use crawlwe_core::export::LibraryCDN;

pub fn run(
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

    // Create output directories
    let data_dir = output.join("data");
    let scripts_dir = output.join("scripts");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&scripts_dir)?;

    // Launch browser
    println!("Launching browser...");
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .window_size(Some((width, height)))
            .sandbox(false)
            .build()
            .map_err(|e| format!("Failed to build launch options: {}", e))?,
    )?;

    // Create tab and navigate
    println!("Opening page...");
    let tab = browser.new_tab()?;
    tab.set_default_timeout(Duration::from_secs(60));
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;

    // Wait for JS
    println!("Waiting {}s for JavaScript...", wait);
    std::thread::sleep(Duration::from_secs(wait));

    // Wait for network idle
    println!("Waiting for network...");
    let _ = tab.wait_for_element("body");

    // Scroll for lazy content
    println!("Scrolling for lazy content...");
    scroll_page(&tab)?;
    std::thread::sleep(Duration::from_secs(1));

    // Extract content
    println!("\nExtracting content...");

    // Get rendered HTML
    let html_raw = tab.get_content()?;
    println!("  HTML: {} bytes", html_raw.len());

    // Get all CSS
    let css_raw = get_all_css(&tab)?;
    println!("  CSS: {} bytes", css_raw.len());

    // Get scripts for library detection
    let external_scripts = get_external_scripts(&tab)?;
    let js_inline = get_inline_js(&tab)?;
    println!("  External scripts: {}", external_scripts.len());

    // Get title
    let title = get_title(&tab).unwrap_or_else(|_| "Untitled".to_string());
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

    // Parse CSS
    let css_parsed = CssMicroparser::parse(&css_raw);
    println!("  CSS variables: {}", css_parsed.variables.len());
    println!("  CSS keyframes: {}", css_parsed.keyframes.len());

    // Screenshot
    if screenshot {
        println!("\nTaking screenshot...");
        let png = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;
        fs::write(output.join("screenshot.png"), &png)?;
        println!("  screenshot.png saved");
    }

    // Optimize
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

    // Clean HTML
    let clean_html = generate_clean_html(&html_optimized.html, &title);
    fs::write(output.join("index.html"), &clean_html)?;
    println!("  index.html");

    // Raw HTML
    fs::write(data_dir.join("raw.html"), &html_raw)?;
    println!("  data/raw.html");

    // CSS
    fs::write(output.join("styles.css"), &css_optimized.css)?;
    println!("  styles.css");

    // Metadata
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
            "external_scripts": external_scripts.len(),
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
    });
    fs::write(data_dir.join("metadata.json"), serde_json::to_string_pretty(&metadata)?)?;
    println!("  data/metadata.json");

    // Project TOML
    if project_toml {
        let toml_content = generate_project_toml(url, &title, &detected_libs, &css_parsed);
        fs::write(output.join("project.toml"), &toml_content)?;
        println!("  project.toml");
    }

    // Scripts info
    fs::write(
        scripts_dir.join("external.json"),
        serde_json::to_string_pretty(&external_scripts)?,
    )?;
    println!("  scripts/external.json");

    println!("\nDone! Output: {:?}", output);
    Ok(())
}

fn scroll_page(tab: &headless_chrome::Tab) -> Result<(), Box<dyn std::error::Error>> {
    tab.evaluate(
        r#"
        (function() {
            const height = document.body.scrollHeight;
            const step = window.innerHeight;
            for (let y = 0; y < height; y += step) {
                window.scrollTo(0, y);
            }
            window.scrollTo(0, 0);
        })()
        "#,
        false,
    )?;
    Ok(())
}

fn get_all_css(tab: &headless_chrome::Tab) -> Result<String, Box<dyn std::error::Error>> {
    let result = tab.evaluate(
        r#"
        (function() {
            let css = '';

            // Get from stylesheets
            for (const sheet of document.styleSheets) {
                try {
                    if (sheet.cssRules) {
                        const src = sheet.href || 'inline';
                        css += '/* Source: ' + src + ' */\n';
                        for (const rule of sheet.cssRules) {
                            css += rule.cssText + '\n';
                        }
                        css += '\n';
                    }
                } catch (e) {
                    if (sheet.href) {
                        css += '/* CORS: ' + sheet.href + ' */\n';
                    }
                }
            }

            // Get inline styles
            for (const style of document.querySelectorAll('style')) {
                if (style.textContent && style.textContent.trim()) {
                    css += '/* Inline style */\n' + style.textContent + '\n\n';
                }
            }

            // Get CSS variables
            const root = getComputedStyle(document.documentElement);
            let vars = ':root {\n';
            let hasVars = false;
            for (let i = 0; i < root.length; i++) {
                if (root[i].startsWith('--')) {
                    vars += '  ' + root[i] + ': ' + root.getPropertyValue(root[i]) + ';\n';
                    hasVars = true;
                }
            }
            vars += '}\n';
            if (hasVars) css += '/* CSS Variables */\n' + vars;

            return css;
        })()
        "#,
        false,
    )?;

    Ok(result.value.unwrap_or_default().as_str().unwrap_or("").to_string())
}

fn get_external_scripts(tab: &headless_chrome::Tab) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let result = tab.evaluate(
        r#"
        Array.from(document.scripts)
            .filter(s => s.src)
            .map(s => s.src)
        "#,
        false,
    )?;

    let scripts: Vec<String> = result.value
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    Ok(scripts)
}

fn get_inline_js(tab: &headless_chrome::Tab) -> Result<String, Box<dyn std::error::Error>> {
    let result = tab.evaluate(
        r#"
        Array.from(document.scripts)
            .filter(s => !s.src && s.textContent)
            .map(s => s.textContent)
            .join('\n')
        "#,
        false,
    )?;

    Ok(result.value.unwrap_or_default().as_str().unwrap_or("").to_string())
}

fn get_title(tab: &headless_chrome::Tab) -> Result<String, Box<dyn std::error::Error>> {
    let result = tab.evaluate("document.title", false)?;
    Ok(result.value.unwrap_or_default().as_str().unwrap_or("").to_string())
}

fn generate_clean_html(html: &str, title: &str) -> String {
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
    toml.push_str("generator = \"crawlwe\"\n\n");

    toml.push_str("[source]\n");
    toml.push_str(&format!("url = \"{}\"\n", url));
    toml.push_str(&format!("title = \"{}\"\n\n", title.replace('"', "\\\"")));

    toml.push_str("[technologies]\n");
    for lib in libs {
        toml.push_str(&format!("{} = \"{}\"\n",
            lib.name.replace('-', "_"),
            lib.version.as_deref().unwrap_or("*")));
    }
    toml.push('\n');

    if !css.fonts.is_empty() {
        toml.push_str("[fonts]\n");
        for font in &css.fonts {
            let url = LibraryCDN::get_font_url(font, &["400".to_string(), "700".to_string()]);
            toml.push_str(&format!("\"{}\" = \"{}\"\n", font, url));
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
