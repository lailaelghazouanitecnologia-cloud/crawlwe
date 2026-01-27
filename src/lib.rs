//! CrawlWe Core - High-performance web page extractor
//!
//! This library provides the Rust core for the CrawlWe web scraper.
//! It handles: browser automation, HTML/CSS parsing, content analysis, and optimization.
//!
//! Also includes ZAD - a template metaprogramming language.
//!
//! Output structure:
//! - index.html (clean semantic HTML)
//! - styles.css (unified CSS)
//! - scripts/main.js (relevant JS only)
//! - project.toml (metadata and dependencies)
//!
//! Exposed to Python via PyO3.

pub mod analyzer;
pub mod models;
pub mod zad;
pub mod export;

// Re-export from subdirectories
pub mod capture {
    pub mod browser;
}

pub mod parser {
    pub mod css;
    pub mod html;
}

use analyzer::Analyzer;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::collections::HashMap;

// ============================================================================
// PYTHON CLASSES
// ============================================================================

/// Python-accessible result of page capture
#[pyclass]
#[derive(Debug, Clone)]
pub struct PageResult {
    #[pyo3(get)]
    pub url: String,
    #[pyo3(get)]
    pub title: Option<String>,
    #[pyo3(get)]
    pub html: String,
    #[pyo3(get)]
    pub css: String,
    #[pyo3(get)]
    pub screenshot: Option<String>,
    #[pyo3(get)]
    pub element_count: usize,
    #[pyo3(get)]
    pub rule_count: usize,
    #[pyo3(get)]
    pub variables: HashMap<String, String>,
    #[pyo3(get)]
    pub colors: Vec<String>,
    #[pyo3(get)]
    pub fonts: Vec<String>,
}

