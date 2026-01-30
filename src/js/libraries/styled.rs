//! styled-components and Emotion parser
//!
//! Extracts CSS from CSS-in-JS patterns like styled-components and Emotion.

use super::{
    LibraryAnalysis, LibraryCategory, RenderElement,
    registry::{LibraryParser, LibraryEffect, EffectType},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for styled-components and Emotion
pub struct StyledComponentsParser {
    // Styled patterns
    styled_tag: Regex,
    styled_component: Regex,
    css_tag: Regex,
    keyframes_tag: Regex,
    create_global_style: Regex,
    theme_prop: Regex,
}

impl StyledComponentsParser {
    pub fn new() -> Self {
        Self {
            styled_tag: Regex::new(r#"styled\.(\w+)`([^`]*)`"#).unwrap(),
            styled_component: Regex::new(r#"styled\((\w+)\)`([^`]*)`"#).unwrap(),
            css_tag: Regex::new(r#"css`([^`]*)`"#).unwrap(),
            keyframes_tag: Regex::new(r#"keyframes`([^`]*)`"#).unwrap(),
            create_global_style: Regex::new(r#"createGlobalStyle`([^`]*)`"#).unwrap(),
            theme_prop: Regex::new(r#"\$\{(?:props\s*=>\s*)?props\.theme\.(\w+(?:\.\w+)*)\}"#).unwrap(),
        }
    }

    /// Clean up CSS from template literal
    fn clean_css(css: &str) -> String {
        // Remove template interpolations and clean whitespace
        let theme_pattern = Regex::new(r#"\$\{[^}]+\}"#).unwrap();
        let cleaned = theme_pattern.replace_all(css, "/* dynamic */");

        // Normalize whitespace
        cleaned
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate a class name for a styled component
    fn generate_class_name(element: &str, index: usize) -> String {
        format!("sc-{}-{}", element.to_lowercase(), index)
    }

    /// Parse props references in CSS
    fn extract_theme_props(&self, css: &str) -> Vec<String> {
        let mut props = Vec::new();

        for cap in self.theme_prop.captures_iter(css) {
            if let Some(prop_path) = cap.get(1) {
                props.push(prop_path.as_str().to_string());
            }
        }

        props
    }
}

impl Default for StyledComponentsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for StyledComponentsParser {
    fn name(&self) -> &'static str {
        "styled-components"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::CssInJs
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        analysis.libraries.iter().any(|lib|
            lib == "styled-components" ||
            lib == "emotion" ||
            lib.starts_with("@emotion")
        ) ||
        analysis.raw.as_ref()
            .map(|r| r.imports.iter().any(|(lib, src)|
                lib == "styled-components" ||
                lib == "emotion" ||
                src.contains("styled-components") ||
                src.contains("@emotion")
            ))
            .unwrap_or(false) ||
        // Also check for CSS-in-JS patterns in the parsed result
        analysis.raw.as_ref()
            .map(|r| !r.css_in_js.is_empty())
            .unwrap_or(false)
    }

    fn analyze(&self, code: &str, analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut component_index = 0;

        // Extract styled.element`css` patterns
        for cap in self.styled_tag.captures_iter(code) {
            let element = cap.get(1).map(|m| m.as_str()).unwrap_or("div");
            let css = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let class_name = Self::generate_class_name(element, component_index);
            let cleaned_css = Self::clean_css(css);

            // Generate scoped CSS
            let scoped_css = format!(".{} {{\n{}\n}}", class_name, cleaned_css);
            result.generated_css.push(scoped_css);

            // Track theme props used
            let theme_props = self.extract_theme_props(css);
            if !theme_props.is_empty() {
                result.warnings.push(format!(
                    "Component {} uses theme props: {:?}",
                    class_name, theme_props
                ));
            }

            result.effects.push(LibraryEffect {
                effect_type: EffectType::StyleChange,
                target: class_name.clone(),
                css_properties: HashMap::new(),
                timing: None,
            });

            result.elements_to_render.push(RenderElement {
                tag: element.to_string(),
                classes: vec![class_name],
                styles: HashMap::new(),
                attributes: HashMap::new(),
                content: None,
                children: Vec::new(),
            });

            component_index += 1;
        }

        // Extract styled(Component)`css` patterns
        for cap in self.styled_component.captures_iter(code) {
            let component = cap.get(1).map(|m| m.as_str()).unwrap_or("Component");
            let css = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let class_name = Self::generate_class_name(component, component_index);
            let cleaned_css = Self::clean_css(css);

            let scoped_css = format!(".{} {{\n{}\n}}", class_name, cleaned_css);
            result.generated_css.push(scoped_css);

            component_index += 1;
        }

        // Extract css`...` utilities
        for cap in self.css_tag.captures_iter(code) {
            let css = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let cleaned_css = Self::clean_css(css);

            if !cleaned_css.is_empty() {
                let class_name = format!("css-util-{}", component_index);
                let scoped_css = format!(".{} {{\n{}\n}}", class_name, cleaned_css);
                result.generated_css.push(scoped_css);
                component_index += 1;
            }
        }

        // Extract keyframes
        for cap in self.keyframes_tag.captures_iter(code) {
            let keyframes_css = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let cleaned = Self::clean_css(keyframes_css);

            if !cleaned.is_empty() {
                let keyframe_name = format!("sc-keyframe-{}", component_index);
                let keyframes = format!("@keyframes {} {{\n{}\n}}", keyframe_name, cleaned);
                result.generated_css.push(keyframes);
                component_index += 1;
            }
        }

        // Extract createGlobalStyle
        for cap in self.create_global_style.captures_iter(code) {
            let global_css = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let cleaned = Self::clean_css(global_css);

            if !cleaned.is_empty() {
                result.generated_css.push(format!("/* Global Styles */\n{}", cleaned));
            }
        }

        // Also include CSS-in-JS from the original analysis
        if let Some(raw) = &analysis.raw {
            for (tag, css) in &raw.css_in_js {
                let class_name = Self::generate_class_name(&tag.replace("styled.", ""), component_index);
                let cleaned = Self::clean_css(css);

                if !cleaned.is_empty() && !result.generated_css.iter().any(|c| c.contains(&cleaned)) {
                    let scoped_css = format!("/* {} */\n.{} {{\n{}\n}}", tag, class_name, cleaned);
                    result.generated_css.push(scoped_css);
                    component_index += 1;
                }
            }
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* styled-components / Emotion CSS */\n\n");

        for block in &analysis.generated_css {
            css.push_str(block);
            css.push_str("\n\n");
        }

        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_styled_component() {
        let parser = StyledComponentsParser::new();
        let code = r#"
            const Button = styled.button`
                background: blue;
                color: white;
                padding: 10px 20px;
            `;
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("styled-components".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.generated_css.is_empty());
    }

    #[test]
    fn test_parse_keyframes() {
        let parser = StyledComponentsParser::new();
        let code = r#"
            const fadeIn = keyframes`
                from { opacity: 0; }
                to { opacity: 1; }
            `;
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("styled-components".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(result.generated_css.iter().any(|c| c.contains("@keyframes")));
    }
}
