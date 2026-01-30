//! JavaScript to CSS Converter
//!
//! Converts JavaScript-based styling to pure CSS:
//! - CSS-in-JS (styled-components, emotion, vanilla-extract)
//! - Animation libraries (GSAP, Framer Motion, anime.js)
//! - Inline styles to CSS classes
//! - Dynamic class computations to static CSS
//!
//! This module focuses on generating clean, usable CSS output.

use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Result of JavaScript to CSS conversion
#[derive(Debug, Clone, Default)]
pub struct JsToCssResult {
    /// Generated CSS content
    pub css: String,
    /// CSS class mappings (original -> generated)
    pub class_map: HashMap<String, String>,
    /// Keyframe animations extracted
    pub keyframes: Vec<KeyframeAnimation>,
    /// CSS variables extracted
    pub variables: HashMap<String, String>,
    /// Warnings about dynamic content that couldn't be converted
    pub warnings: Vec<String>,
    /// Statistics
    pub stats: ConversionStats,
}

#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    pub styled_components_found: usize,
    pub emotion_blocks_found: usize,
    pub gsap_animations_found: usize,
    pub framer_animations_found: usize,
    pub inline_styles_converted: usize,
    pub keyframes_generated: usize,
    pub dynamic_values_skipped: usize,
}

#[derive(Debug, Clone)]
pub struct KeyframeAnimation {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
    pub duration: Option<String>,
    pub easing: Option<String>,
    pub iterations: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub position: String, // "0%", "50%", "100%", "from", "to"
    pub properties: HashMap<String, String>,
}

/// JavaScript to CSS Converter
pub struct JsToCssConverter {
    // Styled-components patterns
    styled_pattern: Regex,
    styled_attrs_pattern: Regex,
    css_tagged_pattern: Regex,
    keyframes_tagged_pattern: Regex,
    global_style_pattern: Regex,

    // GSAP patterns
    gsap_to_pattern: Regex,
    gsap_from_pattern: Regex,
    gsap_fromto_pattern: Regex,
    gsap_timeline_pattern: Regex,
    gsap_set_pattern: Regex,

    // Framer Motion patterns
    motion_animate_pattern: Regex,
    motion_variants_pattern: Regex,
    motion_transition_pattern: Regex,

    // Inline style patterns
    style_object_pattern: Regex,
    style_assignment_pattern: Regex,

    // CSS variable patterns
    css_var_pattern: Regex,
    theme_access_pattern: Regex,

    // Class counter for unique names
    class_counter: usize,
}

impl JsToCssConverter {
    pub fn new() -> Self {
        Self {
            // Styled-components: styled.div`css` or styled(Component)`css`
            styled_pattern: Regex::new(
                r#"styled(?:\.(\w+)|\([\w.]+\))\s*`([^`]+)`"#
            ).unwrap(),

            // Styled with attrs: styled.div.attrs({...})`css`
            styled_attrs_pattern: Regex::new(
                r#"styled(?:\.(\w+)|\([\w.]+\))\.attrs\([^)]*\)\s*`([^`]+)`"#
            ).unwrap(),

            // css`` tagged template
            css_tagged_pattern: Regex::new(
                r#"(?:^|[^a-zA-Z])css\s*`([^`]+)`"#
            ).unwrap(),

            // keyframes`` tagged template
            keyframes_tagged_pattern: Regex::new(
                r#"keyframes\s*`([^`]+)`"#
            ).unwrap(),

            // createGlobalStyle
            global_style_pattern: Regex::new(
                r#"createGlobalStyle\s*`([^`]+)`"#
            ).unwrap(),

            // GSAP: gsap.to(".selector", { props })
            gsap_to_pattern: Regex::new(
                r#"gsap\.to\s*\(\s*["']([^"']+)["']\s*,\s*\{([^}]+)\}"#
            ).unwrap(),

            gsap_from_pattern: Regex::new(
                r#"gsap\.from\s*\(\s*["']([^"']+)["']\s*,\s*\{([^}]+)\}"#
            ).unwrap(),

            gsap_fromto_pattern: Regex::new(
                r#"gsap\.fromTo\s*\(\s*["']([^"']+)["']\s*,\s*\{([^}]+)\}\s*,\s*\{([^}]+)\}"#
            ).unwrap(),

            gsap_timeline_pattern: Regex::new(
                r#"gsap\.timeline\s*\(\s*(?:\{[^}]*\})?\s*\)"#
            ).unwrap(),

            gsap_set_pattern: Regex::new(
                r#"gsap\.set\s*\(\s*["']([^"']+)["']\s*,\s*\{([^}]+)\}"#
            ).unwrap(),

            // Framer Motion: <motion.div animate={{ props }}>
            motion_animate_pattern: Regex::new(
                r#"animate\s*=\s*\{\s*\{([^}]+)\}\s*\}"#
            ).unwrap(),

            motion_variants_pattern: Regex::new(
                r#"variants\s*=\s*\{\s*\{([^}]+)\}\s*\}"#
            ).unwrap(),

            motion_transition_pattern: Regex::new(
                r#"transition\s*=\s*\{\s*\{([^}]+)\}\s*\}"#
            ).unwrap(),

            // Inline styles: style={{ color: "red" }}
            style_object_pattern: Regex::new(
                r#"style\s*=\s*\{\s*\{([^}]+)\}\s*\}"#
            ).unwrap(),

            // element.style.prop = value
            style_assignment_pattern: Regex::new(
                r#"\.style\.(\w+)\s*=\s*["']([^"']+)["']"#
            ).unwrap(),

            // CSS variables: var(--name) or ${theme.color}
            css_var_pattern: Regex::new(
                r#"var\s*\(\s*--([a-zA-Z0-9_-]+)\s*\)"#
            ).unwrap(),

            theme_access_pattern: Regex::new(
                r#"\$\{(?:props\.)?theme\.([a-zA-Z0-9_.]+)\}"#
            ).unwrap(),

            class_counter: 0,
        }
    }