#[pymethods]
impl PageResult {
    fn __str__(&self) -> String {
        format!(
            "PageResult(url='{}', elements={}, rules={})",
            self.url, self.element_count, self.rule_count
        )
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ============================================================================
// PYTHON FUNCTIONS
// ============================================================================

/// Analyze web content (HTML, CSS, JS) and return detailed analysis
#[pyfunction]
#[pyo3(signature = (html, css, js=""))]
fn analyze_content(html: &str, css: &str, js: &str) -> PyResult<String> {
    let result = Analyzer::analyze(html, css, js);
    serde_json::to_string_pretty(&result)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Analyze CSS content only
#[pyfunction]
fn analyze_css(css: &str) -> PyResult<String> {
    let result = analyzer::CssAnalyzer::analyze(css);
    serde_json::to_string_pretty(&result)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Analyze JavaScript content only
#[pyfunction]
fn analyze_js(js: &str) -> PyResult<String> {
    let result = analyzer::JsAnalyzer::analyze(js);
    serde_json::to_string_pretty(&result)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Analyze HTML content only
#[pyfunction]
fn analyze_html(html: &str) -> PyResult<String> {
    let result = analyzer::HtmlAnalyzer::analyze(html);
    serde_json::to_string_pretty(&result)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Extract colors from CSS
#[pyfunction]
fn extract_css_colors(css: &str) -> Vec<String> {
    let analysis = analyzer::CssAnalyzer::analyze(css);
    analysis.colors
}

/// Extract fonts from CSS
#[pyfunction]
fn extract_css_fonts(css: &str) -> Vec<String> {
    let analysis = analyzer::CssAnalyzer::analyze(css);
    analysis.fonts
}

/// Extract CSS variables
#[pyfunction]
fn extract_css_variables(css: &str) -> HashMap<String, String> {
    let analysis = analyzer::CssAnalyzer::analyze(css);
    analysis.variables
}

/// Detect Tailwind CSS usage
#[pyfunction]
fn detect_tailwind(html: &str, css: &str) -> PyResult<String> {
    let result = analyzer::patterns::detect_tailwind(html, css);
    serde_json::to_string_pretty(&result)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Detect WebGL usage in JavaScript
#[pyfunction]
fn detect_webgl(js: &str) -> PyResult<String> {
    let analysis = analyzer::JsAnalyzer::analyze(js);
    serde_json::to_string_pretty(&analysis.webgl)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Detect WebGPU usage in JavaScript
#[pyfunction]
fn detect_webgpu(js: &str) -> PyResult<String> {
    let analysis = analyzer::JsAnalyzer::analyze(js);
    serde_json::to_string_pretty(&analysis.webgpu)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Detect animation libraries
#[pyfunction]
fn detect_animation_libraries(js: &str) -> PyResult<String> {
    let analysis = analyzer::JsAnalyzer::analyze(js);
    serde_json::to_string_pretty(&analysis.animation_libraries)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Detect UI frameworks (React, Vue, etc.)
#[pyfunction]
fn detect_ui_frameworks(js: &str) -> PyResult<String> {
    let analysis = analyzer::JsAnalyzer::analyze(js);
    serde_json::to_string_pretty(&analysis.ui_frameworks)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Get CDN URL for a CSS library
#[pyfunction]
#[pyo3(signature = (name, version=None))]
fn get_css_cdn(name: &str, version: Option<&str>) -> Option<String> {
    export::LibraryCDN::get_css(name, version)
}

/// Get CDN URL for a JS library
#[pyfunction]
#[pyo3(signature = (name, version=None))]
fn get_js_cdn(name: &str, version: Option<&str>) -> Option<String> {
    export::LibraryCDN::get_js(name, version)
}

/// Get Google Fonts URL
#[pyfunction]
#[pyo3(signature = (family, weights=None))]
fn get_font_url(family: &str, weights: Option<Vec<String>>) -> String {
    export::LibraryCDN::get_font_url(family, &weights.unwrap_or_default())
}

/// Generate project configuration based on analysis
#[pyfunction]
#[pyo3(signature = (url, html, css, js, title=None))]
fn generate_project_config(
    url: &str,
    html: &str,
    css: &str,
    js: &str,
    title: Option<String>,
) -> PyResult<String> {
    use chrono::Utc;

    let analysis = Analyzer::analyze(html, css, js);
    let css_analysis = analyzer::CssAnalyzer::analyze(css);
    let js_analysis = analyzer::JsAnalyzer::analyze(js);

    let mut config = export::ProjectConfig {
        meta: export::ProjectMeta {
            name: sanitize_name(title.as_deref().unwrap_or("captured-page")),
            version: "1.0.0".to_string(),
            description: title.clone(),
            captured_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            generator: "crawlwe".to_string(),
        },
        source: export::SourceInfo {
            url: url.to_string(),
            title,
            viewport: export::ViewportInfo {
                width: 1440,
                height: 900,
                device_scale: 1.0,
                device_type: "desktop".to_string(),
            },
            language: detect_language(html),
            charset: "UTF-8".to_string(),
        },
        technologies: export::Technologies::default(),
        dependencies: export::Dependencies::default(),
        export: export::ExportOptions::default(),
    };

    // Detect CSS framework
    if let Some(css_fw) = &analysis.frameworks.css_framework {
        if css_fw.name.to_lowercase().contains("tailwind") {
            config.technologies.css_framework = Some(export::CssFramework {
                name: "tailwind".to_string(),
                version: None,
                cdn_url: export::LibraryCDN::get_css("tailwind", None),
                inline: false,
            });
            config.dependencies.css.push(export::DependencyLink {
                name: "Tailwind CSS".to_string(),
                url: "https://cdn.tailwindcss.com".to_string(),
                version: None,
                integrity: None,
                crossorigin: None,
            });
        } else if css_fw.name.to_lowercase().contains("bootstrap") {
            config.technologies.css_framework = Some(export::CssFramework {
                name: "bootstrap".to_string(),
                version: Some("5".to_string()),
                cdn_url: export::LibraryCDN::get_css("bootstrap", None),
                inline: false,
            });
            if let Some(url) = export::LibraryCDN::get_css("bootstrap", None) {
                config.dependencies.css.push(export::DependencyLink {
                    name: "Bootstrap".to_string(),
                    url,
                    version: Some("5".to_string()),
                    integrity: None,
                    crossorigin: Some("anonymous".to_string()),
                });
            }
        }
    }

    // Detect animation libraries
    for lib in &js_analysis.animation_libraries {
        config.technologies.animation_libs.push(lib.name.clone());
        if let Some(url) = export::LibraryCDN::get_js(&lib.name, None) {
            config.dependencies.js.push(export::DependencyLink {
                name: lib.name.clone(),
                url,
                version: None,
                integrity: None,
                crossorigin: None,
            });
        }
    }

    // Detect 3D/graphics libraries
    if js_analysis.webgl.detected || js_analysis.webgpu.detected {
        if js.contains("THREE") || js.contains("three.js") {
            config.technologies.graphics_libs.push("three.js".to_string());
            if let Some(url) = export::LibraryCDN::get_js("three", None) {
                config.dependencies.js.push(export::DependencyLink {
                    name: "Three.js".to_string(),
                    url,
                    version: None,
                    integrity: None,
                    crossorigin: None,
                });
            }
        }
    }

    // Add fonts from CSS analysis
    for font in &css_analysis.fonts {
        config.dependencies.fonts.push(export::FontImport {
            family: font.clone(),
            weights: vec!["400".to_string(), "700".to_string()],
            source: "google".to_string(),
            url: Some(export::LibraryCDN::get_font_url(font, &["400".to_string(), "700".to_string()])),
        });
    }

    // Extract ALL external resources from HTML (catches everything we might have missed)
    let extracted_scripts = export::ResourceExtractor::extract_scripts(html);
    let extracted_styles = export::ResourceExtractor::extract_stylesheets(html);
    let extracted_fonts = export::ResourceExtractor::extract_fonts(html);

    // Merge extracted dependencies (avoids duplicates)
    export::merge_dependencies(&mut config.dependencies.js, extracted_scripts);
    export::merge_dependencies(&mut config.dependencies.css, extracted_styles);

    // Add extracted fonts (check for duplicates by family name)
    let existing_families: std::collections::HashSet<_> = config.dependencies.fonts
        .iter()
        .map(|f| f.family.to_lowercase())
        .collect();
    for font in extracted_fonts {
        if !existing_families.contains(&font.family.to_lowercase()) {
            config.dependencies.fonts.push(font);
        }
    }

    // Generate TOML
    let exporter = export::ProjectExporter::new(config);
    Ok(exporter.generate_project_toml())
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn detect_language(html: &str) -> Option<String> {
    if let Some(start) = html.find("lang=\"") {
        let rest = &html[start + 6..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    if let Some(start) = html.find("lang='") {
        let rest = &html[start + 6..];
        if let Some(end) = rest.find('\'') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

// ============================================================================
// PYTHON MODULE
// ============================================================================

/// CrawlWe Core Python Module
#[pymodule]
fn crawlwe_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Analysis functions
    m.add_function(wrap_pyfunction!(analyze_content, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_css, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_js, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_html, m)?)?;

    // Extraction functions
    m.add_function(wrap_pyfunction!(extract_css_colors, m)?)?;
    m.add_function(wrap_pyfunction!(extract_css_fonts, m)?)?;
    m.add_function(wrap_pyfunction!(extract_css_variables, m)?)?;

    // Detection functions
    m.add_function(wrap_pyfunction!(detect_tailwind, m)?)?;
    m.add_function(wrap_pyfunction!(detect_webgl, m)?)?;
    m.add_function(wrap_pyfunction!(detect_webgpu, m)?)?;
    m.add_function(wrap_pyfunction!(detect_animation_libraries, m)?)?;
    m.add_function(wrap_pyfunction!(detect_ui_frameworks, m)?)?;

    // Export functions
    m.add_function(wrap_pyfunction!(get_css_cdn, m)?)?;
    m.add_function(wrap_pyfunction!(get_js_cdn, m)?)?;
    m.add_function(wrap_pyfunction!(get_font_url, m)?)?;
    m.add_function(wrap_pyfunction!(generate_project_config, m)?)?;

    // Classes
    m.add_class::<PageResult>()?;

    Ok(())
}
