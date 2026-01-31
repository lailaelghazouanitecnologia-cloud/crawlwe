//! CSS Formatter - Formats CSS for readability
//!
//! Converts minified CSS into properly formatted, readable CSS with:
//! - Proper indentation
//! - One property per line
//! - Organized sections
//! - Color coding comments

use regex::Regex;
use std::collections::HashMap;

/// CSS Formatter configuration
#[derive(Clone)]
pub struct CssFormatOptions {
    /// Number of spaces for indentation
    pub indent_size: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    /// Add blank lines between rule blocks
    pub blank_lines_between_rules: bool,
    /// Sort properties alphabetically
    pub sort_properties: bool,
    /// Add section comments
    pub add_section_comments: bool,
    /// Max line length before wrapping (0 = no wrap)
    pub max_line_length: usize,
    /// Group related properties
    pub group_properties: bool,
}

impl Default for CssFormatOptions {
    fn default() -> Self {
        Self {
            indent_size: 2,
            use_tabs: false,
            blank_lines_between_rules: true,
            sort_properties: false,
            add_section_comments: true,
            max_line_length: 0,
            group_properties: true,
        }
    }
}

/// CSS Formatter result
pub struct CssFormatResult {
    pub css: String,
    pub stats: FormatStats,
}

/// Formatting statistics
#[derive(Default)]
pub struct FormatStats {
    pub rules_formatted: usize,
    pub properties_formatted: usize,
    pub keyframes_formatted: usize,
    pub media_queries_formatted: usize,
    pub font_faces_formatted: usize,
    pub variables_found: usize,
    pub original_size: usize,
    pub formatted_size: usize,
}

/// CSS Formatter
pub struct CssFormatter {
    options: CssFormatOptions,
    // Property groups for organization
    property_groups: HashMap<&'static str, Vec<&'static str>>,
}

impl CssFormatter {
    pub fn new() -> Self {
        Self::with_options(CssFormatOptions::default())
    }

    pub fn with_options(options: CssFormatOptions) -> Self {
        let mut property_groups = HashMap::new();

        // Layout & Box Model
        property_groups.insert("layout", vec![
            "display", "position", "top", "right", "bottom", "left",
            "float", "clear", "z-index", "overflow", "overflow-x", "overflow-y",
            "visibility", "clip", "clip-path",
        ]);

        // Flexbox
        property_groups.insert("flexbox", vec![
            "flex", "flex-direction", "flex-wrap", "flex-flow",
            "flex-grow", "flex-shrink", "flex-basis",
            "justify-content", "align-items", "align-content", "align-self",
            "order", "gap", "row-gap", "column-gap",
        ]);

        // Grid
        property_groups.insert("grid", vec![
            "grid", "grid-template", "grid-template-columns", "grid-template-rows",
            "grid-template-areas", "grid-auto-columns", "grid-auto-rows",
            "grid-auto-flow", "grid-column", "grid-row", "grid-area",
            "place-items", "place-content", "place-self",
        ]);

        // Box Model
        property_groups.insert("box-model", vec![
            "width", "min-width", "max-width",
            "height", "min-height", "max-height",
            "margin", "margin-top", "margin-right", "margin-bottom", "margin-left",
            "padding", "padding-top", "padding-right", "padding-bottom", "padding-left",
            "box-sizing",
        ]);

        // Typography
        property_groups.insert("typography", vec![
            "font", "font-family", "font-size", "font-weight", "font-style",
            "font-variant", "font-stretch", "line-height", "letter-spacing",
            "word-spacing", "text-align", "text-decoration", "text-transform",
            "text-indent", "text-shadow", "white-space", "word-break", "word-wrap",
            "vertical-align",
        ]);

        // Colors & Background
        property_groups.insert("visual", vec![
            "color", "background", "background-color", "background-image",
            "background-position", "background-size", "background-repeat",
            "background-attachment", "background-clip", "background-origin",
            "opacity",
        ]);

        // Borders
        property_groups.insert("border", vec![
            "border", "border-width", "border-style", "border-color",
            "border-top", "border-right", "border-bottom", "border-left",
            "border-radius", "border-top-left-radius", "border-top-right-radius",
            "border-bottom-left-radius", "border-bottom-right-radius",
            "outline", "outline-width", "outline-style", "outline-color", "outline-offset",
        ]);

        // Effects
        property_groups.insert("effects", vec![
            "box-shadow", "filter", "backdrop-filter", "mix-blend-mode",
            "isolation", "object-fit", "object-position",
        ]);

        // Transforms
        property_groups.insert("transform", vec![
            "transform", "transform-origin", "transform-style",
            "perspective", "perspective-origin", "backface-visibility",
        ]);

        // Transitions & Animations
        property_groups.insert("animation", vec![
            "transition", "transition-property", "transition-duration",
            "transition-timing-function", "transition-delay",
            "animation", "animation-name", "animation-duration",
            "animation-timing-function", "animation-delay", "animation-iteration-count",
            "animation-direction", "animation-fill-mode", "animation-play-state",
        ]);

        // Other
        property_groups.insert("other", vec![
            "cursor", "pointer-events", "user-select", "resize",
            "content", "quotes", "counter-reset", "counter-increment",
            "will-change", "contain", "appearance",
        ]);

        Self {
            options,
            property_groups,
        }
    }

