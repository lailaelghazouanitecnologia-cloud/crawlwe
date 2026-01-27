//! HTML Parser and DOM Tree Builder
//!
//! Provides HTML parsing, cleaning, and semantic structure extraction
//! using html5ever for spec-compliant parsing.

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::ParseOpts;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HtmlParserError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),
}

/// HTML AST node
#[derive(Debug, Clone)]
pub struct HtmlNode {
    pub id: String,
    pub tag: String,
    pub element_id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub text_content: Option<String>,
    pub children: Vec<HtmlNode>,
    pub depth: usize,
}

impl HtmlNode {
    pub fn new(tag: String, depth: usize) -> Self {
        Self {
            id: uuid_v4(),
            tag,
            element_id: None,
            classes: Vec::new(),
            attributes: HashMap::new(),
            text_content: None,
            children: Vec::new(),
            depth,
        }
    }

    pub fn get_all_classes(&self) -> HashSet<String> {
        let mut classes = HashSet::new();
        self.collect_classes(&mut classes);
        classes
    }

    fn collect_classes(&self, classes: &mut HashSet<String>) {
        for class in &self.classes {
            classes.insert(class.clone());
        }
        for child in &self.children {
            child.collect_classes(classes);
        }
    }

    pub fn get_all_ids(&self) -> HashSet<String> {
        let mut ids = HashSet::new();
        self.collect_ids(&mut ids);
        ids
    }

    fn collect_ids(&self, ids: &mut HashSet<String>) {
        if let Some(id) = &self.element_id {
            ids.insert(id.clone());
        }
        for child in &self.children {
            child.collect_ids(ids);
        }
    }

    pub fn get_all_tags(&self) -> HashSet<String> {
        let mut tags = HashSet::new();
        self.collect_tags(&mut tags);
        tags
    }

    fn collect_tags(&self, tags: &mut HashSet<String>) {
        tags.insert(self.tag.clone());
        for child in &self.children {
            child.collect_tags(tags);
        }
    }

    pub fn element_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.element_count()).sum::<usize>()
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&HtmlNode> {
        let mut results = Vec::new();
        self.find_by_tag_recursive(tag, &mut results);
        results
    }

    fn find_by_tag_recursive<'a>(&'a self, tag: &str, results: &mut Vec<&'a HtmlNode>) {
        if self.tag == tag {
            results.push(self);
        }
        for child in &self.children {
            child.find_by_tag_recursive(tag, results);
        }
    }

    pub fn find_by_class(&self, class: &str) -> Vec<&HtmlNode> {
        let mut results = Vec::new();
        self.find_by_class_recursive(class, &mut results);
        results
    }

    fn find_by_class_recursive<'a>(&'a self, class: &str, results: &mut Vec<&'a HtmlNode>) {
        if self.classes.contains(&class.to_string()) {
            results.push(self);
        }
        for child in &self.children {
            child.find_by_class_recursive(class, results);
        }
    }
}

/// HTML Parser for extracting clean DOM structure
pub struct HtmlParser {
    pub remove_scripts: bool,
    pub remove_styles: bool,
    pub remove_comments: bool,
    pub remove_data_attrs: bool,
    pub remove_framework_attrs: bool,
    pub remove_empty: bool,
    pub collapse_whitespace: bool,
    pub skip_tags: HashSet<String>,
}

impl Default for HtmlParser {
    fn default() -> Self {
        let mut skip_tags = HashSet::new();
        skip_tags.insert("script".to_string());
        skip_tags.insert("noscript".to_string());
        skip_tags.insert("style".to_string());
        skip_tags.insert("link".to_string());
        skip_tags.insert("meta".to_string());

        Self {
            remove_scripts: true,
            remove_styles: true,
            remove_comments: true,
            remove_data_attrs: true,
            remove_framework_attrs: true,
            remove_empty: true,
            collapse_whitespace: true,
            skip_tags,
        }
    }
}

impl HtmlParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, html: &str) -> Result<HtmlNode, HtmlParserError> {
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map_err(|e| HtmlParserError::ParseError(e.to_string()))?;

