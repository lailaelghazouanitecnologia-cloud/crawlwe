//! Advanced CSS Parser using lightningcss
//!
//! Provides AST manipulation, deduplication, and optimization
//! of CSS rules for generating clean output.

use crate::models::*;
use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::Property;
use lightningcss::rules::CssRule as LightningRule;
use lightningcss::rules::CssRuleList;
use lightningcss::selector::Selector;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::ToCss;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CssParserError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    #[error("Optimization error: {0}")]
    OptimizationError(String),
}

/// CSS AST for manipulation
#[derive(Debug, Clone)]
pub struct CssAst {
    pub rules: Vec<CssRule>,
    pub media_queries: Vec<MediaQuery>,
    pub keyframes: Vec<Keyframes>,
    pub variables: Vec<CssVariable>,
    pub font_faces: Vec<FontFace>,
}

impl CssAst {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            media_queries: Vec::new(),
            keyframes: Vec::new(),
            variables: Vec::new(),
            font_faces: Vec::new(),
        }
    }

    /// Parse CSS text into AST
    pub fn parse(css_text: &str) -> Result<Self, CssParserError> {
        let mut ast = Self::new();

        let stylesheet = StyleSheet::parse(css_text, ParserOptions::default())
            .map_err(|e| CssParserError::ParseError(e.to_string()))?;

        ast.extract_rules(&stylesheet.rules);

        Ok(ast)
    }

    fn extract_rules(&mut self, rules: &CssRuleList) {
        for rule in rules.0.iter() {
            match rule {
                LightningRule::Style(style_rule) => {
                    let selector = style_rule
                        .selectors
                        .to_css_string(PrinterOptions::default())
                        .unwrap_or_default();

                    let mut declarations = HashMap::new();
                    for property in style_rule.declarations.declarations.iter() {
                        let (name, value) = property_to_string(property);
                        declarations.insert(name, value);
                    }

                    self.rules.push(CssRule {
                        selector: selector.clone(),
                        declarations,
                        specificity: calculate_specificity_from_selector(&selector),
                        source_file: None,
                        line: None,
                    });
                }

                LightningRule::Media(media_rule) => {
                    let condition = media_rule
                        .query
                        .to_css_string(PrinterOptions::default())
                        .unwrap_or_default();

                    let mut nested_ast = CssAst::new();
                    nested_ast.extract_rules(&media_rule.rules);

                    self.media_queries.push(MediaQuery {
                        condition,
                        rules: nested_ast.rules,
                    });
                }

                LightningRule::Keyframes(kf_rule) => {
                    let mut frames = Vec::new();
                    for keyframe in kf_rule.keyframes.iter() {
                        let position = keyframe
                            .selectors
                            .iter()
                            .map(|s| format!("{:?}", s))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let mut properties = HashMap::new();
                        for prop in keyframe.declarations.declarations.iter() {
                            let (name, value) = property_to_string(prop);
                            properties.insert(name, value);
                        }

                        frames.push(KeyframeStep { position, properties });
                    }

                    self.keyframes.push(Keyframes {
                        name: kf_rule.name.to_string(),
                        frames,
                    });
                }

                LightningRule::FontFace(ff_rule) => {
                    // Extract font face info
                    let mut family = String::new();
                    let mut src = Vec::new();
                    let mut weight = None;
                    let mut style = None;
                    let mut display = None;

                    for prop in ff_rule.properties.iter() {
                        match prop {
                            lightningcss::rules::font_face::FontFaceProperty::FontFamily(f) => {
                                family = f.to_string();
                            }
                            lightningcss::rules::font_face::FontFaceProperty::Source(s) => {
                                src = s.iter().map(|x| format!("{:?}", x)).collect();
                            }
                            lightningcss::rules::font_face::FontFaceProperty::FontWeight(w) => {
                                weight = Some(format!("{:?}", w));
                            }
                            lightningcss::rules::font_face::FontFaceProperty::FontStyle(s) => {
                                style = Some(format!("{:?}", s));
                            }
                            lightningcss::rules::font_face::FontFaceProperty::FontDisplay(d) => {
                                display = Some(format!("{:?}", d));
                            }
                            _ => {}
                        }
                    }

                    self.font_faces.push(FontFace {
                        family,
                        src,
                        weight,
                        style,
                        display,
                    });
                }

                _ => {}
            }
        }
    }

    /// Merge with another AST
    pub fn merge(&mut self, other: CssAst) {
        self.rules.extend(other.rules);
        self.media_queries.extend(other.media_queries);
        self.keyframes.extend(other.keyframes);
        self.variables.extend(other.variables);
        self.font_faces.extend(other.font_faces);
    }
}