    /// Convert JavaScript code to CSS
    pub fn convert(&mut self, js_code: &str) -> JsToCssResult {
        let mut result = JsToCssResult::default();
        let mut css_blocks: Vec<String> = Vec::new();

        // 1. Extract styled-components
        self.extract_styled_components(js_code, &mut result, &mut css_blocks);

        // 2. Extract css`` blocks
        self.extract_css_tagged(js_code, &mut result, &mut css_blocks);

        // 3. Extract keyframes
        self.extract_keyframes_tagged(js_code, &mut result);

        // 4. Extract global styles
        self.extract_global_styles(js_code, &mut css_blocks);

        // 5. Convert GSAP animations to CSS
        self.convert_gsap_to_css(js_code, &mut result);

        // 6. Convert Framer Motion to CSS
        self.convert_framer_to_css(js_code, &mut result);

        // 7. Extract inline styles
        self.extract_inline_styles(js_code, &mut result, &mut css_blocks);

        // 8. Build final CSS
        result.css = self.build_css(&css_blocks, &result.keyframes, &result.variables);

        result
    }

    fn extract_styled_components(
        &mut self,
        js_code: &str,
        result: &mut JsToCssResult,
        css_blocks: &mut Vec<String>,
    ) {
        // Collect matches first to avoid borrow issues
        let styled_matches: Vec<(String, String)> = self.styled_pattern
            .captures_iter(js_code)
            .map(|cap| {
                let element = cap.get(1).map(|m| m.as_str()).unwrap_or("div").to_string();
                let css_content = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                (element, css_content)
            })
            .collect();

        for (element, css_content) in styled_matches {
            if css_content.is_empty() {
                continue;
            }

            let class_name = self.generate_class_name("sc", &element);
            let processed_css = self.process_css_content(&css_content, result);

            css_blocks.push(format!(".{} {{\n{}\n}}", class_name, processed_css));
            result.stats.styled_components_found += 1;
        }

        // Handle styled with attrs
        let attrs_matches: Vec<(String, String)> = self.styled_attrs_pattern
            .captures_iter(js_code)
            .map(|cap| {
                let element = cap.get(1).map(|m| m.as_str()).unwrap_or("div").to_string();
                let css_content = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                (element, css_content)
            })
            .collect();

        for (element, css_content) in attrs_matches {
            if css_content.is_empty() {
                continue;
            }

            let class_name = self.generate_class_name("sc", &element);
            let processed_css = self.process_css_content(&css_content, result);

            css_blocks.push(format!(".{} {{\n{}\n}}", class_name, processed_css));
            result.stats.styled_components_found += 1;
        }
    }

