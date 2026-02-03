//! Browser-based fetch - Uses headless Chrome for full JavaScript rendering
//!
//! This mode:
//! - Executes JavaScript for dynamic content
//! - Captures computed styles (final rendered CSS)
//! - Handles SPAs and client-side rendering
//! - Extracts all assets after full page load

use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::protocol::cdp::Page;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use url::Url;

use crawlwe_core::pipeline::{CssMicroparser, LibMicroparser, HtmlOptimizer, HtmlOptimizeOptions};
use crawlwe_core::css::{CssFormatter, CssFormatOptions};
use crawlwe_core::assets::{AssetRegistry, CssStats};

/// Browser fetch options
pub struct BrowserFetchOptions {
    /// Wait time after page load (ms)
    pub wait_after_load: u64,
    /// Viewport width
    pub viewport_width: u32,
    /// Viewport height
    pub viewport_height: u32,
    /// Take screenshot
    pub take_screenshot: bool,
    /// Extract computed styles
    pub extract_computed_styles: bool,
    /// Maximum wait time (ms)
    pub timeout: u64,
    /// Optical mode - capture ALL visible elements with full computed styles
    pub optical_mode: bool,
}

impl Default for BrowserFetchOptions {
    fn default() -> Self {
        Self {
            wait_after_load: 2000,
            viewport_width: 1920,
            viewport_height: 1080,
            take_screenshot: true,
            extract_computed_styles: true,
            timeout: 30000,
            optical_mode: true, // Default to optical mode for best visual capture
        }
    }
}

/// Main browser fetch function
pub async fn run(
    url: &str,
    output: &Path,
    project_toml: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    run_with_options(url, output, project_toml, BrowserFetchOptions::default()).await
}