/// Convert a lightningcss Property to (name, value) string pair
fn property_to_string(property: &Property) -> (String, String) {
    let css_string = property
        .to_css_string(PrinterOptions::default())
        .unwrap_or_default();

    // Parse "property: value" format
    if let Some(colon_pos) = css_string.find(':') {
        let name = css_string[..colon_pos].trim().to_string();
        let value = css_string[colon_pos + 1..]
            .trim()
            .trim_end_matches(';')
            .to_string();
        (name, value)
    } else {
        (css_string, String::new())
    }
}

/// Calculate specificity from selector string
fn calculate_specificity_from_selector(selector: &str) -> Specificity {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;

    for part in selector.split(',') {
        let part = part.trim();

        // Count IDs (#)
        ids += part.matches('#').count() as u32;

        // Count classes (.), attributes ([), and pseudo-classes (:)
        classes += part.matches('.').count() as u32;
        classes += part.matches('[').count() as u32;

        // Pseudo-classes (single :) but not pseudo-elements (::)
        let single_colons = part.matches(':').count();
        let double_colons = part.matches("::").count();
        classes += (single_colons - double_colons * 2) as u32;

        // Count elements (tag names) and pseudo-elements (::)
        let tokens: Vec<&str> = part
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        for token in tokens {
            // If it's not starting with # or . and looks like a tag
            if !token.starts_with('#')
                && !token.starts_with('.')
                && token.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
                && token.to_lowercase() == token
            {
                elements += 1;
            }
        }

        elements += double_colons as u32;
    }

    Specificity { ids, classes, elements }
}

/// CSS Optimizer for deduplication and minification
pub struct CssOptimizer {
    /// Remove duplicate rules
    pub deduplicate: bool,
    /// Merge rules with identical declarations
    pub merge_identical: bool,
    /// Sort by specificity
    pub sort_by_specificity: bool,
    /// Remove !important where not needed
    pub clean_important: bool,
    /// Minify values (e.g., 0px -> 0)
    pub minify_values: bool,
    /// Remove unused rules (requires DOM)
    pub remove_unused: bool,
}

impl Default for CssOptimizer {
    fn default() -> Self {
        Self {
            deduplicate: true,
            merge_identical: true,
            sort_by_specificity: true,
            clean_important: false,
            minify_values: true,
            remove_unused: false,
        }
    }
}

