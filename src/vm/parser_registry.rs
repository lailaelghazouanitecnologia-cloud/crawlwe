//! Parser Registry - Pluggable parser backends
//!
//! Like Bun's approach to multiple parsers: register different backends
//! and switch between them based on content type or performance needs.

use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

use super::opcodes::Value;
use super::vm::VMError;

lazy_static! {
    // CSS patterns
    static ref KEYFRAMES_RE: Regex = Regex::new(r"@keyframes\s+([a-zA-Z0-9_-]+)\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}").unwrap();
    static ref CSS_VAR_RE: Regex = Regex::new(r"--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);").unwrap();
    static ref COLOR_RE: Regex = Regex::new(r"#[0-9a-fA-F]{3,8}\b|rgba?\([^)]+\)|hsla?\([^)]+\)").unwrap();

    // HTML patterns
    static ref SCRIPT_TAG_RE: Regex = Regex::new(r#"<script[^>]*(?:src=["']([^"']+)["'])?[^>]*>([^<]*)</script>"#).unwrap();
    static ref LINK_CSS_RE: Regex = Regex::new(r#"<link[^>]*href=["']([^"']+\.css[^"']*)["'][^>]*>"#).unwrap();
    static ref STYLE_TAG_RE: Regex = Regex::new(r"<style[^>]*>([^<]*)</style>").unwrap();

    // Library detection patterns
    static ref LIB_PATTERNS: Vec<(&'static str, Regex)> = vec![
        ("gsap", Regex::new(r"gsap|greensock").unwrap()),
        ("three", Regex::new(r"three(?:\.min)?\.js").unwrap()),
        ("react", Regex::new(r"react(?:\.production)?(?:\.min)?\.js").unwrap()),
        ("vue", Regex::new(r"vue(?:\.runtime)?(?:\.min)?\.js").unwrap()),
        ("jquery", Regex::new(r"jquery(?:\.min)?\.js").unwrap()),
        ("anime", Regex::new(r"anime(?:\.min)?\.js").unwrap()),
        ("lottie", Regex::new(r"lottie(?:-web)?(?:\.min)?\.js").unwrap()),
        ("d3", Regex::new(r"d3(?:\.min)?\.js").unwrap()),
        ("tailwind", Regex::new(r"tailwind").unwrap()),
        ("bootstrap", Regex::new(r"bootstrap").unwrap()),
    ];
}

/// Parser backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParserKind {
    /// Fast regex-based parser (default)
    Regex,
    /// Tree-sitter based parser (AST)
    TreeSitter,
    /// HTML5 compliant parser
    Html5Ever,
    /// Lightning CSS parser
    LightningCss,
    /// Custom parser
    Custom,
}

/// A parser backend interface
pub trait ParserBackend: Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> ParserKind;
    fn parse(&self, source: &str) -> Result<Value, VMError>;
}

/// Regex-based fast parser (default)
pub struct RegexParser {
    patterns: HashMap<String, Regex>,
}

impl RegexParser {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }
}

impl ParserBackend for RegexParser {
    fn name(&self) -> &str { "regex" }
    fn kind(&self) -> ParserKind { ParserKind::Regex }

    fn parse(&self, source: &str) -> Result<Value, VMError> {
        Ok(Value::String(source.to_string()))
    }
}

