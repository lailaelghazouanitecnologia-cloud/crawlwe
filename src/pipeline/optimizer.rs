//! Optimizer - Optimize parsed content for output
//!
//! - CSS: Remove unused rules, deduplicate, minify
//! - HTML: Clean structure, remove junk, format
//! - JS: Extract essential code
//! - Output: Format nicely for ML training

use std::collections::{HashMap, HashSet};
use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    // CSS optimization patterns
    static ref CSS_COMMENT: Regex = Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    static ref CSS_WHITESPACE: Regex = Regex::new(r"\s+").unwrap();
    static ref CSS_EMPTY_RULE: Regex = Regex::new(r"[^{}]+\{\s*\}").unwrap();

    // HTML optimization patterns
    static ref HTML_COMMENT: Regex = Regex::new(r"<!--[\s\S]*?-->").unwrap();
    static ref HTML_SCRIPT_TAG: Regex = Regex::new(r"<script[\s\S]*?</script>").unwrap();
    static ref HTML_STYLE_TAG: Regex = Regex::new(r"<style[\s\S]*?</style>").unwrap();
    static ref HTML_WHITESPACE: Regex = Regex::new(r">\s+<").unwrap();
    static ref HTML_DATA_ATTR: Regex = Regex::new(r#"\s+data-[a-zA-Z0-9_-]+="[^"]*""#).unwrap();
    static ref HTML_EMPTY_ATTR: Regex = Regex::new(r#"\s+\w+=""\s*"#).unwrap();

    // Framework-specific cleanup
    static ref REACT_ATTRS: Regex = Regex::new(r#"\s+(?:data-reactroot|data-reactid|data-react-\w+)="[^"]*""#).unwrap();
    static ref VUE_ATTRS: Regex = Regex::new(r#"\s+(?:data-v-[a-f0-9]+|v-\w+(?::[^=]+)?(?:="[^"]*")?)"#).unwrap();
    static ref ANGULAR_ATTRS: Regex = Regex::new(r#"\s+(?:_ngcontent-[^=]+|ng-\w+)(?:="[^"]*")?"#).unwrap();
    static ref NEXT_ATTRS: Regex = Regex::new(r#"\s+(?:data-nscript|data-n-\w+)="[^"]*""#).unwrap();
}

// ============================================================================
// CSS OPTIMIZER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssOptimizeOptions {
    pub remove_comments: bool,
    pub remove_empty_rules: bool,
    pub deduplicate_rules: bool,
    pub minify: bool,
    pub sort_properties: bool,
    pub merge_media_queries: bool,
    pub remove_unused: bool,
    pub used_classes: Option<HashSet<String>>,
}

impl Default for CssOptimizeOptions {
    fn default() -> Self {
        Self {
            remove_comments: true,
            remove_empty_rules: true,
            deduplicate_rules: true,
            minify: false,
            sort_properties: false,
            merge_media_queries: false,
            remove_unused: false,
            used_classes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssOptimizeResult {
    pub css: String,
    pub stats: CssOptimizeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CssOptimizeStats {
    pub original_size: usize,
    pub optimized_size: usize,
    pub rules_removed: usize,
    pub duplicates_removed: usize,
    pub comments_removed: usize,
}

pub struct CssOptimizer;

impl CssOptimizer {
    pub fn optimize(css: &str, options: &CssOptimizeOptions) -> CssOptimizeResult {
        let mut result = css.to_string();
        let mut stats = CssOptimizeStats {
            original_size: css.len(),
            ..Default::default()
        };

        // Remove comments
        if options.remove_comments {
            let before = result.matches("/*").count();
            result = CSS_COMMENT.replace_all(&result, "").to_string();
            stats.comments_removed = before - result.matches("/*").count();
        }

        // Remove empty rules
        if options.remove_empty_rules {
            let before_len = result.len();
            result = CSS_EMPTY_RULE.replace_all(&result, "").to_string();
            if result.len() < before_len {
                stats.rules_removed += 1;
            }
        }

        // Deduplicate rules
        if options.deduplicate_rules {
            result = Self::deduplicate_rules(&result, &mut stats);
        }

        // Minify
        if options.minify {
            result = Self::minify(&result);
        } else {
            // Just normalize whitespace
            result = Self::format(&result);
        }

        stats.optimized_size = result.len();

        CssOptimizeResult { css: result, stats }
    }

    fn deduplicate_rules(css: &str, stats: &mut CssOptimizeStats) -> String {
        let mut seen: HashMap<String, String> = HashMap::new();
        let mut output = String::new();

        // Simple rule-based deduplication
        // This is a simplified version - a real implementation would parse properly
        let rule_re = Regex::new(r"([^{}]+)\{([^{}]+)\}").unwrap();

        for cap in rule_re.captures_iter(css) {
            let selector = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let body = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            let key = format!("{}{{{}}}", selector, body);
            if !seen.contains_key(&key) {
                seen.insert(key, selector.to_string());
                output.push_str(&format!("{} {{\n  {}\n}}\n\n", selector, body));
            } else {
                stats.duplicates_removed += 1;
            }
        }

        output
    }

    fn minify(css: &str) -> String {
        let mut result = css.to_string();
        result = CSS_WHITESPACE.replace_all(&result, " ").to_string();
        result = result.replace(" {", "{");
        result = result.replace("{ ", "{");
        result = result.replace(" }", "}");
        result = result.replace("} ", "}");
        result = result.replace(": ", ":");
        result = result.replace("; ", ";");
        result = result.replace(";\n", ";");
        result = result.replace("\n", "");
        result
    }

    fn format(css: &str) -> String {
        let mut result = String::new();
        let mut indent: usize = 0;

        for line in css.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.ends_with('}') {
                indent = indent.saturating_sub(1);
            }

            result.push_str(&"  ".repeat(indent));
            result.push_str(trimmed);
            result.push('\n');

            if trimmed.ends_with('{') {
                indent += 1;
            }
        }

        result
    }
}

// ============================================================================
// HTML OPTIMIZER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlOptimizeOptions {
    pub remove_comments: bool,
    pub remove_scripts: bool,
    pub remove_styles: bool,
    pub remove_data_attrs: bool,
    pub remove_framework_attrs: bool,
    pub remove_empty_attrs: bool,
    pub minify_whitespace: bool,
    pub format_output: bool,
    pub preserve_structure: bool,
}

impl Default for HtmlOptimizeOptions {
    fn default() -> Self {
        Self {
            remove_comments: true,
            remove_scripts: true,
            remove_styles: true,
            remove_data_attrs: false,
            remove_framework_attrs: true,
            remove_empty_attrs: true,
            minify_whitespace: false,
            format_output: true,
            preserve_structure: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlOptimizeResult {
    pub html: String,
    pub stats: HtmlOptimizeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HtmlOptimizeStats {
    pub original_size: usize,
    pub optimized_size: usize,
    pub scripts_removed: usize,
    pub styles_removed: usize,
    pub comments_removed: usize,
    pub attrs_removed: usize,
}

pub struct HtmlOptimizer;

impl HtmlOptimizer {
    pub fn optimize(html: &str, options: &HtmlOptimizeOptions) -> HtmlOptimizeResult {
        let mut result = html.to_string();
        let mut stats = HtmlOptimizeStats {
            original_size: html.len(),
            ..Default::default()
        };

        // Remove comments
        if options.remove_comments {
            let before = result.matches("<!--").count();
            result = HTML_COMMENT.replace_all(&result, "").to_string();
            stats.comments_removed = before - result.matches("<!--").count();
        }

        // Remove script tags
        if options.remove_scripts {
            let before = result.matches("<script").count();
            result = HTML_SCRIPT_TAG.replace_all(&result, "").to_string();
            stats.scripts_removed = before - result.matches("<script").count();
        }

        // Remove style tags
        if options.remove_styles {
            let before = result.matches("<style").count();
            result = HTML_STYLE_TAG.replace_all(&result, "").to_string();
            stats.styles_removed = before - result.matches("<style").count();
        }

        // Remove framework-specific attributes
        if options.remove_framework_attrs {
            let before_len = result.len();
            result = REACT_ATTRS.replace_all(&result, "").to_string();
            result = VUE_ATTRS.replace_all(&result, "").to_string();
            result = ANGULAR_ATTRS.replace_all(&result, "").to_string();
            result = NEXT_ATTRS.replace_all(&result, "").to_string();
            stats.attrs_removed += (before_len - result.len()) / 20; // Rough estimate
        }

        // Remove data attributes
        if options.remove_data_attrs {
            let before_len = result.len();
            result = HTML_DATA_ATTR.replace_all(&result, "").to_string();
            stats.attrs_removed += (before_len - result.len()) / 15;
        }

        // Remove empty attributes
        if options.remove_empty_attrs {
            result = HTML_EMPTY_ATTR.replace_all(&result, " ").to_string();
        }

        // Minify whitespace
        if options.minify_whitespace {
            result = HTML_WHITESPACE.replace_all(&result, "><").to_string();
        }

        // Format output
        if options.format_output {
            result = Self::format(&result);
        }

        stats.optimized_size = result.len();

        HtmlOptimizeResult { html: result, stats }
    }

    fn format(html: &str) -> String {
        let mut result = String::new();
        let mut indent: usize = 0;
        let void_elements = ["area", "base", "br", "col", "embed", "hr", "img", "input",
                            "link", "meta", "param", "source", "track", "wbr"];

        // Simple line-by-line formatting
        for line in html.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check for closing tag
            if trimmed.starts_with("</") {
                indent = indent.saturating_sub(1);
            }

            result.push_str(&"  ".repeat(indent));
            result.push_str(trimmed);
            result.push('\n');

            // Check for opening tag (not void or self-closing)
            if trimmed.starts_with('<') && !trimmed.starts_with("</") && !trimmed.starts_with("<!") {
                let tag_end = trimmed.find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(trimmed.len());
                let tag_name = &trimmed[1..tag_end].to_lowercase();

                if !void_elements.contains(&tag_name.as_str())
                    && !trimmed.ends_with("/>")
                    && !trimmed.contains(&format!("</{}>", tag_name))
                {
                    indent += 1;
                }
            }
        }

        result
    }
}

// ============================================================================
// OUTPUT FORMATTER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    pub indent: String,
    pub line_ending: String,
    pub max_line_width: usize,
    pub sort_attributes: bool,
    pub quote_style: QuoteStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuoteStyle {
    Double,
    Single,
    Auto,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self {
            indent: "  ".to_string(),
            line_ending: "\n".to_string(),
            max_line_width: 120,
            sort_attributes: false,
            quote_style: QuoteStyle::Double,
        }
    }
}

pub struct OutputFormatter;

impl OutputFormatter {
    /// Format the final output package
    pub fn format_package(
        html: &str,
        css: &str,
        js_libs: &[String],
        format: &OutputFormat,
    ) -> FormattedOutput {
        FormattedOutput {
            html: Self::format_html(html, format),
            css: Self::format_css(css, format),
            dependencies: Self::format_dependencies(js_libs),
        }
    }

    fn format_html(html: &str, format: &OutputFormat) -> String {
        let mut lines: Vec<String> = Vec::new();
        let mut current_indent: usize = 0;

        for line in html.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Decrease indent for closing tags
            if trimmed.starts_with("</") {
                current_indent = current_indent.saturating_sub(1);
            }

            // Add indented line
            let indented = format!("{}{}", format.indent.repeat(current_indent), trimmed);

            // Handle long lines
            if indented.len() > format.max_line_width && trimmed.contains(' ') {
                lines.push(Self::wrap_long_line(&indented, format));
            } else {
                lines.push(indented);
            }

            // Increase indent for opening tags
            if trimmed.starts_with('<')
                && !trimmed.starts_with("</")
                && !trimmed.starts_with("<!")
                && !trimmed.ends_with("/>")
                && !Self::is_void_element(trimmed)
            {
                current_indent += 1;
            }
        }

        lines.join(&format.line_ending)
    }

    fn format_css(css: &str, format: &OutputFormat) -> String {
        let mut lines: Vec<String> = Vec::new();
        let mut current_indent: usize = 0;

        for line in css.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('}') {
                current_indent = current_indent.saturating_sub(1);
            }

            let indented = format!("{}{}", format.indent.repeat(current_indent), trimmed);
            lines.push(indented);

            if trimmed.ends_with('{') {
                current_indent += 1;
            }
        }

        lines.join(&format.line_ending)
    }

    fn format_dependencies(libs: &[String]) -> Vec<String> {
        libs.iter()
            .map(|lib| lib.trim().to_string())
            .filter(|lib| !lib.is_empty())
            .collect()
    }

    fn wrap_long_line(line: &str, format: &OutputFormat) -> String {
        if line.len() <= format.max_line_width {
            return line.to_string();
        }

        // Simple wrapping at spaces
        let indent = line.len() - line.trim_start().len();
        let continuation_indent = format!("{}{}", " ".repeat(indent), &format.indent);

        let mut result = String::new();
        let mut current_line = String::new();

        for word in line.split_whitespace() {
            if current_line.is_empty() {
                current_line = " ".repeat(indent) + word;
            } else if current_line.len() + word.len() + 1 <= format.max_line_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                result.push_str(&current_line);
                result.push_str(&format.line_ending);
                current_line = continuation_indent.clone() + word;
            }
        }

        if !current_line.is_empty() {
            result.push_str(&current_line);
        }

        result
    }

    fn is_void_element(tag: &str) -> bool {
        let void_elements = ["area", "base", "br", "col", "embed", "hr", "img", "input",
                           "link", "meta", "param", "source", "track", "wbr"];
        for elem in void_elements {
            if tag.starts_with(&format!("<{}", elem)) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedOutput {
    pub html: String,
    pub css: String,
    pub dependencies: Vec<String>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_optimizer() {
        let css = r#"
            /* Comment to remove */
            .button { color: red; }
            .button { color: red; }  /* Duplicate */
            .empty { }
        "#;

        let result = CssOptimizer::optimize(css, &CssOptimizeOptions::default());
        assert!(result.stats.comments_removed > 0);
        assert!(result.stats.duplicates_removed > 0);
    }

    #[test]
    fn test_html_optimizer() {
        let html = r#"
            <!-- Comment -->
            <div data-reactroot="" data-v-abc123>
                <script>alert('x')</script>
                <p>Content</p>
            </div>
        "#;

        let result = HtmlOptimizer::optimize(html, &HtmlOptimizeOptions::default());
        assert!(!result.html.contains("<!--"));
        assert!(!result.html.contains("<script"));
        assert!(!result.html.contains("data-reactroot"));
    }
}
