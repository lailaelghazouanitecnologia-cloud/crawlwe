//! Chrome DevTools Protocol client for capturing web pages
//!
//! Uses chromiumoxide to control Chrome/Chromium headless browser
//! and extract computed styles, DOM structure, and CSS rules.

use crate::models::*;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::dom::{GetDocumentParams, Node};
use chromiumoxide::cdp::browser_protocol::css::{
    CSSStyleSheetHeader, GetComputedStyleForNodeParams, GetMatchedStylesForNodeParams,
};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams;
use chromiumoxide::page::Page;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    LaunchError(String),
    #[error("Navigation failed: {0}")]
    NavigationError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("CDP error: {0}")]
    CdpError(String),
    #[error("Element not found: {0}")]
    ElementNotFound(String),
}

/// Browser controller for capturing pages
pub struct BrowserController {
    browser: Browser,
    config: CaptureConfig,
}

impl BrowserController {
    /// Launch a new browser instance
    pub async fn launch(config: CaptureConfig) -> Result<Self, BrowserError> {
        let browser_config = BrowserConfig::builder()
            .with_head(false) // headless
            .viewport(chromiumoxide::handler::viewport::Viewport {
                width: config.viewports.first().map(|v| v.width).unwrap_or(1440),
                height: config.viewports.first().map(|v| v.height).unwrap_or(900),
                device_scale_factor: Some(1.0),
                ..Default::default()
            })
            .build()
            .map_err(|e| BrowserError::LaunchError(e.to_string()))?;

        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| BrowserError::LaunchError(e.to_string()))?;

        // Spawn handler in background
        tokio::spawn(async move {
            while let Some(_) = handler.next().await {}
        });

