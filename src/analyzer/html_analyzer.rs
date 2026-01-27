//! HTML content analyzer
//!
//! Analyzes HTML structure, semantic elements, UI components,
//! media elements, and accessibility features.

use std::collections::HashMap;

use super::patterns::*;
use super::{
    AccessibilityAnalysis, HtmlAnalysis, MediaElementAnalysis, SemanticElements,
    UiComponentAnalysis,
};

pub struct HtmlAnalyzer;

impl HtmlAnalyzer {
    /// Perform complete HTML analysis
    pub fn analyze(html: &str) -> HtmlAnalysis {
        let html_lower = html.to_lowercase();

        HtmlAnalysis {
            element_count: Self::count_elements(&html_lower),
            max_depth: Self::estimate_max_depth(&html_lower),
            semantic_elements: Self::analyze_semantic(&html_lower),
            ui_components: Self::analyze_ui_components(html),
            media_elements: Self::analyze_media(&html_lower),
            accessibility: Self::analyze_accessibility(html),
            data_attributes: Self::extract_data_attributes(html),
        }
    }

    /// Count total elements (rough estimate)
    fn count_elements(html: &str) -> usize {
        // Count opening tags
        html.matches('<').count().saturating_sub(
            html.matches("</").count() + // closing tags
            html.matches("<!").count() + // doctype/comments
            html.matches("<?").count()   // xml declarations
        )
    }

    /// Estimate max depth (heuristic)
    fn estimate_max_depth(html: &str) -> usize {
        let mut max_depth: usize = 0;
        let mut current_depth: usize = 0;

        for c in html.chars() {
            if c == '<' {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
            } else if c == '>' {
                // Very rough heuristic
                if current_depth > 0 {
                    current_depth = current_depth.saturating_sub(1);
                }
            }
        }

        // Divide by 2 as rough estimate (open + close = 2 per level)
        max_depth / 2
    }

    /// Analyze semantic HTML elements
    fn analyze_semantic(html: &str) -> SemanticElements {
        SemanticElements {
            header: Self::count_tag(html, "header"),
            nav: Self::count_tag(html, "nav"),
            main: Self::count_tag(html, "main"),
            article: Self::count_tag(html, "article"),
            section: Self::count_tag(html, "section"),
            aside: Self::count_tag(html, "aside"),
            footer: Self::count_tag(html, "footer"),
        }
    }

    /// Count occurrences of a tag
    fn count_tag(html: &str, tag: &str) -> usize {
        let open_tag = format!("<{}", tag);
        html.matches(&open_tag).count()
    }

    /// Analyze UI components
    fn analyze_ui_components(html: &str) -> UiComponentAnalysis {
        let mut components = UiComponentAnalysis::default();

        // Extract all class attributes
        for cap in CLASS_ATTR_PATTERN.captures_iter(html) {
            if let Some(classes) = cap.get(1) {
                let class_str = classes.as_str();

                // Check for each component type
                if let Some(component) = detect_ui_component(class_str) {
                    match component {
                        "modal" => components.modals += 1,
                        "dropdown" => components.dropdowns += 1,
                        "carousel" => components.carousels += 1,
                        "tabs" => components.tabs += 1,
                        "accordion" => components.accordions += 1,
                        "form" => components.forms += 1,
                        "card" => components.cards += 1,
                        "tooltip" => components.tooltips += 1,
                        "notification" => components.notifications += 1,
                        _ => {}
                    }
                }
            }
        }

        // Count buttons
        components.buttons = Self::count_tag(&html.to_lowercase(), "button");

        // Count forms
        components.forms = components.forms.max(
            Self::count_tag(&html.to_lowercase(), "form")
        );

        components
    }

    /// Analyze media elements
    fn analyze_media(html: &str) -> MediaElementAnalysis {
        MediaElementAnalysis {
            images: Self::count_tag(html, "img"),
            videos: Self::count_tag(html, "video"),
            audio: Self::count_tag(html, "audio"),
            canvas: Self::count_tag(html, "canvas"),
            svg: Self::count_tag(html, "svg"),
            iframes: Self::count_tag(html, "iframe"),
        }
    }

    /// Analyze accessibility features
    fn analyze_accessibility(html: &str) -> AccessibilityAnalysis {
        // Extract ARIA attributes
        let aria_attributes: Vec<String> = ARIA_ATTR_PATTERN
            .captures_iter(html)
            .filter_map(|cap| cap.get(1).map(|m| format!("aria-{}", m.as_str())))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(20)
            .collect();

        // Extract roles
        let roles: Vec<String> = ROLE_ATTR_PATTERN
            .captures_iter(html)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(20)
            .collect();

        // Check for alt texts
        let has_alt_texts = html.contains("alt=");

        // Check for labels
        let has_labels = html.to_lowercase().contains("<label");

        AccessibilityAnalysis {
            aria_attributes,
            roles,
            has_alt_texts,
            has_labels,
        }
    }

    /// Extract data attributes
    fn extract_data_attributes(html: &str) -> HashMap<String, usize> {
        let mut data_attrs: HashMap<String, usize> = HashMap::new();

        for cap in DATA_ATTR_PATTERN.captures_iter(html) {
            if let Some(attr_name) = cap.get(1) {
                let key = format!("data-{}", attr_name.as_str());
                *data_attrs.entry(key).or_insert(0) += 1;
            }
        }

        // Return top 30 by count
        let mut sorted: Vec<_> = data_attrs.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(30);
        sorted.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_semantic() {
        let html = r#"
            <header>Header</header>
            <nav>Nav</nav>
            <main>
                <article>Article 1</article>
                <article>Article 2</article>
            </main>
            <footer>Footer</footer>
        "#;
        let analysis = HtmlAnalyzer::analyze(html);
        assert_eq!(analysis.semantic_elements.header, 1);
        assert_eq!(analysis.semantic_elements.nav, 1);
        assert_eq!(analysis.semantic_elements.main, 1);
        assert_eq!(analysis.semantic_elements.article, 2);
        assert_eq!(analysis.semantic_elements.footer, 1);
    }

    #[test]
    fn test_analyze_media() {
        let html = r#"
            <img src="test.jpg">
            <video src="test.mp4"></video>
            <canvas id="gl"></canvas>
            <svg viewBox="0 0 100 100"></svg>
        "#;
        let analysis = HtmlAnalyzer::analyze(html);
        assert_eq!(analysis.media_elements.images, 1);
        assert_eq!(analysis.media_elements.videos, 1);
        assert_eq!(analysis.media_elements.canvas, 1);
        assert_eq!(analysis.media_elements.svg, 1);
    }

    #[test]
    fn test_analyze_accessibility() {
        let html = r#"
            <button aria-label="Close" role="button">X</button>
            <nav role="navigation" aria-expanded="true">
                <img src="logo.png" alt="Logo">
            </nav>
        "#;
        let analysis = HtmlAnalyzer::analyze(html);
        assert!(analysis.accessibility.has_alt_texts);
        assert!(analysis.accessibility.aria_attributes.len() >= 2);
        assert!(analysis.accessibility.roles.contains(&"button".to_string()));
    }
}
