//! CSS Modules Parser
//!
//! Parses CSS Module class names from compiled JavaScript bundles and HTML.
//! Extracts the mapping between hashed class names and their semantic equivalents.
//!
//! Pattern: ComponentName_className__hash -> .componentName-className
//!
//! Example:
//!   GridLayout_grid__lsHnz -> .grid-layout-grid
//!   Nav_nav__link__2UQWF -> .nav-link

use regex::Regex;
use std::collections::{HashMap, HashSet};

/// CSS Modules extraction result
#[derive(Debug, Clone, Default)]
pub struct CssModulesResult {
    /// Mapping: hashed class name -> semantic class name
    pub class_map: HashMap<String, String>,
    /// Components found
    pub components: Vec<CssModuleComponent>,
    /// Generated CSS with semantic names
    pub generated_css: String,
    /// Statistics
    pub stats: CssModulesStats,
}

#[derive(Debug, Clone, Default)]
pub struct CssModulesStats {
    pub total_classes_found: usize,
    pub unique_components: usize,
    pub classes_mapped: usize,
    pub css_rules_generated: usize,
}

#[derive(Debug, Clone)]
pub struct CssModuleComponent {
    pub name: String,
    pub classes: Vec<CssModuleClass>,
}

#[derive(Debug, Clone)]
pub struct CssModuleClass {
    pub original: String,     // GridLayout_grid__lsHnz
    pub semantic: String,     // grid-layout-grid
    pub component: String,    // GridLayout
    pub class_name: String,   // grid
    pub hash: String,         // lsHnz
}

/// CSS Modules Parser
pub struct CssModulesParser {
    // Pattern: ComponentName_className__hash
    module_class_pattern: Regex,
    // Pattern: ComponentName_className_subClass__hash
    nested_class_pattern: Regex,
    // Pattern for extracting CSS rules from compiled JS
    css_rule_pattern: Regex,
    // CSS variable pattern
    css_var_pattern: Regex,
}

impl CssModulesParser {
    pub fn new() -> Self {
        Self {
            // Match: ComponentName_className__hash (5+ char hash)
            module_class_pattern: Regex::new(
                r"([A-Z][a-zA-Z0-9]*)_([a-zA-Z][a-zA-Z0-9_]*)__([a-zA-Z0-9_]{4,})"
            ).unwrap(),

            // Match nested: ComponentName_className_subClass__hash
            nested_class_pattern: Regex::new(
                r"([A-Z][a-zA-Z0-9]*)_([a-zA-Z][a-zA-Z0-9]*)_([a-zA-Z][a-zA-Z0-9]*)__([a-zA-Z0-9_]{4,})"
            ).unwrap(),

            // Match CSS rules in compiled JS: ".ClassName_class__hash{...}"
            css_rule_pattern: Regex::new(
                r#"\.([A-Z][a-zA-Z0-9]*_[a-zA-Z0-9_]+__[a-zA-Z0-9_]+)\s*\{([^}]+)\}"#
            ).unwrap(),

            // CSS custom properties
            css_var_pattern: Regex::new(
                r"--([a-zA-Z][a-zA-Z0-9-]*)\s*:\s*([^;]+)"
            ).unwrap(),
        }
    }