impl CssOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Optimize a CSS AST
    pub fn optimize(&self, mut ast: CssAst) -> CssAst {
        if self.deduplicate {
            ast = self.deduplicate_rules(ast);
        }

        if self.merge_identical {
            ast = self.merge_identical_rules(ast);
        }

        if self.sort_by_specificity {
            ast.rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));

            for mq in &mut ast.media_queries {
                mq.rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));
            }
        }

        if self.minify_values {
            ast = self.minify_css_values(ast);
        }

        ast
    }

    /// Remove duplicate rules (same selector and declarations)
    fn deduplicate_rules(&self, mut ast: CssAst) -> CssAst {
        let mut seen: HashSet<String> = HashSet::new();
        ast.rules.retain(|rule| {
            let key = format!("{}|{:?}", rule.selector, rule.declarations);
            seen.insert(key)
        });

        for mq in &mut ast.media_queries {
            let mut seen: HashSet<String> = HashSet::new();
            mq.rules.retain(|rule| {
                let key = format!("{}|{:?}", rule.selector, rule.declarations);
                seen.insert(key)
            });
        }

        ast
    }

    /// Merge rules with identical declarations into combined selectors
    fn merge_identical_rules(&self, mut ast: CssAst) -> CssAst {
        let mut declaration_map: HashMap<String, Vec<String>> = HashMap::new();

        // Group by declarations
        for rule in &ast.rules {
            let decl_key = format!("{:?}", rule.declarations);
            declaration_map
                .entry(decl_key)
                .or_default()
                .push(rule.selector.clone());
        }

        // Rebuild rules with merged selectors
        let mut new_rules = Vec::new();
        let mut processed_selectors: HashSet<String> = HashSet::new();

        for rule in &ast.rules {
            if processed_selectors.contains(&rule.selector) {
                continue;
            }

            let decl_key = format!("{:?}", rule.declarations);
            if let Some(selectors) = declaration_map.get(&decl_key) {
                if selectors.len() > 1 {
                    // Merge selectors
                    let merged_selector = selectors.join(", ");
                    new_rules.push(CssRule {
                        selector: merged_selector,
                        declarations: rule.declarations.clone(),
                        specificity: rule.specificity,
                        source_file: rule.source_file.clone(),
                        line: rule.line,
                    });

                    for s in selectors {
                        processed_selectors.insert(s.clone());
                    }
                } else {
                    new_rules.push(rule.clone());
                    processed_selectors.insert(rule.selector.clone());
                }
            }
        }

        ast.rules = new_rules;
        ast
    }

    /// Minify CSS values
    fn minify_css_values(&self, mut ast: CssAst) -> CssAst {
        for rule in &mut ast.rules {
            for value in rule.declarations.values_mut() {
                *value = minify_value(value);
            }
        }

        for mq in &mut ast.media_queries {
            for rule in &mut mq.rules {
                for value in rule.declarations.values_mut() {
                    *value = minify_value(value);
                }
            }
        }

        ast
    }

    /// Remove rules that don't match any element in DOM
    pub fn remove_unused_rules(&self, ast: CssAst, used_selectors: &HashSet<String>) -> CssAst {
        let mut new_ast = ast.clone();

        new_ast.rules.retain(|rule| {
            // Keep if any selector part is used
            rule.selector
                .split(',')
                .any(|s| used_selectors.contains(s.trim()))
        });

        new_ast
    }
}

/// Minify a single CSS value
fn minify_value(value: &str) -> String {
    let mut result = value.to_string();

    // 0px -> 0
    result = result.replace("0px", "0");
    result = result.replace("0em", "0");
    result = result.replace("0rem", "0");
    result = result.replace("0%", "0");

    // Remove leading zeros: 0.5 -> .5
    // (careful with this one, only in safe contexts)

    // Collapse multiple spaces
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    // Lowercase hex colors
    if result.starts_with('#') && result.len() == 7 {
        result = result.to_lowercase();
    }

    // #ffffff -> #fff
    if result.starts_with('#') && result.len() == 7 {
        let chars: Vec<char> = result.chars().collect();
        if chars[1] == chars[2] && chars[3] == chars[4] && chars[5] == chars[6] {
            result = format!("#{}{}{}", chars[1], chars[3], chars[5]);
        }
    }

    result
}

/// CSS Generator - converts AST back to CSS string
pub struct CssGenerator {
    /// Indent string (default: 2 spaces)
    pub indent: String,
    /// Whether to minify output
    pub minify: bool,
}

impl Default for CssGenerator {
    fn default() -> Self {
        Self {
            indent: "  ".to_string(),
            minify: false,
        }
    }
}

