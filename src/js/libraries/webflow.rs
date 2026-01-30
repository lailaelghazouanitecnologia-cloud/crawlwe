//! Webflow/Tram animation parser
//!
//! Webflow uses "tram" for animations - a custom animation library.
//! This parser extracts animation patterns and generates CSS equivalents.

use super::{
    AnimationDefinition, AnimationProperty, LibraryAnalysis, LibraryCategory,
    registry::{LibraryParser, LibraryEffect, EffectType, EffectTiming},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for Webflow animations (tram library)
pub struct WebflowParser {
    // Tram patterns
    tram_init: Regex,
    tram_add: Regex,
    tram_start: Regex,
    tram_set: Regex,
    ix2_trigger: Regex,
    ix2_action: Regex,
    webflow_ready: Regex,
}

impl WebflowParser {
    pub fn new() -> Self {
        Self {
            tram_init: Regex::new(r#"window\.tram|tram\("#).unwrap(),
            tram_add: Regex::new(r#"\.add\(["']([^"']+)["']\s*,\s*\{([^}]+)\}"#).unwrap(),
            tram_start: Regex::new(r#"\.start\(\{([^}]+)\}\)"#).unwrap(),
            tram_set: Regex::new(r#"\.set\(\{([^}]+)\}\)"#).unwrap(),
            ix2_trigger: Regex::new(r#"ix2-trigger|data-w-id|w-ix-|\.w--"#).unwrap(),
            ix2_action: Regex::new(r#"["']actionType["']\s*:\s*["']([^"']+)["']"#).unwrap(),
            webflow_ready: Regex::new(r#"Webflow\.ready|Webflow\.require"#).unwrap(),
        }
    }

    /// Parse tram animation properties
    fn parse_tram_props(&self, props_str: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();
        let prop_re = Regex::new(r#"(\w+)\s*:\s*(?:["']([^"']+)["']|(\d+(?:\.\d+)?(?:s|ms)?)|([a-zA-Z-]+))"#).unwrap();

        for cap in prop_re.captures_iter(props_str) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2)
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
                .map(|m| m.as_str())
                .unwrap_or("");

            if !key.is_empty() && !value.is_empty() {
                props.insert(key.to_string(), value.to_string());
            }
        }

        props
    }

    /// Convert tram property to CSS
    fn to_css_property(tram_prop: &str) -> Option<&str> {
        match tram_prop {
            "x" | "translateX" => Some("transform"),
            "y" | "translateY" => Some("transform"),
            "z" | "translateZ" => Some("transform"),
            "rotate" | "rotateZ" => Some("transform"),
            "rotateX" => Some("transform"),
            "rotateY" => Some("transform"),
            "scale" => Some("transform"),
            "scaleX" => Some("transform"),
            "scaleY" => Some("transform"),
            "skew" => Some("transform"),
            "opacity" => Some("opacity"),
            "width" => Some("width"),
            "height" => Some("height"),
            "color" => Some("color"),
            "background" | "background-color" => Some("background-color"),
            "font-size" => Some("font-size"),
            "padding" => Some("padding"),
            "margin" => Some("margin"),
            _ => None,
        }
    }

    /// Convert tram value to CSS value
    fn to_css_value(prop: &str, value: &str) -> String {
        match prop {
            "x" | "translateX" => format!("translateX({})", Self::add_unit(value, "px")),
            "y" | "translateY" => format!("translateY({})", Self::add_unit(value, "px")),
            "z" | "translateZ" => format!("translateZ({})", Self::add_unit(value, "px")),
            "rotate" | "rotateZ" => format!("rotate({})", Self::add_unit(value, "deg")),
            "rotateX" => format!("rotateX({})", Self::add_unit(value, "deg")),
            "rotateY" => format!("rotateY({})", Self::add_unit(value, "deg")),
            "scale" | "scaleX" | "scaleY" => format!("{}({})", prop, value),
            _ => Self::add_unit(value, "px"),
        }
    }

    /// Add unit if not present
    fn add_unit(value: &str, default_unit: &str) -> String {
        if value.ends_with("px") || value.ends_with("em") || value.ends_with("rem") ||
           value.ends_with("%") || value.ends_with("deg") || value.ends_with("s") ||
           value.ends_with("ms") || value.parse::<f64>().is_err() {
            value.to_string()
        } else {
            format!("{}{}", value, default_unit)
        }
    }

    /// Convert Webflow IX2 action type to CSS animation
    fn ix2_action_to_css(action_type: &str) -> Option<String> {
        match action_type {
            "TRANSFORM_MOVE" => Some("transform: translate".to_string()),
            "TRANSFORM_SCALE" => Some("transform: scale".to_string()),
            "TRANSFORM_ROTATE" => Some("transform: rotate".to_string()),
            "STYLE_OPACITY" => Some("opacity".to_string()),
            "STYLE_SIZE" => Some("width, height".to_string()),
            "STYLE_BACKGROUND_COLOR" => Some("background-color".to_string()),
            "STYLE_TEXT_COLOR" => Some("color".to_string()),
            _ => None,
        }
    }
}

impl Default for WebflowParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for WebflowParser {
    fn name(&self) -> &'static str {
        "webflow"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::Animation
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        // Check for Webflow patterns in code or imports
        if let Some(raw) = &analysis.raw {
            // Check if any code contains tram/webflow patterns
            for (_, src) in &raw.imports {
                if src.contains("webflow") || src.contains("w-") {
                    return true;
                }
            }
        }

        // Also check classes for Webflow patterns
        analysis.classes.iter().any(|c|
            c.starts_with("w-") ||
            c.starts_with("w--") ||
            c.contains("webflow")
        )
    }

    fn analyze(&self, code: &str, _analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut anim_index = 0;

        // Check for tram initialization
        let has_tram = self.tram_init.is_match(code);
        let has_webflow = self.webflow_ready.is_match(code);
        let has_ix2 = self.ix2_trigger.is_match(code);

        if has_tram {
            result.warnings.push("Webflow tram animation library detected".to_string());
        }

        if has_webflow {
            result.warnings.push("Webflow.ready initialization found".to_string());
        }

        if has_ix2 {
            result.warnings.push("Webflow IX2 interactions detected".to_string());
        }

        // Parse tram.add() calls
        for cap in self.tram_add.captures_iter(code) {
            let target = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let props_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let props = self.parse_tram_props(props_str);

            if !props.is_empty() {
                let anim_name = format!("webflow-{}", anim_index);
                let mut css_props = HashMap::new();
                let mut transforms = Vec::new();

                for (key, value) in &props {
                    if let Some(css_prop) = Self::to_css_property(key) {
                        let css_val = Self::to_css_value(key, value);
                        if css_prop == "transform" {
                            transforms.push(css_val);
                        } else {
                            css_props.insert(css_prop.to_string(), css_val);
                        }
                    }
                }

                // Generate keyframes
                let mut keyframes = format!("@keyframes {} {{\n", anim_name);
                keyframes.push_str("  0% { /* initial state */ }\n");
                keyframes.push_str("  100% {\n");

                if !transforms.is_empty() {
                    keyframes.push_str(&format!("    transform: {};\n", transforms.join(" ")));
                }
                for (prop, val) in &css_props {
                    keyframes.push_str(&format!("    {}: {};\n", prop, val));
                }
                keyframes.push_str("  }\n}\n");

                // Animation class
                let duration = props.get("duration").cloned().unwrap_or_else(|| "0.5s".to_string());
                let easing = props.get("easing").cloned().unwrap_or_else(|| "ease".to_string());

                let anim_class = format!(
                    "{} {{\n  animation: {} {} {} forwards;\n}}\n",
                    target, anim_name, duration, easing
                );

                result.animations.insert(anim_name.clone(), AnimationDefinition {
                    name: anim_name,
                    target: target.to_string(),
                    duration: duration.trim_end_matches('s').parse().unwrap_or(0.5),
                    properties: HashMap::new(),
                    easing,
                    keyframes_css: Some(format!("{}\n{}", keyframes, anim_class)),
                });

                anim_index += 1;
            }
        }

        // Parse IX2 action types
        for cap in self.ix2_action.captures_iter(code) {
            let action_type = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if let Some(css_hint) = Self::ix2_action_to_css(action_type) {
                result.effects.push(LibraryEffect {
                    effect_type: EffectType::Animation,
                    target: format!("[data-w-id]"),
                    css_properties: {
                        let mut props = HashMap::new();
                        props.insert("animation-type".to_string(), css_hint);
                        props
                    },
                    timing: None,
                });
            }
        }

        // Generate base Webflow animation utilities CSS
        result.generated_css.push(r#"
/* Webflow Animation Utilities */
.w-ix-hide {
  opacity: 0;
}

[data-w-id] {
  will-change: transform, opacity;
}

/* Common Webflow animation states */
.w--current {
  /* Active state styling */
}
"#.to_string());

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* Webflow Animation CSS */\n\n");

        for block in &analysis.generated_css {
            css.push_str(block);
            css.push_str("\n\n");
        }

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
    fn test_detect_webflow() {
        let parser = WebflowParser::new();

        let mut analysis = JsAnalysisResult::default();
        analysis.classes = vec!["w-nav".to_string(), "w--current".to_string()];

        assert!(parser.handles(&analysis));
    }

    #[test]
    fn test_detect_tram() {
        let parser = WebflowParser::new();
        let code = r#"
            window.tram = function(t) {
                // tram implementation
            };
        "#;

        let analysis = JsAnalysisResult::default();
        let result = parser.analyze(code, &analysis);

        assert!(result.warnings.iter().any(|w| w.contains("tram")));
    }
}
