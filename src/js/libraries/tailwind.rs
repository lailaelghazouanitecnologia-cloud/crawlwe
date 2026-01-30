//! Tailwind CSS parser
//!
//! Analyzes Tailwind utility classes and generates equivalent CSS.

use super::{
    LibraryAnalysis, LibraryCategory,
    registry::{LibraryParser, LibraryEffect, EffectType},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for Tailwind CSS utility classes
pub struct TailwindParser {
    // Common Tailwind patterns
    spacing_pattern: Regex,
    color_pattern: Regex,
    sizing_pattern: Regex,
    flex_pattern: Regex,
    grid_pattern: Regex,
    text_pattern: Regex,
    border_pattern: Regex,
    shadow_pattern: Regex,
    hover_pattern: Regex,
    responsive_pattern: Regex,
}

impl TailwindParser {
    pub fn new() -> Self {
        Self {
            spacing_pattern: Regex::new(r"^([pm][xytblr]?)-(\d+(?:\.\d+)?|px|auto)$").unwrap(),
            color_pattern: Regex::new(r"^(bg|text|border)-(\w+)-(\d+)$").unwrap(),
            sizing_pattern: Regex::new(r"^([wh])-(\d+(?:\.\d+)?|full|screen|auto|min|max|fit)$").unwrap(),
            flex_pattern: Regex::new(r"^(flex|items|justify|gap|grow|shrink|basis)-?(.*)$").unwrap(),
            grid_pattern: Regex::new(r"^(grid|col|row)-?(.*)$").unwrap(),
            text_pattern: Regex::new(r"^(text|font|leading|tracking)-(.+)$").unwrap(),
            border_pattern: Regex::new(r"^(border|rounded)-?(.*)$").unwrap(),
            shadow_pattern: Regex::new(r"^shadow(-.*)?$").unwrap(),
            hover_pattern: Regex::new(r"^hover:(.+)$").unwrap(),
            responsive_pattern: Regex::new(r"^(sm|md|lg|xl|2xl):(.+)$").unwrap(),
        }
    }

    /// Convert Tailwind spacing value to CSS
    fn spacing_to_css(value: &str) -> String {
        match value {
            "px" => "1px".to_string(),
            "auto" => "auto".to_string(),
            _ => {
                if let Ok(num) = value.parse::<f64>() {
                    format!("{}rem", num * 0.25)
                } else {
                    value.to_string()
                }
            }
        }
    }

    /// Convert Tailwind color to CSS
    fn color_to_css(color: &str, shade: &str) -> String {
        // Common Tailwind color values (simplified)
        let base = match color {
            "slate" => match shade {
                "50" => "#f8fafc", "100" => "#f1f5f9", "200" => "#e2e8f0",
                "300" => "#cbd5e1", "400" => "#94a3b8", "500" => "#64748b",
                "600" => "#475569", "700" => "#334155", "800" => "#1e293b",
                "900" => "#0f172a", _ => "#64748b"
            },
            "gray" => match shade {
                "50" => "#f9fafb", "100" => "#f3f4f6", "200" => "#e5e7eb",
                "300" => "#d1d5db", "400" => "#9ca3af", "500" => "#6b7280",
                "600" => "#4b5563", "700" => "#374151", "800" => "#1f2937",
                "900" => "#111827", _ => "#6b7280"
            },
            "red" => match shade {
                "50" => "#fef2f2", "100" => "#fee2e2", "200" => "#fecaca",
                "300" => "#fca5a5", "400" => "#f87171", "500" => "#ef4444",
                "600" => "#dc2626", "700" => "#b91c1c", "800" => "#991b1b",
                "900" => "#7f1d1d", _ => "#ef4444"
            },
            "blue" => match shade {
                "50" => "#eff6ff", "100" => "#dbeafe", "200" => "#bfdbfe",
                "300" => "#93c5fd", "400" => "#60a5fa", "500" => "#3b82f6",
                "600" => "#2563eb", "700" => "#1d4ed8", "800" => "#1e40af",
                "900" => "#1e3a8a", _ => "#3b82f6"
            },
            "green" => match shade {
                "50" => "#f0fdf4", "100" => "#dcfce7", "200" => "#bbf7d0",
                "300" => "#86efac", "400" => "#4ade80", "500" => "#22c55e",
                "600" => "#16a34a", "700" => "#15803d", "800" => "#166534",
                "900" => "#14532d", _ => "#22c55e"
            },
            "yellow" => match shade {
                "50" => "#fefce8", "100" => "#fef9c3", "200" => "#fef08a",
                "300" => "#fde047", "400" => "#facc15", "500" => "#eab308",
                "600" => "#ca8a04", "700" => "#a16207", "800" => "#854d0e",
                "900" => "#713f12", _ => "#eab308"
            },
            "purple" => match shade {
                "50" => "#faf5ff", "100" => "#f3e8ff", "200" => "#e9d5ff",
                "300" => "#d8b4fe", "400" => "#c084fc", "500" => "#a855f7",
                "600" => "#9333ea", "700" => "#7e22ce", "800" => "#6b21a8",
                "900" => "#581c87", _ => "#a855f7"
            },
            "white" => "#ffffff",
            "black" => "#000000",
            "transparent" => "transparent",
            _ => color,
        };
        base.to_string()
    }

    /// Convert Tailwind sizing value to CSS
    fn sizing_to_css(value: &str) -> String {
        match value {
            "full" => "100%".to_string(),
            "screen" => "100vh".to_string(),
            "auto" => "auto".to_string(),
            "min" => "min-content".to_string(),
            "max" => "max-content".to_string(),
            "fit" => "fit-content".to_string(),
            _ => {
                if let Ok(num) = value.parse::<f64>() {
                    format!("{}rem", num * 0.25)
                } else {
                    value.to_string()
                }
            }
        }
    }

    /// Generate CSS for a single Tailwind class
    pub fn class_to_css(&self, class: &str) -> Option<(String, String)> {
        // Spacing (margin, padding)
        if let Some(cap) = self.spacing_pattern.captures(class) {
            let prop_prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let css_value = Self::spacing_to_css(value);
            let css_prop = match prop_prefix {
                "m" => "margin",
                "mx" => "margin-inline",
                "my" => "margin-block",
                "mt" => "margin-top",
                "mr" => "margin-right",
                "mb" => "margin-bottom",
                "ml" => "margin-left",
                "p" => "padding",
                "px" => "padding-inline",
                "py" => "padding-block",
                "pt" => "padding-top",
                "pr" => "padding-right",
                "pb" => "padding-bottom",
                "pl" => "padding-left",
                _ => return None,
            };

            return Some((css_prop.to_string(), css_value));
        }

        // Colors
        if let Some(cap) = self.color_pattern.captures(class) {
            let prop_type = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let color = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let shade = cap.get(3).map(|m| m.as_str()).unwrap_or("500");

            let css_value = Self::color_to_css(color, shade);
            let css_prop = match prop_type {
                "bg" => "background-color",
                "text" => "color",
                "border" => "border-color",
                _ => return None,
            };

            return Some((css_prop.to_string(), css_value));
        }

        // Sizing
        if let Some(cap) = self.sizing_pattern.captures(class) {
            let prop_type = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let css_value = Self::sizing_to_css(value);
            let css_prop = match prop_type {
                "w" => "width",
                "h" => "height",
                _ => return None,
            };

            return Some((css_prop.to_string(), css_value));
        }

        // Common utilities
        match class {
            // Display
            "flex" => Some(("display".to_string(), "flex".to_string())),
            "grid" => Some(("display".to_string(), "grid".to_string())),
            "block" => Some(("display".to_string(), "block".to_string())),
            "inline" => Some(("display".to_string(), "inline".to_string())),
            "inline-block" => Some(("display".to_string(), "inline-block".to_string())),
            "hidden" => Some(("display".to_string(), "none".to_string())),

            // Flex
            "flex-row" => Some(("flex-direction".to_string(), "row".to_string())),
            "flex-col" => Some(("flex-direction".to_string(), "column".to_string())),
            "flex-wrap" => Some(("flex-wrap".to_string(), "wrap".to_string())),
            "items-center" => Some(("align-items".to_string(), "center".to_string())),
            "items-start" => Some(("align-items".to_string(), "flex-start".to_string())),
            "items-end" => Some(("align-items".to_string(), "flex-end".to_string())),
            "justify-center" => Some(("justify-content".to_string(), "center".to_string())),
            "justify-between" => Some(("justify-content".to_string(), "space-between".to_string())),
            "justify-around" => Some(("justify-content".to_string(), "space-around".to_string())),

            // Position
            "relative" => Some(("position".to_string(), "relative".to_string())),
            "absolute" => Some(("position".to_string(), "absolute".to_string())),
            "fixed" => Some(("position".to_string(), "fixed".to_string())),
            "sticky" => Some(("position".to_string(), "sticky".to_string())),

            // Text
            "text-center" => Some(("text-align".to_string(), "center".to_string())),
            "text-left" => Some(("text-align".to_string(), "left".to_string())),
            "text-right" => Some(("text-align".to_string(), "right".to_string())),
            "font-bold" => Some(("font-weight".to_string(), "700".to_string())),
            "font-semibold" => Some(("font-weight".to_string(), "600".to_string())),
            "font-medium" => Some(("font-weight".to_string(), "500".to_string())),
            "font-normal" => Some(("font-weight".to_string(), "400".to_string())),
            "font-light" => Some(("font-weight".to_string(), "300".to_string())),

            // Text sizes
            "text-xs" => Some(("font-size".to_string(), "0.75rem".to_string())),
            "text-sm" => Some(("font-size".to_string(), "0.875rem".to_string())),
            "text-base" => Some(("font-size".to_string(), "1rem".to_string())),
            "text-lg" => Some(("font-size".to_string(), "1.125rem".to_string())),
            "text-xl" => Some(("font-size".to_string(), "1.25rem".to_string())),
            "text-2xl" => Some(("font-size".to_string(), "1.5rem".to_string())),
            "text-3xl" => Some(("font-size".to_string(), "1.875rem".to_string())),
            "text-4xl" => Some(("font-size".to_string(), "2.25rem".to_string())),

            // Border radius
            "rounded" => Some(("border-radius".to_string(), "0.25rem".to_string())),
            "rounded-sm" => Some(("border-radius".to_string(), "0.125rem".to_string())),
            "rounded-md" => Some(("border-radius".to_string(), "0.375rem".to_string())),
            "rounded-lg" => Some(("border-radius".to_string(), "0.5rem".to_string())),
            "rounded-xl" => Some(("border-radius".to_string(), "0.75rem".to_string())),
            "rounded-2xl" => Some(("border-radius".to_string(), "1rem".to_string())),
            "rounded-full" => Some(("border-radius".to_string(), "9999px".to_string())),

            // Shadow
            "shadow" => Some(("box-shadow".to_string(), "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)".to_string())),
            "shadow-sm" => Some(("box-shadow".to_string(), "0 1px 2px 0 rgb(0 0 0 / 0.05)".to_string())),
            "shadow-md" => Some(("box-shadow".to_string(), "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)".to_string())),
            "shadow-lg" => Some(("box-shadow".to_string(), "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)".to_string())),
            "shadow-xl" => Some(("box-shadow".to_string(), "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)".to_string())),
            "shadow-none" => Some(("box-shadow".to_string(), "none".to_string())),

            // Overflow
            "overflow-hidden" => Some(("overflow".to_string(), "hidden".to_string())),
            "overflow-auto" => Some(("overflow".to_string(), "auto".to_string())),
            "overflow-scroll" => Some(("overflow".to_string(), "scroll".to_string())),

            // Transition
            "transition" => Some(("transition".to_string(), "all 150ms cubic-bezier(0.4, 0, 0.2, 1)".to_string())),
            "transition-all" => Some(("transition".to_string(), "all 150ms cubic-bezier(0.4, 0, 0.2, 1)".to_string())),
            "transition-colors" => Some(("transition".to_string(), "color, background-color, border-color 150ms cubic-bezier(0.4, 0, 0.2, 1)".to_string())),
            "transition-opacity" => Some(("transition".to_string(), "opacity 150ms cubic-bezier(0.4, 0, 0.2, 1)".to_string())),
            "transition-transform" => Some(("transition".to_string(), "transform 150ms cubic-bezier(0.4, 0, 0.2, 1)".to_string())),

            // Cursor
            "cursor-pointer" => Some(("cursor".to_string(), "pointer".to_string())),
            "cursor-default" => Some(("cursor".to_string(), "default".to_string())),
            "cursor-not-allowed" => Some(("cursor".to_string(), "not-allowed".to_string())),

            _ => None,
        }
    }
}

impl Default for TailwindParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for TailwindParser {
    fn name(&self) -> &'static str {
        "tailwind"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::CssUtility
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        // Check if any classes look like Tailwind utilities
        let tailwind_prefixes = ["bg-", "text-", "p-", "m-", "flex", "grid", "w-", "h-", "rounded", "shadow"];

        for class in &analysis.classes {
            for prefix in &tailwind_prefixes {
                if class.starts_with(prefix) || class == *prefix {
                    return true;
                }
            }
        }

        if let Some(raw) = &analysis.raw {
            for class in &raw.classes {
                for prefix in &tailwind_prefixes {
                    if class.starts_with(prefix) || class == *prefix {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn analyze(&self, _code: &str, analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut all_classes = Vec::new();

        // Collect all classes
        all_classes.extend(analysis.classes.clone());
        if let Some(raw) = &analysis.raw {
            all_classes.extend(raw.classes.clone());
            for jsx_class in &raw.jsx_classnames {
                for class in jsx_class.split_whitespace() {
                    if !all_classes.contains(&class.to_string()) {
                        all_classes.push(class.to_string());
                    }
                }
            }
        }

        // Generate CSS for each Tailwind class
        let mut css_rules = HashMap::new();

        for class in &all_classes {
            // Skip hover/responsive variants for now (would need media queries)
            if class.contains(':') {
                continue;
            }

            if let Some((prop, value)) = self.class_to_css(class) {
                css_rules.insert(class.clone(), (prop, value));
            }
        }

        // Generate CSS output
        if !css_rules.is_empty() {
            let mut css = String::from("/* Tailwind CSS Utilities */\n\n");

            for (class, (prop, value)) in &css_rules {
                css.push_str(&format!(".{} {{\n  {}: {};\n}}\n\n", class, prop, value));
            }

            result.generated_css.push(css);
        }

        // Add effects
        for (class, (prop, value)) in css_rules {
            result.effects.push(LibraryEffect {
                effect_type: EffectType::StyleChange,
                target: format!(".{}", class),
                css_properties: {
                    let mut props = HashMap::new();
                    props.insert(prop, value);
                    props
                },
                timing: None,
            });
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        analysis.generated_css.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_conversion() {
        let parser = TailwindParser::new();

        assert_eq!(parser.class_to_css("p-4"), Some(("padding".to_string(), "1rem".to_string())));
        assert_eq!(parser.class_to_css("mt-2"), Some(("margin-top".to_string(), "0.5rem".to_string())));
    }

    #[test]
    fn test_color_conversion() {
        let parser = TailwindParser::new();

        assert_eq!(parser.class_to_css("bg-blue-500"), Some(("background-color".to_string(), "#3b82f6".to_string())));
        assert_eq!(parser.class_to_css("text-gray-700"), Some(("color".to_string(), "#374151".to_string())));
    }

    #[test]
    fn test_detect_tailwind() {
        let parser = TailwindParser::new();

        let mut analysis = JsAnalysisResult::default();
        analysis.classes = vec!["flex".to_string(), "bg-blue-500".to_string(), "p-4".to_string()];

        assert!(parser.handles(&analysis));
    }
}
