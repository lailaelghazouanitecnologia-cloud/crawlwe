//! CSS Variable and Style Extraction from JavaScript
//!
//! Extracts CSS-related patterns from JS bundles:
//! - style.setProperty("--var", value) - CSS variable assignments
//! - style.getPropertyValue("--var") || defaultValue - variable fallbacks
//! - element.style.property = value - inline style assignments
//! - Computed style patterns: hsl(), rgb(), linear-gradient()
//! - Animation frame patterns: requestAnimationFrame with style updates

use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Result of CSS extraction from JavaScript
#[derive(Debug, Clone, Default)]
pub struct JsCssExtractionResult {
    /// CSS variables found with their values
    pub css_variables: HashMap<String, CssVariableInfo>,
    /// Inline styles extracted (selector pattern -> properties)
    pub inline_styles: Vec<ExtractedStyle>,
    /// Dynamic animations found
    pub animations: Vec<DynamicAnimation>,
    /// Color values found
    pub colors: HashSet<String>,
    /// Gradient patterns found
    pub gradients: Vec<String>,
    /// Statistics
    pub stats: ExtractionStats,
}

#[derive(Debug, Clone)]
pub struct CssVariableInfo {
    /// Variable name (without --)
    pub name: String,
    /// Possible values found
    pub values: Vec<String>,
    /// Default/fallback value if found
    pub default_value: Option<String>,
    /// Context where it was found
    pub context: VariableContext,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableContext {
    SetProperty,      // style.setProperty("--var", value)
    GetPropertyFallback, // getPropertyValue("--var") || default
    CssVarFunction,   // var(--name, default)
    ThemeAccess,      // theme.colors.primary
}

#[derive(Debug, Clone)]
pub struct ExtractedStyle {
    /// Element or selector hint
    pub selector_hint: String,
    /// CSS properties
    pub properties: HashMap<String, String>,
    /// Source context
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct DynamicAnimation {
    /// Animation identifier
    pub name: String,
    /// Properties being animated
    pub properties: Vec<String>,
    /// Duration if found
    pub duration: Option<String>,
    /// Keyframe-like values
    pub values: Vec<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractionStats {
    pub variables_found: usize,
    pub set_property_calls: usize,
    pub get_property_calls: usize,
    pub inline_styles_found: usize,
    pub colors_found: usize,
    pub gradients_found: usize,
    pub animations_found: usize,
}

/// CSS extractor from JavaScript code
pub struct JsCssExtractor {
    // CSS variable patterns
    set_property_pattern: Regex,
    get_property_pattern: Regex,
    get_property_fallback_pattern: Regex,
    css_var_pattern: Regex,

    // Style assignment patterns
    style_dot_pattern: Regex,
    style_bracket_pattern: Regex,

    // Color patterns
    hsl_pattern: Regex,
    rgb_pattern: Regex,
    hex_pattern: Regex,

    // Gradient patterns
    linear_gradient_pattern: Regex,
    radial_gradient_pattern: Regex,

    // Animation patterns
    request_animation_frame_pattern: Regex,
    transition_pattern: Regex,
}

impl JsCssExtractor {
    pub fn new() -> Self {
        Self {
            // style.setProperty("--varName", value)
            set_property_pattern: Regex::new(
                r#"\.setProperty\s*\(\s*["']--([a-zA-Z][a-zA-Z0-9_-]*)["']\s*,\s*([^)]+)\)"#
            ).unwrap(),

            // style.getPropertyValue("--varName")
            get_property_pattern: Regex::new(
                r#"\.getPropertyValue\s*\(\s*["']--([a-zA-Z][a-zA-Z0-9_-]*)["']\s*\)"#
            ).unwrap(),

            // getPropertyValue("--var") || "default"
            get_property_fallback_pattern: Regex::new(
                r#"\.getPropertyValue\s*\(\s*["']--([a-zA-Z][a-zA-Z0-9_-]*)["']\s*\)\s*\|\|\s*["']([^"']+)["']"#
            ).unwrap(),

            // var(--name) or var(--name, default)
            css_var_pattern: Regex::new(
                r#"var\s*\(\s*--([a-zA-Z][a-zA-Z0-9_-]*)(?:\s*,\s*([^)]+))?\)"#
            ).unwrap(),

            // element.style.propertyName = value
            style_dot_pattern: Regex::new(
                r#"\.style\.([a-zA-Z][a-zA-Z0-9]*)\s*=\s*([^;,\n]+)"#
            ).unwrap(),

            // element.style["property-name"] = value
            style_bracket_pattern: Regex::new(
                r#"\.style\s*\[\s*["']([a-zA-Z-]+)["']\s*\]\s*=\s*([^;,\n]+)"#
            ).unwrap(),

            // hsl(deg, %, %) patterns
            hsl_pattern: Regex::new(
                r#"hsl\s*\(\s*([0-9.]+)(?:deg)?\s*,\s*([0-9.]+)%?\s*,\s*([0-9.]+)%?\s*\)"#
            ).unwrap(),

            // rgb(r, g, b) patterns
            rgb_pattern: Regex::new(
                r#"rgb\s*\(\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*\)"#
            ).unwrap(),

            // Hex colors
            hex_pattern: Regex::new(
                r#"["']#([0-9a-fA-F]{3,8})["']"#
            ).unwrap(),

            // linear-gradient patterns
            linear_gradient_pattern: Regex::new(
                r#"linear-gradient\s*\([^)]+\)"#
            ).unwrap(),

            // radial-gradient patterns
            radial_gradient_pattern: Regex::new(
                r#"radial-gradient\s*\([^)]+\)"#
            ).unwrap(),

            // requestAnimationFrame
            request_animation_frame_pattern: Regex::new(
                r#"requestAnimationFrame\s*\(\s*(\w+)\s*\)"#
            ).unwrap(),

            // transition patterns
            transition_pattern: Regex::new(
                r#"transition\s*:\s*([^;}"']+)"#
            ).unwrap(),
        }
    }

    /// Extract CSS information from JavaScript code
    pub fn extract(&self, js_code: &str) -> JsCssExtractionResult {
        let mut result = JsCssExtractionResult::default();

        // Extract CSS variables with setProperty
        self.extract_set_property(js_code, &mut result);

        // Extract CSS variables with getPropertyValue and fallbacks
        self.extract_get_property(js_code, &mut result);

        // Extract inline style assignments
        self.extract_inline_styles(js_code, &mut result);

        // Extract colors
        self.extract_colors(js_code, &mut result);

        // Extract gradients
        self.extract_gradients(js_code, &mut result);

        // Update stats
        result.stats.variables_found = result.css_variables.len();
        result.stats.colors_found = result.colors.len();
        result.stats.gradients_found = result.gradients.len();

        result
    }

    fn extract_set_property(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        for cap in self.set_property_pattern.captures_iter(js_code) {
            if let (Some(name), Some(value)) = (cap.get(1), cap.get(2)) {
                let var_name = name.as_str().to_string();
                let value_str = self.clean_value(value.as_str());

                result.stats.set_property_calls += 1;

                let entry = result.css_variables
                    .entry(var_name.clone())
                    .or_insert_with(|| CssVariableInfo {
                        name: var_name,
                        values: Vec::new(),
                        default_value: None,
                        context: VariableContext::SetProperty,
                    });

                if !entry.values.contains(&value_str) {
                    entry.values.push(value_str);
                }
            }
        }
    }

    fn extract_get_property(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // First, extract with fallbacks
        for cap in self.get_property_fallback_pattern.captures_iter(js_code) {
            if let (Some(name), Some(default)) = (cap.get(1), cap.get(2)) {
                let var_name = name.as_str().to_string();
                let default_str = default.as_str().to_string();

                result.stats.get_property_calls += 1;

                let entry = result.css_variables
                    .entry(var_name.clone())
                    .or_insert_with(|| CssVariableInfo {
                        name: var_name,
                        values: Vec::new(),
                        default_value: None,
                        context: VariableContext::GetPropertyFallback,
                    });

                entry.default_value = Some(default_str.clone());
                if !entry.values.contains(&default_str) {
                    entry.values.push(default_str);
                }
            }
        }

        // Then simple getPropertyValue calls (without fallback)
        for cap in self.get_property_pattern.captures_iter(js_code) {
            if let Some(name) = cap.get(1) {
                let var_name = name.as_str().to_string();

                if !result.css_variables.contains_key(&var_name) {
                    result.stats.get_property_calls += 1;
                    result.css_variables.insert(var_name.clone(), CssVariableInfo {
                        name: var_name,
                        values: Vec::new(),
                        default_value: None,
                        context: VariableContext::GetPropertyFallback,
                    });
                }
            }
        }
    }

    fn extract_inline_styles(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Extract style.propertyName = value patterns
        for cap in self.style_dot_pattern.captures_iter(js_code) {
            if let (Some(prop), Some(value)) = (cap.get(1), cap.get(2)) {
                let property = self.camel_to_kebab(prop.as_str());
                let value_str = self.clean_value(value.as_str());

                // Skip if value is a variable reference
                if !value_str.contains("var(") &&
                   !value_str.starts_with("e.") &&
                   !value_str.starts_with("t.") &&
                   !self.is_js_expression(&value_str) {
                    let mut props = HashMap::new();
                    props.insert(property.clone(), value_str);

                    result.inline_styles.push(ExtractedStyle {
                        selector_hint: format!(".js-style-{}", property),
                        properties: props,
                        source: "style.property".to_string(),
                    });
                    result.stats.inline_styles_found += 1;
                }
            }
        }

        // Extract style["property-name"] = value patterns
        for cap in self.style_bracket_pattern.captures_iter(js_code) {
            if let (Some(prop), Some(value)) = (cap.get(1), cap.get(2)) {
                let property = prop.as_str().to_string();
                let value_str = self.clean_value(value.as_str());

                if !self.is_js_expression(&value_str) {
                    let mut props = HashMap::new();
                    props.insert(property.clone(), value_str);

                    result.inline_styles.push(ExtractedStyle {
                        selector_hint: format!(".js-style-{}", property),
                        properties: props,
                        source: "style[property]".to_string(),
                    });
                    result.stats.inline_styles_found += 1;
                }
            }
        }
    }

    fn extract_colors(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // HSL colors
        for cap in self.hsl_pattern.captures_iter(js_code) {
            if let (Some(h), Some(s), Some(l)) = (cap.get(1), cap.get(2), cap.get(3)) {
                let color = format!("hsl({}deg, {}%, {}%)", h.as_str(), s.as_str(), l.as_str());
                result.colors.insert(color);
            }
        }

        // RGB colors
        for cap in self.rgb_pattern.captures_iter(js_code) {
            if let (Some(r), Some(g), Some(b)) = (cap.get(1), cap.get(2), cap.get(3)) {
                let color = format!("rgb({}, {}, {})", r.as_str(), g.as_str(), b.as_str());
                result.colors.insert(color);
            }
        }

        // Hex colors
        for cap in self.hex_pattern.captures_iter(js_code) {
            if let Some(hex) = cap.get(1) {
                let color = format!("#{}", hex.as_str());
                result.colors.insert(color);
            }
        }
    }

    fn extract_gradients(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Linear gradients
        for mat in self.linear_gradient_pattern.find_iter(js_code) {
            result.gradients.push(mat.as_str().to_string());
        }

        // Radial gradients
        for mat in self.radial_gradient_pattern.find_iter(js_code) {
            result.gradients.push(mat.as_str().to_string());
        }
    }

    fn clean_value(&self, value: &str) -> String {
        value.trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches('`')
            .to_string()
    }

    fn camel_to_kebab(&self, s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }

    fn is_js_expression(&self, value: &str) -> bool {
        // Check if value is a JS expression rather than a CSS value
        value.contains("concat(") ||
        value.contains("function") ||
        value.contains("=>") ||
        value.starts_with("e.") ||
        value.starts_with("t.") ||
        value.starts_with("r.") ||
        value.starts_with("n.") ||
        value.starts_with("o.") ||
        (value.len() < 3 && value.chars().all(|c| c.is_alphabetic()))
    }

    /// Generate CSS from extraction result
    pub fn generate_css(&self, result: &JsCssExtractionResult) -> String {
        let mut css = String::new();

        // Generate :root with CSS variables
        if !result.css_variables.is_empty() {
            css.push_str("/* CSS Variables extracted from JavaScript */\n");
            css.push_str(":root {\n");

            for (name, info) in &result.css_variables {
                // Use default value if available, otherwise first value
                let value = info.default_value.as_ref()
                    .or_else(|| info.values.first())
                    .cloned()
                    .unwrap_or_else(|| "inherit".to_string());

                // Skip if value looks like a JS expression
                if !self.is_js_expression(&value) {
                    css.push_str(&format!("  --{}: {};\n", name, value));
                }
            }

            css.push_str("}\n\n");
        }

        // Generate utility classes for colors
        if !result.colors.is_empty() {
            css.push_str("/* Colors found in JavaScript */\n");
            css.push_str("/*\n");
            for color in &result.colors {
                css.push_str(&format!("  {}\n", color));
            }
            css.push_str("*/\n\n");
        }

        // Generate gradient examples
        if !result.gradients.is_empty() {
            css.push_str("/* Gradients found in JavaScript */\n");
            css.push_str("/*\n");
            for gradient in &result.gradients {
                css.push_str(&format!("  background: {};\n", gradient));
            }
            css.push_str("*/\n\n");
        }

        css
    }
}

impl Default for JsCssExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_property_extraction() {
        let extractor = JsCssExtractor::new();
        let js_code = r##"
            g.style.setProperty("--fontColor", d);
            g.style.setProperty("--highlightColor", "#ff0000");
        "##;

        let result = extractor.extract(js_code);
        assert!(result.css_variables.contains_key("fontColor"));
        assert!(result.css_variables.contains_key("highlightColor"));
    }

    #[test]
    fn test_get_property_fallback() {
        let extractor = JsCssExtractor::new();
        let js_code = r##"
            e.strokeStyle = document.body.style.getPropertyValue("--artStroke") || "#1d1d1d";
        "##;

        let result = extractor.extract(js_code);
        assert!(result.css_variables.contains_key("artStroke"));
        assert_eq!(
            result.css_variables.get("artStroke").unwrap().default_value,
            Some("#1d1d1d".to_string())
        );
    }
}