    fn extract_css_tagged(
        &mut self,
        js_code: &str,
        result: &mut JsToCssResult,
        css_blocks: &mut Vec<String>,
    ) {
        let css_matches: Vec<String> = self.css_tagged_pattern
            .captures_iter(js_code)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|s| s.len() >= 10)
            .collect();

        for css_content in css_matches {
            let class_name = self.generate_class_name("css", "block");
            let processed_css = self.process_css_content(&css_content, result);

            // Check if it's already a full rule or just properties
            if processed_css.contains('{') {
                css_blocks.push(processed_css);
            } else {
                css_blocks.push(format!(".{} {{\n{}\n}}", class_name, processed_css));
            }
            result.stats.emotion_blocks_found += 1;
        }
    }

    fn extract_keyframes_tagged(&mut self, js_code: &str, result: &mut JsToCssResult) {
        let kf_matches: Vec<String> = self.keyframes_tagged_pattern
            .captures_iter(js_code)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|s| !s.is_empty())
            .collect();

        for keyframe_css in kf_matches {
            let animation_name = self.generate_class_name("anim", "keyframe");
            let keyframes = self.parse_keyframes_css(&keyframe_css);

            result.keyframes.push(KeyframeAnimation {
                name: animation_name,
                keyframes,
                duration: Some("1s".to_string()),
                easing: Some("ease".to_string()),
                iterations: None,
            });
            result.stats.keyframes_generated += 1;
        }
    }

    fn extract_global_styles(&self, js_code: &str, css_blocks: &mut Vec<String>) {
        for cap in self.global_style_pattern.captures_iter(js_code) {
            let css_content = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            if !css_content.is_empty() {
                css_blocks.push(format!("/* Global styles */\n{}", css_content));
            }
        }
    }

    fn convert_gsap_to_css(&mut self, js_code: &str, result: &mut JsToCssResult) {
        // Collect gsap.to() matches
        let to_matches: Vec<(String, String)> = self.gsap_to_pattern
            .captures_iter(js_code)
            .map(|cap| {
                let selector = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let props = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                (selector, props)
            })
            .collect();

        for (selector, props) in to_matches {
            if let Some(animation) = self.gsap_props_to_keyframes(&selector, None, Some(&props)) {
                result.keyframes.push(animation);
                result.stats.gsap_animations_found += 1;
            }
        }

        // Collect gsap.from() matches
        let from_matches: Vec<(String, String)> = self.gsap_from_pattern
            .captures_iter(js_code)
            .map(|cap| {
                let selector = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let props = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                (selector, props)
            })
            .collect();

        for (selector, props) in from_matches {
            if let Some(animation) = self.gsap_props_to_keyframes(&selector, Some(&props), None) {
                result.keyframes.push(animation);
                result.stats.gsap_animations_found += 1;
            }
        }

        // Collect gsap.fromTo() matches
        let fromto_matches: Vec<(String, String, String)> = self.gsap_fromto_pattern
            .captures_iter(js_code)
            .map(|cap| {
                let selector = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let from_props = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                let to_props = cap.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
                (selector, from_props, to_props)
            })
            .collect();

        for (selector, from_props, to_props) in fromto_matches {
            if let Some(animation) =
                self.gsap_props_to_keyframes(&selector, Some(&from_props), Some(&to_props))
            {
                result.keyframes.push(animation);
                result.stats.gsap_animations_found += 1;
            }
        }
    }

    fn gsap_props_to_keyframes(
        &mut self,
        selector: &str,
        from_props: Option<&str>,
        to_props: Option<&str>,
    ) -> Option<KeyframeAnimation> {
        let animation_name = self.generate_class_name("gsap", &selector.replace(['.', '#'], ""));

        let mut keyframes = Vec::new();
        let mut duration = "1s".to_string();
        let mut easing = "ease".to_string();

        // Parse from props (0%)
        if let Some(props) = from_props {
            let (css_props, dur, ease) = self.parse_gsap_props(props);
            if let Some(d) = dur {
                duration = d;
            }
            if let Some(e) = ease {
                easing = e;
            }
            keyframes.push(Keyframe {
                position: "0%".to_string(),
                properties: css_props,
            });
        }

        // Parse to props (100%)
        if let Some(props) = to_props {
            let (css_props, dur, ease) = self.parse_gsap_props(props);
            if let Some(d) = dur {
                duration = d;
            }
            if let Some(e) = ease {
                easing = e;
            }
            keyframes.push(Keyframe {
                position: "100%".to_string(),
                properties: css_props,
            });
        }

        if keyframes.is_empty() {
            return None;
        }

        Some(KeyframeAnimation {
            name: animation_name,
            keyframes,
            duration: Some(duration),
            easing: Some(easing),
            iterations: None,
        })
    }

    fn parse_gsap_props(&self, props: &str) -> (HashMap<String, String>, Option<String>, Option<String>) {
        let mut css_props = HashMap::new();
        let mut duration = None;
        let mut easing = None;

        // Simple property parser
        let prop_re = Regex::new(r#"(\w+)\s*:\s*([^,}]+)"#).unwrap();

        for cap in prop_re.captures_iter(props) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            match key {
                // Transform properties -> CSS transform
                "x" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} translateX({})", existing, self.parse_gsap_value(value));
                }
                "y" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} translateY({})", existing, self.parse_gsap_value(value));
                }
                "rotation" | "rotate" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} rotate({}deg)", existing, value.trim_matches('"'));
                }
                "scale" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} scale({})", existing, value.trim_matches('"'));
                }
                "scaleX" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} scaleX({})", existing, value.trim_matches('"'));
                }
                "scaleY" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} scaleY({})", existing, value.trim_matches('"'));
                }
                // Opacity
                "opacity" | "autoAlpha" => {
                    css_props.insert("opacity".to_string(), value.trim_matches('"').to_string());
                }
                // Duration
                "duration" => {
                    let dur_val: f64 = value.trim_matches('"').parse().unwrap_or(1.0);
                    duration = Some(format!("{}s", dur_val));
                }
                // Easing
                "ease" => {
                    easing = Some(self.gsap_ease_to_css(value.trim_matches('"')));
                }
                // Direct CSS properties
                "backgroundColor" => {
                    css_props.insert("background-color".to_string(), value.trim_matches('"').to_string());
                }
                "color" => {
                    css_props.insert("color".to_string(), value.trim_matches('"').to_string());
                }
                "width" | "height" | "left" | "top" | "right" | "bottom" => {
                    css_props.insert(key.to_string(), self.parse_gsap_value(value));
                }
                _ => {}
            }
        }

        // Clean up transform
        if let Some(transform) = css_props.get_mut("transform") {
            *transform = transform.trim().to_string();
        }

        (css_props, duration, easing)
    }

    fn parse_gsap_value(&self, value: &str) -> String {
        let clean = value.trim().trim_matches('"').trim_matches('\'');

        // If it's a number, assume pixels
        if let Ok(_) = clean.parse::<f64>() {
            format!("{}px", clean)
        } else {
            clean.to_string()
        }
    }

    fn gsap_ease_to_css(&self, ease: &str) -> String {
        match ease {
            "none" | "linear" => "linear".to_string(),
            "power1.in" | "power1.inOut" | "power1.out" => "ease".to_string(),
            "power2.in" => "ease-in".to_string(),
            "power2.out" => "ease-out".to_string(),
            "power2.inOut" => "ease-in-out".to_string(),
            "power3.in" => "cubic-bezier(0.55, 0.055, 0.675, 0.19)".to_string(),
            "power3.out" => "cubic-bezier(0.215, 0.61, 0.355, 1)".to_string(),
            "power3.inOut" => "cubic-bezier(0.645, 0.045, 0.355, 1)".to_string(),
            "power4.in" => "cubic-bezier(0.895, 0.03, 0.685, 0.22)".to_string(),
            "power4.out" => "cubic-bezier(0.165, 0.84, 0.44, 1)".to_string(),
            "power4.inOut" => "cubic-bezier(0.77, 0, 0.175, 1)".to_string(),
            "back.in" => "cubic-bezier(0.6, -0.28, 0.735, 0.045)".to_string(),
            "back.out" => "cubic-bezier(0.175, 0.885, 0.32, 1.275)".to_string(),
            "back.inOut" => "cubic-bezier(0.68, -0.55, 0.265, 1.55)".to_string(),
            "elastic.out" => "cubic-bezier(0.175, 0.885, 0.32, 1.275)".to_string(),
            "bounce.out" => "cubic-bezier(0.175, 0.885, 0.32, 1.275)".to_string(),
            _ => "ease".to_string(),
        }
    }

    fn convert_framer_to_css(&mut self, js_code: &str, result: &mut JsToCssResult) {
        // Collect animate={{ }} props
        let animate_matches: Vec<String> = self.motion_animate_pattern
            .captures_iter(js_code)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        for props in animate_matches {
            if let Some(animation) = self.framer_props_to_keyframes(&props) {
                result.keyframes.push(animation);
                result.stats.framer_animations_found += 1;
            }
        }
    }

    fn framer_props_to_keyframes(&mut self, props: &str) -> Option<KeyframeAnimation> {
        let animation_name = self.generate_class_name("framer", "motion");
        let css_props = self.parse_framer_props(props);

        if css_props.is_empty() {
            return None;
        }

        // Create a simple 0% -> 100% animation
        let keyframes = vec![
            Keyframe {
                position: "0%".to_string(),
                properties: HashMap::new(), // Initial state (browser default)
            },
            Keyframe {
                position: "100%".to_string(),
                properties: css_props,
            },
        ];

        Some(KeyframeAnimation {
            name: animation_name,
            keyframes,
            duration: Some("0.3s".to_string()),
            easing: Some("ease-out".to_string()),
            iterations: None,
        })
    }

    fn parse_framer_props(&self, props: &str) -> HashMap<String, String> {
        let mut css_props = HashMap::new();
        let prop_re = Regex::new(r#"(\w+)\s*:\s*([^,}]+)"#).unwrap();

        for cap in prop_re.captures_iter(props) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            match key {
                "x" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} translateX({}px)", existing, value);
                }
                "y" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} translateY({}px)", existing, value);
                }
                "rotate" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} rotate({}deg)", existing, value);
                }
                "scale" => {
                    let existing = css_props.entry("transform".to_string()).or_default();
                    *existing = format!("{} scale({})", existing, value);
                }
                "opacity" => {
                    css_props.insert("opacity".to_string(), value.to_string());
                }
                _ => {}
            }
        }

        if let Some(transform) = css_props.get_mut("transform") {
            *transform = transform.trim().to_string();
        }

        css_props
    }

    fn extract_inline_styles(
        &mut self,
        js_code: &str,
        result: &mut JsToCssResult,
        css_blocks: &mut Vec<String>,
    ) {
        // Collect style={{ }} objects
        let style_matches: Vec<String> = self.style_object_pattern
            .captures_iter(js_code)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|s| s.len() >= 5)
            .collect();

        for style_content in style_matches {
            let class_name = self.generate_class_name("style", "inline");
            let css_props = self.parse_style_object(&style_content);

            if !css_props.is_empty() {
                let css_text: String = css_props
                    .iter()
                    .map(|(k, v)| format!("  {}: {};", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                css_blocks.push(format!(".{} {{\n{}\n}}", class_name, css_text));
                result.stats.inline_styles_converted += 1;
            }
        }
    }

    fn parse_style_object(&self, content: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();
        let prop_re = Regex::new(r#"(\w+)\s*:\s*["']([^"']+)["']"#).unwrap();

        for cap in prop_re.captures_iter(content) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Convert camelCase to kebab-case
            let css_key = self.camel_to_kebab(key);
            props.insert(css_key, value.to_string());
        }

        props
    }

    fn process_css_content(&self, content: &str, result: &mut JsToCssResult) -> String {
        let mut processed = content.to_string();

        // Replace theme interpolations with CSS variables
        for cap in self.theme_access_pattern.captures_iter(content) {
            let theme_path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let var_name = theme_path.replace('.', "-");
            let original = cap.get(0).map(|m| m.as_str()).unwrap_or("");

            processed = processed.replace(original, &format!("var(--{})", var_name));
            result.variables.insert(
                format!("--{}", var_name),
                format!("/* from theme.{} */", theme_path),
            );
            result.stats.dynamic_values_skipped += 1;
        }

        // Clean up the CSS
        processed = self.clean_css(&processed);

        processed
    }

    fn parse_keyframes_css(&self, css: &str) -> Vec<Keyframe> {
        let mut keyframes = Vec::new();
        let frame_re = Regex::new(r#"(\d+%|from|to)\s*\{([^}]+)\}"#).unwrap();

        for cap in frame_re.captures_iter(css) {
            let position = cap.get(1).map(|m| m.as_str()).unwrap_or("0%");
            let props_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut properties = HashMap::new();
            let prop_re = Regex::new(r#"([a-zA-Z-]+)\s*:\s*([^;]+)"#).unwrap();

            for prop_cap in prop_re.captures_iter(props_str) {
                let key = prop_cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let value = prop_cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                properties.insert(key.to_string(), value.to_string());
            }

            keyframes.push(Keyframe {
                position: position.to_string(),
                properties,
            });
        }

        keyframes
    }

    fn clean_css(&self, css: &str) -> String {
        let mut result = css.to_string();

        // Remove JS comments that might have leaked
        result = Regex::new(r#"//[^\n]*"#)
            .unwrap()
            .replace_all(&result, "")
            .to_string();

        // Clean up extra whitespace
        result = Regex::new(r#"\s+"#)
            .unwrap()
            .replace_all(&result, " ")
            .to_string();

        // Add proper formatting
        result = result.replace(";", ";\n  ");
        result = result.replace("{", "{\n  ");
        result = result.replace("}", "\n}");

        result.trim().to_string()
    }

    fn generate_class_name(&mut self, prefix: &str, suffix: &str) -> String {
        self.class_counter += 1;
        format!("{}-{}-{}", prefix, suffix, self.class_counter)
    }

    fn camel_to_kebab(&self, s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }

    fn build_css(
        &self,
        css_blocks: &[String],
        keyframes: &[KeyframeAnimation],
        variables: &HashMap<String, String>,
    ) -> String {
        let mut result = String::new();

        // CSS Variables
        if !variables.is_empty() {
            result.push_str(":root {\n");
            for (name, value) in variables {
                result.push_str(&format!("  {}: {};\n", name, value));
            }
            result.push_str("}\n\n");
        }

        // Keyframe animations
        for anim in keyframes {
            result.push_str(&format!("@keyframes {} {{\n", anim.name));
            for kf in &anim.keyframes {
                result.push_str(&format!("  {} {{\n", kf.position));
                for (prop, value) in &kf.properties {
                    result.push_str(&format!("    {}: {};\n", prop, value));
                }
                result.push_str("  }\n");
            }
            result.push_str("}\n\n");

            // Also generate animation class
            let duration = anim.duration.as_deref().unwrap_or("1s");
            let easing = anim.easing.as_deref().unwrap_or("ease");
            let iterations = anim.iterations.as_deref().unwrap_or("1");

            result.push_str(&format!(
                ".animate-{} {{\n  animation: {} {} {} {};\n}}\n\n",
                anim.name, anim.name, duration, easing, iterations
            ));
        }

        // CSS blocks
        for block in css_blocks {
            result.push_str(block);
            result.push_str("\n\n");
        }

        result
    }
}

impl Default for JsToCssConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_components_extraction() {
        let mut converter = JsToCssConverter::new();
        let js = r#"
            const Button = styled.button`
                background: blue;
                color: white;
                padding: 10px 20px;
            `;
        "#;

        let result = converter.convert(js);
        assert!(result.css.contains("background:"));
        assert!(result.css.contains("color:"));
        assert_eq!(result.stats.styled_components_found, 1);
    }

    #[test]
    fn test_gsap_to_keyframes() {
        let mut converter = JsToCssConverter::new();
        let js = r#"
            gsap.to(".box", { x: 100, opacity: 0.5, duration: 1 });
        "#;

        let result = converter.convert(js);
        assert!(!result.keyframes.is_empty());
        assert!(result.css.contains("@keyframes"));
        assert!(result.css.contains("translateX"));
    }

    #[test]
    fn test_keyframes_extraction() {
        let mut converter = JsToCssConverter::new();
        let js = r#"
            const fadeIn = keyframes`
                from { opacity: 0; }
                to { opacity: 1; }
            `;
        "#;

        let result = converter.convert(js);
        assert_eq!(result.stats.keyframes_generated, 1);
        assert!(result.css.contains("@keyframes"));
    }

    #[test]
    fn test_inline_style_conversion() {
        let mut converter = JsToCssConverter::new();
        let js = r#"
            <div style={{ backgroundColor: "red", fontSize: "16px" }} />
        "#;

        let result = converter.convert(js);
        assert!(result.css.contains("background-color"));
        assert!(result.css.contains("font-size"));
    }
}