/// Parser registry - manages multiple parser backends
pub struct ParserRegistry {
    parsers: HashMap<String, Box<dyn ParserBackend>>,
    current: String,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: HashMap::new(),
            current: "regex".to_string(),
        };

        // Register default parser
        registry.register(Box::new(RegexParser::new()));

        registry
    }

    /// Register a new parser backend
    pub fn register(&mut self, parser: Box<dyn ParserBackend>) {
        self.parsers.insert(parser.name().to_string(), parser);
    }

    /// Set the current parser
    pub fn set_current(&mut self, name: &str) {
        if self.parsers.contains_key(name) {
            self.current = name.to_string();
        }
    }

    /// Parse using a specific parser
    pub fn parse(&self, parser_name: &str, source: &str) -> Result<Value, VMError> {
        self.parsers.get(parser_name)
            .ok_or_else(|| VMError::ParserError(format!("unknown parser: {}", parser_name)))?
            .parse(source)
    }

    // ========================================
    // CSS Operations
    // ========================================

    /// Find @keyframes definitions
    pub fn css_find_keyframes(&self, css: &str) -> Result<Vec<Value>, VMError> {
        let mut keyframes = Vec::new();

        for cap in KEYFRAMES_RE.captures_iter(css) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut kf = HashMap::new();
            kf.insert("name".to_string(), Value::String(name.to_string()));
            kf.insert("body".to_string(), Value::String(body.to_string()));

            keyframes.push(Value::Map(kf));
        }

        Ok(keyframes)
    }

    /// Find CSS custom properties (variables)
    pub fn css_find_vars(&self, css: &str) -> Result<Vec<Value>, VMError> {
        let mut vars = Vec::new();

        for cap in CSS_VAR_RE.captures_iter(css) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut var = HashMap::new();
            var.insert("name".to_string(), Value::String(format!("--{}", name)));
            var.insert("value".to_string(), Value::String(value.trim().to_string()));

            vars.push(Value::Map(var));
        }

        Ok(vars)
    }

    /// Find color values in CSS
    pub fn css_find_colors(&self, css: &str) -> Result<Vec<Value>, VMError> {
        let colors: Vec<Value> = COLOR_RE.find_iter(css)
            .map(|m| Value::String(m.as_str().to_string()))
            .collect();

        Ok(colors)
    }

    // ========================================
    // HTML Operations
    // ========================================

    /// Parse HTML (returns DOM-like structure)
    pub fn html_parse(&self, html: &str) -> Result<Value, VMError> {
        // For now, return the HTML as-is
        // TODO: Implement proper DOM parsing
        Ok(Value::String(html.to_string()))
    }

    /// Query single element
    pub fn html_query_selector(&self, html: &str, selector: &str) -> Result<Value, VMError> {
        // Simple tag-based query for now
        let tag_pattern = format!(r"<{}\b[^>]*>([^<]*)</{}>", selector, selector);
        if let Ok(re) = Regex::new(&tag_pattern) {
            if let Some(cap) = re.captures(html) {
                return Ok(Value::String(cap.get(0).map(|m| m.as_str()).unwrap_or("").to_string()));
            }
        }
        Ok(Value::Null)
    }

    /// Query all matching elements
    pub fn html_query_all(&self, html: &str, selector: &str) -> Result<Vec<Value>, VMError> {
        let mut results = Vec::new();

        // Simple tag-based query
        let tag_pattern = format!(r"<{}\b[^>]*>([^<]*)</{}>", selector, selector);
        if let Ok(re) = Regex::new(&tag_pattern) {
            for cap in re.captures_iter(html) {
                if let Some(m) = cap.get(0) {
                    results.push(Value::String(m.as_str().to_string()));
                }
            }
        }

        Ok(results)
    }

    // ========================================
    // Resource Extraction
    // ========================================

    /// Extract script tags and their sources
    pub fn extract_scripts(&self, html: &str) -> Result<Vec<Value>, VMError> {
        let mut scripts = Vec::new();

        for cap in SCRIPT_TAG_RE.captures_iter(html) {
            let mut script = HashMap::new();

            if let Some(src) = cap.get(1) {
                script.insert("src".to_string(), Value::String(src.as_str().to_string()));
                script.insert("external".to_string(), Value::Bool(true));
            } else if let Some(content) = cap.get(2) {
                let content_str = content.as_str().trim();
                if !content_str.is_empty() {
                    script.insert("content".to_string(), Value::String(content_str.to_string()));
                    script.insert("external".to_string(), Value::Bool(false));
                }
            }

            if !script.is_empty() {
                scripts.push(Value::Map(script));
            }
        }

        Ok(scripts)
    }

    /// Extract stylesheet links and inline styles
    pub fn extract_styles(&self, html: &str) -> Result<Vec<Value>, VMError> {
        let mut styles = Vec::new();

        // External stylesheets
        for cap in LINK_CSS_RE.captures_iter(html) {
            if let Some(href) = cap.get(1) {
                let mut style = HashMap::new();
                style.insert("href".to_string(), Value::String(href.as_str().to_string()));
                style.insert("external".to_string(), Value::Bool(true));
                styles.push(Value::Map(style));
            }
        }

        // Inline styles
        for cap in STYLE_TAG_RE.captures_iter(html) {
            if let Some(content) = cap.get(1) {
                let content_str = content.as_str().trim();
                if !content_str.is_empty() {
                    let mut style = HashMap::new();
                    style.insert("content".to_string(), Value::String(content_str.to_string()));
                    style.insert("external".to_string(), Value::Bool(false));
                    styles.push(Value::Map(style));
                }
            }
        }

        Ok(styles)
    }

    /// Detect library from URL
    pub fn detect_library(&self, url: &str) -> Result<Value, VMError> {
        let url_lower = url.to_lowercase();

        for (name, pattern) in LIB_PATTERNS.iter() {
            if pattern.is_match(&url_lower) {
                let mut lib = HashMap::new();
                lib.insert("name".to_string(), Value::String(name.to_string()));
                lib.insert("url".to_string(), Value::String(url.to_string()));

                // Try to extract version
                if let Some(version) = extract_version(url) {
                    lib.insert("version".to_string(), Value::String(version));
                }

                return Ok(Value::Map(lib));
            }
        }

        // Unknown library - extract name from filename
        if let Some(filename) = url.split('/').last() {
            let name = filename.split('.').next().unwrap_or(filename);
            let mut lib = HashMap::new();
            lib.insert("name".to_string(), Value::String(name.to_string()));
            lib.insert("url".to_string(), Value::String(url.to_string()));
            lib.insert("unknown".to_string(), Value::Bool(true));
            return Ok(Value::Map(lib));
        }

        Ok(Value::Null)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract version from URL
fn extract_version(url: &str) -> Option<String> {
    lazy_static! {
        static ref VERSION_RE: Regex = Regex::new(r"[/@](\d+\.\d+(?:\.\d+)?)").unwrap();
    }

    VERSION_RE.captures(url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_find_vars() {
        let registry = ParserRegistry::new();
        let css = r#"
            :root {
                --primary-color: #3b82f6;
                --secondary-color: rgb(100, 200, 150);
            }
        "#;

        let vars = registry.css_find_vars(css).unwrap();
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_detect_library() {
        let registry = ParserRegistry::new();

        let result = registry.detect_library("https://cdn.jsdelivr.net/npm/gsap@3.12.2/dist/gsap.min.js").unwrap();
        if let Value::Map(lib) = result {
            assert_eq!(lib.get("name"), Some(&Value::String("gsap".to_string())));
        }
    }

    #[test]
    fn test_extract_scripts() {
        let registry = ParserRegistry::new();
        let html = r#"
            <script src="app.js"></script>
            <script>console.log('inline');</script>
        "#;

        let scripts = registry.extract_scripts(html).unwrap();
        assert_eq!(scripts.len(), 2);
    }
}