/// Browser fetch with custom options
pub async fn run_with_options(
    url: &str,
    output: &Path,
    project_toml: bool,
    options: BrowserFetchOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe - Browser Fetch (Headless Chrome)");
    println!("==========================================");
    println!("URL: {}", url);
    if options.optical_mode {
        println!("Mode: OPTICAL - Capture all visible elements with computed styles");
    } else {
        println!("Mode: Basic - JavaScript rendering + key element styles");
    }
    println!();

    let base_url = Url::parse(url)?;

    // Create output structure
    let data_dir = output.join("data");
    let assets_dir = output.join("assets");
    let fonts_dir = assets_dir.join("fonts");
    let images_dir = assets_dir.join("images");
    let svgs_dir = assets_dir.join("svgs");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&fonts_dir)?;
    fs::create_dir_all(&images_dir)?;
    fs::create_dir_all(&svgs_dir)?;

    // Initialize Asset Registry
    let mut asset_registry = AssetRegistry::new();

    // Launch browser
    println!("1. Launching browser...");
    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        .sandbox(false) // Required when running as root
        .window_size(Some((options.viewport_width, options.viewport_height)))
        .build()
        .map_err(|e| format!("Failed to build launch options: {}", e))?;

    let browser = Browser::new(launch_options)
        .map_err(|e| format!("Failed to launch browser: {}", e))?;

    let tab = browser.new_tab()
        .map_err(|e| format!("Failed to create tab: {}", e))?;

    // Navigate to page
    println!("2. Navigating to page...");
    tab.navigate_to(url)?;

    // Wait for page load
    println!("   Waiting for page load...");
    tab.wait_until_navigated()?;

    // Additional wait for dynamic content
    if options.wait_after_load > 0 {
        println!("   Waiting {}ms for dynamic content...", options.wait_after_load);
        std::thread::sleep(Duration::from_millis(options.wait_after_load));
    }

    // Get page title using JavaScript
    let title = tab.evaluate("document.title", false)
        .map(|r| r.value.map(|v| v.to_string().trim_matches('"').to_string()))
        .ok()
        .flatten()
        .unwrap_or_else(|| "Untitled".to_string());
    println!("   Title: {}", title);

    // Take screenshot
    if options.take_screenshot {
        println!("   Taking screenshot...");
        match tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true
        ) {
            Ok(data) => {
                let screenshot_path = output.join("screenshot.png");
                fs::write(&screenshot_path, &data)?;
                println!("   + screenshot.png");
            }
            Err(e) => {
                println!("   ! Screenshot failed: {}", e);
            }
        }
    }

    // Get rendered HTML using JavaScript
    println!("\n3. Extracting rendered HTML...");
    let html = tab.evaluate("document.documentElement.outerHTML", false)
        .map(|r| r.value.map(|v| {
            // Remove surrounding quotes if present
            let s = v.to_string();
            if s.starts_with('"') && s.ends_with('"') {
                unescape_js_string(&s[1..s.len()-1])
            } else {
                s
            }
        }))
        .ok()
        .flatten()
        .unwrap_or_else(|| "<html><body>Failed to get HTML</body></html>".to_string());
    println!("   HTML: {} bytes", html.len());

    // Get all stylesheets using JavaScript
    println!("\n4. Extracting CSS...");
    let all_css = get_stylesheets_js(&tab)?;
    println!("   Total CSS: {} bytes", all_css.len());

    // Get computed styles - either optical (full) or basic mode
    let (computed_styles, optical_result) = if options.optical_mode {
        println!("\n5. Optical capture (full computed styles)...");
        let optical = get_optical_styles_js(&tab)?;
        println!("   Total elements: {}", optical.stats.total);
        println!("   Visible elements: {}", optical.stats.visible);
        println!("   Style groups (deduplicated): {}", optical.stats.groups);
        (HashMap::new(), Some(optical))
    } else if options.extract_computed_styles {
        println!("\n5. Extracting computed styles (basic)...");
        let computed = get_computed_styles_js(&tab)?;
        println!("   Computed styles for {} elements", computed.len());
        (computed, None)
    } else {
        (HashMap::new(), None)
    };

    // Format the CSS
    println!("\n6. Formatting CSS...");
    let formatter = CssFormatter::with_options(CssFormatOptions {
        indent_size: 2,
        blank_lines_between_rules: true,
        add_section_comments: true,
        group_properties: true,
        ..Default::default()
    });
    let format_result = formatter.format(&all_css);
    println!("   Rules formatted: {}", format_result.stats.rules_formatted);
    println!("   Keyframes: {}", format_result.stats.keyframes_formatted);
    println!("   Media queries: {}", format_result.stats.media_queries_formatted);
    println!("   Font faces: {}", format_result.stats.font_faces_formatted);
    println!("   Variables: {}", format_result.stats.variables_found);

    // Extract HTML classes
    let html_classes = extract_classes_from_html(&html);
    println!("   Classes in HTML: {}", html_classes.len());

    // Generate computed/optical styles CSS
    let (computed_css, optical_css) = if let Some(ref optical) = optical_result {
        (String::new(), generate_optical_css(optical))
    } else {
        (generate_computed_styles_css(&computed_styles), String::new())
    };

    // Combine all CSS
    let mut final_css = format_result.css;
    if !optical_css.is_empty() {
        final_css.push_str("\n\n");
        final_css.push_str(&optical_css);
    } else if !computed_css.is_empty() {
        final_css.push_str("\n\n/* === Computed Styles (from browser) === */\n");
        final_css.push_str(&computed_css);
    }

    // SVG extraction
    println!("\n7. Extracting SVGs...");
    asset_registry.extract_svgs(&html);
    println!("   Total SVGs: {}", asset_registry.stats.total_svgs_found);
    println!("   Unique: {}", asset_registry.stats.unique_svgs);
    println!("   Icons: {}", asset_registry.stats.icons_found);
    println!("   Logos: {}", asset_registry.stats.logos_found);

    // Library detection
    println!("\n8. Analyzing...");
    let script_urls = extract_script_urls(&html, &base_url);
    let lib_result = LibMicroparser::parse(&script_urls, "");
    println!("   Libraries: {}", lib_result.libraries.len());
    for lib in &lib_result.libraries {
        println!("    - {} ({})", lib.name, lib.category);
    }

    let css_parsed = CssMicroparser::parse(&final_css);
    println!("   CSS Variables: {}", css_parsed.variables.len());
    println!("   Keyframes: {}", css_parsed.keyframes.len());

    // Optimize HTML
    println!("\n9. Optimizing & saving...");
    let html_optimized = HtmlOptimizer::optimize(&html, &HtmlOptimizeOptions {
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

    println!("   HTML: {} -> {} bytes", html.len(), html_optimized.stats.optimized_size);
    println!("   CSS: {} bytes (formatted)", final_css.len());

    // Save files
    let clean_html = generate_clean_html(&html_optimized.html, &title, &final_css);
    fs::write(output.join("index.html"), &clean_html)?;
    println!("   + index.html");

    fs::write(data_dir.join("raw.html"), &html)?;
    println!("   + data/raw.html");

    fs::write(output.join("styles.css"), &final_css)?;
    println!("   + styles.css");

    // Save computed/optical styles separately
    if let Some(ref optical) = optical_result {
        let optical_json = serde_json::to_string_pretty(optical)?;
        fs::write(data_dir.join("optical_capture.json"), &optical_json)?;
        println!("   + data/optical_capture.json ({} groups)", optical.stats.groups);
    } else if !computed_styles.is_empty() {
        let computed_json = serde_json::to_string_pretty(&computed_styles)?;
        fs::write(data_dir.join("computed_styles.json"), &computed_json)?;
        println!("   + data/computed_styles.json");
    }

    // Metadata
    let metadata = serde_json::json!({
        "url": url,
        "domain": base_url.host_str(),
        "title": title,
        "mode": "browser",
        "extracted_at": chrono::Utc::now().to_rfc3339(),
        "options": {
            "viewport": format!("{}x{}", options.viewport_width, options.viewport_height),
            "wait_after_load_ms": options.wait_after_load,
        },
        "stats": {
            "html_bytes": html.len(),
            "html_optimized_bytes": html_optimized.stats.optimized_size,
            "css_bytes": final_css.len(),
            "optical_mode": options.optical_mode,
            "optical_elements_total": optical_result.as_ref().map(|o| o.stats.total).unwrap_or(0),
            "optical_elements_visible": optical_result.as_ref().map(|o| o.stats.visible).unwrap_or(0),
            "optical_style_groups": optical_result.as_ref().map(|o| o.stats.groups).unwrap_or(0),
            "computed_styles_elements": computed_styles.len(),
            "html_classes": html_classes.len(),
            "svgs_found": asset_registry.stats.total_svgs_found,
            "svgs_unique": asset_registry.stats.unique_svgs,
        },
        "css_format_stats": {
            "rules": format_result.stats.rules_formatted,
            "keyframes": format_result.stats.keyframes_formatted,
            "media_queries": format_result.stats.media_queries_formatted,
            "font_faces": format_result.stats.font_faces_formatted,
            "variables": format_result.stats.variables_found,
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

    // Classes JSON
    fs::write(data_dir.join("classes.json"), serde_json::to_string_pretty(&html_classes)?)?;
    println!("   + data/classes.json");

    // Project.toml
    if project_toml {
        let libraries: Vec<(String, String)> = lib_result.libraries
            .iter()
            .map(|l| (l.name.clone(), l.category.clone()))
            .collect();
        let css_stats = CssStats {
            variables: css_parsed.variables.len(),
            keyframes: css_parsed.keyframes.len(),
            colors: css_parsed.colors.len(),
        };
        let toml_content = asset_registry.generate_toml(url, &title, "browser", &libraries, &css_stats);
        fs::write(output.join("project.toml"), &toml_content)?;
        println!("   + project.toml");
    }

    println!("\n✓ Done! Output: {:?}", output);
    Ok(())
}

/// Unescape JavaScript string
fn unescape_js_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('u') => {
                    // Unicode escape \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Get all stylesheets using JavaScript
fn get_stylesheets_js(tab: &headless_chrome::Tab) -> Result<String, Box<dyn std::error::Error>> {
    let js = r#"
        (function() {
            var css = '';

            // Get all stylesheets
            for (var i = 0; i < document.styleSheets.length; i++) {
                try {
                    var sheet = document.styleSheets[i];
                    var source = sheet.href || 'inline';
                    css += '\n/* Source: ' + source + ' */\n';

                    var rules = sheet.cssRules || sheet.rules;
                    for (var j = 0; j < rules.length; j++) {
                        css += rules[j].cssText + '\n';
                    }
                    css += '\n';
                } catch (e) {
                    // Cross-origin stylesheet, skip
                    css += '/* Cross-origin stylesheet: ' + (document.styleSheets[i].href || 'unknown') + ' */\n';
                }
            }

            // Get all <style> tags content
            var styles = document.querySelectorAll('style');
            for (var k = 0; k < styles.length; k++) {
                css += '\n/* Inline style tag #' + k + ' */\n';
                css += styles[k].textContent + '\n';
            }

            return css;
        })()
    "#;

    let result = tab.evaluate(js, false)?;

    if let Some(value) = result.value {
        let s = value.to_string();
        if s.starts_with('"') && s.ends_with('"') {
            return Ok(unescape_js_string(&s[1..s.len()-1]));
        }
        return Ok(s);
    }

    Ok(String::new())
}

/// Get computed styles for important elements using JavaScript
fn get_computed_styles_js(tab: &headless_chrome::Tab) -> Result<HashMap<String, HashMap<String, String>>, Box<dyn std::error::Error>> {
    let js = r#"
        (function() {
            var result = {};
            var selectors = ['body', 'header', 'nav', 'main', 'footer', 'h1', 'h2', 'h3', 'p', 'a', 'button'];
            var props = ['display', 'position', 'width', 'height', 'margin', 'padding',
                        'color', 'background-color', 'font-family', 'font-size', 'font-weight',
                        'border', 'border-radius', 'box-shadow', 'flex', 'gap'];

            selectors.forEach(function(sel) {
                var el = document.querySelector(sel);
                if (el) {
                    var computed = window.getComputedStyle(el);
                    var styles = {};
                    props.forEach(function(prop) {
                        var val = computed.getPropertyValue(prop);
                        if (val && val !== 'none' && val !== 'normal' && val !== '0px') {
                            styles[prop] = val;
                        }
                    });
                    if (Object.keys(styles).length > 0) {
                        result[sel] = styles;
                    }
                }
            });

            return JSON.stringify(result);
        })()
    "#;

    let result = tab.evaluate(js, false)?;

    if let Some(value) = result.value {
        let s = value.to_string();
        let json_str = if s.starts_with('"') && s.ends_with('"') {
            unescape_js_string(&s[1..s.len()-1])
        } else {
            s
        };

        if let Ok(parsed) = serde_json::from_str::<HashMap<String, HashMap<String, String>>>(&json_str) {
            return Ok(parsed);
        }
    }

    Ok(HashMap::new())
}

/// Optical capture - extracts computed styles for ALL visible elements
/// This is the core of the "visual" capture mode
fn get_optical_styles_js(tab: &headless_chrome::Tab) -> Result<OpticalCaptureResult, Box<dyn std::error::Error>> {
    let js = r#"
        (function() {
            // All CSS properties we care about for visual reproduction
            const visualProps = [
                // Layout
                'display', 'position', 'top', 'right', 'bottom', 'left',
                'width', 'height', 'min-width', 'max-width', 'min-height', 'max-height',
                'margin', 'margin-top', 'margin-right', 'margin-bottom', 'margin-left',
                'padding', 'padding-top', 'padding-right', 'padding-bottom', 'padding-left',
                'box-sizing', 'overflow', 'overflow-x', 'overflow-y',
                // Flexbox
                'flex', 'flex-direction', 'flex-wrap', 'flex-grow', 'flex-shrink', 'flex-basis',
                'justify-content', 'align-items', 'align-content', 'align-self', 'gap', 'row-gap', 'column-gap',
                // Grid
                'grid-template-columns', 'grid-template-rows', 'grid-column', 'grid-row',
                'grid-gap', 'grid-auto-flow',
                // Typography
                'font-family', 'font-size', 'font-weight', 'font-style', 'font-variant',
                'line-height', 'letter-spacing', 'text-align', 'text-decoration', 'text-transform',
                'white-space', 'word-break', 'word-wrap',
                // Colors & Background
                'color', 'background', 'background-color', 'background-image', 'background-size',
                'background-position', 'background-repeat',
                // Borders
                'border', 'border-width', 'border-style', 'border-color', 'border-radius',
                'border-top', 'border-right', 'border-bottom', 'border-left',
                // Effects
                'box-shadow', 'opacity', 'visibility', 'z-index',
                'transform', 'transition', 'filter', 'backdrop-filter',
                // SVG
                'fill', 'stroke', 'stroke-width'
            ];

            // Default values to skip (browser defaults)
            const defaults = {
                'display': 'block',
                'position': 'static',
                'top': 'auto', 'right': 'auto', 'bottom': 'auto', 'left': 'auto',
                'margin': '0px', 'margin-top': '0px', 'margin-right': '0px', 'margin-bottom': '0px', 'margin-left': '0px',
                'padding': '0px', 'padding-top': '0px', 'padding-right': '0px', 'padding-bottom': '0px', 'padding-left': '0px',
                'border': '0px none rgb(0, 0, 0)', 'border-width': '0px', 'border-style': 'none',
                'border-radius': '0px',
                'box-shadow': 'none',
                'opacity': '1',
                'visibility': 'visible',
                'z-index': 'auto',
                'transform': 'none',
                'transition': 'all 0s ease 0s',
                'filter': 'none',
                'backdrop-filter': 'none',
                'background-image': 'none',
                'flex': '0 1 auto',
                'flex-grow': '0', 'flex-shrink': '1',
                'gap': 'normal', 'row-gap': 'normal', 'column-gap': 'normal'
            };

            const result = {
                elements: [],
                styleGroups: {},
                stats: { total: 0, visible: 0, groups: 0 }
            };

            // Check if element is visible
            function isVisible(el) {
                if (!el.offsetParent && el.tagName !== 'BODY' && el.tagName !== 'HTML') return false;
                const style = window.getComputedStyle(el);
                if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                const rect = el.getBoundingClientRect();
                if (rect.width === 0 && rect.height === 0) return false;
                return true;
            }

            // Get unique selector for element
            function getSelector(el) {
                if (el.id) return '#' + el.id;

                let selector = el.tagName.toLowerCase();
                if (el.className && typeof el.className === 'string') {
                    const classes = el.className.trim().split(/\s+/).filter(c => c && !c.includes('__'));
                    if (classes.length > 0) {
                        selector += '.' + classes.slice(0, 2).join('.');
                    }
                }

                // Add nth-child for uniqueness
                const parent = el.parentElement;
                if (parent) {
                    const siblings = Array.from(parent.children).filter(c => c.tagName === el.tagName);
                    if (siblings.length > 1) {
                        const index = siblings.indexOf(el) + 1;
                        selector += ':nth-child(' + index + ')';
                    }
                }

                return selector;
            }

            // Extract non-default styles
            function extractStyles(el) {
                const computed = window.getComputedStyle(el);
                const styles = {};

                for (const prop of visualProps) {
                    const value = computed.getPropertyValue(prop);
                    if (!value || value === '' || value === defaults[prop]) continue;
                    if (value === 'none' || value === 'normal' || value === 'auto') continue;
                    if (value === '0px' || value === '0' || value === '0px 0px 0px 0px') continue;
                    if (value === 'rgba(0, 0, 0, 0)' || value === 'transparent') continue;

                    styles[prop] = value;
                }

                return styles;
            }

            // Hash styles for grouping
            function hashStyles(styles) {
                const keys = Object.keys(styles).sort();
                return keys.map(k => k + ':' + styles[k]).join(';');
            }

            // Process all elements
            const allElements = document.querySelectorAll('*');
            const styleHashes = {};

            for (const el of allElements) {
                result.stats.total++;

                if (!isVisible(el)) continue;
                result.stats.visible++;

                const styles = extractStyles(el);
                if (Object.keys(styles).length === 0) continue;

                const hash = hashStyles(styles);
                const selector = getSelector(el);

                if (!styleHashes[hash]) {
                    const groupId = 'g' + Object.keys(styleHashes).length;
                    styleHashes[hash] = {
                        id: groupId,
                        styles: styles,
                        selectors: []
                    };
                }

                styleHashes[hash].selectors.push(selector);
                result.elements.push({
                    selector: selector,
                    tag: el.tagName.toLowerCase(),
                    groupId: styleHashes[hash].id
                });
            }

            // Convert to groups
            for (const hash in styleHashes) {
                const group = styleHashes[hash];
                result.styleGroups[group.id] = {
                    selectors: group.selectors,
                    styles: group.styles
                };
                result.stats.groups++;
            }

            return JSON.stringify(result);
        })()
    "#;

    let result = tab.evaluate(js, false)?;

    if let Some(value) = result.value {
        let s = value.to_string();
        let json_str = if s.starts_with('"') && s.ends_with('"') {
            unescape_js_string(&s[1..s.len()-1])
        } else {
            s
        };

        if let Ok(parsed) = serde_json::from_str::<OpticalCaptureResult>(&json_str) {
            return Ok(parsed);
        }
    }

    Ok(OpticalCaptureResult::default())
}

/// Result of optical capture
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct OpticalCaptureResult {
    pub elements: Vec<CapturedElement>,
    #[serde(rename = "styleGroups")]
    pub style_groups: HashMap<String, StyleGroup>,
    pub stats: OpticalStats,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct CapturedElement {
    pub selector: String,
    pub tag: String,
    #[serde(rename = "groupId")]
    pub group_id: String,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct StyleGroup {
    pub selectors: Vec<String>,
    pub styles: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct OpticalStats {
    pub total: usize,
    pub visible: usize,
    pub groups: usize,
}

/// Generate optimized CSS from optical capture
fn generate_optical_css(optical: &OpticalCaptureResult) -> String {
    let mut css = String::new();

    css.push_str("/* =========================================\n");
    css.push_str("   Optical Capture - Computed Styles\n");
    css.push_str(&format!("   Elements: {} visible / {} total\n", optical.stats.visible, optical.stats.total));
    css.push_str(&format!("   Style groups: {} (deduplicated)\n", optical.stats.groups));
    css.push_str("   ========================================= */\n\n");

    // Sort groups by number of selectors (most common first)
    let mut groups: Vec<_> = optical.style_groups.iter().collect();
    groups.sort_by(|a, b| b.1.selectors.len().cmp(&a.1.selectors.len()));

    for (group_id, group) in groups {
        if group.styles.is_empty() || group.selectors.is_empty() {
            continue;
        }

        // Comment showing group info
        if group.selectors.len() > 3 {
            css.push_str(&format!("/* {} - {} elements */\n", group_id, group.selectors.len()));
        }

        // Combine selectors (max 5 per line for readability)
        let selector_str = if group.selectors.len() > 5 {
            let first_five: Vec<_> = group.selectors.iter().take(5).cloned().collect();
            format!("{} /* +{} more */", first_five.join(",\n"), group.selectors.len() - 5)
        } else {
            group.selectors.join(",\n")
        };

        css.push_str(&selector_str);
        css.push_str(" {\n");

        // Sort properties by category
        let mut props: Vec<_> = group.styles.iter().collect();
        props.sort_by(|a, b| {
            let order_a = property_order(a.0);
            let order_b = property_order(b.0);
            order_a.cmp(&order_b)
        });

        for (prop, value) in props {
            css.push_str(&format!("  {}: {};\n", prop, value));
        }

        css.push_str("}\n\n");
    }

    css
}

/// Property ordering for readable CSS output
fn property_order(prop: &str) -> usize {
    match prop {
        // Layout
        p if p.starts_with("display") => 0,
        p if p.starts_with("position") => 1,
        p if p == "top" || p == "right" || p == "bottom" || p == "left" => 2,
        p if p.starts_with("width") || p.starts_with("height") => 3,
        p if p.starts_with("min-") || p.starts_with("max-") => 4,
        // Box model
        p if p.starts_with("margin") => 10,
        p if p.starts_with("padding") => 11,
        p if p.starts_with("border") => 12,
        // Flexbox/Grid
        p if p.starts_with("flex") => 20,
        p if p.starts_with("grid") => 21,
        p if p.starts_with("justify") || p.starts_with("align") => 22,
        p if p.starts_with("gap") => 23,
        // Typography
        p if p.starts_with("font") => 30,
        p if p.starts_with("line-height") || p.starts_with("letter-spacing") => 31,
        p if p.starts_with("text") => 32,
        // Colors
        p if p == "color" => 40,
        p if p.starts_with("background") => 41,
        // Effects
        p if p.starts_with("box-shadow") => 50,
        p if p.starts_with("opacity") => 51,
        p if p.starts_with("transform") => 52,
        _ => 100,
    }
}

/// Generate CSS from computed styles
fn generate_computed_styles_css(computed: &HashMap<String, HashMap<String, String>>) -> String {
    let mut css = String::new();

    for (selector, styles) in computed {
        if styles.is_empty() {
            continue;
        }

        css.push_str(selector);
        css.push_str(" {\n");

        for (prop, value) in styles {
            css.push_str(&format!("  {}: {};\n", prop, value));
        }

        css.push_str("}\n\n");
    }

    css
}

/// Extract classes from HTML
fn extract_classes_from_html(html: &str) -> Vec<String> {
    let mut classes = Vec::new();
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

/// Extract script URLs from HTML
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

/// Generate clean HTML with embedded CSS
fn generate_clean_html(html: &str, title: &str, css: &str) -> String {
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
