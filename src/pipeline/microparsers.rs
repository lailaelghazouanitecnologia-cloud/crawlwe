//! Microparsers - Specialized fast parsers for specific content types
//!
//! Each microparser is optimized for a specific task:
//! - CssMicroparser: CSS parsing (variables, keyframes, selectors, colors)
//! - JsMicroparser: JS parsing (imports, exports, functions, libraries)
//! - LibMicroparser: Library detection from URLs and code
//! - HtmlMicroparser: HTML structure parsing

use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

// ============================================================================
// CSS MICROPARSER
// ============================================================================

lazy_static! {
    // CSS patterns
    static ref CSS_VAR_DEF: Regex = Regex::new(r"--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);").unwrap();
    static ref CSS_VAR_USE: Regex = Regex::new(r"var\s*\(\s*--([a-zA-Z0-9_-]+)").unwrap();
    static ref CSS_KEYFRAMES: Regex = Regex::new(r"@keyframes\s+([a-zA-Z0-9_-]+)\s*\{").unwrap();
    static ref CSS_ANIMATION: Regex = Regex::new(r"animation(?:-name)?\s*:\s*([^;,\s]+)").unwrap();
    static ref CSS_COLOR_HEX: Regex = Regex::new(r"#([0-9a-fA-F]{3,8})\b").unwrap();
    static ref CSS_COLOR_RGB: Regex = Regex::new(r"rgba?\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)").unwrap();
    static ref CSS_COLOR_HSL: Regex = Regex::new(r"hsla?\s*\(\s*(\d+)\s*,\s*(\d+)%?\s*,\s*(\d+)%?").unwrap();
    static ref CSS_FONT_FAMILY: Regex = Regex::new(r#"font-family\s*:\s*([^;]+)"#).unwrap();
    static ref CSS_IMPORT: Regex = Regex::new(r#"@import\s+(?:url\s*\(\s*)?["']([^"']+)["']"#).unwrap();
    static ref CSS_MEDIA: Regex = Regex::new(r"@media\s+([^{]+)\{").unwrap();
    static ref CSS_SELECTOR: Regex = Regex::new(r"([.#]?[a-zA-Z_][a-zA-Z0-9_-]*(?:\[[^\]]+\])?(?::[\w-]+(?:\([^)]*\))?)*)\s*\{").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssVariable {
    pub name: String,
    pub value: String,
    pub usages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssKeyframe {
    pub name: String,
    pub used_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssColor {
    pub original: String,
    pub normalized: String,  // Always hex
    pub usages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CssParseResult {
    pub variables: Vec<CssVariable>,
    pub keyframes: Vec<CssKeyframe>,
    pub colors: Vec<CssColor>,
    pub fonts: Vec<String>,
    pub imports: Vec<String>,
    pub media_queries: Vec<String>,
    pub selectors: Vec<String>,
    pub stats: CssStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CssStats {
    pub total_rules: usize,
    pub total_selectors: usize,
    pub total_properties: usize,
    pub has_variables: bool,
    pub has_animations: bool,
    pub has_media_queries: bool,
}

pub struct CssMicroparser;

impl CssMicroparser {
    pub fn parse(css: &str) -> CssParseResult {
        let mut result = CssParseResult::default();

        // Variables
        let mut var_usage: HashMap<String, usize> = HashMap::new();
        for cap in CSS_VAR_USE.captures_iter(css) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            *var_usage.entry(name.to_string()).or_default() += 1;
        }

        for cap in CSS_VAR_DEF.captures_iter(css) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            result.variables.push(CssVariable {
                name: format!("--{}", name),
                value: value.to_string(),
                usages: var_usage.get(name).copied().unwrap_or(0),
            });
        }
        result.stats.has_variables = !result.variables.is_empty();

        // Keyframes
        let mut animation_usages: HashMap<String, Vec<String>> = HashMap::new();
        for cap in CSS_ANIMATION.captures_iter(css) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            animation_usages.entry(name.to_string()).or_default();
        }

        for cap in CSS_KEYFRAMES.captures_iter(css) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            result.keyframes.push(CssKeyframe {
                name: name.to_string(),
                used_by: animation_usages.get(name).cloned().unwrap_or_default(),
            });
        }
        result.stats.has_animations = !result.keyframes.is_empty();

        // Colors
        let mut color_counts: HashMap<String, usize> = HashMap::new();
        for cap in CSS_COLOR_HEX.captures_iter(css) {
            let hex = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            *color_counts.entry(hex.to_lowercase()).or_default() += 1;
        }
        for cap in CSS_COLOR_RGB.captures_iter(css) {
            let full = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            *color_counts.entry(full.to_string()).or_default() += 1;
        }
        for (color, count) in color_counts {
            result.colors.push(CssColor {
                original: color.clone(),
                normalized: normalize_color(&color),
                usages: count,
            });
        }

        // Fonts
        let mut seen_fonts = std::collections::HashSet::new();
        for cap in CSS_FONT_FAMILY.captures_iter(css) {
            let fonts_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            for font in fonts_str.split(',') {
                let font = font.trim().trim_matches('"').trim_matches('\'');
                if !font.is_empty() && !is_generic_font(font) && seen_fonts.insert(font.to_string()) {
                    result.fonts.push(font.to_string());
                }
            }
        }

        // Imports
        for cap in CSS_IMPORT.captures_iter(css) {
            if let Some(url) = cap.get(1) {
                result.imports.push(url.as_str().to_string());
            }
        }

        // Media queries
        for cap in CSS_MEDIA.captures_iter(css) {
            if let Some(query) = cap.get(1) {
                result.media_queries.push(query.as_str().trim().to_string());
            }
        }
        result.stats.has_media_queries = !result.media_queries.is_empty();

        // Selectors
        for cap in CSS_SELECTOR.captures_iter(css) {
            if let Some(sel) = cap.get(1) {
                result.selectors.push(sel.as_str().to_string());
            }
        }
        result.stats.total_selectors = result.selectors.len();

        result
    }
}

fn normalize_color(color: &str) -> String {
    if color.starts_with('#') {
        let hex = &color[1..];
        if hex.len() == 3 {
            // Expand #RGB to #RRGGBB
            let chars: Vec<char> = hex.chars().collect();
            return format!("#{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]);
        }
    }
    color.to_lowercase()
}

fn is_generic_font(font: &str) -> bool {
    matches!(font.to_lowercase().as_str(),
        "serif" | "sans-serif" | "monospace" | "cursive" | "fantasy" |
        "system-ui" | "ui-serif" | "ui-sans-serif" | "ui-monospace" | "ui-rounded" |
        "inherit" | "initial" | "unset"
    )
}

// ============================================================================
// JS MICROPARSER
// ============================================================================

lazy_static! {
    static ref JS_IMPORT_ES6: Regex = Regex::new(r#"import\s+(?:\{[^}]+\}|\*\s+as\s+\w+|\w+)\s+from\s+["']([^"']+)["']"#).unwrap();
    static ref JS_IMPORT_DYNAMIC: Regex = Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
    static ref JS_REQUIRE: Regex = Regex::new(r#"require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
    static ref JS_EXPORT: Regex = Regex::new(r"export\s+(?:(default)\s+)?(?:(const|let|var|function|class)\s+)?(\w+)").unwrap();
    static ref JS_FUNC_DEF: Regex = Regex::new(r"(?:function\s+(\w+)|(\w+)\s*=\s*(?:async\s+)?(?:function|\([^)]*\)\s*=>))").unwrap();
    static ref JS_CLASS_DEF: Regex = Regex::new(r"class\s+(\w+)(?:\s+extends\s+(\w+))?").unwrap();
    static ref JS_ASYNC_AWAIT: Regex = Regex::new(r"\b(async|await)\b").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsImport {
    pub source: String,
    pub is_dynamic: bool,
    pub is_require: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsExport {
    pub name: String,
    pub kind: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsFunction {
    pub name: String,
    pub is_async: bool,
    pub is_arrow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsClass {
    pub name: String,
    pub extends: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsParseResult {
    pub imports: Vec<JsImport>,
    pub exports: Vec<JsExport>,
    pub functions: Vec<JsFunction>,
    pub classes: Vec<JsClass>,
    pub stats: JsStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsStats {
    pub uses_esm: bool,
    pub uses_cjs: bool,
    pub uses_async: bool,
    pub total_imports: usize,
    pub total_exports: usize,
}

pub struct JsMicroparser;

impl JsMicroparser {
    pub fn parse(js: &str) -> JsParseResult {
        let mut result = JsParseResult::default();

        // ES6 imports
        for cap in JS_IMPORT_ES6.captures_iter(js) {
            if let Some(src) = cap.get(1) {
                result.imports.push(JsImport {
                    source: src.as_str().to_string(),
                    is_dynamic: false,
                    is_require: false,
                });
                result.stats.uses_esm = true;
            }
        }

        // Dynamic imports
        for cap in JS_IMPORT_DYNAMIC.captures_iter(js) {
            if let Some(src) = cap.get(1) {
                result.imports.push(JsImport {
                    source: src.as_str().to_string(),
                    is_dynamic: true,
                    is_require: false,
                });
            }
        }

        // Require
        for cap in JS_REQUIRE.captures_iter(js) {
            if let Some(src) = cap.get(1) {
                result.imports.push(JsImport {
                    source: src.as_str().to_string(),
                    is_dynamic: false,
                    is_require: true,
                });
                result.stats.uses_cjs = true;
            }
        }
        result.stats.total_imports = result.imports.len();

        // Exports
        for cap in JS_EXPORT.captures_iter(js) {
            let is_default = cap.get(1).is_some();
            let kind = cap.get(2).map(|m| m.as_str()).unwrap_or("unknown");
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            result.exports.push(JsExport {
                name: name.to_string(),
                kind: kind.to_string(),
                is_default,
            });
        }
        result.stats.total_exports = result.exports.len();

        // Functions
        for cap in JS_FUNC_DEF.captures_iter(js) {
            let name = cap.get(1).or(cap.get(2)).map(|m| m.as_str()).unwrap_or("");
            if !name.is_empty() {
                result.functions.push(JsFunction {
                    name: name.to_string(),
                    is_async: false, // Would need more context to determine
                    is_arrow: cap.get(2).is_some(),
                });
            }
        }

        // Classes
        for cap in JS_CLASS_DEF.captures_iter(js) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let extends = cap.get(2).map(|m| m.as_str().to_string());
            result.classes.push(JsClass {
                name: name.to_string(),
                extends,
            });
        }

        // Async/await detection
        result.stats.uses_async = JS_ASYNC_AWAIT.is_match(js);

        result
    }
}

// ============================================================================
// LIBRARY MICROPARSER
// ============================================================================

lazy_static! {
    static ref LIB_PATTERNS: Vec<LibPattern> = vec![
        // Animation
        LibPattern::new("gsap", &["gsap", "greensock", "TweenMax", "TweenLite", "TimelineMax"], "animation"),
        LibPattern::new("anime.js", &["anime", "animejs"], "animation"),
        LibPattern::new("lottie", &["lottie", "lottie-web", "bodymovin"], "animation"),
        LibPattern::new("framer-motion", &["framer-motion", "motion"], "animation"),
        LibPattern::new("popmotion", &["popmotion"], "animation"),
        LibPattern::new("velocity", &["velocity", "velocityjs"], "animation"),

        // 3D/Graphics
        LibPattern::new("three.js", &["three", "THREE"], "3d"),
        LibPattern::new("babylon.js", &["babylon", "BABYLON"], "3d"),
        LibPattern::new("pixi.js", &["pixi", "PIXI"], "2d"),
        LibPattern::new("p5.js", &["p5"], "2d"),
        LibPattern::new("fabric.js", &["fabric"], "2d"),
        LibPattern::new("paper.js", &["paper"], "2d"),

        // UI Frameworks
        LibPattern::new("react", &["react", "React", "ReactDOM"], "ui"),
        LibPattern::new("vue", &["vue", "Vue"], "ui"),
        LibPattern::new("angular", &["angular", "@angular"], "ui"),
        LibPattern::new("svelte", &["svelte"], "ui"),
        LibPattern::new("alpine.js", &["alpine", "Alpine"], "ui"),
        LibPattern::new("htmx", &["htmx"], "ui"),

        // CSS Frameworks
        LibPattern::new("tailwind", &["tailwind", "tailwindcss"], "css"),
        LibPattern::new("bootstrap", &["bootstrap"], "css"),
        LibPattern::new("bulma", &["bulma"], "css"),
        LibPattern::new("foundation", &["foundation"], "css"),

        // Utilities
        LibPattern::new("lodash", &["lodash", "_"], "utility"),
        LibPattern::new("jquery", &["jquery", "jQuery", "$"], "utility"),
        LibPattern::new("axios", &["axios"], "utility"),
        LibPattern::new("moment", &["moment"], "utility"),
        LibPattern::new("dayjs", &["dayjs"], "utility"),
        LibPattern::new("date-fns", &["date-fns", "dateFns"], "utility"),

        // Scroll
        LibPattern::new("locomotive-scroll", &["locomotive", "LocomotiveScroll"], "scroll"),
        LibPattern::new("scrollmagic", &["ScrollMagic"], "scroll"),
        LibPattern::new("aos", &["AOS"], "scroll"),
        LibPattern::new("scrollreveal", &["ScrollReveal"], "scroll"),
        LibPattern::new("lenis", &["lenis", "Lenis"], "scroll"),

        // State Management
        LibPattern::new("redux", &["redux", "Redux"], "state"),
        LibPattern::new("mobx", &["mobx", "MobX"], "state"),
        LibPattern::new("zustand", &["zustand"], "state"),
        LibPattern::new("recoil", &["recoil"], "state"),

        // Data Visualization
        LibPattern::new("d3", &["d3"], "viz"),
        LibPattern::new("chart.js", &["Chart"], "viz"),
        LibPattern::new("echarts", &["echarts"], "viz"),
        LibPattern::new("highcharts", &["Highcharts"], "viz"),
    ];

    static ref VERSION_PATTERN: Regex = Regex::new(r"[/@v](\d+\.\d+(?:\.\d+)?(?:-[\w.]+)?)").unwrap();
}

struct LibPattern {
    name: String,
    patterns: Vec<String>,
    category: String,
}

impl LibPattern {
    fn new(name: &str, patterns: &[&str], category: &str) -> Self {
        Self {
            name: name.to_string(),
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
            category: category.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLibrary {
    pub name: String,
    pub category: String,
    pub version: Option<String>,
    pub source: LibrarySource,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LibrarySource {
    Url(String),
    Code,
    WindowGlobal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibParseResult {
    pub libraries: Vec<DetectedLibrary>,
    pub by_category: HashMap<String, Vec<String>>,
}

pub struct LibMicroparser;

impl LibMicroparser {
    /// Detect libraries from URL
    pub fn from_url(url: &str) -> Option<DetectedLibrary> {
        let url_lower = url.to_lowercase();

        for lib in LIB_PATTERNS.iter() {
            for pattern in &lib.patterns {
                if url_lower.contains(&pattern.to_lowercase()) {
                    return Some(DetectedLibrary {
                        name: lib.name.clone(),
                        category: lib.category.clone(),
                        version: Self::extract_version(url),
                        source: LibrarySource::Url(url.to_string()),
                        confidence: 0.9,
                    });
                }
            }
        }

        // Unknown library - extract from filename
        if let Some(filename) = url.split('/').last() {
            let name = filename
                .split('.')
                .next()
                .unwrap_or(filename)
                .replace(".min", "")
                .replace("-min", "");

            if !name.is_empty() && name.len() > 2 {
                return Some(DetectedLibrary {
                    name,
                    category: "unknown".to_string(),
                    version: Self::extract_version(url),
                    source: LibrarySource::Url(url.to_string()),
                    confidence: 0.5,
                });
            }
        }

        None
    }

    /// Detect libraries from code content
    pub fn from_code(code: &str) -> Vec<DetectedLibrary> {
        let mut detected = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for lib in LIB_PATTERNS.iter() {
            for pattern in &lib.patterns {
                // Check for window.X or global X usage
                let window_pattern = format!(r"\bwindow\.{}\b", regex::escape(pattern));
                let global_pattern = format!(r"\b{}\b", regex::escape(pattern));

                if let Ok(re) = Regex::new(&window_pattern) {
                    if re.is_match(code) && seen.insert(lib.name.clone()) {
                        detected.push(DetectedLibrary {
                            name: lib.name.clone(),
                            category: lib.category.clone(),
                            version: None,
                            source: LibrarySource::Code,
                            confidence: 0.8,
                        });
                        break;
                    }
                }

                if let Ok(re) = Regex::new(&global_pattern) {
                    // Count matches for confidence
                    let count = re.find_iter(code).count();
                    if count >= 3 && seen.insert(lib.name.clone()) {
                        detected.push(DetectedLibrary {
                            name: lib.name.clone(),
                            category: lib.category.clone(),
                            version: None,
                            source: LibrarySource::Code,
                            confidence: (count as f32 / 10.0).min(0.9),
                        });
                        break;
                    }
                }
            }
        }

        detected
    }

    /// Parse multiple sources and deduplicate
    pub fn parse(urls: &[String], code: &str) -> LibParseResult {
        let mut result = LibParseResult::default();
        let mut seen = std::collections::HashSet::new();

        // From URLs
        for url in urls {
            if let Some(lib) = Self::from_url(url) {
                if seen.insert(lib.name.clone()) {
                    result.by_category
                        .entry(lib.category.clone())
                        .or_default()
                        .push(lib.name.clone());
                    result.libraries.push(lib);
                }
            }
        }

        // From code
        for lib in Self::from_code(code) {
            if seen.insert(lib.name.clone()) {
                result.by_category
                    .entry(lib.category.clone())
                    .or_default()
                    .push(lib.name.clone());
                result.libraries.push(lib);
            }
        }

        result
    }

    fn extract_version(url: &str) -> Option<String> {
        VERSION_PATTERN.captures(url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }
}

// ============================================================================
// HTML MICROPARSER
// ============================================================================

lazy_static! {
    static ref HTML_DOCTYPE: Regex = Regex::new(r"<!DOCTYPE\s+([^>]+)>").unwrap();
    static ref HTML_META: Regex = Regex::new(r#"<meta\s+([^>]+)>"#).unwrap();
    static ref HTML_TITLE: Regex = Regex::new(r"<title[^>]*>([^<]*)</title>").unwrap();
    static ref HTML_SCRIPT_SRC: Regex = Regex::new(r#"<script[^>]*\ssrc=["']([^"']+)["'][^>]*>"#).unwrap();
    static ref HTML_LINK_HREF: Regex = Regex::new(r#"<link[^>]*\shref=["']([^"']+)["'][^>]*>"#).unwrap();
    static ref HTML_IMG_SRC: Regex = Regex::new(r#"<img[^>]*\ssrc=["']([^"']+)["'][^>]*>"#).unwrap();
    static ref HTML_DATA_ATTR: Regex = Regex::new(r#"\s(data-[a-zA-Z0-9_-]+)=["']([^"']*)["']"#).unwrap();
    static ref HTML_CLASS_ATTR: Regex = Regex::new(r#"\sclass=["']([^"']+)["']"#).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HtmlParseResult {
    pub title: Option<String>,
    pub meta: HashMap<String, String>,
    pub scripts: Vec<HtmlScript>,
    pub stylesheets: Vec<String>,
    pub images: Vec<String>,
    pub data_attributes: HashMap<String, Vec<String>>,
    pub class_names: Vec<String>,
    pub stats: HtmlStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlScript {
    pub src: Option<String>,
    pub is_module: bool,
    pub is_async: bool,
    pub is_defer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HtmlStats {
    pub total_elements: usize,
    pub total_scripts: usize,
    pub total_stylesheets: usize,
    pub total_images: usize,
    pub has_viewport: bool,
    pub has_charset: bool,
}

pub struct HtmlMicroparser;

impl HtmlMicroparser {
    pub fn parse(html: &str) -> HtmlParseResult {
        let mut result = HtmlParseResult::default();

        // Title
        if let Some(cap) = HTML_TITLE.captures(html) {
            result.title = cap.get(1).map(|m| m.as_str().trim().to_string());
        }

        // Meta tags
        for cap in HTML_META.captures_iter(html) {
            if let Some(attrs) = cap.get(1) {
                let attrs_str = attrs.as_str();

                // Parse name/property and content
                let name = extract_attr(attrs_str, "name")
                    .or_else(|| extract_attr(attrs_str, "property"));
                let content = extract_attr(attrs_str, "content");

                if let (Some(n), Some(c)) = (name, content) {
                    if n == "viewport" { result.stats.has_viewport = true; }
                    if n == "charset" { result.stats.has_charset = true; }
                    result.meta.insert(n, c);
                }

                // Charset
                if let Some(charset) = extract_attr(attrs_str, "charset") {
                    result.meta.insert("charset".to_string(), charset);
                    result.stats.has_charset = true;
                }
            }
        }

        // Scripts
        for cap in HTML_SCRIPT_SRC.captures_iter(html) {
            let full_tag = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            result.scripts.push(HtmlScript {
                src: cap.get(1).map(|m| m.as_str().to_string()),
                is_module: full_tag.contains("type=\"module\"") || full_tag.contains("type='module'"),
                is_async: full_tag.contains("async"),
                is_defer: full_tag.contains("defer"),
            });
        }
        result.stats.total_scripts = result.scripts.len();

        // Stylesheets
        for cap in HTML_LINK_HREF.captures_iter(html) {
            let full_tag = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            if full_tag.contains("stylesheet") || full_tag.contains(".css") {
                if let Some(href) = cap.get(1) {
                    result.stylesheets.push(href.as_str().to_string());
                }
            }
        }
        result.stats.total_stylesheets = result.stylesheets.len();

        // Images
        for cap in HTML_IMG_SRC.captures_iter(html) {
            if let Some(src) = cap.get(1) {
                result.images.push(src.as_str().to_string());
            }
        }
        result.stats.total_images = result.images.len();

        // Data attributes
        for cap in HTML_DATA_ATTR.captures_iter(html) {
            let attr = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            result.data_attributes
                .entry(attr.to_string())
                .or_default()
                .push(value.to_string());
        }

        // Class names
        let mut seen_classes = std::collections::HashSet::new();
        for cap in HTML_CLASS_ATTR.captures_iter(html) {
            if let Some(classes) = cap.get(1) {
                for class in classes.as_str().split_whitespace() {
                    if seen_classes.insert(class.to_string()) {
                        result.class_names.push(class.to_string());
                    }
                }
            }
        }

        // Count elements (rough estimate)
        result.stats.total_elements = html.matches('<').count() / 2;

        result
    }
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let patterns = [
        format!(r#"{}=["']([^"']*)["']"#, attr),
        format!(r#"{}=(\S+)"#, attr),
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(cap) = re.captures(tag) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
    }
    None
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_microparser() {
        let css = r#"
            :root {
                --primary: #3b82f6;
                --secondary: rgb(100, 200, 150);
            }
            @keyframes fade {
                from { opacity: 0; }
                to { opacity: 1; }
            }
            .button {
                color: var(--primary);
                animation: fade 0.3s;
            }
        "#;

        let result = CssMicroparser::parse(css);
        assert!(!result.variables.is_empty());
        assert!(!result.keyframes.is_empty());
        assert!(!result.colors.is_empty());
    }

    #[test]
    fn test_js_microparser() {
        let js = r#"
            import React from 'react';
            import { useState } from 'react';

            export default function App() {
                const [count, setCount] = useState(0);
                return <div>{count}</div>;
            }
        "#;

        let result = JsMicroparser::parse(js);
        assert!(result.stats.uses_esm);
        assert!(!result.imports.is_empty());
        assert!(!result.exports.is_empty());
    }

    #[test]
    fn test_lib_detection() {
        let url = "https://cdn.jsdelivr.net/npm/gsap@3.12.2/dist/gsap.min.js";
        let lib = LibMicroparser::from_url(url).unwrap();
        assert_eq!(lib.name, "gsap");
        assert_eq!(lib.version, Some("3.12.2".to_string()));
    }
}
