//! GSAP (GreenSock Animation Platform) parser
//!
//! Parses GSAP animations and generates equivalent CSS keyframes.

use super::{
    AnimationDefinition, AnimationProperty, LibraryAnalysis, LibraryCategory,
    registry::{LibraryParser, LibraryEffect, EffectType, EffectTiming},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for GSAP animations
pub struct GsapParser {
    // Regex patterns for GSAP calls
    to_pattern: Regex,
    from_pattern: Regex,
    fromto_pattern: Regex,
    timeline_pattern: Regex,
    property_pattern: Regex,
}

impl GsapParser {
    pub fn new() -> Self {
        Self {
            to_pattern: Regex::new(r#"gsap\.to\s*\(\s*["']([^"']+)["']\s*,\s*\{([^}]+)\}"#).unwrap(),
            from_pattern: Regex::new(r#"gsap\.from\s*\(\s*["']([^"']+)["']\s*,\s*\{([^}]+)\}"#).unwrap(),
            fromto_pattern: Regex::new(r#"gsap\.fromTo\s*\(\s*["']([^"']+)["']"#).unwrap(),
            timeline_pattern: Regex::new(r#"gsap\.timeline\s*\("#).unwrap(),
            property_pattern: Regex::new(r#"(\w+)\s*:\s*(?:["']([^"']+)["']|(\d+(?:\.\d+)?))"#).unwrap(),
        }
    }

    /// Parse GSAP properties object
    fn parse_properties(&self, props_str: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();

        for cap in self.property_pattern.captures_iter(props_str) {
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

    /// Convert GSAP property to CSS property
    fn to_css_property(gsap_prop: &str) -> &str {
        match gsap_prop {
            "x" => "transform",
            "y" => "transform",
            "rotation" => "transform",
            "scale" => "transform",
            "scaleX" => "transform",
            "scaleY" => "transform",
            "skewX" => "transform",
            "skewY" => "transform",
            "opacity" => "opacity",
            "backgroundColor" => "background-color",
            "color" => "color",
            "width" => "width",
            "height" => "height",
            "padding" => "padding",
            "margin" => "margin",
            "borderRadius" => "border-radius",
            "fontSize" => "font-size",
            _ => gsap_prop,
        }
    }

    /// Convert GSAP transform value to CSS
    fn to_css_value(gsap_prop: &str, value: &str) -> String {
        match gsap_prop {
            "x" => format!("translateX({}px)", value),
            "y" => format!("translateY({}px)", value),
            "rotation" => format!("rotate({}deg)", value),
            "scale" => format!("scale({})", value),
            "scaleX" => format!("scaleX({})", value),
            "scaleY" => format!("scaleY({})", value),
            "skewX" => format!("skewX({}deg)", value),
            "skewY" => format!("skewY({}deg)", value),
            _ => value.to_string(),
        }
    }

    /// Convert GSAP easing to CSS timing function
    fn to_css_easing(gsap_ease: &str) -> &str {
        match gsap_ease {
            "none" | "linear" | "none.none" => "linear",
            "power1" | "power1.out" => "ease-out",
            "power1.in" => "ease-in",
            "power1.inOut" => "ease-in-out",
            "power2" | "power2.out" => "cubic-bezier(0.25, 0.46, 0.45, 0.94)",
            "power2.in" => "cubic-bezier(0.55, 0.06, 0.68, 0.19)",
            "power2.inOut" => "cubic-bezier(0.45, 0.03, 0.51, 0.95)",
            "power3" | "power3.out" => "cubic-bezier(0.22, 0.61, 0.36, 1)",
            "power3.in" => "cubic-bezier(0.64, 0, 0.78, 0)",
            "power3.inOut" => "cubic-bezier(0.77, 0, 0.175, 1)",
            "power4" | "power4.out" => "cubic-bezier(0.16, 1, 0.3, 1)",
            "power4.in" => "cubic-bezier(0.7, 0, 0.84, 0)",
            "back.out" => "cubic-bezier(0.34, 1.56, 0.64, 1)",
            "back.in" => "cubic-bezier(0.36, 0, 0.66, -0.56)",
            "elastic.out" => "cubic-bezier(0.68, -0.55, 0.27, 1.55)",
            "bounce.out" => "cubic-bezier(0.34, 1.56, 0.64, 1)",
            _ => "ease",
        }
    }

    /// Generate CSS keyframes from GSAP animation
    fn generate_keyframes(&self, target: &str, props: &HashMap<String, String>, is_from: bool) -> AnimationDefinition {
        let duration = props.get("duration")
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(1.0);

        let easing = props.get("ease")
            .map(|e| Self::to_css_easing(e))
            .unwrap_or("ease")
            .to_string();

        let anim_name = format!("gsap-{}", target.replace(['.', '#', ' '], "-"));

        let mut anim_props = HashMap::new();
        let mut transforms = Vec::new();

        for (key, value) in props {
            // Skip meta properties
            if ["duration", "ease", "delay", "stagger", "repeat", "yoyo"].contains(&key.as_str()) {
                continue;
            }

            let css_prop = Self::to_css_property(key);
            let css_val = Self::to_css_value(key, value);

            if css_prop == "transform" {
                transforms.push(css_val.clone());
            }

            anim_props.insert(key.clone(), AnimationProperty {
                from: if is_from { css_val.clone() } else { "initial".to_string() },
                to: if is_from { "initial".to_string() } else { css_val },
                css_property: css_prop.to_string(),
            });
        }

        // Generate keyframes CSS
        let mut keyframes = format!("@keyframes {} {{\n", anim_name);

        if is_from {
            keyframes.push_str("  0% {\n");
            if !transforms.is_empty() {
                keyframes.push_str(&format!("    transform: {};\n", transforms.join(" ")));
            }
            for (key, value) in props {
                if !["duration", "ease", "delay", "stagger", "repeat", "yoyo"].contains(&key.as_str()) {
                    let css_prop = Self::to_css_property(key);
                    if css_prop != "transform" {
                        let css_val = Self::to_css_value(key, value);
                        keyframes.push_str(&format!("    {}: {};\n", css_prop, css_val));
                    }
                }
            }
            keyframes.push_str("  }\n");
            keyframes.push_str("  100% {\n    /* Final state */\n  }\n");
        } else {
            keyframes.push_str("  0% {\n    /* Initial state */\n  }\n");
            keyframes.push_str("  100% {\n");
            if !transforms.is_empty() {
                keyframes.push_str(&format!("    transform: {};\n", transforms.join(" ")));
            }
            for (key, value) in props {
                if !["duration", "ease", "delay", "stagger", "repeat", "yoyo"].contains(&key.as_str()) {
                    let css_prop = Self::to_css_property(key);
                    if css_prop != "transform" {
                        let css_val = Self::to_css_value(key, value);
                        keyframes.push_str(&format!("    {}: {};\n", css_prop, css_val));
                    }
                }
            }
            keyframes.push_str("  }\n");
        }

        keyframes.push('}');

        // Also generate the animation class
        let anim_class = format!(
            "{} {{\n  animation: {} {}s {} forwards;\n}}\n",
            target, anim_name, duration, easing
        );

        AnimationDefinition {
            name: anim_name,
            target: target.to_string(),
            duration,
            properties: anim_props,
            easing,
            keyframes_css: Some(format!("{}\n\n{}", keyframes, anim_class)),
        }
    }
}

impl Default for GsapParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for GsapParser {
    fn name(&self) -> &'static str {
        "gsap"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::Animation
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        analysis.libraries.iter().any(|lib| lib == "gsap") ||
        analysis.raw.as_ref()
            .map(|r| r.imports.iter().any(|(lib, src)| lib == "gsap" || src.contains("gsap")))
            .unwrap_or(false)
    }

    fn analyze(&self, code: &str, _analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();

        // Find gsap.to() calls
        for cap in self.to_pattern.captures_iter(code) {
            let target = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let props_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let props = self.parse_properties(props_str);
            let anim = self.generate_keyframes(target, &props, false);

            result.animations.insert(anim.name.clone(), anim.clone());

            result.effects.push(LibraryEffect {
                effect_type: EffectType::Animation,
                target: target.to_string(),
                css_properties: props.clone(),
                timing: Some(EffectTiming {
                    start: 0.0,
                    duration: props.get("duration")
                        .and_then(|d| d.parse().ok())
                        .unwrap_or(1.0) * 1000.0,
                    easing: props.get("ease").cloned().unwrap_or_else(|| "ease".to_string()),
                }),
            });
        }

        // Find gsap.from() calls
        for cap in self.from_pattern.captures_iter(code) {
            let target = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let props_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let props = self.parse_properties(props_str);
            let anim = self.generate_keyframes(target, &props, true);

            result.animations.insert(anim.name.clone(), anim);
        }

        // Detect timeline usage
        if self.timeline_pattern.is_match(code) {
            result.warnings.push("GSAP timeline detected - complex sequencing may not be fully captured".to_string());
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* GSAP Animation CSS */\n\n");

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
    fn test_parse_gsap_to() {
        let parser = GsapParser::new();
        let code = r#"gsap.to(".box", { x: 100, opacity: 0.5, duration: 1 });"#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("gsap".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.animations.is_empty());
    }

    #[test]
    fn test_parse_gsap_from() {
        let parser = GsapParser::new();
        let code = r#"gsap.from(".title", { y: -50, opacity: 0, duration: 0.5 });"#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("gsap".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.animations.is_empty());
    }

    #[test]
    fn test_css_easing_conversion() {
        assert_eq!(GsapParser::to_css_easing("power2.out"), "cubic-bezier(0.25, 0.46, 0.45, 0.94)");
        assert_eq!(GsapParser::to_css_easing("linear"), "linear");
    }
}
