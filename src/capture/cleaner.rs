//! HTML Cleaner Module
//!
//! Cleans HTML by removing scripts, data attributes, and framework-specific code.

use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Script tags (with content)
    static ref SCRIPT_RE: Regex = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();

    // Noscript tags
    static ref NOSCRIPT_RE: Regex = Regex::new(r"(?is)<noscript[^>]*>.*?</noscript>").unwrap();

    // Style tags (we keep the CSS separately)
    static ref STYLE_RE: Regex = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();

    // Link stylesheet tags (we handle CSS separately)
    static ref LINK_STYLE_RE: Regex = Regex::new(r#"(?i)<link[^>]*rel=["']stylesheet["'][^>]*>"#).unwrap();

    // Meta tags (except charset and viewport)
    static ref META_RE: Regex = Regex::new(r#"(?i)<meta[^>]*(?:property|name)=["'][^"']+["'][^>]*>"#).unwrap();

    // Data attributes
    static ref DATA_ATTR_RE: Regex = Regex::new(r#"\sdata-[a-z0-9-]+="[^"]*""#).unwrap();

    // Framework attributes (ng-, v-, x-, @, :)
    static ref FRAMEWORK_ATTR_RE: Regex = Regex::new(r#"\s(?:ng-|v-|x-|@|:)[a-z0-9-]+="[^"]*""#).unwrap();

    // Internal attributes (_*)
    static ref INTERNAL_ATTR_RE: Regex = Regex::new(r#"\s_[a-z0-9-]+="[^"]*""#).unwrap();

    // Empty attributes
    static ref EMPTY_ATTR_RE: Regex = Regex::new(r#"\s[a-z-]+=""(?!\s)"#).unwrap();

    // Multiple whitespace
    static ref MULTI_SPACE_RE: Regex = Regex::new(r"  +").unwrap();

    // Multiple newlines
    static ref MULTI_NEWLINE_RE: Regex = Regex::new(r"\n\n\n+").unwrap();

    // Comments
    static ref COMMENT_RE: Regex = Regex::new(r"(?s)<!--.*?-->").unwrap();
}

/// HTML cleaner options
#[derive(Debug, Clone)]
pub struct CleanerOptions {
    pub remove_scripts: bool,
    pub remove_styles: bool,
    pub remove_data_attrs: bool,
    pub remove_framework_attrs: bool,
    pub remove_comments: bool,
    pub remove_empty_attrs: bool,
    pub minify_whitespace: bool,
    pub add_stylesheet_link: bool,
    pub add_script_link: bool,
    pub stylesheet_path: String,
    pub script_path: String,
}

impl Default for CleanerOptions {
    fn default() -> Self {
        Self {
            remove_scripts: true,
            remove_styles: true,
            remove_data_attrs: true,
            remove_framework_attrs: true,
            remove_comments: true,
            remove_empty_attrs: true,
            minify_whitespace: false,
            add_stylesheet_link: true,
            add_script_link: true,
            stylesheet_path: "styles.css".to_string(),
            script_path: "scripts/main.js".to_string(),
        }
    }
}

/// HTML Cleaner
pub struct HtmlCleaner {
    options: CleanerOptions,
}

impl HtmlCleaner {
    pub fn new(options: CleanerOptions) -> Self {
        Self { options }
    }

    pub fn with_defaults() -> Self {
        Self::new(CleanerOptions::default())
    }

    /// Clean HTML content
    pub fn clean(&self, html: &str) -> String {
        let mut result = html.to_string();

        // Remove scripts
        if self.options.remove_scripts {
            result = SCRIPT_RE.replace_all(&result, "").to_string();
            result = NOSCRIPT_RE.replace_all(&result, "").to_string();
        }

        // Remove styles (we'll link to external CSS)
        if self.options.remove_styles {
            result = STYLE_RE.replace_all(&result, "").to_string();
            result = LINK_STYLE_RE.replace_all(&result, "").to_string();
        }

        // Remove comments
        if self.options.remove_comments {
            result = COMMENT_RE.replace_all(&result, "").to_string();
        }

        // Remove data attributes
        if self.options.remove_data_attrs {
            result = DATA_ATTR_RE.replace_all(&result, "").to_string();
        }

        // Remove framework attributes
        if self.options.remove_framework_attrs {
            result = FRAMEWORK_ATTR_RE.replace_all(&result, "").to_string();
            result = INTERNAL_ATTR_RE.replace_all(&result, "").to_string();
        }

        // Remove empty attributes
        if self.options.remove_empty_attrs {
            result = EMPTY_ATTR_RE.replace_all(&result, "").to_string();
        }

        // Add stylesheet link
        if self.options.add_stylesheet_link {
            result = self.add_stylesheet(&result);
        }

        // Add script link
        if self.options.add_script_link {
            result = self.add_script(&result);
        }

        // Minify whitespace
        if self.options.minify_whitespace {
            result = MULTI_SPACE_RE.replace_all(&result, " ").to_string();
            result = MULTI_NEWLINE_RE.replace_all(&result, "\n\n").to_string();
        }

        result
    }

    /// Add stylesheet link to head
    fn add_stylesheet(&self, html: &str) -> String {
        let link = format!(r#"  <link rel="stylesheet" href="{}">"#, self.options.stylesheet_path);

        if html.contains("</head>") {
            html.replace("</head>", &format!("{}\n</head>", link))
        } else if html.contains("<body") {
            // No head tag, insert before body
            let re = Regex::new(r"(?i)(<body[^>]*>)").unwrap();
            re.replace(html, &format!("<head>\n{}\n</head>\n$1", link)).to_string()
        } else {
            html.to_string()
        }
    }

    /// Add script link before </body>
    fn add_script(&self, html: &str) -> String {
        let script = format!(r#"  <script src="{}"></script>"#, self.options.script_path);

        if html.contains("</body>") {
            html.replace("</body>", &format!("{}\n</body>", script))
        } else {
            html.to_string()
        }
    }
}

/// Quick clean function
pub fn clean_html(html: &str) -> String {
    HtmlCleaner::with_defaults().clean(html)
}

/// Clean HTML with custom options
pub fn clean_html_with_options(html: &str, options: CleanerOptions) -> String {
    HtmlCleaner::new(options).clean(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_scripts() {
        let html = r#"<html><head></head><body><script>alert('test');</script><p>Hello</p></body></html>"#;
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("<script>"));
        assert!(cleaned.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_remove_data_attrs() {
        let html = r#"<div data-id="123" data-value="test" class="box">Content</div>"#;
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("data-id"));
        assert!(cleaned.contains("class=\"box\""));
    }

    #[test]
    fn test_add_stylesheet() {
        let html = r#"<html><head><title>Test</title></head><body></body></html>"#;
        let cleaned = clean_html(html);
        assert!(cleaned.contains(r#"<link rel="stylesheet" href="styles.css">"#));
    }
}