        Ok(Self { browser, config })
    }

    /// Capture a single page
    pub async fn capture_page(&self, url: &str) -> Result<CapturedPage, BrowserError> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::NavigationError(e.to_string()))?;

        // Wait for page to load
        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::NavigationError(e.to_string()))?;

        let viewport = self.config.viewports.first().cloned().unwrap_or(Viewport::desktop());
        let mut captured = CapturedPage::new(url.to_string(), viewport);

        // Get page title
        captured.title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.into_value().ok());

        // Enable CSS domain
        page.execute(chromiumoxide::cdp::browser_protocol::css::EnableParams::default())
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        // Get document root
        let doc = page
            .execute(GetDocumentParams::builder().depth(-1).build())
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        // Extract DOM tree
        self.extract_dom_tree(&page, &doc.root, &mut captured, None, 0)
            .await?;

        // Get all stylesheets
        self.extract_stylesheets(&page, &mut captured).await?;

        // Get computed styles for each element
        self.extract_computed_styles(&page, &mut captured).await?;

        // Capture states (hover, focus, etc.)
        if self.config.capture_hover || self.config.capture_focus || self.config.capture_active {
            self.capture_element_states(&page, &mut captured).await?;
        }

        // Take screenshot if configured
        if self.config.take_screenshots {
            captured.screenshot = self.take_screenshot(&page).await.ok();
        }

        // Generate clean HTML
        captured.html = self.generate_clean_html(&page).await?;

        // Generate unified CSS
        captured.css = self.generate_unified_css(&captured);

        Ok(captured)
    }

    /// Extract DOM tree recursively
    async fn extract_dom_tree(
        &self,
        page: &Page,
        node: &Node,
        captured: &mut CapturedPage,
        parent_id: Option<String>,
        depth: usize,
    ) -> Result<(), BrowserError> {
        // Only process element nodes (nodeType 1)
        if node.node_type != 1 {
            return Ok(());
        }

        let element_id = format!("el_{}", node.node_id);
        let tag = node.node_name.to_lowercase();

        // Skip script, style, svg internals
        if matches!(tag.as_str(), "script" | "noscript" | "style" | "link" | "meta") {
            return Ok(());
        }

        let mut element = CapturedElement::new(element_id.clone(), tag);
        element.parent_id = parent_id.clone();
        element.depth = depth;

        // Extract attributes
        if let Some(attrs) = &node.attributes {
            for chunk in attrs.chunks(2) {
                if chunk.len() == 2 {
                    let name = &chunk[0];
                    let value = &chunk[1];

                    match name.as_str() {
                        "id" => element.element_id = Some(value.clone()),
                        "class" => {
                            element.classes = value
                                .split_whitespace()
                                .map(String::from)
                                .collect();
                        }
                        "style" => {} // Handled separately
                        _ => {
                            // Skip data-* and framework-specific attributes
                            if !name.starts_with("data-")
                                && !name.starts_with("ng-")
                                && !name.starts_with("v-")
                                && !name.starts_with("_")
                            {
                                element.attributes.insert(name.clone(), value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Get text content (direct text only)
        if let Some(children) = &node.children {
            for child in children {
                if child.node_type == 3 {
                    // Text node
                    if let Some(text) = &child.node_value {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            element.text_content = Some(trimmed.to_string());
                        }
                    }
                }
            }
        }

        // Store root ID
        if captured.root_id.is_none() && element.tag == "html" {
            captured.root_id = Some(element_id.clone());
        }

        // Process children
        if let Some(children) = &node.children {
            for child in children {
                if child.node_type == 1 {
                    let child_id = format!("el_{}", child.node_id);
                    element.children_ids.push(child_id);

                    // Recurse
                    Box::pin(self.extract_dom_tree(
                        page,
                        child,
                        captured,
                        Some(element_id.clone()),
                        depth + 1,
                    ))
                    .await?;
                }
            }
        }

        captured.elements.insert(element_id, element);

        Ok(())
    }

    /// Extract all stylesheets from the page
    async fn extract_stylesheets(
        &self,
        page: &Page,
        captured: &mut CapturedPage,
    ) -> Result<(), BrowserError> {
        // Get all stylesheet headers
        let stylesheets = page
            .execute(chromiumoxide::cdp::browser_protocol::css::GetAllStyleSheetsParams::default())
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        for header in &stylesheets.style_sheet_infos {
            // Get stylesheet text
            if let Ok(text_result) = page
                .execute(
                    chromiumoxide::cdp::browser_protocol::css::GetStyleSheetTextParams::builder()
                        .style_sheet_id(header.style_sheet_id.clone())
                        .build(),
                )
                .await
            {
                // Parse CSS text into rules
                self.parse_css_text(&text_result.text, &header.source_url, captured);
            }
        }

        Ok(())
    }

    /// Parse CSS text into rules
    fn parse_css_text(&self, css_text: &str, source_url: &Option<String>, captured: &mut CapturedPage) {
        // Simple CSS parser - in production use lightningcss
        let mut current_selector = String::new();
        let mut current_declarations: HashMap<String, String> = HashMap::new();
        let mut in_rule = false;
        let mut brace_depth = 0;
        let mut current_media: Option<String> = None;

        for line in css_text.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("/*") || line.starts_with("//") {
                continue;
            }

            // Media query start
            if line.starts_with("@media") {
                let condition = line
                    .trim_start_matches("@media")
                    .trim()
                    .trim_end_matches('{')
                    .trim()
                    .to_string();
                current_media = Some(condition);
                brace_depth += 1;
                continue;
            }

            // Keyframes
            if line.starts_with("@keyframes") {
                // TODO: Parse keyframes
                continue;
            }

            // CSS variables (:root)
            if line.contains("--") && line.contains(":") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().to_string();
                    let value = parts[1].trim().trim_end_matches(';').to_string();
                    if name.starts_with("--") {
                        captured.css_variables.push(CssVariable {
                            name,
                            value,
                            scope: current_selector.clone(),
                        });
                    }
                }
            }

            // Rule start
            if line.contains('{') && !line.starts_with("@") {
                current_selector = line.trim_end_matches('{').trim().to_string();
                in_rule = true;
                brace_depth += 1;
                continue;
            }

            // Rule end
            if line.contains('}') {
                if in_rule && !current_selector.is_empty() {
                    let rule = CssRule {
                        selector: current_selector.clone(),
                        declarations: current_declarations.clone(),
                        specificity: calculate_specificity(&current_selector),
                        source_file: source_url.clone(),
                        line: None,
                    };

                    if let Some(media) = &current_media {
                        // Add to media query
                        if let Some(mq) = captured
                            .media_queries
                            .iter_mut()
                            .find(|m| &m.condition == media)
                        {
                            mq.rules.push(rule);
                        } else {
                            captured.media_queries.push(MediaQuery {
                                condition: media.clone(),
                                rules: vec![rule],
                            });
                        }
                    } else {
                        captured.css_rules.push(rule);
                    }

                    current_declarations.clear();
                    current_selector.clear();
                    in_rule = false;
                }

                brace_depth -= 1;
                if brace_depth == 0 {
                    current_media = None;
                }
                continue;
            }

            // Declaration
            if in_rule && line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let property = parts[0].trim().to_string();
                    let value = parts[1].trim().trim_end_matches(';').to_string();
                    current_declarations.insert(property, value);
                }
            }
        }
    }

    /// Extract computed styles for all elements
    async fn extract_computed_styles(
        &self,
        page: &Page,
        captured: &mut CapturedPage,
    ) -> Result<(), BrowserError> {
        let element_ids: Vec<String> = captured.elements.keys().cloned().collect();
        let max = if self.config.max_elements > 0 {
            self.config.max_elements.min(element_ids.len())
        } else {
            element_ids.len()
        };

        for element_id in element_ids.iter().take(max) {
            // Extract node_id from element_id
            let node_id: i64 = element_id
                .trim_start_matches("el_")
                .parse()
                .unwrap_or(0);

            if node_id == 0 {
                continue;
            }

            // Get computed style
            if let Ok(computed) = page
                .execute(
                    GetComputedStyleForNodeParams::builder()
                        .node_id(node_id.into())
                        .build(),
                )
                .await
            {
                let mut styles = ComputedStyles::new();
                for prop in &computed.computed_style {
                    styles.set(prop.name.clone(), prop.value.clone());
                }

                if let Some(element) = captured.elements.get_mut(element_id) {
                    element.set_styles("base".to_string(), styles);
                }
            }
        }

        Ok(())
    }

    /// Capture element states (hover, focus, active)
    async fn capture_element_states(
        &self,
        page: &Page,
        captured: &mut CapturedPage,
    ) -> Result<(), BrowserError> {
        // Find interactive elements
        let interactive_elements: Vec<String> = captured
            .elements
            .iter()
            .filter(|(_, el)| {
                matches!(
                    el.tag.as_str(),
                    "a" | "button" | "input" | "select" | "textarea" | "label"
                ) || el.classes.iter().any(|c| {
                    c.contains("btn")
                        || c.contains("button")
                        || c.contains("link")
                        || c.contains("clickable")
                })
            })
            .map(|(id, _)| id.clone())
            .collect();

        for element_id in interactive_elements {
            let node_id: i64 = element_id.trim_start_matches("el_").parse().unwrap_or(0);
            if node_id == 0 {
                continue;
            }

            // Force pseudo states
            let states_to_capture: Vec<(&str, Vec<&str>)> = vec![
                ("hover", vec!["hover"]),
                ("focus", vec!["focus"]),
                ("active", vec!["active"]),
                ("hover_focus", vec!["hover", "focus"]),
            ];

            for (state_name, pseudo_classes) in &states_to_capture {
                // Force pseudo state
                let _ = page
                    .execute(
                        chromiumoxide::cdp::browser_protocol::css::ForcePseudoStateParams::builder()
                            .node_id(node_id.into())
                            .forced_pseudo_classes(
                                pseudo_classes.iter().map(|s| s.to_string()).collect(),
                            )
                            .build(),
                    )
                    .await;

                // Get computed style in this state
                if let Ok(computed) = page
                    .execute(
                        GetComputedStyleForNodeParams::builder()
                            .node_id(node_id.into())
                            .build(),
                    )
                    .await
                {
                    let mut styles = ComputedStyles::new();
                    for prop in &computed.computed_style {
                        styles.set(prop.name.clone(), prop.value.clone());
                    }

                    if let Some(element) = captured.elements.get_mut(&element_id) {
                        // Only store if different from base
                        if let Some(base) = element.get_styles("base") {
                            if styles.properties != base.properties {
                                element.set_styles(state_name.to_string(), styles);
                            }
                        }
                    }
                }

                // Reset pseudo state
                let _ = page
                    .execute(
                        chromiumoxide::cdp::browser_protocol::css::ForcePseudoStateParams::builder()
                            .node_id(node_id.into())
                            .forced_pseudo_classes(vec![])
                            .build(),
                    )
                    .await;
            }
        }

        Ok(())
    }

    /// Take a screenshot of the page
    async fn take_screenshot(&self, page: &Page) -> Result<String, BrowserError> {
        let screenshot = page
            .execute(
                CaptureScreenshotParams::builder()
                    .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
                    .build(),
            )
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        Ok(screenshot.data)
    }

    /// Generate clean HTML from captured DOM
    async fn generate_clean_html(&self, page: &Page) -> Result<String, BrowserError> {
        let html: String = page
            .evaluate(
                r#"
                (function() {
                    const clone = document.documentElement.cloneNode(true);

                    // Remove scripts
                    clone.querySelectorAll('script, noscript, style, link[rel="stylesheet"]')
                        .forEach(el => el.remove());

                    // Remove data-* attributes
                    clone.querySelectorAll('*').forEach(el => {
                        [...el.attributes].forEach(attr => {
                            if (attr.name.startsWith('data-') ||
                                attr.name.startsWith('ng-') ||
                                attr.name.startsWith('v-') ||
                                attr.name.startsWith('_')) {
                                el.removeAttribute(attr.name);
                            }
                        });
                    });

                    return clone.outerHTML;
                })()
            "#,
            )
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        Ok(html)
    }

    /// Generate unified CSS from all captured rules and states
    fn generate_unified_css(&self, captured: &CapturedPage) -> String {
        let mut css = String::new();

        // CSS Variables
        if !captured.css_variables.is_empty() {
            css.push_str(":root {\n");
            for var in &captured.css_variables {
                css.push_str(&format!("  {}: {};\n", var.name, var.value));
            }
            css.push_str("}\n\n");
        }

        // Font faces
        for font in &captured.font_faces {
            css.push_str("@font-face {\n");
            css.push_str(&format!("  font-family: '{}';\n", font.family));
            if let Some(weight) = &font.weight {
                css.push_str(&format!("  font-weight: {};\n", weight));
            }
            if let Some(style) = &font.style {
                css.push_str(&format!("  font-style: {};\n", style));
            }
            css.push_str(&format!("  src: {};\n", font.src.join(", ")));
            css.push_str("}\n\n");
        }

        // Base rules (sorted by specificity)
        let mut sorted_rules = captured.css_rules.clone();
        sorted_rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));

        for rule in &sorted_rules {
            css.push_str(&rule.to_css_string());
            css.push_str("\n\n");
        }

        // Media queries
        for mq in &captured.media_queries {
            css.push_str(&format!("@media {} {{\n", mq.condition));
            for rule in &mq.rules {
                css.push_str("  ");
                css.push_str(&rule.to_css_string().replace('\n', "\n  "));
                css.push_str("\n");
            }
            css.push_str("}\n\n");
        }

        // Keyframes
        for kf in &captured.keyframes {
            css.push_str(&format!("@keyframes {} {{\n", kf.name));
            for frame in &kf.frames {
                css.push_str(&format!("  {} {{\n", frame.position));
                for (prop, val) in &frame.properties {
                    css.push_str(&format!("    {}: {};\n", prop, val));
                }
                css.push_str("  }\n");
            }
            css.push_str("}\n\n");
        }

        css
    }

    /// Close the browser
    pub async fn close(self) -> Result<(), BrowserError> {
        self.browser
            .close()
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;
        Ok(())
    }
}