    /// Format CSS string
    pub fn format(&self, css: &str) -> CssFormatResult {
        let mut stats = FormatStats {
            original_size: css.len(),
            ..Default::default()
        };

        let mut output = String::with_capacity(css.len() * 2);
        let indent = self.get_indent();

        // First, normalize the CSS
        let normalized = self.normalize_css(css);

        // Parse and format different blocks
        let mut pos = 0;
        let chars: Vec<char> = normalized.chars().collect();
        let len = chars.len();

        while pos < len {
            // Skip whitespace
            while pos < len && chars[pos].is_whitespace() {
                pos += 1;
            }
            if pos >= len {
                break;
            }

            // Check for comments
            if pos + 1 < len && chars[pos] == '/' && chars[pos + 1] == '*' {
                let comment_end = self.find_comment_end(&chars, pos);
                let comment: String = chars[pos..comment_end].iter().collect();
                output.push_str(&self.format_comment(&comment));
                output.push('\n');
                if self.options.blank_lines_between_rules {
                    output.push('\n');
                }
                pos = comment_end;
                continue;
            }

            // Find the start of the block
            let block_start = self.find_next(&chars, pos, '{');
            if block_start.is_none() {
                // No more blocks, append rest as-is
                let rest: String = chars[pos..].iter().collect();
                output.push_str(rest.trim());
                break;
            }
            let block_start = block_start.unwrap();

            // Get the selector/at-rule
            let selector: String = chars[pos..block_start].iter().collect();
            let selector = selector.trim();

            // Find matching closing brace
            let block_end = self.find_matching_brace(&chars, block_start);
            let block_content: String = chars[block_start + 1..block_end].iter().collect();

            // Format based on type
            if selector.starts_with("@keyframes") || selector.starts_with("@-webkit-keyframes") {
                output.push_str(&self.format_keyframes(selector, &block_content, &indent));
                stats.keyframes_formatted += 1;
            } else if selector.starts_with("@media") || selector.starts_with("@supports") {
                output.push_str(&self.format_media_query(selector, &block_content, &indent));
                stats.media_queries_formatted += 1;
            } else if selector.starts_with("@font-face") {
                output.push_str(&self.format_font_face(&block_content, &indent));
                stats.font_faces_formatted += 1;
            } else if selector.starts_with("@property") {
                output.push_str(&self.format_property_rule(selector, &block_content, &indent));
            } else if selector.starts_with("@import") || selector.starts_with("@charset") {
                output.push_str(selector);
                output.push_str(" {\n");
                output.push_str(&block_content.trim());
                output.push_str("\n}\n");
            } else if selector.starts_with(":root") || selector.contains("--") {
                let (formatted, var_count) = self.format_variables(selector, &block_content, &indent);
                output.push_str(&formatted);
                stats.variables_found += var_count;
                stats.rules_formatted += 1;
            } else {
                // Regular rule
                let (formatted, prop_count) = self.format_rule(selector, &block_content, &indent);
                output.push_str(&formatted);
                stats.properties_formatted += prop_count;
                stats.rules_formatted += 1;
            }

            if self.options.blank_lines_between_rules {
                output.push('\n');
            }

            pos = block_end + 1;
        }

        stats.formatted_size = output.len();

        CssFormatResult {
            css: output.trim().to_string(),
            stats,
        }
    }

    /// Get the indent string
    fn get_indent(&self) -> String {
        if self.options.use_tabs {
            "\t".to_string()
        } else {
            " ".repeat(self.options.indent_size)
        }
    }

