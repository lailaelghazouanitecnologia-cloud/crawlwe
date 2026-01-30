//! anime.js parser
//!
//! Parses anime.js animations and generates CSS keyframes.

use super::{
    AnimationDefinition, AnimationProperty, LibraryAnalysis, LibraryCategory,
    registry::{LibraryParser, LibraryEffect, EffectType, EffectTiming},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for anime.js animations
pub struct AnimeJsParser {
    // Anime.js patterns
    anime_call: Regex,
    targets_pattern: Regex,
    property_pattern: Regex,
    keyframes_pattern: Regex,
    stagger_pattern: Regex,
    timeline_pattern: Regex,
}

impl AnimeJsParser {
    pub fn new() -> Self {
        Self {
            anime_call: Regex::new(r#"anime\s*\(\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}"#).unwrap(),
            targets_pattern: Regex::new(r#"targets\s*:\s*["']([^"']+)["']"#).unwrap(),
            property_pattern: Regex::new(r#"(\w+)\s*:\s*(?:["']([^"']+)["']|(\d+(?:\.\d+)?)|(\[[^\]]+\]))"#).unwrap(),
            keyframes_pattern: Regex::new(r#"keyframes\s*:\s*\[([^\]]+)\]"#).unwrap(),
            stagger_pattern: Regex::new(r#"anime\.stagger\s*\(([^)]+)\)"#).unwrap(),
            timeline_pattern: Regex::new(r#"anime\.timeline\s*\("#).unwrap(),
        }
    }

    /// Parse anime.js config object
    fn parse_config(&self, config_str: &str) -> HashMap<String, String> {
        let mut config = HashMap::new();

        for cap in self.property_pattern.captures_iter(config_str) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2)
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
                .map(|m| m.as_str())
                .unwrap_or("");

            if !key.is_empty() && !value.is_empty() {
                config.insert(key.to_string(), value.to_string());
            }
        }

        config
    }

    /// Convert anime.js property to CSS
    fn to_css_property(anime_prop: &str) -> Option<&str> {
        match anime_prop {
            "translateX" | "translateY" | "translateZ" |
            "rotate" | "rotateX" | "rotateY" | "rotateZ" |
            "scale" | "scaleX" | "scaleY" => Some("transform"),
            "opacity" => Some("opacity"),
            "backgroundColor" => Some("background-color"),
            "color" => Some("color"),
            "width" => Some("width"),
            "height" => Some("height"),
            "borderRadius" => Some("border-radius"),
            "padding" => Some("padding"),
            "margin" => Some("margin"),
            "top" | "right" | "bottom" | "left" => Some(anime_prop),
            // Skip non-CSS properties
            "targets" | "duration" | "delay" | "easing" |
            "loop" | "direction" | "autoplay" | "round" => None,
            _ => Some(anime_prop),
        }
    }

    /// Convert anime.js value to CSS value
    fn to_css_value(prop: &str, value: &str) -> String {
        match prop {
            "translateX" | "translateY" => {
                if value.ends_with("px") || value.ends_with("%") || value.ends_with("em") {
                    format!("{}({})", prop, value)
                } else {
                    format!("{}({}px)", prop, value)
                }
            }
            "translateZ" => format!("translateZ({}px)", value.trim_end_matches("px")),
            "rotate" | "rotateX" | "rotateY" | "rotateZ" => {
                let val = value.trim_end_matches("deg").trim_end_matches("turn");
                format!("{}({}deg)", prop, val)
            }
            "scale" | "scaleX" | "scaleY" => format!("{}({})", prop, value),
            _ => value.to_string(),
        }
    }

    /// Convert anime.js easing to CSS timing function
    fn to_css_easing(anime_ease: &str) -> &str {
        match anime_ease {
            "linear" => "linear",
            "easeInQuad" => "cubic-bezier(0.55, 0.085, 0.68, 0.53)",
            "easeOutQuad" => "cubic-bezier(0.25, 0.46, 0.45, 0.94)",
            "easeInOutQuad" => "cubic-bezier(0.455, 0.03, 0.515, 0.955)",
            "easeInCubic" => "cubic-bezier(0.55, 0.055, 0.675, 0.19)",
            "easeOutCubic" => "cubic-bezier(0.215, 0.61, 0.355, 1)",
            "easeInOutCubic" => "cubic-bezier(0.645, 0.045, 0.355, 1)",
            "easeInQuart" => "cubic-bezier(0.895, 0.03, 0.685, 0.22)",
            "easeOutQuart" => "cubic-bezier(0.165, 0.84, 0.44, 1)",
            "easeInOutQuart" => "cubic-bezier(0.77, 0, 0.175, 1)",
            "easeInExpo" => "cubic-bezier(0.95, 0.05, 0.795, 0.035)",
            "easeOutExpo" => "cubic-bezier(0.19, 1, 0.22, 1)",
            "easeInOutExpo" => "cubic-bezier(1, 0, 0, 1)",
            "easeInBack" => "cubic-bezier(0.6, -0.28, 0.735, 0.045)",
            "easeOutBack" => "cubic-bezier(0.175, 0.885, 0.32, 1.275)",
            "easeInOutBack" => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
            _ => "ease",
        }
    }

    /// Generate CSS keyframes from anime.js config
    fn generate_animation(&self, target: &str, config: &HashMap<String, String>, index: usize) -> AnimationDefinition {
        let anim_name = format!("anime-{}", index);

        let duration = config.get("duration")
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(1000.0) / 1000.0; // Convert ms to s

        let easing = config.get("easing")
            .map(|e| Self::to_css_easing(e))
            .unwrap_or("ease")
            .to_string();

        let mut properties = HashMap::new();
        let mut to_transforms = Vec::new();

        for (key, value) in config {
            if let Some(css_prop) = Self::to_css_property(key) {
                let css_val = Self::to_css_value(key, value);

                if css_prop == "transform" {
                    to_transforms.push(css_val.clone());
                }

                properties.insert(key.clone(), AnimationProperty {
                    from: "initial".to_string(),
                    to: css_val,
                    css_property: css_prop.to_string(),
                });
            }
        }

        // Generate keyframes
        let mut keyframes = format!("@keyframes {} {{\n", anim_name);

        keyframes.push_str("  0% {\n    /* Initial state */\n  }\n");
        keyframes.push_str("  100% {\n");

        if !to_transforms.is_empty() {
            keyframes.push_str(&format!("    transform: {};\n", to_transforms.join(" ")));
        }

        for (key, value) in config {
            if let Some(css_prop) = Self::to_css_property(key) {
                if css_prop != "transform" {
                    keyframes.push_str(&format!("    {}: {};\n", css_prop, value));
                }
            }
        }

        keyframes.push_str("  }\n}\n");

        // Animation class
        let anim_class = format!(
            "{} {{\n  animation: {} {}s {} forwards;\n}}\n",
            target, anim_name, duration, easing
        );

        AnimationDefinition {
            name: anim_name,
            target: target.to_string(),
            duration,
            properties,
            easing,
            keyframes_css: Some(format!("{}\n{}", keyframes, anim_class)),
        }
    }
}

impl Default for AnimeJsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for AnimeJsParser {
    fn name(&self) -> &'static str {
        "animejs"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::Animation
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        analysis.libraries.iter().any(|lib| lib == "animejs" || lib.contains("anime")) ||
        analysis.raw.as_ref()
            .map(|r| r.imports.iter().any(|(_, src)| src.contains("animejs") || src.contains("anime")))
            .unwrap_or(false)
    }

    fn analyze(&self, code: &str, _analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut anim_index = 0;

        // Find anime() calls
        for cap in self.anime_call.captures_iter(code) {
            let config_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Extract targets
            let target = self.targets_pattern.captures(config_str)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str())
                .unwrap_or(".animated");

            let config = self.parse_config(config_str);

            if !config.is_empty() {
                let anim = self.generate_animation(target, &config, anim_index);

                let duration = config.get("duration")
                    .and_then(|d| d.parse::<f64>().ok())
                    .unwrap_or(1000.0);

                result.effects.push(LibraryEffect {
                    effect_type: EffectType::Animation,
                    target: target.to_string(),
                    css_properties: config.clone(),
                    timing: Some(EffectTiming {
                        start: config.get("delay").and_then(|d| d.parse().ok()).unwrap_or(0.0),
                        duration,
                        easing: config.get("easing").cloned().unwrap_or_else(|| "ease".to_string()),
                    }),
                });

                result.animations.insert(anim.name.clone(), anim);
                anim_index += 1;
            }
        }

        // Check for timeline
        if self.timeline_pattern.is_match(code) {
            result.warnings.push("anime.js timeline detected - sequenced animations may need manual handling".to_string());
        }

        // Check for stagger
        if self.stagger_pattern.is_match(code) {
            result.warnings.push("anime.stagger() detected - staggered animations require JavaScript".to_string());
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* anime.js Animation CSS */\n\n");

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
    fn test_parse_anime_call() {
        let parser = AnimeJsParser::new();
        let code = r#"
            anime({
                targets: '.box',
                translateX: 250,
                rotate: '1turn',
                duration: 800
            });
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("animejs".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.animations.is_empty());
    }

    #[test]
    fn test_css_easing_conversion() {
        assert_eq!(AnimeJsParser::to_css_easing("easeOutQuad"), "cubic-bezier(0.25, 0.46, 0.45, 0.94)");
        assert_eq!(AnimeJsParser::to_css_easing("linear"), "linear");
    }
}
