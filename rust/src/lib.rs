//! CrawlWe Core - High-performance web page extractor
//!
//! This library provides the Rust core for the CrawlWe web scraper.
//! It handles heavy lifting: HTTP requests, browser automation,
//! HTML/CSS parsing, and optimization.
//!
//! Exposed to Python via PyO3.

pub mod browser;
pub mod css_parser;
pub mod html_parser;
pub mod models;

use crate::browser::BrowserController;
use crate::css_parser::{CssAst, CssGenerator, CssOptimizer};
use crate::html_parser::{HtmlGenerator, HtmlNode, HtmlParser};
use crate::models::*;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

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

/// Capture a single page and return clean HTML + CSS
#[pyfunction]
#[pyo3(signature = (url, capture_states=true, capture_screenshot=true, viewport_width=1440, viewport_height=900))]
fn capture_page(
    url: &str,
    capture_states: bool,
    capture_screenshot: bool,
    viewport_width: u32,
    viewport_height: u32,
) -> PyResult<PageResult> {
    let rt = Runtime::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    rt.block_on(async {
        let config = CaptureConfig {
            viewports: vec![Viewport::new(viewport_width, viewport_height)],
            capture_hover: capture_states,
            capture_focus: capture_states,
            capture_active: false,
            capture_pseudo_elements: true,
            take_screenshots: capture_screenshot,
            download_fonts: false,
            download_images: false,
            max_elements: 2000,
            timeout_ms: 30000,
            user_agent: None,
        };

        let browser = BrowserController::launch(config)
            .await
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let captured = browser
            .capture_page(url)
            .await
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        browser
            .close()
            .await
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        // Extract design tokens
        let variables: HashMap<String, String> = captured
            .css_variables
            .iter()
            .map(|v| (v.name.clone(), v.value.clone()))
            .collect();

        let colors = extract_colors(&captured.css);
        let fonts = extract_fonts(&captured.css);

        Ok(PageResult {
            url: captured.url,
            title: captured.title,
            html: captured.html,
            css: captured.css,
            screenshot: captured.screenshot,
            element_count: captured.elements.len(),
            rule_count: captured.css_rules.len(),
            variables,
            colors,
            fonts,
        })
    })
}

/// Parse CSS text into optimized CSS
#[pyfunction]
#[pyo3(signature = (css_text, deduplicate=true, minify=false))]
fn parse_css(css_text: &str, deduplicate: bool, minify: bool) -> PyResult<String> {
    let ast = CssAst::parse(css_text).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let optimizer = CssOptimizer {
        deduplicate,
        merge_identical: deduplicate,
        sort_by_specificity: true,
        clean_important: false,
        minify_values: true,
        remove_unused: false,
    };

    let optimized = optimizer.optimize(ast);

    let generator = if minify {
        CssGenerator::minified()
    } else {
        CssGenerator::new()
    };

    Ok(generator.generate(&optimized))
}

/// Parse HTML and return clean version
#[pyfunction]
#[pyo3(signature = (html_text, stylesheet_path=None))]
fn parse_html(html_text: &str, stylesheet_path: Option<&str>) -> PyResult<String> {
    let parser = HtmlParser::new();
    let ast = parser
        .parse(html_text)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let generator = HtmlGenerator::new();
    Ok(generator.generate(&ast, stylesheet_path))
}

/// Extract colors used in CSS
#[pyfunction]
fn extract_css_colors(css_text: &str) -> Vec<String> {
    extract_colors(css_text)
}

/// Extract font families used in CSS
#[pyfunction]
fn extract_css_fonts(css_text: &str) -> Vec<String> {
    extract_fonts(css_text)
}

/// Merge multiple CSS texts into one optimized CSS
#[pyfunction]
fn merge_css(css_texts: Vec<String>) -> PyResult<String> {
    let mut combined_ast = CssAst::new();

    for css_text in css_texts {
        if let Ok(ast) = CssAst::parse(&css_text) {
            combined_ast.merge(ast);
        }
    }

    let optimizer = CssOptimizer::default();
    let optimized = optimizer.optimize(combined_ast);

    let generator = CssGenerator::new();
    Ok(generator.generate(&optimized))
}

/// Calculate CSS specificity for a selector
#[pyfunction]
fn calculate_specificity(selector: &str) -> (u32, u32, u32) {
    let spec = css_parser::CssAst::parse(&format!("{} {{}}", selector))
        .ok()
        .and_then(|ast| ast.rules.first().map(|r| r.specificity))
        .unwrap_or_default();

    (spec.ids, spec.classes, spec.elements)
}