    /// Normalize CSS - add spaces around braces, etc.
    fn normalize_css(&self, css: &str) -> String {
        let mut result = css.to_string();

        // Normalize line endings
        result = result.replace("\r\n", "\n").replace("\r", "\n");

        // Remove excessive whitespace but preserve structure
        let re = Regex::new(r"\s+").unwrap();
        result = re.replace_all(&result, " ").to_string();

        // Add newlines after braces for easier parsing
        result = result.replace("}", "}\n");
        result = result.replace("{", " {\n");

        result
    }

    /// Find the end of a comment
    fn find_comment_end(&self, chars: &[char], start: usize) -> usize {
        let mut pos = start + 2;
        while pos + 1 < chars.len() {
            if chars[pos] == '*' && chars[pos + 1] == '/' {
                return pos + 2;
            }
            pos += 1;
        }
        chars.len()
    }

    /// Find next occurrence of a character
    fn find_next(&self, chars: &[char], start: usize, target: char) -> Option<usize> {
        let mut pos = start;
        let mut in_string = false;
        let mut string_char = '"';

        while pos < chars.len() {
            let c = chars[pos];

            // Handle strings
            if (c == '"' || c == '\'') && (pos == 0 || chars[pos - 1] != '\\') {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char {
                    in_string = false;
                }
            }

            if !in_string && c == target {
                return Some(pos);
            }
            pos += 1;
        }
        None
    }

