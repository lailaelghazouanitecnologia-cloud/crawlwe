//! CSS Parser using regex-based analysis
//!
//! Provides CSS parsing, rule extraction, and optimization
//! without external CSS parser dependencies.

use crate::models::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Error, Debug)]
pub enum CssParserError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    #[error("Optimization error: {0}")]
    OptimizationError(String),
}

lazy_static! {
    static ref RULE_RE: Regex = Regex::new(r"([^{}]+)\{([^{}]*)\}").unwrap();
    static ref DECL_RE: Regex = Regex::new(r"([^:]+):([^;]+);?").unwrap();
    static ref KEYFRAMES_RE: Regex = Regex::new(r"@keyframes\s+([^\s{]+)\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}").unwrap();
    static ref MEDIA_RE: Regex = Regex::new(r"@media\s*([^{]+)\{((?:[^{}]*\{[^{}]*\})*[^{}]*)\}").unwrap();
    static ref FONTFACE_RE: Regex = Regex::new(r"@font-face\s*\{([^}]+)\}").unwrap();
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

        // Remove comments
        let css_clean = remove_css_comments(css_text);

        // Extract keyframes first
        for cap in KEYFRAMES_RE.captures_iter(&css_clean) {
            let name = cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
            let body = cap.get(2).map_or("", |m| m.as_str());

            let mut frames = Vec::new();
            for rule_cap in RULE_RE.captures_iter(body) {
                let position = rule_cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
                let decls = rule_cap.get(2).map_or("", |m| m.as_str());

                let mut properties = HashMap::new();
                for decl_cap in DECL_RE.captures_iter(decls) {
                    let prop = decl_cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
                    let val = decl_cap.get(2).map_or("", |m| m.as_str()).trim().to_string();
                    properties.insert(prop, val);
                }

                frames.push(KeyframeStep { position, properties });
            }

            ast.keyframes.push(Keyframes { name, frames });
        }

        // Extract media queries
        for cap in MEDIA_RE.captures_iter(&css_clean) {
            let condition = cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
            let body = cap.get(2).map_or("", |m| m.as_str());

            let mut rules = Vec::new();
            for rule_cap in RULE_RE.captures_iter(body) {
                let selector = rule_cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
                let decls = rule_cap.get(2).map_or("", |m| m.as_str());

                if selector.starts_with('@') {
                    continue; // Skip nested at-rules
                }

                let mut declarations = HashMap::new();
                for decl_cap in DECL_RE.captures_iter(decls) {
                    let prop = decl_cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
                    let val = decl_cap.get(2).map_or("", |m| m.as_str()).trim().to_string();
                    declarations.insert(prop, val);
                }

                rules.push(CssRule {
                    selector,
                    declarations,
                    specificity: Specificity::default(),
                    source_file: None,
                    line: None,
                });
            }

            ast.media_queries.push(MediaQuery { condition, rules });
        }

        // Extract font-faces
        for cap in FONTFACE_RE.captures_iter(&css_clean) {
            let body = cap.get(1).map_or("", |m| m.as_str());

            let mut family = String::new();
            let mut src = Vec::new();
            let mut weight = None;
            let mut style = None;
            let mut display = None;

            for decl_cap in DECL_RE.captures_iter(body) {
                let prop = decl_cap.get(1).map_or("", |m| m.as_str()).trim().to_lowercase();
                let val = decl_cap.get(2).map_or("", |m| m.as_str()).trim().to_string();

                match prop.as_str() {
                    "font-family" => family = val.trim_matches(|c| c == '"' || c == '\'').to_string(),
                    "src" => src.push(val),
                    "font-weight" => weight = Some(val),
                    "font-style" => style = Some(val),
                    "font-display" => display = Some(val),
                    _ => {}
                }
            }

            ast.font_faces.push(FontFace { family, src, weight, style, display });
        }

        // Extract regular rules (excluding @-rules)
        let css_no_atrules = remove_at_rules(&css_clean);

        for cap in RULE_RE.captures_iter(&css_no_atrules) {
            let selector = cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
            let decls = cap.get(2).map_or("", |m| m.as_str());

            if selector.starts_with('@') || selector.is_empty() {
                continue;
            }

            let mut declarations = HashMap::new();
            for decl_cap in DECL_RE.captures_iter(decls) {
                let prop = decl_cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
                let val = decl_cap.get(2).map_or("", |m| m.as_str()).trim().to_string();

                // Extract CSS variables
                if prop.starts_with("--") {
                    ast.variables.push(CssVariable {
                        name: prop.clone(),
                        value: val.clone(),
                        scope: selector.clone(),
                    });
                }

                declarations.insert(prop, val);
            }

            let specificity = calculate_specificity(&selector);

            ast.rules.push(CssRule {
                selector,
                declarations,
                specificity,
                source_file: None,
                line: None,
            });
        }

        Ok(ast)
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

/// Remove CSS comments
fn remove_css_comments(css: &str) -> String {
    let mut result = String::new();
    let mut chars = css.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next(); // consume *
            // Skip until */
            while let Some(c2) = chars.next() {
                if c2 == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Remove @-rules for simpler parsing of regular rules
fn remove_at_rules(css: &str) -> String {
    let mut result = css.to_string();

    // Remove @keyframes blocks
    result = KEYFRAMES_RE.replace_all(&result, "").to_string();
    // Remove @media blocks
    result = MEDIA_RE.replace_all(&result, "").to_string();
    // Remove @font-face blocks
    result = FONTFACE_RE.replace_all(&result, "").to_string();

    result
}

/// Calculate specificity from selector string
fn calculate_specificity(selector: &str) -> Specificity {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;

    for part in selector.split(',') {
        let part = part.trim();

        // Count IDs (#)
        ids += part.matches('#').count() as u32;

        // Count classes (.)
        classes += part.matches('.').count() as u32;
        // Count attributes ([)
        classes += part.matches('[').count() as u32;

        // Pseudo-classes (single :)
        let single_colons = part.matches(':').count();
        let double_colons = part.matches("::").count();
        classes += (single_colons.saturating_sub(double_colons * 2)) as u32;

        // Count elements
        let tokens: Vec<&str> = part
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        for token in tokens {
            if token.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
                && token.to_lowercase() == *token
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
    pub deduplicate: bool,
    pub merge_identical: bool,
    pub sort_by_specificity: bool,
    pub minify_values: bool,
}

impl Default for CssOptimizer {
    fn default() -> Self {
        Self {
            deduplicate: true,
            merge_identical: true,
            sort_by_specificity: true,
            minify_values: true,
        }
    }
}

impl CssOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn optimize(&self, mut ast: CssAst) -> CssAst {
        if self.deduplicate {
            ast = self.deduplicate_rules(ast);
        }

        if self.merge_identical {
            ast = self.merge_identical_rules(ast);
        }

        if self.sort_by_specificity {
            ast.rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));
        }

        if self.minify_values {
            ast = self.minify_css_values(ast);
        }

        ast
    }

    fn deduplicate_rules(&self, mut ast: CssAst) -> CssAst {
        let mut seen: HashSet<String> = HashSet::new();
        ast.rules.retain(|rule| {
            let key = format!("{}|{:?}", rule.selector, rule.declarations);
            seen.insert(key)
        });
        ast
    }

    fn merge_identical_rules(&self, mut ast: CssAst) -> CssAst {
        let mut declaration_map: HashMap<String, Vec<String>> = HashMap::new();

        for rule in &ast.rules {
            let decl_key = format!("{:?}", rule.declarations);
            declaration_map
                .entry(decl_key)
                .or_default()
                .push(rule.selector.clone());
        }

        let mut new_rules = Vec::new();
        let mut processed_selectors: HashSet<String> = HashSet::new();

        for rule in &ast.rules {
            if processed_selectors.contains(&rule.selector) {
                continue;
            }

            let decl_key = format!("{:?}", rule.declarations);
            if let Some(selectors) = declaration_map.get(&decl_key) {
                if selectors.len() > 1 {
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

    fn minify_css_values(&self, mut ast: CssAst) -> CssAst {
        for rule in &mut ast.rules {
            for value in rule.declarations.values_mut() {
                *value = minify_value(value);
            }
        }
        ast
    }
}

fn minify_value(value: &str) -> String {
    let mut result = value.to_string();

    result = result.replace("0px", "0");
    result = result.replace("0em", "0");
    result = result.replace("0rem", "0");
    result = result.replace("0%", "0");
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    // #ffffff -> #fff
    if result.starts_with('#') && result.len() == 7 {
        result = result.to_lowercase();
        let chars: Vec<char> = result.chars().collect();
        if chars[1] == chars[2] && chars[3] == chars[4] && chars[5] == chars[6] {
            result = format!("#{}{}{}", chars[1], chars[3], chars[5]);
        }
    }

    result
}

/// CSS Generator - converts AST back to CSS string
pub struct CssGenerator {
    pub indent: String,
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

    pub fn generate(&self, ast: &CssAst) -> String {
        let mut css = String::new();
        let newline = if self.minify { "" } else { "\n" };
        let space = if self.minify { "" } else { " " };

        // CSS Variables in :root
        if !ast.variables.is_empty() {
            css.push_str(":root {");
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
            css.push_str("@font-face {");
            css.push_str(newline);
            css.push_str(&self.indent);
            css.push_str("font-family: '");
            css.push_str(&font.family);
            css.push_str("';");
            css.push_str(newline);
            if let Some(weight) = &font.weight {
                css.push_str(&self.indent);
                css.push_str("font-weight: ");
                css.push_str(weight);
                css.push(';');
                css.push_str(newline);
            }
            if !font.src.is_empty() {
                css.push_str(&self.indent);
                css.push_str("src: ");
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
            css.push_str("@media ");
            css.push_str(&mq.condition);
            css.push_str(" {");
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
            css.push_str("@keyframes ");
            css.push_str(&kf.name);
            css.push_str(" {");
            css.push_str(newline);
            for frame in &kf.frames {
                css.push_str(&self.indent);
                css.push_str(&frame.position);
                css.push_str(" {");
                css.push_str(newline);
                for (prop, val) in &frame.properties {
                    css.push_str(&self.indent);
                    css.push_str(&self.indent);
                    css.push_str(prop);
                    css.push_str(": ");
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
        assert_eq!(minify_value("#ffffff"), "#fff");
    }

    #[test]
    fn test_parse_simple_css() {
        let css = ".button { color: red; padding: 10px; }";
        let ast = CssAst::parse(css).unwrap();
        assert_eq!(ast.rules.len(), 1);
        assert_eq!(ast.rules[0].selector, ".button");
    }
}
