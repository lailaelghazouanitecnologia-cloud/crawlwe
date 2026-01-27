//! Pattern definitions for web content analysis
//!
//! Contains all regex patterns and detection logic for CSS, JS, and HTML analysis.

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use super::CssFrameworkInfo;

// ============================================================================
// CSS PATTERNS
// ============================================================================

lazy_static! {
    // Animation patterns
    pub static ref KEYFRAMES_PATTERN: Regex =
        Regex::new(r"@keyframes\s+([a-zA-Z0-9_-]+)\s*\{((?:[^{}]+|\{[^{}]*\})*)\}").unwrap();
    pub static ref ANIMATION_PATTERN: Regex =
        Regex::new(r"animation\s*:\s*([^;]+);").unwrap();
    pub static ref ANIMATION_NAME_PATTERN: Regex =
        Regex::new(r"animation-name\s*:\s*([^;]+);").unwrap();

    // Transition patterns
    pub static ref TRANSITION_PATTERN: Regex =
        Regex::new(r"transition\s*:\s*([^;{}]+);").unwrap();
    pub static ref TRANSITION_DURATION_PATTERN: Regex =
        Regex::new(r"transition-duration\s*:\s*([^;]+);").unwrap();
    pub static ref TIMING_FUNCTION_PATTERN: Regex =
        Regex::new(r"(ease|ease-in|ease-out|ease-in-out|linear|cubic-bezier\([^)]+\))").unwrap();
    pub static ref DURATION_PATTERN: Regex =
        Regex::new(r"(\d+\.?\d*m?s)").unwrap();

    // Transform patterns
    pub static ref TRANSFORM_PATTERN: Regex =
        Regex::new(r"transform\s*:\s*([^;{}]+);").unwrap();

    // Layout patterns
    pub static ref DISPLAY_FLEX_PATTERN: Regex =
        Regex::new(r"display\s*:\s*flex").unwrap();
    pub static ref DISPLAY_GRID_PATTERN: Regex =
        Regex::new(r"display\s*:\s*grid").unwrap();
    pub static ref POSITION_FIXED_PATTERN: Regex =
        Regex::new(r"position\s*:\s*fixed").unwrap();
    pub static ref POSITION_STICKY_PATTERN: Regex =
        Regex::new(r"position\s*:\s*sticky").unwrap();
    pub static ref POSITION_ABSOLUTE_PATTERN: Regex =
        Regex::new(r"position\s*:\s*absolute").unwrap();

    // CSS Variables
    pub static ref CSS_VAR_DEF_PATTERN: Regex =
        Regex::new(r"--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);").unwrap();
    pub static ref CSS_VAR_USAGE_PATTERN: Regex =
        Regex::new(r"var\s*\(\s*--([a-zA-Z0-9_-]+)").unwrap();

    // Colors
    pub static ref HEX_COLOR_PATTERN: Regex =
        Regex::new(r"#[0-9a-fA-F]{3,8}\b").unwrap();
    pub static ref RGB_COLOR_PATTERN: Regex =
        Regex::new(r"rgba?\([^)]+\)").unwrap();
    pub static ref HSL_COLOR_PATTERN: Regex =
        Regex::new(r"hsla?\([^)]+\)").unwrap();

    // Fonts
    pub static ref FONT_FAMILY_PATTERN: Regex =
        Regex::new(r#"font-family:\s*([^;]+)"#).unwrap();

    // Visual effects
    pub static ref BOX_SHADOW_PATTERN: Regex =
        Regex::new(r"box-shadow\s*:\s*([^;]+);").unwrap();
    pub static ref FILTER_PATTERN: Regex =
        Regex::new(r"filter\s*:\s*([^;]+);").unwrap();
    pub static ref BACKDROP_FILTER_PATTERN: Regex =
        Regex::new(r"backdrop-filter\s*:\s*([^;]+);").unwrap();

    // Media queries
    pub static ref MEDIA_QUERY_PATTERN: Regex =
        Regex::new(r"@media\s*\(([^)]+)\)").unwrap();

    // Pseudo states
    pub static ref HOVER_PATTERN: Regex =
        Regex::new(r":hover\s*\{").unwrap();
    pub static ref FOCUS_PATTERN: Regex =
        Regex::new(r":focus\s*\{").unwrap();
    pub static ref ACTIVE_PATTERN: Regex =
        Regex::new(r":active\s*\{").unwrap();
    pub static ref BEFORE_PATTERN: Regex =
        Regex::new(r"::before").unwrap();
    pub static ref AFTER_PATTERN: Regex =
        Regex::new(r"::after").unwrap();
}

// ============================================================================
// JAVASCRIPT PATTERNS
// ============================================================================

lazy_static! {
    // Animation libraries
    pub static ref GSAP_PATTERN: Regex =
        Regex::new(r"gsap\.|TweenMax|TweenLite|TimelineMax|ScrollTrigger").unwrap();
    pub static ref ANIMEJS_PATTERN: Regex =
        Regex::new(r"anime\s*\(|anime\.timeline").unwrap();
    pub static ref FRAMER_PATTERN: Regex =
        Regex::new(r"motion\.|useAnimation|AnimatePresence").unwrap();
    pub static ref LOTTIE_PATTERN: Regex =
        Regex::new(r"lottie\.|bodymovin|lottie-web").unwrap();
    pub static ref VELOCITY_PATTERN: Regex =
        Regex::new(r"Velocity\s*\(|\.velocity\s*\(").unwrap();

    // 3D/Graphics libraries
    pub static ref THREEJS_PATTERN: Regex =
        Regex::new(r"THREE\.|WebGLRenderer|PerspectiveCamera").unwrap();
    pub static ref BABYLON_PATTERN: Regex =
        Regex::new(r"BABYLON\.|BabylonJS").unwrap();
    pub static ref PIXI_PATTERN: Regex =
        Regex::new(r"PIXI\.|PixiJS").unwrap();
    pub static ref AFRAME_PATTERN: Regex =
        Regex::new(r"AFRAME\.|a-scene|a-entity").unwrap();

    // WebGL patterns
    pub static ref WEBGL_CONTEXT_PATTERN: Regex =
        Regex::new(r#"getContext\s*\(\s*['"]webgl"#).unwrap();
    pub static ref WEBGL2_CONTEXT_PATTERN: Regex =
        Regex::new(r#"getContext\s*\(\s*['"]webgl2"#).unwrap();
    pub static ref WEBGL_RENDERING_PATTERN: Regex =
        Regex::new(r"WebGLRenderingContext|WebGL2RenderingContext").unwrap();
    pub static ref SHADER_CREATE_PATTERN: Regex =
        Regex::new(r"createShader|shaderSource|compileShader").unwrap();
    pub static ref GL_CALLS_PATTERN: Regex =
        Regex::new(r"gl\.(bindBuffer|bindTexture|drawArrays|drawElements)").unwrap();

    // WebGPU patterns
    pub static ref WEBGPU_NAVIGATOR_PATTERN: Regex =
        Regex::new(r"navigator\.gpu").unwrap();
    pub static ref WEBGPU_DEVICE_PATTERN: Regex =
        Regex::new(r"GPUDevice|GPUAdapter|GPUBuffer").unwrap();
    pub static ref WEBGPU_SHADER_PATTERN: Regex =
        Regex::new(r"createShaderModule|GPUShaderModule").unwrap();

    // Scroll libraries
    pub static ref LOCOMOTIVE_PATTERN: Regex =
        Regex::new(r"LocomotiveScroll|locomotive-scroll").unwrap();
    pub static ref SCROLLMAGIC_PATTERN: Regex =
        Regex::new(r"ScrollMagic|ScrollScene").unwrap();
    pub static ref AOS_PATTERN: Regex =
        Regex::new(r"AOS\.init|data-aos").unwrap();
    pub static ref LENIS_PATTERN: Regex =
        Regex::new(r"Lenis|@studio-freight/lenis").unwrap();

    // Native animation APIs
    pub static ref RAF_PATTERN: Regex =
        Regex::new(r"requestAnimationFrame\s*\(").unwrap();
    pub static ref WAAPI_PATTERN: Regex =
        Regex::new(r"\.animate\s*\(\s*\[").unwrap();

    // Observers
    pub static ref INTERSECTION_OBSERVER_PATTERN: Regex =
        Regex::new(r"IntersectionObserver").unwrap();
    pub static ref MUTATION_OBSERVER_PATTERN: Regex =
        Regex::new(r"MutationObserver").unwrap();
    pub static ref RESIZE_OBSERVER_PATTERN: Regex =
        Regex::new(r"ResizeObserver").unwrap();

    // UI Frameworks
    pub static ref REACT_PATTERN: Regex =
        Regex::new(r"React\.|createElement|__jsx").unwrap();
    pub static ref VUE_PATTERN: Regex =
        Regex::new(r"Vue\.|createApp|defineComponent").unwrap();
    pub static ref SVELTE_PATTERN: Regex =
        Regex::new(r"svelte|SvelteComponent").unwrap();
    pub static ref ANGULAR_PATTERN: Regex =
        Regex::new(r"@angular|ngModule").unwrap();
    pub static ref NEXTJS_PATTERN: Regex =
        Regex::new(r"next/|_next|__NEXT").unwrap();
    pub static ref NUXT_PATTERN: Regex =
        Regex::new(r"nuxt|__NUXT").unwrap();

    // Event handlers
    pub static ref SCROLL_EVENT_PATTERN: Regex =
        Regex::new(r#"addEventListener\s*\(\s*['"]scroll['""]"#).unwrap();
    pub static ref MOUSE_EVENT_PATTERN: Regex =
        Regex::new(r"mousemove|mouseenter|mouseleave").unwrap();
    pub static ref TOUCH_EVENT_PATTERN: Regex =
        Regex::new(r"touchstart|touchmove|touchend").unwrap();
    pub static ref WHEEL_EVENT_PATTERN: Regex =
        Regex::new(r"wheel|mousewheel").unwrap();

    // Three.js specific
    pub static ref THREEJS_GEOMETRY_PATTERN: Regex =
        Regex::new(r"(BoxGeometry|SphereGeometry|PlaneGeometry|CylinderGeometry|TorusGeometry|BufferGeometry)").unwrap();
    pub static ref THREEJS_MATERIAL_PATTERN: Regex =
        Regex::new(r"(MeshBasicMaterial|MeshStandardMaterial|MeshPhongMaterial|ShaderMaterial)").unwrap();
    pub static ref THREEJS_LIGHT_PATTERN: Regex =
        Regex::new(r"(AmbientLight|DirectionalLight|PointLight|SpotLight)").unwrap();
    pub static ref THREEJS_CONTROLS_PATTERN: Regex =
        Regex::new(r"(OrbitControls|TrackballControls|FlyControls)").unwrap();

    // Shader patterns
    pub static ref UNIFORM_PATTERN: Regex =
        Regex::new(r"uniform\s+(\w+)\s+(\w+)").unwrap();
    pub static ref ATTRIBUTE_PATTERN: Regex =
        Regex::new(r"attribute\s+(\w+)\s+(\w+)").unwrap();
}

// ============================================================================
// TAILWIND PATTERNS
// ============================================================================

lazy_static! {
    pub static ref TAILWIND_PATTERNS: HashMap<&'static str, Regex> = {
        let mut m = HashMap::new();
        m.insert("display", Regex::new(r"\b(flex|grid|block|inline-block|hidden)\b").unwrap());
        m.insert("flex", Regex::new(r"\b(flex-row|flex-col|flex-wrap|flex-1|items-center|justify-center|justify-between)\b").unwrap());
        m.insert("spacing", Regex::new(r"\b(p-\d+|m-\d+|px-\d+|py-\d+|mx-\d+|my-\d+|gap-\d+)\b").unwrap());
        m.insert("sizing", Regex::new(r"\b(w-\d+|h-\d+|w-full|h-full|w-screen|h-screen|max-w-\w+)\b").unwrap());
        m.insert("typography", Regex::new(r"\b(text-\w+|font-\w+|leading-\w+|tracking-\w+)\b").unwrap());
        m.insert("colors", Regex::new(r"\b(bg-\w+-\d+|text-\w+-\d+|border-\w+-\d+)\b").unwrap());
        m.insert("borders", Regex::new(r"\b(border|border-\d+|rounded|rounded-\w+)\b").unwrap());
        m.insert("effects", Regex::new(r"\b(shadow|shadow-\w+|opacity-\d+)\b").unwrap());
        m.insert("transitions", Regex::new(r"\b(transition|duration-\d+|ease-\w+)\b").unwrap());
        m.insert("transforms", Regex::new(r"\b(scale-\d+|rotate-\d+|translate-\w+-\d+)\b").unwrap());
        m.insert("states", Regex::new(r"\b(hover:|focus:|active:|dark:|sm:|md:|lg:|xl:)").unwrap());
        m
    };
}

// ============================================================================
// HTML PATTERNS
// ============================================================================

lazy_static! {
    pub static ref CLASS_ATTR_PATTERN: Regex =
        Regex::new(r#"class=["']([^"']+)["']"#).unwrap();
    pub static ref DATA_ATTR_PATTERN: Regex =
        Regex::new(r"data-([a-zA-Z0-9-]+)").unwrap();
    pub static ref ARIA_ATTR_PATTERN: Regex =
        Regex::new(r"aria-([a-zA-Z0-9-]+)").unwrap();
    pub static ref ROLE_ATTR_PATTERN: Regex =
        Regex::new(r#"role=["']([^"']+)["']"#).unwrap();
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Count regex matches in text
pub fn count_matches(pattern: &Regex, text: &str) -> usize {
    pattern.find_iter(text).count()
}

/// Extract all matches as strings
pub fn extract_matches(pattern: &Regex, text: &str) -> Vec<String> {
    pattern
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Extract capture groups
pub fn extract_captures(pattern: &Regex, text: &str) -> Vec<Vec<String>> {
    pattern
        .captures_iter(text)
        .map(|cap| {
            cap.iter()
                .skip(1)
                .filter_map(|m| m.map(|m| m.as_str().to_string()))
                .collect()
        })
        .collect()
}

/// Detect Tailwind CSS usage
pub fn detect_tailwind(html: &str, _css: &str) -> Option<CssFrameworkInfo> {
    let mut total_classes = 0;

    // Extract all class attributes
    let class_contents: Vec<String> = CLASS_ATTR_PATTERN
        .captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    let all_classes: Vec<&str> = class_contents
        .iter()
        .flat_map(|s| s.split_whitespace())
        .collect();

    // Check Tailwind patterns
    for (_category, pattern) in TAILWIND_PATTERNS.iter() {
        for class in &all_classes {
            if pattern.is_match(class) {
                total_classes += 1;
            }
        }
    }

    if total_classes > 10 {
        Some(CssFrameworkInfo {
            name: "Tailwind CSS".to_string(),
            confidence: (total_classes.min(100) as f32) / 100.0,
            utility_classes_found: total_classes,
        })
    } else {
        None
    }
}

// ============================================================================
// UI COMPONENT PATTERNS
// ============================================================================

pub const UI_COMPONENT_KEYWORDS: &[(&str, &[&str])] = &[
    ("modal", &["modal", "dialog", "popup", "overlay", "lightbox"]),
    ("dropdown", &["dropdown", "select", "combobox", "menu"]),
    ("carousel", &["carousel", "slider", "swiper", "slideshow", "gallery"]),
    ("tabs", &["tab", "tablist", "tabpanel"]),
    ("accordion", &["accordion", "collapse", "expandable", "collapsible"]),
    ("navigation", &["nav", "navbar", "sidebar", "drawer"]),
    ("form", &["form", "input", "textarea", "checkbox", "radio"]),
    ("card", &["card", "tile", "panel"]),
    ("tooltip", &["tooltip", "popover", "hint"]),
    ("notification", &["toast", "notification", "alert", "snackbar", "banner"]),
    ("progress", &["progress", "loader", "spinner", "skeleton", "loading"]),
];

/// Detect UI component in class string
pub fn detect_ui_component(class_str: &str) -> Option<&'static str> {
    let lower = class_str.to_lowercase();
    for (component, keywords) in UI_COMPONENT_KEYWORDS {
        for keyword in *keywords {
            if lower.contains(keyword) {
                return Some(component);
            }
        }
    }
    None
}
