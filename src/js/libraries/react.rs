//! React/JSX parser
//!
//! Parses React components and extracts styling information.

use super::{
    LibraryAnalysis, LibraryCategory, RenderElement,
    registry::{LibraryParser, LibraryEffect, EffectType},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for React/JSX patterns
pub struct ReactParser {
    // JSX patterns
    jsx_element: Regex,
    classname_attr: Regex,
    style_attr: Regex,
    conditional_class: Regex,
    clsx_pattern: Regex,
    use_state: Regex,
    use_effect: Regex,
}

impl ReactParser {
    pub fn new() -> Self {
        Self {
            jsx_element: Regex::new(r#"<(\w+)([^>]*)>"#).unwrap(),
            classname_attr: Regex::new(r#"className\s*=\s*(?:"([^"]+)"|'([^']+)'|\{([^}]+)\})"#).unwrap(),
            style_attr: Regex::new(r#"style\s*=\s*\{\s*\{([^}]+)\}"#).unwrap(),
            conditional_class: Regex::new(r#"(\w+)\s*\?\s*["']([^"']+)["']\s*:\s*["']([^"']+)["']"#).unwrap(),
            clsx_pattern: Regex::new(r#"(?:clsx|classnames|cn)\s*\(([^)]+)\)"#).unwrap(),
            use_state: Regex::new(r#"useState\s*\(([^)]*)\)"#).unwrap(),
            use_effect: Regex::new(r#"useEffect\s*\("#).unwrap(),
        }
    }

    /// Parse inline style object
    fn parse_style_object(style_str: &str) -> HashMap<String, String> {
        let mut styles = HashMap::new();
        let prop_pattern = Regex::new(r#"(\w+)\s*:\s*(?:["']([^"']+)["']|(\d+(?:\.\d+)?(?:px|em|rem|%)?)|(\w+))"#).unwrap();

        for cap in prop_pattern.captures_iter(style_str) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2)
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
                .map(|m| m.as_str())
                .unwrap_or("");

            if !key.is_empty() && !value.is_empty() {
                // Convert camelCase to kebab-case
                let css_key = Self::to_kebab_case(key);
                styles.insert(css_key, value.to_string());
            }
        }

        styles
    }

    /// Convert camelCase to kebab-case
    fn to_kebab_case(s: &str) -> String {
        let mut result = String::new();

        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('-');
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c.to_lowercase().next().unwrap());
            }
        }

        result
    }

    /// Extract classes from clsx/classnames patterns
    fn extract_clsx_classes(&self, clsx_call: &str) -> Vec<String> {
        let mut classes = Vec::new();

        // Extract string literals
        let string_pattern = Regex::new(r#"["']([^"']+)["']"#).unwrap();
        for cap in string_pattern.captures_iter(clsx_call) {
            if let Some(class_str) = cap.get(1) {
                for class in class_str.as_str().split_whitespace() {
                    classes.push(class.to_string());
                }
            }
        }

        // Extract conditional classes: condition && "class"
        let cond_pattern = Regex::new(r#"\w+\s*&&\s*["']([^"']+)["']"#).unwrap();
        for cap in cond_pattern.captures_iter(clsx_call) {
            if let Some(class_str) = cap.get(1) {
                for class in class_str.as_str().split_whitespace() {
                    classes.push(class.to_string());
                }
            }
        }

        classes
    }
}

impl Default for ReactParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for ReactParser {
    fn name(&self) -> &'static str {
        "react"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::UIFramework
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        analysis.libraries.iter().any(|lib| lib == "react") ||
        analysis.raw.as_ref()
            .map(|r| r.imports.iter().any(|(lib, src)| lib == "react" || src.contains("react")))
            .unwrap_or(false) ||
        // Check for JSX patterns
        analysis.raw.as_ref()
            .map(|r| !r.jsx_classnames.is_empty())
            .unwrap_or(false)
    }

    fn analyze(&self, code: &str, analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut all_classes: Vec<String> = Vec::new();
        let mut inline_styles: Vec<(String, HashMap<String, String>)> = Vec::new();

        // Extract className attributes
        for cap in self.classname_attr.captures_iter(code) {
            // Direct string value
            if let Some(class_str) = cap.get(1).or_else(|| cap.get(2)) {
                for class in class_str.as_str().split_whitespace() {
                    if !all_classes.contains(&class.to_string()) {
                        all_classes.push(class.to_string());
                    }
                }
            }

            // Expression (could be clsx, ternary, etc.)
            if let Some(expr) = cap.get(3) {
                let expr_str = expr.as_str();

                // Check for clsx/classnames
                if let Some(clsx_cap) = self.clsx_pattern.captures(expr_str) {
                    if let Some(args) = clsx_cap.get(1) {
                        for class in self.extract_clsx_classes(args.as_str()) {
                            if !all_classes.contains(&class) {
                                all_classes.push(class);
                            }
                        }
                    }
                }

                // Check for ternary conditions
                for cond_cap in self.conditional_class.captures_iter(expr_str) {
                    if let Some(true_class) = cond_cap.get(2) {
                        for class in true_class.as_str().split_whitespace() {
                            if !all_classes.contains(&class.to_string()) {
                                all_classes.push(class.to_string());
                            }
                        }
                    }
                    if let Some(false_class) = cond_cap.get(3) {
                        for class in false_class.as_str().split_whitespace() {
                            if !all_classes.contains(&class.to_string()) {
                                all_classes.push(class.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Extract inline styles
        for cap in self.style_attr.captures_iter(code) {
            if let Some(style_str) = cap.get(1) {
                let styles = Self::parse_style_object(style_str.as_str());
                if !styles.is_empty() {
                    inline_styles.push((format!("inline-style-{}", inline_styles.len()), styles));
                }
            }
        }

        // Include classes from the original analysis
        if let Some(raw) = &analysis.raw {
            for class in &raw.classes {
                if !all_classes.contains(class) {
                    all_classes.push(class.clone());
                }
            }

            for jsx_class in &raw.jsx_classnames {
                for class in jsx_class.split_whitespace() {
                    if !all_classes.contains(&class.to_string()) {
                        all_classes.push(class.to_string());
                    }
                }
            }
        }

        // Generate CSS for inline styles (as utility classes)
        for (class_name, styles) in &inline_styles {
            let mut css = format!(".{} {{\n", class_name);
            for (prop, value) in styles {
                css.push_str(&format!("  {}: {};\n", prop, value));
            }
            css.push_str("}\n");
            result.generated_css.push(css);

            result.effects.push(LibraryEffect {
                effect_type: EffectType::StyleChange,
                target: class_name.clone(),
                css_properties: styles.clone(),
                timing: None,
            });
        }

        // Build render elements
        for class in &all_classes {
            result.elements_to_render.push(RenderElement {
                tag: "div".to_string(),
                classes: vec![class.clone()],
                styles: HashMap::new(),
                attributes: HashMap::new(),
                content: None,
                children: Vec::new(),
            });
        }

        // Check for state that might affect rendering
        if self.use_state.is_match(code) {
            result.warnings.push("useState detected - component has dynamic state".to_string());
        }

        if self.use_effect.is_match(code) {
            result.warnings.push("useEffect detected - side effects may modify DOM".to_string());
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* React Component CSS */\n\n");

        for block in &analysis.generated_css {
            css.push_str(block);
            css.push_str("\n\n");
        }

        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_classname() {
        let parser = ReactParser::new();
        let code = r#"
            <div className="container flex items-center">
                <span className="text-blue-500">Hello</span>
            </div>
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("react".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.elements_to_render.is_empty());
    }

    #[test]
    fn test_parse_inline_style() {
        let parser = ReactParser::new();
        let code = r#"
            <div style={{ backgroundColor: 'red', padding: '20px' }}>
                Content
            </div>
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("react".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.generated_css.is_empty());
    }

    #[test]
    fn test_kebab_case_conversion() {
        assert_eq!(ReactParser::to_kebab_case("backgroundColor"), "background-color");
        assert_eq!(ReactParser::to_kebab_case("fontSize"), "font-size");
        assert_eq!(ReactParser::to_kebab_case("borderRadius"), "border-radius");
    }
}