    /// Find matching closing brace
    fn find_matching_brace(&self, chars: &[char], start: usize) -> usize {
        let mut depth = 1;
        let mut pos = start + 1;
        let mut in_string = false;
        let mut string_char = '"';

        while pos < chars.len() && depth > 0 {
            let c = chars[pos];

            // Handle strings
            if (c == '"' || c == '\'') && (pos == 0 || chars[pos - 1] != '\\') {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char {
                    in_string = false;
                }
            }

            if !in_string {
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                }
            }
            pos += 1;
        }
        pos - 1
    }

    /// Format a comment
    fn format_comment(&self, comment: &str) -> String {
        let trimmed = comment.trim();

        // Check if it's a section comment
        if trimmed.contains("===") || trimmed.contains("---") {
            return format!("\n{}", trimmed);
        }

        // Multi-line comment
        if trimmed.contains('\n') {
            return trimmed.to_string();
        }

        trimmed.to_string()
    }

    /// Format a regular CSS rule
    fn format_rule(&self, selector: &str, content: &str, indent: &str) -> (String, usize) {
        let mut output = String::new();

        // Format selector
        let formatted_selector = self.format_selector(selector);
        output.push_str(&formatted_selector);
        output.push_str(" {\n");

        // Parse and format properties
        let properties = self.parse_properties(content);
        let prop_count = properties.len();

        // Optionally sort or group properties
        let formatted_props = if self.options.sort_properties {
            let mut sorted = properties.clone();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            sorted
        } else if self.options.group_properties {
            self.group_properties_list(&properties)
        } else {
            properties
        };

        for (prop, value) in &formatted_props {
            output.push_str(indent);
            output.push_str(prop);
            output.push_str(": ");
            output.push_str(&self.format_value(value));
            output.push_str(";\n");
        }

        output.push_str("}\n");

        (output, prop_count)
    }

    /// Format a selector (handle multiple selectors)
    fn format_selector(&self, selector: &str) -> String {
        let selector = selector.trim();

        // Split by comma and rejoin with proper formatting
        if selector.contains(',') {
            let parts: Vec<&str> = selector.split(',').map(|s| s.trim()).collect();
            if parts.len() > 3 {
                // Multiple selectors - put each on its own line
                parts.join(",\n")
            } else {
                parts.join(", ")
            }
        } else {
            selector.to_string()
        }
    }

    /// Parse properties from a rule content
    fn parse_properties(&self, content: &str) -> Vec<(String, String)> {
        let mut properties = Vec::new();

        // Split by semicolon
        for part in content.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Split by first colon
            if let Some(colon_pos) = part.find(':') {
                let prop = part[..colon_pos].trim().to_string();
                let value = part[colon_pos + 1..].trim().to_string();

                if !prop.is_empty() && !value.is_empty() {
                    properties.push((prop, value));
                }
            }
        }

        properties
    }

    /// Format a property value
    fn format_value(&self, value: &str) -> String {
        let value = value.trim();

        // Format color values
        if value.starts_with('#') && value.len() == 4 {
            // Expand shorthand colors: #fff -> #ffffff
            // (optional, keeping as-is for now)
            return value.to_string();
        }

        // Format multi-value properties
        if value.contains(',') && !value.contains('(') {
            let parts: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
            return parts.join(", ");
        }

        value.to_string()
    }

    /// Group properties by category
    fn group_properties_list(&self, properties: &[(String, String)]) -> Vec<(String, String)> {
        let mut grouped: Vec<(String, String)> = Vec::new();
        let mut remaining: Vec<(String, String)> = properties.to_vec();

        // Process each group in order
        let group_order = [
            "layout", "flexbox", "grid", "box-model",
            "typography", "visual", "border", "effects",
            "transform", "animation", "other"
        ];

        for group_name in &group_order {
            if let Some(group_props) = self.property_groups.get(*group_name) {
                let mut found_in_group: Vec<(String, String)> = Vec::new();

                remaining.retain(|(prop, value)| {
                    let prop_base = prop.split('-').next().unwrap_or(prop);
                    if group_props.contains(&prop.as_str()) ||
                       group_props.iter().any(|gp| prop.starts_with(gp)) {
                        found_in_group.push((prop.clone(), value.clone()));
                        false
                    } else {
                        true
                    }
                });

                grouped.extend(found_in_group);
            }
        }

        // Add any remaining properties
        grouped.extend(remaining);

        grouped
    }

    /// Format @keyframes
    fn format_keyframes(&self, name: &str, content: &str, indent: &str) -> String {
        let mut output = String::new();

        output.push_str(name.trim());
        output.push_str(" {\n");

        // Parse keyframe blocks
        let re = Regex::new(r"(\d+%|from|to)\s*\{([^}]*)\}").unwrap();

        for cap in re.captures_iter(content) {
            let keyframe = cap.get(1).map_or("", |m| m.as_str()).trim();
            let props = cap.get(2).map_or("", |m| m.as_str());

            output.push_str(indent);
            output.push_str(keyframe);
            output.push_str(" {\n");

            // Format properties
            let properties = self.parse_properties(props);
            for (prop, value) in &properties {
                output.push_str(indent);
                output.push_str(indent);
                output.push_str(prop);
                output.push_str(": ");
                output.push_str(&self.format_value(value));
                output.push_str(";\n");
            }

            output.push_str(indent);
            output.push_str("}\n");
        }

        output.push_str("}\n");

        output
    }

    /// Format @media query
    fn format_media_query(&self, query: &str, content: &str, indent: &str) -> String {
        let mut output = String::new();

        output.push_str(query.trim());
        output.push_str(" {\n");

        // Recursively format inner rules
        let inner_formatted = self.format(content);

        // Indent each line
        for line in inner_formatted.css.lines() {
            if !line.trim().is_empty() {
                output.push_str(indent);
                output.push_str(line);
                output.push('\n');
            }
        }

        output.push_str("}\n");

        output
    }

    /// Format @font-face
    fn format_font_face(&self, content: &str, indent: &str) -> String {
        let mut output = String::new();

        output.push_str("@font-face {\n");

        let properties = self.parse_properties(content);

        // Order font-face properties logically
        let order = ["font-family", "src", "font-weight", "font-style", "font-display", "unicode-range"];
        let mut ordered_props: Vec<(String, String)> = Vec::new();

        for prop_name in &order {
            for (prop, value) in &properties {
                if prop == *prop_name {
                    ordered_props.push((prop.clone(), value.clone()));
                }
            }
        }

        // Add any remaining properties
        for (prop, value) in &properties {
            if !order.contains(&prop.as_str()) {
                ordered_props.push((prop.clone(), value.clone()));
            }
        }

        for (prop, value) in &ordered_props {
            output.push_str(indent);
            output.push_str(prop);
            output.push_str(": ");

            // Format src specially
            if prop == "src" {
                output.push_str(&self.format_font_src(value));
            } else {
                output.push_str(&self.format_value(value));
            }
            output.push_str(";\n");
        }

        output.push_str("}\n");

        output
    }

    /// Format font src value
    fn format_font_src(&self, src: &str) -> String {
        // Split multiple sources
        if src.contains("url(") && src.matches("url(").count() > 1 {
            let re = Regex::new(r"(url\([^)]+\)\s*format\([^)]+\))").unwrap();
            let sources: Vec<&str> = re.find_iter(src).map(|m| m.as_str()).collect();

            if sources.len() > 1 {
                return sources.join(",\n       ");
            }
        }

        src.trim().to_string()
    }

    /// Format @property rule
    fn format_property_rule(&self, name: &str, content: &str, indent: &str) -> String {
        let mut output = String::new();

        output.push_str(name.trim());
        output.push_str(" {\n");

        let properties = self.parse_properties(content);
        for (prop, value) in &properties {
            output.push_str(indent);
            output.push_str(prop);
            output.push_str(": ");
            output.push_str(&self.format_value(value));
            output.push_str(";\n");
        }

        output.push_str("}\n");

        output
    }

    /// Format :root with CSS variables
    fn format_variables(&self, selector: &str, content: &str, indent: &str) -> (String, usize) {
        let mut output = String::new();
        let properties = self.parse_properties(content);

        // Separate variables from regular properties
        let (vars, regular): (Vec<_>, Vec<_>) = properties
            .iter()
            .partition(|(prop, _)| prop.starts_with("--"));

        let var_count = vars.len();

        output.push_str(selector);
        output.push_str(" {\n");

        // Group variables by prefix if add_section_comments is enabled
        if self.options.add_section_comments && var_count > 5 {
            let mut grouped: HashMap<String, Vec<(&String, &String)>> = HashMap::new();

            for (prop, value) in &vars {
                // Extract prefix (e.g., --color-primary -> color)
                let prefix = prop.trim_start_matches("--")
                    .split('-')
                    .next()
                    .unwrap_or("other")
                    .to_string();

                grouped.entry(prefix).or_default().push((prop, value));
            }

            // Output grouped variables
            let mut group_names: Vec<_> = grouped.keys().collect();
            group_names.sort();

            for group_name in group_names {
                if let Some(group_vars) = grouped.get(group_name) {
                    output.push_str(indent);
                    output.push_str(&format!("/* {} */\n", group_name));

                    for (prop, value) in group_vars {
                        output.push_str(indent);
                        output.push_str(prop);
                        output.push_str(": ");
                        output.push_str(value);
                        output.push_str(";\n");
                    }
                    output.push('\n');
                }
            }
        } else {
            // Output variables without grouping
            for (prop, value) in &vars {
                output.push_str(indent);
                output.push_str(prop);
                output.push_str(": ");
                output.push_str(value);
                output.push_str(";\n");
            }
        }

        // Output regular properties
        for (prop, value) in &regular {
            output.push_str(indent);
            output.push_str(prop);
            output.push_str(": ");
            output.push_str(value);
            output.push_str(";\n");
        }

        output.push_str("}\n");

        (output, var_count)
    }
}

