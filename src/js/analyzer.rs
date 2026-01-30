//! JavaScript analyzer
//!
//! High-level analysis combining parsing with pattern detection.

use super::parser::{JsParser, JsParseResult};
use serde::{Deserialize, Serialize};

/// Combined analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsAnalysisResult {
    /// All CSS classes found (deduplicated)
    pub classes: Vec<String>,

    /// Style assignments (property, value)
    pub styles: Vec<(String, String)>,

    /// CSS-in-JS content
    pub css_in_js: Vec<String>,

    /// Libraries detected from imports
    pub libraries: Vec<String>,

    /// Elements created dynamically
    pub elements_created: Vec<String>,

    /// Event listeners registered
    pub event_listeners: Vec<String>,

    /// Parse errors (non-fatal)
    pub errors: Vec<String>,

    /// Raw parse result for detailed inspection
    #[serde(skip)]
    pub raw: Option<JsParseResult>,
}

/// JavaScript analyzer
pub struct JsAnalyzer;

impl JsAnalyzer {
    /// Analyze JavaScript code
    pub fn analyze(code: &str) -> JsAnalysisResult {
        let parse_result = JsParser::parse_to_string_result(code);
        Self::from_parse_result(parse_result)
    }

    /// Analyze TypeScript code
    pub fn analyze_typescript(code: &str) -> JsAnalysisResult {
        let parse_result = JsParser::parse_typescript(code);
        Self::from_parse_result(parse_result)
    }

    /// Convert parse result to analysis result
    fn from_parse_result(parse: JsParseResult) -> JsAnalysisResult {
        let mut result = JsAnalysisResult::default();

        // Deduplicate classes
        let mut classes: Vec<String> = parse.classes.clone();
        classes.sort();
        classes.dedup();
        result.classes = classes;

        // Styles
        result.styles = parse.style_assignments.clone();

        // CSS-in-JS
        for (tag, css) in &parse.css_in_js {
            result.css_in_js.push(format!("/* {} */\n{}", tag, css));
        }

        // Libraries
        for (lib, _source) in &parse.imports {
            if !result.libraries.contains(lib) {
                result.libraries.push(lib.clone());
            }
        }

        // Elements
        result.elements_created = parse.elements_created.clone();

        // Event listeners
        result.event_listeners = parse.event_listeners.clone();

        // Errors
        result.errors = parse.errors.clone();

        // Keep raw for inspection
        result.raw = Some(parse);

        result
    }

    /// Analyze multiple JavaScript files and merge results
    pub fn analyze_multiple(codes: &[&str]) -> JsAnalysisResult {
        let mut combined = JsAnalysisResult::default();

        for code in codes {
            let result = Self::analyze(code);
            combined.merge(result);
        }

        combined
    }
}

impl JsAnalysisResult {
    /// Merge another result into this one
    pub fn merge(&mut self, other: JsAnalysisResult) {
        // Merge classes
        for class in other.classes {
            if !self.classes.contains(&class) {
                self.classes.push(class);
            }
        }

        // Merge styles
        self.styles.extend(other.styles);

        // Merge CSS-in-JS
        self.css_in_js.extend(other.css_in_js);

        // Merge libraries
        for lib in other.libraries {
            if !self.libraries.contains(&lib) {
                self.libraries.push(lib);
            }
        }

        // Merge elements
        self.elements_created.extend(other.elements_created);

        // Merge events
        self.event_listeners.extend(other.event_listeners);

        // Merge errors
        self.errors.extend(other.errors);
    }

    /// Generate CSS from analysis
    pub fn to_css(&self) -> String {
        let mut css = String::new();

        // CSS-in-JS
        if !self.css_in_js.is_empty() {
            css.push_str("/* CSS-in-JS extracted */\n");
            for block in &self.css_in_js {
                css.push_str(block);
                css.push_str("\n\n");
            }
        }

        // Style assignments as utility classes
        if !self.styles.is_empty() {
            css.push_str("/* Style assignments */\n");
            css.push_str(".js-styles {\n");
            for (prop, val) in &self.styles {
                css.push_str(&format!("  {}: {};\n", prop, val));
            }
            css.push_str("}\n");
        }

        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_classname() {
        let code = r#"element.className = "active hover";"#;
        let result = JsAnalyzer::analyze(code);
        assert!(result.classes.contains(&"active".to_string()));
        assert!(result.classes.contains(&"hover".to_string()));
    }

    #[test]
    fn test_analyze_jsx() {
        let code = r#"const App = () => <div className="container flex">Hello</div>;"#;
        let result = JsAnalyzer::analyze(code);
        assert!(result.classes.contains(&"container".to_string()));
        assert!(result.classes.contains(&"flex".to_string()));
    }

    #[test]
    fn test_analyze_styled() {
        let code = r#"
            const Button = styled.button`
                background: blue;
                color: white;
            `;
        "#;
        let result = JsAnalyzer::analyze(code);
        assert!(!result.css_in_js.is_empty());
    }
}