/// Get info about HTML structure
#[pyfunction]
fn analyze_html(html_text: &str) -> PyResult<HashMap<String, PyObject>> {
    let parser = HtmlParser::new();
    let ast = parser
        .parse(html_text)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let info = html_parser::SemanticAnalyzer::analyze(&ast);

    Python::with_gil(|py| {
        let mut result = HashMap::new();

        result.insert("has_header".to_string(), info.has_header.into_py(py));
        result.insert("has_nav".to_string(), info.has_nav.into_py(py));
        result.insert("has_main".to_string(), info.has_main.into_py(py));
        result.insert("has_footer".to_string(), info.has_footer.into_py(py));
        result.insert("has_aside".to_string(), info.has_aside.into_py(py));
        result.insert("article_count".to_string(), info.article_count.into_py(py));
        result.insert("section_count".to_string(), info.section_count.into_py(py));
        result.insert("h1_count".to_string(), info.h1_count.into_py(py));
        result.insert("h2_count".to_string(), info.h2_count.into_py(py));
        result.insert("h3_count".to_string(), info.h3_count.into_py(py));
        result.insert("form_count".to_string(), info.form_count.into_py(py));
        result.insert("button_count".to_string(), info.button_count.into_py(py));
        result.insert("link_count".to_string(), info.link_count.into_py(py));
        result.insert("image_count".to_string(), info.image_count.into_py(py));
        result.insert(
            "uses_modern_layout".to_string(),
            info.uses_modern_layout.into_py(py),
        );
        result.insert("element_count".to_string(), ast.element_count().into_py(py));

        Ok(result)
    })
}

/// Helper: extract colors from CSS
fn extract_colors(css: &str) -> Vec<String> {
    use std::collections::HashSet;

    let mut colors = HashSet::new();

    // Hex colors
    for cap in regex_lite::Regex::new(r"#[0-9a-fA-F]{3,8}\b")
        .unwrap()
        .find_iter(css)
    {
        colors.insert(cap.as_str().to_lowercase());
    }

    // rgb/rgba
    for cap in regex_lite::Regex::new(r"rgba?\([^)]+\)")
        .unwrap()
        .find_iter(css)
    {
        colors.insert(cap.as_str().to_string());
    }

    // hsl/hsla
    for cap in regex_lite::Regex::new(r"hsla?\([^)]+\)")
        .unwrap()
        .find_iter(css)
    {
        colors.insert(cap.as_str().to_string());
    }

    // Named colors (common ones)
    let named = [
        "black", "white", "red", "green", "blue", "yellow", "orange", "purple", "pink", "gray",
        "grey", "transparent", "currentColor",
    ];

    for name in named {
        if css.contains(name) {
            colors.insert(name.to_string());
        }
    }

    let mut result: Vec<_> = colors.into_iter().collect();
    result.sort();
    result
}

/// Helper: extract fonts from CSS
fn extract_fonts(css: &str) -> Vec<String> {
    use std::collections::HashSet;

    let mut fonts = HashSet::new();

    // font-family declarations
    for cap in regex_lite::Regex::new(r#"font-family:\s*([^;]+)"#)
        .unwrap()
        .captures_iter(css)
    {
        if let Some(families) = cap.get(1) {
            for family in families.as_str().split(',') {
                let family = family
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                if !family.is_empty()
                    && !matches!(
                        family.to_lowercase().as_str(),
                        "sans-serif"
                            | "serif"
                            | "monospace"
                            | "cursive"
                            | "fantasy"
                            | "system-ui"
                            | "inherit"
                    )
                {
                    fonts.insert(family);
                }
            }
        }
    }

    let mut result: Vec<_> = fonts.into_iter().collect();
    result.sort();
    result
}

/// Python module definition
#[pymodule]
fn crawlwe_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(capture_page, m)?)?;
    m.add_function(wrap_pyfunction!(parse_css, m)?)?;
    m.add_function(wrap_pyfunction!(parse_html, m)?)?;
    m.add_function(wrap_pyfunction!(extract_css_colors, m)?)?;
    m.add_function(wrap_pyfunction!(extract_css_fonts, m)?)?;
    m.add_function(wrap_pyfunction!(merge_css, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_specificity, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_html, m)?)?;

    // Classes
    m.add_class::<PageResult>()?;
    m.add_class::<CssState>()?;
    m.add_class::<BoxModel>()?;
    m.add_class::<ComputedStyles>()?;
    m.add_class::<Specificity>()?;
    m.add_class::<CssRule>()?;
    m.add_class::<CapturedElement>()?;
    m.add_class::<Keyframes>()?;
    m.add_class::<KeyframeStep>()?;
    m.add_class::<Transition>()?;
    m.add_class::<MediaQuery>()?;
    m.add_class::<CssVariable>()?;
    m.add_class::<FontFace>()?;
    m.add_class::<Viewport>()?;
    m.add_class::<CapturedPage>()?;
    m.add_class::<CaptureConfig>()?;

    Ok(())
}
