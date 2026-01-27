//! CrawlWe Core - High-performance web page extractor
//!
//! This library provides the Rust core for the CrawlWe web scraper.
//! It handles: browser automation, HTML/CSS parsing, content analysis, and optimization.
//!
//! Also includes ZAD - a template metaprogramming language.
//!
//! Exposed to Python via PyO3.

pub mod analyzer;
pub mod models;
pub mod zad;

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

    // Classes
    m.add_class::<PageResult>()?;

    Ok(())
}