        let root = self.find_document_element(&dom.document)?;
        Ok(root)
    }

    fn find_document_element(&self, handle: &Handle) -> Result<HtmlNode, HtmlParserError> {
        match &handle.data {
            NodeData::Document => {
                for child in handle.children.borrow().iter() {
                    if let Ok(result) = self.find_document_element(child) {
                        if result.tag == "html" {
                            return Ok(result);
                        }
                    }
                }
                Err(HtmlParserError::InvalidHtml("No html element found".to_string()))
            }
            NodeData::Element { name, attrs, .. } => {
                let tag = name.local.to_string().to_lowercase();

                if self.skip_tags.contains(&tag) {
                    return Err(HtmlParserError::InvalidHtml("Skipped tag".to_string()));
                }

                let mut html_node = HtmlNode::new(tag.clone(), 0);

                for attr in attrs.borrow().iter() {
                    let attr_name = attr.name.local.to_string();
                    let attr_value = attr.value.to_string();

                    match attr_name.as_str() {
                        "id" => html_node.element_id = Some(attr_value),
                        "class" => {
                            html_node.classes = attr_value
                                .split_whitespace()
                                .map(String::from)
                                .collect();
                        }
                        "style" => {
                            html_node.attributes.insert("style".to_string(), attr_value);
                        }
                        _ => {
                            if self.should_keep_attribute(&attr_name) {
                                html_node.attributes.insert(attr_name, attr_value);
                            }
                        }
                    }
                }

                for child in handle.children.borrow().iter() {
                    if let Ok(child_node) = self.process_node(child, 1) {
                        html_node.children.push(child_node);
                    }
                }

                Ok(html_node)
            }
            _ => Err(HtmlParserError::InvalidHtml("Not an element".to_string())),
        }
    }

    fn process_node(&self, handle: &Handle, depth: usize) -> Result<HtmlNode, HtmlParserError> {
        match &handle.data {
            NodeData::Element { name, attrs, .. } => {
                let tag = name.local.to_string().to_lowercase();

                if self.skip_tags.contains(&tag) {
                    return Err(HtmlParserError::InvalidHtml("Skipped tag".to_string()));
                }

                let mut html_node = HtmlNode::new(tag.clone(), depth);

                for attr in attrs.borrow().iter() {
                    let attr_name = attr.name.local.to_string();
                    let attr_value = attr.value.to_string();

                    match attr_name.as_str() {
                        "id" => html_node.element_id = Some(attr_value),
                        "class" => {
                            html_node.classes = attr_value
                                .split_whitespace()
                                .map(String::from)
                                .collect();
                        }
                        "style" => {
                            html_node.attributes.insert("style".to_string(), attr_value);
                        }
                        _ => {
                            if self.should_keep_attribute(&attr_name) {
                                html_node.attributes.insert(attr_name, attr_value);
                            }
                        }
                    }
                }

                let mut text_parts = Vec::new();

                for child in handle.children.borrow().iter() {
                    match &child.data {
                        NodeData::Text { contents } => {
                            let text = contents.borrow().to_string();
                            let trimmed = if self.collapse_whitespace {
                                text.split_whitespace().collect::<Vec<_>>().join(" ")
                            } else {
                                text
                            };

                            if !trimmed.is_empty() {
                                text_parts.push(trimmed);
                            }
                        }
                        NodeData::Element { .. } => {
                            if let Ok(child_node) = self.process_node(child, depth + 1) {
                                html_node.children.push(child_node);
                            }
                        }
                        NodeData::Comment { .. } => {}
                        _ => {}
                    }
                }

                if !text_parts.is_empty() {
                    html_node.text_content = Some(text_parts.join(" "));
                }

                if self.remove_empty
                    && html_node.children.is_empty()
                    && html_node.text_content.is_none()
                    && !is_void_element(&html_node.tag)
                {
                    return Err(HtmlParserError::InvalidHtml("Empty element".to_string()));
                }

                Ok(html_node)
            }
            _ => Err(HtmlParserError::InvalidHtml("Not an element".to_string())),
        }
    }

    fn should_keep_attribute(&self, name: &str) -> bool {
        if self.remove_data_attrs && name.starts_with("data-") {
            return false;
        }

        if self.remove_framework_attrs {
            if name.starts_with("ng-")
                || name.starts_with("v-")
                || name.starts_with("x-")
                || name.starts_with('_')
                || name.starts_with('@')
                || name.starts_with(':')
            {
                return false;
            }
        }

        if name.starts_with("on") {
            return false;
        }

        true
    }
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

/// HTML Generator - converts AST back to HTML string
pub struct HtmlGenerator {
    pub indent: String,
    pub minify: bool,
    pub include_doctype: bool,
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self {
            indent: "  ".to_string(),
            minify: false,
            include_doctype: true,
        }
    }
}