impl Default for CssFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_rule() {
        let formatter = CssFormatter::new();
        let css = ".test{color:red;font-size:14px}";
        let result = formatter.format(css);

        assert!(result.css.contains(".test {"));
        assert!(result.css.contains("color: red;"));
        assert!(result.css.contains("font-size: 14px;"));
        assert_eq!(result.stats.rules_formatted, 1);
    }

    #[test]
    fn test_format_keyframes() {
        let formatter = CssFormatter::new();
        let css = "@keyframes spin{0%{transform:rotate(0deg)}100%{transform:rotate(360deg)}}";
        let result = formatter.format(css);

        assert!(result.css.contains("@keyframes spin {"));
        assert!(result.css.contains("0% {"));
        assert!(result.css.contains("100% {"));
        assert_eq!(result.stats.keyframes_formatted, 1);
    }

    #[test]
    fn test_format_media_query() {
        let formatter = CssFormatter::new();
        let css = "@media (min-width:768px){.test{display:flex}}";
        let result = formatter.format(css);

        assert!(result.css.contains("@media (min-width:768px) {"));
        assert!(result.css.contains("display: flex;"));
        assert_eq!(result.stats.media_queries_formatted, 1);
    }

    #[test]
    fn test_format_font_face() {
        let formatter = CssFormatter::new();
        let css = "@font-face{font-family:test;src:url(test.woff2);font-weight:400}";
        let result = formatter.format(css);

        assert!(result.css.contains("@font-face {"));
        assert!(result.css.contains("font-family: test;"));
        assert_eq!(result.stats.font_faces_formatted, 1);
    }

    #[test]
    fn test_format_variables() {
        let formatter = CssFormatter::new();
        let css = ":root{--color-primary:#ff0000;--color-secondary:#00ff00}";
        let result = formatter.format(css);

        assert!(result.css.contains(":root {"));
        assert!(result.css.contains("--color-primary: #ff0000;"));
        assert_eq!(result.stats.variables_found, 2);
    }

    #[test]
    fn test_format_multiple_selectors() {
        let formatter = CssFormatter::new();
        let css = ".a,.b,.c,.d,.e{color:red}";
        let result = formatter.format(css);

        // Multiple selectors should be on separate lines
        assert!(result.css.contains(",\n"));
    }
}