    /// Extract CSS Module classes from HTML
    pub fn extract_from_html(&self, html: &str) -> CssModulesResult {
        let mut result = CssModulesResult::default();
        let mut seen_classes: HashSet<String> = HashSet::new();
        let mut components: HashMap<String, Vec<CssModuleClass>> = HashMap::new();

        // Extract class attributes
        let class_attr_pattern = Regex::new(r#"class(?:Name)?="([^"]+)""#).unwrap();

        for cap in class_attr_pattern.captures_iter(html) {
            let classes = &cap[1];
            for class in classes.split_whitespace() {
                if seen_classes.contains(class) {
                    continue;
                }
                seen_classes.insert(class.to_string());

                // Try nested pattern first (more specific)
                if let Some(nested_cap) = self.nested_class_pattern.captures(class) {
                    let component = nested_cap[1].to_string();
                    let base_class = nested_cap[2].to_string();
                    let sub_class = nested_cap[3].to_string();
                    let hash = nested_cap[4].to_string();

                    let semantic = format!(
                        "{}-{}-{}",
                        self.to_kebab_case(&component),
                        self.to_kebab_case(&base_class),
                        self.to_kebab_case(&sub_class)
                    );

                    let css_class = CssModuleClass {
                        original: class.to_string(),
                        semantic: semantic.clone(),
                        component: component.clone(),
                        class_name: format!("{}_{}", base_class, sub_class),
                        hash,
                    };

                    result.class_map.insert(class.to_string(), semantic);
                    components.entry(component).or_default().push(css_class);
                    result.stats.classes_mapped += 1;
                }
                // Try simple pattern
                else if let Some(simple_cap) = self.module_class_pattern.captures(class) {
                    let component = simple_cap[1].to_string();
                    let class_name = simple_cap[2].to_string();
                    let hash = simple_cap[3].to_string();

                    let semantic = format!(
                        "{}-{}",
                        self.to_kebab_case(&component),
                        self.to_kebab_case(&class_name)
                    );

                    let css_class = CssModuleClass {
                        original: class.to_string(),
                        semantic: semantic.clone(),
                        component: component.clone(),
                        class_name,
                        hash,
                    };

                    result.class_map.insert(class.to_string(), semantic);
                    components.entry(component).or_default().push(css_class);
                    result.stats.classes_mapped += 1;
                }

                result.stats.total_classes_found += 1;
            }
        }

        // Convert to component list
        result.components = components
            .into_iter()
            .map(|(name, classes)| CssModuleComponent { name, classes })
            .collect();

        result.stats.unique_components = result.components.len();
        result
    }

    /// Extract CSS rules from compiled JavaScript
    pub fn extract_css_from_js(&self, js_code: &str) -> CssModulesResult {
        let mut result = CssModulesResult::default();
        let mut css_output = String::new();

        // Look for CSS rules in the JS bundle
        for cap in self.css_rule_pattern.captures_iter(js_code) {
            let class_name = &cap[1];
            let css_body = &cap[2];

            // Parse the class name
            if let Some(parsed) = self.parse_class_name(class_name) {
                // Generate semantic CSS rule
                css_output.push_str(&format!(
                    ".{} {{\n  {}\n}}\n\n",
                    parsed.semantic,
                    self.format_css_body(css_body)
                ));

                result.class_map.insert(
                    parsed.original.clone(),
                    parsed.semantic.clone(),
                );
                result.stats.css_rules_generated += 1;
            }
        }

        // Also look for inline CSS in template literals
        let template_css_pattern = Regex::new(r#"`([^`]*\{[^`]*:[^`]*\}[^`]*)`"#).unwrap();
        for cap in template_css_pattern.captures_iter(js_code) {
            let content = &cap[1];
            if content.contains('{') && content.contains(':') && content.contains(';') {
                // Looks like CSS
                if !css_output.contains(content) {
                    css_output.push_str("/* Extracted from template literal */\n");
                    css_output.push_str(content);
                    css_output.push_str("\n\n");
                }
            }
        }

        result.generated_css = css_output;
        result
    }

    /// Parse a CSS Module class name
    fn parse_class_name(&self, class: &str) -> Option<CssModuleClass> {
        // Try nested first
        if let Some(cap) = self.nested_class_pattern.captures(class) {
            let component = cap[1].to_string();
            let base = cap[2].to_string();
            let sub = cap[3].to_string();
            let hash = cap[4].to_string();

            return Some(CssModuleClass {
                original: class.to_string(),
                semantic: format!(
                    "{}-{}-{}",
                    self.to_kebab_case(&component),
                    self.to_kebab_case(&base),
                    self.to_kebab_case(&sub)
                ),
                component,
                class_name: format!("{}_{}", base, sub),
                hash,
            });
        }

        // Try simple
        if let Some(cap) = self.module_class_pattern.captures(class) {
            let component = cap[1].to_string();
            let class_name = cap[2].to_string();
            let hash = cap[3].to_string();

            return Some(CssModuleClass {
                original: class.to_string(),
                semantic: format!(
                    "{}-{}",
                    self.to_kebab_case(&component),
                    self.to_kebab_case(&class_name)
                ),
                component,
                class_name,
                hash,
            });
        }

        None
    }

    /// Convert PascalCase or camelCase to kebab-case
    fn to_kebab_case(&self, s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Format CSS body for output
    fn format_css_body(&self, body: &str) -> String {
        // Replace escaped characters
        let cleaned = body
            .replace("\\n", "\n  ")
            .replace("\\\"", "\"")
            .replace("\\'", "'");

        // Add proper indentation
        cleaned
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("  {};", s.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate CSS import/use statements for a project
    pub fn generate_import_map(&self, result: &CssModulesResult) -> String {
        let mut output = String::new();

        output.push_str("/* CSS Module Class Mapping */\n");
        output.push_str("/* Original -> Semantic */\n\n");

        for component in &result.components {
            output.push_str(&format!("/* Component: {} */\n", component.name));
            for class in &component.classes {
                output.push_str(&format!(
                    "/* .{} -> .{} */\n",
                    class.original, class.semantic
                ));
            }
            output.push('\n');
        }

        output
    }

    /// Replace CSS Module classes in HTML with semantic names
    pub fn replace_classes_in_html(&self, html: &str, class_map: &HashMap<String, String>) -> String {
        let mut result = html.to_string();

        // Sort by length (longest first) to avoid partial replacements
        let mut classes: Vec<_> = class_map.iter().collect();
        classes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (original, semantic) in classes {
            result = result.replace(original, semantic);
        }

        result
    }
}

impl Default for CssModulesParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_class() {
        let parser = CssModulesParser::new();
        let html = r#"<div class="GridLayout_grid__lsHnz Nav_nav__W3s5_"></div>"#;

        let result = parser.extract_from_html(html);

        assert_eq!(result.stats.classes_mapped, 2);
        assert!(result.class_map.contains_key("GridLayout_grid__lsHnz"));
        assert_eq!(
            result.class_map.get("GridLayout_grid__lsHnz"),
            Some(&"grid-layout-grid".to_string())
        );
    }

    #[test]
    fn test_parse_nested_class() {
        let parser = CssModulesParser::new();
        let html = r#"<div class="Nav_nav__link__2UQWF"></div>"#;

        let result = parser.extract_from_html(html);

        assert_eq!(result.stats.classes_mapped, 1);
        assert!(result.class_map.contains_key("Nav_nav__link__2UQWF"));
    }

    #[test]
    fn test_to_kebab_case() {
        let parser = CssModulesParser::new();

        assert_eq!(parser.to_kebab_case("GridLayout"), "grid-layout");
        assert_eq!(parser.to_kebab_case("navLink"), "nav-link");
        assert_eq!(parser.to_kebab_case("Icon"), "icon");
    }

    #[test]
    fn test_replace_classes() {
        let parser = CssModulesParser::new();
        let html = r#"<div class="GridLayout_grid__lsHnz"></div>"#;

        let result = parser.extract_from_html(html);
        let new_html = parser.replace_classes_in_html(html, &result.class_map);

        assert!(new_html.contains("grid-layout-grid"));
        assert!(!new_html.contains("GridLayout_grid__lsHnz"));
    }
}
