//! Framer Motion parser
//!
//! Parses Framer Motion components and generates CSS animations.

use super::{
    AnimationDefinition, AnimationProperty, LibraryAnalysis, LibraryCategory,
    registry::{LibraryParser, LibraryEffect, EffectType, EffectTiming},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for Framer Motion animations
pub struct FramerMotionParser {
    // Patterns for motion components
    motion_component: Regex,
    animate_prop: Regex,
    initial_prop: Regex,
    variants_prop: Regex,
    transition_prop: Regex,
    property_value: Regex,
}

impl FramerMotionParser {
    pub fn new() -> Self {
        Self {
            motion_component: Regex::new(r#"<motion\.(\w+)"#).unwrap(),
            animate_prop: Regex::new(r#"animate\s*=\s*\{\s*\{([^}]+)\}"#).unwrap(),
            initial_prop: Regex::new(r#"initial\s*=\s*\{\s*\{([^}]+)\}"#).unwrap(),
            variants_prop: Regex::new(r#"variants\s*=\s*\{"#).unwrap(),
            transition_prop: Regex::new(r#"transition\s*=\s*\{\s*\{([^}]+)\}"#).unwrap(),
            property_value: Regex::new(r#"(\w+)\s*:\s*(?:["']([^"']+)["']|(\d+(?:\.\d+)?))"#).unwrap(),
        }
    }

    /// Parse properties from JSX object expression
    fn parse_jsx_props(&self, props_str: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();

        for cap in self.property_value.captures_iter(props_str) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2)
                .or_else(|| cap.get(3))
                .map(|m| m.as_str())
                .unwrap_or("");

            if !key.is_empty() && !value.is_empty() {
                props.insert(key.to_string(), value.to_string());
            }
        }

        props
    }

    /// Convert Framer Motion property to CSS
    fn to_css_property(framer_prop: &str) -> &str {
        match framer_prop {
            "x" => "transform",
            "y" => "transform",
            "rotate" => "transform",
            "rotateX" => "transform",
            "rotateY" => "transform",
            "rotateZ" => "transform",
            "scale" => "transform",
            "scaleX" => "transform",
            "scaleY" => "transform",
            "skew" => "transform",
            "skewX" => "transform",
            "skewY" => "transform",
            "opacity" => "opacity",
            "backgroundColor" => "background-color",
            "color" => "color",
            "width" => "width",
            "height" => "height",
            "borderRadius" => "border-radius",
            _ => framer_prop,
        }
    }

    /// Convert Framer Motion transform value to CSS
    fn to_css_value(prop: &str, value: &str) -> String {
        match prop {
            "x" => format!("translateX({}px)", value),
            "y" => format!("translateY({}px)", value),
            "rotate" | "rotateZ" => format!("rotate({}deg)", value),
            "rotateX" => format!("rotateX({}deg)", value),
            "rotateY" => format!("rotateY({}deg)", value),
            "scale" => format!("scale({})", value),
            "scaleX" => format!("scaleX({})", value),
            "scaleY" => format!("scaleY({})", value),
            "skew" => format!("skew({}deg)", value),
            "skewX" => format!("skewX({}deg)", value),
            "skewY" => format!("skewY({}deg)", value),
            _ => value.to_string(),
        }
    }

    /// Convert Framer Motion easing to CSS
    fn to_css_easing(framer_ease: &str) -> &str {
        match framer_ease {
            "linear" => "linear",
            "easeIn" => "ease-in",
            "easeOut" => "ease-out",
            "easeInOut" => "ease-in-out",
            "circIn" => "cubic-bezier(0.55, 0, 1, 0.45)",
            "circOut" => "cubic-bezier(0, 0.55, 0.45, 1)",
            "circInOut" => "cubic-bezier(0.85, 0, 0.15, 1)",
            "backIn" => "cubic-bezier(0.36, 0, 0.66, -0.56)",
            "backOut" => "cubic-bezier(0.34, 1.56, 0.64, 1)",
            "backInOut" => "cubic-bezier(0.68, -0.6, 0.32, 1.6)",
            "anticipate" => "cubic-bezier(0.36, 0, 0.66, -0.56)",
            _ => "ease",
        }
    }

    /// Generate CSS animation from Framer Motion props
    fn generate_animation(
        &self,
        element_type: &str,
        initial: &HashMap<String, String>,
        animate: &HashMap<String, String>,
        transition: &HashMap<String, String>,
        index: usize,
    ) -> AnimationDefinition {
        let anim_name = format!("framer-{}-{}", element_type, index);

        let duration = transition.get("duration")
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(0.3);

        let easing = transition.get("ease")
            .or_else(|| transition.get("type"))
            .map(|e| Self::to_css_easing(e))
            .unwrap_or("ease")
            .to_string();

        let mut properties = HashMap::new();
        let mut from_transforms = Vec::new();
        let mut to_transforms = Vec::new();

        // Process initial state
        for (key, value) in initial {
            let css_prop = Self::to_css_property(key);
            let css_val = Self::to_css_value(key, value);

            if css_prop == "transform" {
                from_transforms.push(css_val.clone());
            }

            properties.insert(key.clone(), AnimationProperty {
                from: css_val,
                to: String::new(),
                css_property: css_prop.to_string(),
            });
        }

        // Process animate state
        for (key, value) in animate {
            let css_prop = Self::to_css_property(key);
            let css_val = Self::to_css_value(key, value);

            if css_prop == "transform" {
                to_transforms.push(css_val.clone());
            }

            if let Some(prop) = properties.get_mut(key) {
                prop.to = css_val;
            } else {
                properties.insert(key.clone(), AnimationProperty {
                    from: "initial".to_string(),
                    to: css_val,
                    css_property: css_prop.to_string(),
                });
            }
        }

        // Generate keyframes
        let mut keyframes = format!("@keyframes {} {{\n", anim_name);

        // From state (0%)
        keyframes.push_str("  0% {\n");
        if !from_transforms.is_empty() {
            keyframes.push_str(&format!("    transform: {};\n", from_transforms.join(" ")));
        }
        for (key, value) in initial {
            let css_prop = Self::to_css_property(key);
            if css_prop != "transform" {
                keyframes.push_str(&format!("    {}: {};\n", css_prop, value));
            }
        }
        keyframes.push_str("  }\n");

        // To state (100%)
        keyframes.push_str("  100% {\n");
        if !to_transforms.is_empty() {
            keyframes.push_str(&format!("    transform: {};\n", to_transforms.join(" ")));
        }
        for (key, value) in animate {
            let css_prop = Self::to_css_property(key);
            if css_prop != "transform" {
                keyframes.push_str(&format!("    {}: {};\n", css_prop, value));
            }
        }
        keyframes.push_str("  }\n");

        keyframes.push('}');

        AnimationDefinition {
            name: anim_name,
            target: format!("motion.{}", element_type),
            duration,
            properties,
            easing,
            keyframes_css: Some(keyframes),
        }
    }
}

impl Default for FramerMotionParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for FramerMotionParser {
    fn name(&self) -> &'static str {
        "framer-motion"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::Animation
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        analysis.libraries.iter().any(|lib| lib == "framer-motion") ||
        analysis.raw.as_ref()
            .map(|r| r.imports.iter().any(|(lib, src)| lib == "framer-motion" || src.contains("framer-motion")))
            .unwrap_or(false)
    }

    fn analyze(&self, code: &str, _analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut anim_index = 0;

        // Find motion components
        for cap in self.motion_component.captures_iter(code) {
            let element_type = cap.get(1).map(|m| m.as_str()).unwrap_or("div");

            // Find the component's context (rough extraction)
            let start_pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
            let search_area = &code[start_pos..std::cmp::min(start_pos + 500, code.len())];

            // Extract initial props
            let initial = self.initial_prop.captures(search_area)
                .and_then(|c| c.get(1))
                .map(|m| self.parse_jsx_props(m.as_str()))
                .unwrap_or_default();

            // Extract animate props
            let animate = self.animate_prop.captures(search_area)
                .and_then(|c| c.get(1))
                .map(|m| self.parse_jsx_props(m.as_str()))
                .unwrap_or_default();

            // Extract transition props
            let transition = self.transition_prop.captures(search_area)
                .and_then(|c| c.get(1))
                .map(|m| self.parse_jsx_props(m.as_str()))
                .unwrap_or_default();

            if !initial.is_empty() || !animate.is_empty() {
                let anim = self.generate_animation(
                    element_type,
                    &initial,
                    &animate,
                    &transition,
                    anim_index,
                );

                let duration = transition.get("duration")
                    .and_then(|d| d.parse::<f64>().ok())
                    .unwrap_or(0.3);

                result.effects.push(LibraryEffect {
                    effect_type: EffectType::Animation,
                    target: format!("motion.{}", element_type),
                    css_properties: animate.clone(),
                    timing: Some(EffectTiming {
                        start: 0.0,
                        duration: duration * 1000.0,
                        easing: transition.get("ease").cloned().unwrap_or_else(|| "ease".to_string()),
                    }),
                });

                result.animations.insert(anim.name.clone(), anim);
                anim_index += 1;
            }
        }

        // Check for variants (complex animation states)
        if self.variants_prop.is_match(code) {
            result.warnings.push("Framer Motion variants detected - complex state animations may need manual handling".to_string());
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* Framer Motion Animation CSS */\n\n");

        for (_, anim) in &analysis.animations {
            if let Some(keyframes) = &anim.keyframes_css {
                css.push_str(keyframes);
                css.push_str("\n\n");
            }
        }

        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_framer_motion() {
        let parser = FramerMotionParser::new();

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("framer-motion".to_string());

        assert!(parser.handles(&analysis));
    }

    #[test]
    fn test_parse_motion_component() {
        let parser = FramerMotionParser::new();
        let code = r#"
            <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
            />
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("framer-motion".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.animations.is_empty());
    }
}