impl HtmlGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minified() -> Self {
        Self {
            indent: String::new(),
            minify: true,
            include_doctype: true,
        }
    }

    pub fn generate(&self, node: &HtmlNode, stylesheet_path: Option<&str>) -> String {
        let mut html = String::new();

        if self.include_doctype {
            html.push_str("<!DOCTYPE html>\n");
        }

        self.generate_node(&mut html, node, 0, stylesheet_path);
        html
    }

    fn generate_node(
        &self,
        html: &mut String,
        node: &HtmlNode,
        depth: usize,
        stylesheet_path: Option<&str>,
    ) {
        let indent = if self.minify {
            String::new()
        } else {
            self.indent.repeat(depth)
        };
        let newline = if self.minify { "" } else { "\n" };

        html.push_str(&indent);
        html.push('<');
        html.push_str(&node.tag);

        if let Some(id) = &node.element_id {
            html.push_str(" id=\"");
            html.push_str(&escape_html(id));
            html.push('"');
        }

        if !node.classes.is_empty() {
            html.push_str(" class=\"");
            html.push_str(&node.classes.join(" "));
            html.push('"');
        }

        for (name, value) in &node.attributes {
            if name == "style" {
                continue;
            }
            html.push(' ');
            html.push_str(name);
            html.push_str("=\"");
            html.push_str(&escape_html(value));
            html.push('"');
        }

        if node.tag == "head" {
            html.push('>');
            html.push_str(newline);

            html.push_str(&indent);
            html.push_str(&self.indent);
            html.push_str("<meta charset=\"utf-8\">");
            html.push_str(newline);

            html.push_str(&indent);
            html.push_str(&self.indent);
            html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
            html.push_str(newline);

            if let Some(path) = stylesheet_path {
                html.push_str(&indent);
                html.push_str(&self.indent);
                html.push_str("<link rel=\"stylesheet\" href=\"");
                html.push_str(path);
                html.push_str("\">");
                html.push_str(newline);
            }

            for child in &node.children {
                self.generate_node(html, child, depth + 1, None);
            }

            html.push_str(&indent);
            html.push_str("</head>");
            html.push_str(newline);
            return;
        }

        if is_void_element(&node.tag) {
            html.push_str(">");
            html.push_str(newline);
            return;
        }

        html.push('>');

        if let Some(text) = &node.text_content {
            if node.children.is_empty() {
                html.push_str(&escape_html(text));
            } else {
                html.push_str(newline);
                html.push_str(&indent);
                html.push_str(&self.indent);
                html.push_str(&escape_html(text));
            }
        }

        if !node.children.is_empty() {
            html.push_str(newline);
            for child in &node.children {
                self.generate_node(html, child, depth + 1, None);
            }
            html.push_str(&indent);
        }

        html.push_str("</");
        html.push_str(&node.tag);
        html.push('>');
        html.push_str(newline);
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    format!("{:032x}", now)
}

/// Semantic analyzer
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn analyze(node: &HtmlNode) -> SemanticInfo {
        let mut info = SemanticInfo::default();
        Self::analyze_recursive(node, &mut info);
        info
    }

    fn analyze_recursive(node: &HtmlNode, info: &mut SemanticInfo) {
        match node.tag.as_str() {
            "header" => info.has_header = true,
            "nav" => info.has_nav = true,
            "main" => info.has_main = true,
            "footer" => info.has_footer = true,
            "article" => info.article_count += 1,
            "section" => info.section_count += 1,
            "aside" => info.has_aside = true,
            "h1" => info.h1_count += 1,
            "h2" => info.h2_count += 1,
            "h3" => info.h3_count += 1,
            "form" => info.form_count += 1,
            "button" => info.button_count += 1,
            "a" => info.link_count += 1,
            "img" => info.image_count += 1,
            _ => {}
        }

        if node.classes.iter().any(|c| {
            c.contains("flex")
                || c.contains("grid")
                || c.contains("container")
                || c.contains("row")
                || c.contains("col")
        }) {
            info.uses_modern_layout = true;
        }

        for child in &node.children {
            Self::analyze_recursive(child, info);
        }
    }
}

#[derive(Debug, Default)]
pub struct SemanticInfo {
    pub has_header: bool,
    pub has_nav: bool,
    pub has_main: bool,
    pub has_footer: bool,
    pub has_aside: bool,
    pub article_count: usize,
    pub section_count: usize,
    pub h1_count: usize,
    pub h2_count: usize,
    pub h3_count: usize,
    pub form_count: usize,
    pub button_count: usize,
    pub link_count: usize,
    pub image_count: usize,
    pub uses_modern_layout: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test</title></head>
            <body>
                <div class="container">
                    <h1>Hello</h1>
                    <p>World</p>
                </div>
            </body>
            </html>
        "#;

        let result = parser.parse(html);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.tag, "html");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }
}
