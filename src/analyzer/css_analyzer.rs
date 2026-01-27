//! CSS content analyzer
//!
//! Analyzes CSS for animations, transitions, transforms, layout patterns,
//! variables, colors, fonts, and visual effects.

use std::collections::{HashMap, HashSet};

use super::patterns::*;
use super::{
    AnimationAnalysis, CssAnalysis, KeyframeInfo, LayoutAnalysis, PseudoStateAnalysis,
    TransformAnalysis,
};

pub struct CssAnalyzer;

impl CssAnalyzer {
    /// Perform complete CSS analysis
    pub fn analyze(css: &str) -> CssAnalysis {
        CssAnalysis {
            animations: Self::analyze_animations(css),
            transitions: Self::extract_transitions(css),
            transforms: Self::analyze_transforms(css),
            layout: Self::analyze_layout(css),
            variables: Self::extract_variables(css),
            colors: Self::extract_colors(css),
            fonts: Self::extract_fonts(css),
            shadows: Self::extract_shadows(css),
            filters: Self::extract_filters(css),
            media_queries: Self::extract_media_queries(css),
            pseudo_states: Self::analyze_pseudo_states(css),
        }
    }

    /// Analyze CSS animations
    fn analyze_animations(css: &str) -> AnimationAnalysis {
        // Extract keyframes
        let keyframes: Vec<KeyframeInfo> = KEYFRAMES_PATTERN
            .captures_iter(css)
            .map(|cap| KeyframeInfo {
                name: cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
                body: cap.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default(),
            })
            .collect();

        // Extract animation properties
        let animation_properties: Vec<String> = ANIMATION_PATTERN
            .captures_iter(css)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .filter(|s| !s.is_empty() && s != "none" && s.len() < 200)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Extract timing functions
        let timing_functions: Vec<String> = TIMING_FUNCTION_PATTERN
            .find_iter(css)
            .map(|m| m.as_str().to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Extract durations
        let durations: Vec<String> = DURATION_PATTERN
            .find_iter(css)
            .map(|m| m.as_str().to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        AnimationAnalysis {
            keyframes,
            animation_properties,
            timing_functions,
            durations,
        }
    }

    /// Extract transitions
    fn extract_transitions(css: &str) -> Vec<String> {
        TRANSITION_PATTERN
            .captures_iter(css)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .filter(|s| !s.is_empty() && s != "none" && s.len() < 150)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Analyze transforms
    fn analyze_transforms(css: &str) -> TransformAnalysis {
        let values: Vec<String> = TRANSFORM_PATTERN
            .captures_iter(css)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .filter(|s| !s.is_empty() && s != "none" && s.len() < 200)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let combined = values.join(" ");

        TransformAnalysis {
            values,
            has_translate: combined.contains("translate"),
            has_rotate: combined.contains("rotate"),
            has_scale: combined.contains("scale"),
            has_skew: combined.contains("skew"),
            has_matrix: combined.contains("matrix"),
            has_3d: combined.contains("3d"),
        }
    }

    /// Analyze layout patterns
    fn analyze_layout(css: &str) -> LayoutAnalysis {
        LayoutAnalysis {
            flexbox_count: count_matches(&DISPLAY_FLEX_PATTERN, css),
            grid_count: count_matches(&DISPLAY_GRID_PATTERN, css),
            position_fixed: count_matches(&POSITION_FIXED_PATTERN, css),
            position_sticky: count_matches(&POSITION_STICKY_PATTERN, css),
            position_absolute: count_matches(&POSITION_ABSOLUTE_PATTERN, css),
        }
    }

    /// Extract CSS variables
    fn extract_variables(css: &str) -> HashMap<String, String> {
        CSS_VAR_DEF_PATTERN
            .captures_iter(css)
            .take(100) // Limit to 100 variables
            .filter_map(|cap| {
                let name = cap.get(1)?.as_str().to_string();
                let value = cap.get(2)?.as_str().trim().to_string();
                Some((name, value))
            })
            .collect()
    }

    /// Extract colors from CSS
    fn extract_colors(css: &str) -> Vec<String> {
        let mut colors = HashSet::new();

        // Hex colors
        for m in HEX_COLOR_PATTERN.find_iter(css) {
            colors.insert(m.as_str().to_lowercase());
        }

        // RGB/RGBA
        for m in RGB_COLOR_PATTERN.find_iter(css) {
            colors.insert(m.as_str().to_string());
        }

        // HSL/HSLA
        for m in HSL_COLOR_PATTERN.find_iter(css) {
            colors.insert(m.as_str().to_string());
        }

        let mut result: Vec<_> = colors.into_iter().collect();
        result.sort();
        result.truncate(50); // Limit colors
        result
    }

    /// Extract font families
    fn extract_fonts(css: &str) -> Vec<String> {
        let generic_fonts = [
            "sans-serif", "serif", "monospace", "cursive", "fantasy",
            "system-ui", "inherit", "initial", "unset",
        ];

        let mut fonts = HashSet::new();

        for cap in FONT_FAMILY_PATTERN.captures_iter(css) {
            if let Some(families) = cap.get(1) {
                for family in families.as_str().split(',') {
                    let family = family
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();

                    if !family.is_empty()
                        && !generic_fonts.contains(&family.to_lowercase().as_str())
                    {
                        fonts.insert(family);
                    }
                }
            }
        }

        let mut result: Vec<_> = fonts.into_iter().collect();
        result.sort();
        result
    }

    /// Extract box shadows
    fn extract_shadows(css: &str) -> Vec<String> {
        BOX_SHADOW_PATTERN
            .captures_iter(css)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .filter(|s| !s.is_empty() && s != "none")
            .take(20)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Extract filters
    fn extract_filters(css: &str) -> Vec<String> {
        let mut filters = HashSet::new();

        for cap in FILTER_PATTERN.captures_iter(css) {
            if let Some(value) = cap.get(1) {
                let v = value.as_str().trim();
                if !v.is_empty() && v != "none" {
                    filters.insert(v.to_string());
                }
            }
        }

        for cap in BACKDROP_FILTER_PATTERN.captures_iter(css) {
            if let Some(value) = cap.get(1) {
                let v = value.as_str().trim();
                if !v.is_empty() && v != "none" {
                    filters.insert(format!("backdrop: {}", v));
                }
            }
        }

        filters.into_iter().take(15).collect()
    }

    /// Extract media queries
    fn extract_media_queries(css: &str) -> Vec<String> {
        MEDIA_QUERY_PATTERN
            .captures_iter(css)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .collect::<HashSet<_>>()
            .into_iter()
            .take(20)
            .collect()
    }

    /// Analyze pseudo states
    fn analyze_pseudo_states(css: &str) -> PseudoStateAnalysis {
        PseudoStateAnalysis {
            hover_count: count_matches(&HOVER_PATTERN, css),
            focus_count: count_matches(&FOCUS_PATTERN, css),
            active_count: count_matches(&ACTIVE_PATTERN, css),
            before_count: count_matches(&BEFORE_PATTERN, css),
            after_count: count_matches(&AFTER_PATTERN, css),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_colors() {
        let css = r#"
            body { color: #fff; background: rgb(0, 0, 0); }
            a { color: hsl(200, 100%, 50%); }
        "#;
        let colors = CssAnalyzer::extract_colors(css);
        assert!(colors.contains(&"#fff".to_string()));
    }

    #[test]
    fn test_analyze_layout() {
        let css = r#"
            .flex { display: flex; }
            .grid { display: grid; }
            .fixed { position: fixed; }
        "#;
        let layout = CssAnalyzer::analyze_layout(css);
        assert_eq!(layout.flexbox_count, 1);
        assert_eq!(layout.grid_count, 1);
        assert_eq!(layout.position_fixed, 1);
    }
}