/// Calculate CSS specificity for a selector
fn calculate_specificity(selector: &str) -> Specificity {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;

    // Simple parsing - in production use cssparser
    for part in selector.split_whitespace() {
        for segment in part.split(|c| c == '>' || c == '+' || c == '~') {
            let segment = segment.trim();

            // Count IDs
            ids += segment.matches('#').count() as u32;

            // Count classes, attributes, pseudo-classes
            classes += segment.matches('.').count() as u32;
            classes += segment.matches('[').count() as u32;
            classes += segment
                .matches(':')
                .count()
                .saturating_sub(segment.matches("::").count()) as u32;

            // Count elements and pseudo-elements
            if !segment.is_empty()
                && !segment.starts_with('#')
                && !segment.starts_with('.')
                && !segment.starts_with('[')
                && !segment.starts_with(':')
            {
                elements += 1;
            }
            elements += segment.matches("::").count() as u32;
        }
    }

    Specificity { ids, classes, elements }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity() {
        assert_eq!(
            calculate_specificity("div"),
            Specificity { ids: 0, classes: 0, elements: 1 }
        );
        assert_eq!(
            calculate_specificity(".class"),
            Specificity { ids: 0, classes: 1, elements: 0 }
        );
        assert_eq!(
            calculate_specificity("#id"),
            Specificity { ids: 1, classes: 0, elements: 0 }
        );
        assert_eq!(
            calculate_specificity("div.class#id"),
            Specificity { ids: 1, classes: 1, elements: 1 }
        );
    }
}