impl CssGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minified() -> Self {
        Self {
            indent: String::new(),
            minify: true,
        }
    }

    /// Generate CSS string from AST
    pub fn generate(&self, ast: &CssAst) -> String {
        let mut css = String::new();
        let newline = if self.minify { "" } else { "\n" };
        let space = if self.minify { "" } else { " " };

        // CSS Variables
        if !ast.variables.is_empty() {
            css.push_str(":root{");
            css.push_str(newline);

            for var in &ast.variables {
                css.push_str(&self.indent);
                css.push_str(&var.name);
                css.push(':');
                css.push_str(space);
                css.push_str(&var.value);
                css.push(';');
                css.push_str(newline);
            }

            css.push('}');
            css.push_str(newline);
            css.push_str(newline);
        }

        // Font faces
        for font in &ast.font_faces {
            css.push_str("@font-face{");
            css.push_str(newline);

            css.push_str(&self.indent);
            css.push_str("font-family:");
            css.push_str(space);
            css.push('\'');
            css.push_str(&font.family);
            css.push('\'');
            css.push(';');
            css.push_str(newline);

            if let Some(weight) = &font.weight {
                css.push_str(&self.indent);
                css.push_str("font-weight:");
                css.push_str(space);
                css.push_str(weight);
                css.push(';');
                css.push_str(newline);
            }

            if let Some(style) = &font.style {
                css.push_str(&self.indent);
                css.push_str("font-style:");
                css.push_str(space);
                css.push_str(style);
                css.push(';');
                css.push_str(newline);
            }

            if !font.src.is_empty() {
                css.push_str(&self.indent);
                css.push_str("src:");
                css.push_str(space);
                css.push_str(&font.src.join(", "));
                css.push(';');
                css.push_str(newline);
            }

            css.push('}');
            css.push_str(newline);
            css.push_str(newline);
        }

        // Rules
        for rule in &ast.rules {
            css.push_str(&self.format_rule(rule));
            css.push_str(newline);
        }

        // Media queries
        for mq in &ast.media_queries {
            css.push_str("@media");
            css.push_str(space);
            css.push_str(&mq.condition);
            css.push_str(space);
            css.push('{');
            css.push_str(newline);

            for rule in &mq.rules {
                css.push_str(&self.indent);
                css.push_str(&self.format_rule(rule).replace('\n', &format!("\n{}", self.indent)));
                css.push_str(newline);
            }

            css.push('}');
            css.push_str(newline);
            css.push_str(newline);
        }

        // Keyframes
        for kf in &ast.keyframes {
            css.push_str("@keyframes");
            css.push_str(space);
            css.push_str(&kf.name);
            css.push_str(space);
            css.push('{');
            css.push_str(newline);

            for frame in &kf.frames {
                css.push_str(&self.indent);
                css.push_str(&frame.position);
                css.push_str(space);
                css.push('{');
                css.push_str(newline);

                for (prop, val) in &frame.properties {
                    css.push_str(&self.indent);
                    css.push_str(&self.indent);
                    css.push_str(prop);
                    css.push(':');
                    css.push_str(space);
                    css.push_str(val);
                    css.push(';');
                    css.push_str(newline);
                }

                css.push_str(&self.indent);
                css.push('}');
                css.push_str(newline);
            }

            css.push('}');
            css.push_str(newline);
            css.push_str(newline);
        }

        css
    }

    fn format_rule(&self, rule: &CssRule) -> String {
        let mut css = String::new();
        let newline = if self.minify { "" } else { "\n" };
        let space = if self.minify { "" } else { " " };

        css.push_str(&rule.selector);
        css.push_str(space);
        css.push('{');
        css.push_str(newline);

        // Sort declarations for consistent output
        let mut declarations: Vec<_> = rule.declarations.iter().collect();
        declarations.sort_by_key(|(k, _)| *k);

        for (prop, val) in declarations {
            css.push_str(&self.indent);
            css.push_str(prop);
            css.push(':');
            css.push_str(space);
            css.push_str(val);
            css.push(';');
            css.push_str(newline);
        }

        css.push('}');
        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_value() {
        assert_eq!(minify_value("0px"), "0");
        assert_eq!(minify_value("10px 0px 5px 0px"), "10px 0 5px 0");
        assert_eq!(minify_value("#ffffff"), "#fff");
        assert_eq!(minify_value("#AABBCC"), "#abc");
    }

    #[test]
    fn test_specificity() {
        let s = calculate_specificity_from_selector("div");
        assert_eq!(s.elements, 1);

        let s = calculate_specificity_from_selector(".class");
        assert_eq!(s.classes, 1);

        let s = calculate_specificity_from_selector("#id");
        assert_eq!(s.ids, 1);

        let s = calculate_specificity_from_selector("div.class#id");
        assert_eq!(s.ids, 1);
        assert_eq!(s.classes, 1);
    }
}
