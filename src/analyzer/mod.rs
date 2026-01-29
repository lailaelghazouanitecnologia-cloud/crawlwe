//! Web content analyzer module
//!
//! Provides deep analysis of CSS, JavaScript, and HTML content
//! to detect frameworks, libraries, animations, and UI patterns.

pub mod css_analyzer;
pub mod html_analyzer;
pub mod js_analyzer;
pub mod patterns;

pub use css_analyzer::CssAnalyzer;
pub use html_analyzer::HtmlAnalyzer;
pub use js_analyzer::JsAnalyzer;
pub use patterns::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete analysis result for a web page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisResult {
    pub css: CssAnalysis,
    pub js: JsAnalysis,
    pub html: HtmlAnalysis,
    pub frameworks: FrameworkDetection,
    pub score: AnalysisScore,
}

/// CSS analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CssAnalysis {
    pub animations: AnimationAnalysis,
    pub transitions: Vec<String>,
    pub transforms: TransformAnalysis,
    pub layout: LayoutAnalysis,
    pub variables: HashMap<String, String>,
    pub colors: Vec<String>,
    pub fonts: Vec<String>,
    pub shadows: Vec<String>,
    pub filters: Vec<String>,
    pub media_queries: Vec<String>,
    pub pseudo_states: PseudoStateAnalysis,
}

/// Animation analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationAnalysis {
    pub keyframes: Vec<KeyframeInfo>,
    pub animation_properties: Vec<String>,
    pub timing_functions: Vec<String>,
    pub durations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeInfo {
    pub name: String,
    pub body: String,
}

/// Transform analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransformAnalysis {
    pub values: Vec<String>,
    pub has_translate: bool,
    pub has_rotate: bool,
    pub has_scale: bool,
    pub has_skew: bool,
    pub has_matrix: bool,
    pub has_3d: bool,
}

/// Layout analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutAnalysis {
    pub flexbox_count: usize,
    pub grid_count: usize,
    pub position_fixed: usize,
    pub position_sticky: usize,
    pub position_absolute: usize,
}

/// Pseudo state analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PseudoStateAnalysis {
    pub hover_count: usize,
    pub focus_count: usize,
    pub active_count: usize,
    pub before_count: usize,
    pub after_count: usize,
}

/// JavaScript analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsAnalysis {
    pub animation_libraries: Vec<LibraryDetection>,
    pub graphics_libraries: Vec<LibraryDetection>,
    pub scroll_libraries: Vec<LibraryDetection>,
    pub ui_frameworks: Vec<LibraryDetection>,
    pub webgl: WebGLAnalysis,
    pub webgpu: WebGPUAnalysis,
    pub observers: ObserverAnalysis,
    pub event_handlers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDetection {
    pub name: String,
    pub occurrences: usize,
    pub confidence: f32,
}

/// WebGL analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebGLAnalysis {
    pub detected: bool,
    pub version: Option<String>,
    pub patterns_found: Vec<String>,
    pub shaders: ShaderAnalysis,
    pub three_js: ThreeJsAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderAnalysis {
    pub vertex_shaders: usize,
    pub fragment_shaders: usize,
    pub uniforms: Vec<String>,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreeJsAnalysis {
    pub detected: bool,
    pub geometries: Vec<String>,
    pub materials: Vec<String>,
    pub lights: Vec<String>,
    pub controls: Vec<String>,
}

/// WebGPU analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebGPUAnalysis {
    pub detected: bool,
    pub patterns_found: Vec<String>,
}

/// Observer API usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObserverAnalysis {
    pub intersection: usize,
    pub mutation: usize,
    pub resize: usize,
    pub performance: usize,
}

/// HTML analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HtmlAnalysis {
    pub element_count: usize,
    pub max_depth: usize,
    pub semantic_elements: SemanticElements,
    pub ui_components: UiComponentAnalysis,
    pub media_elements: MediaElementAnalysis,
    pub accessibility: AccessibilityAnalysis,
    pub data_attributes: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticElements {
    pub header: usize,
    pub nav: usize,
    pub main: usize,
    pub article: usize,
    pub section: usize,
    pub aside: usize,
    pub footer: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiComponentAnalysis {
    pub modals: usize,
    pub dropdowns: usize,
    pub carousels: usize,
    pub tabs: usize,
    pub accordions: usize,
    pub forms: usize,
    pub buttons: usize,
    pub cards: usize,
    pub tooltips: usize,
    pub notifications: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaElementAnalysis {
    pub images: usize,
    pub videos: usize,
    pub audio: usize,
    pub canvas: usize,
    pub svg: usize,
    pub iframes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccessibilityAnalysis {
    pub aria_attributes: Vec<String>,
    pub roles: Vec<String>,
    pub has_alt_texts: bool,
    pub has_labels: bool,
}

/// Framework detection results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameworkDetection {
    pub css_framework: Option<CssFrameworkInfo>,
    pub js_framework: Option<String>,
    pub css_methodology: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssFrameworkInfo {
    pub name: String,
    pub confidence: f32,
    pub utility_classes_found: usize,
}

/// Analysis complexity score
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisScore {
    pub animation_complexity: u32,
    pub interactivity_score: u32,
    pub visual_effects_score: u32,
    pub accessibility_score: u32,
    pub total: u32,
}

/// Main analyzer that combines all sub-analyzers
pub struct Analyzer;

impl Analyzer {
    pub fn analyze(html: &str, css: &str, js: &str) -> AnalysisResult {
        let css_analysis = CssAnalyzer::analyze(css);
        let js_analysis = JsAnalyzer::analyze(js);
        let html_analysis = HtmlAnalyzer::analyze(html);

        let frameworks = Self::detect_frameworks(html, css, js);
        let score = Self::calculate_score(&css_analysis, &js_analysis, &html_analysis);

        AnalysisResult {
            css: css_analysis,
            js: js_analysis,
            html: html_analysis,
            frameworks,
            score,
        }
    }

    fn detect_frameworks(html: &str, css: &str, _js: &str) -> FrameworkDetection {
        let css_framework = patterns::detect_tailwind(html, css);

        FrameworkDetection {
            css_framework,
            js_framework: None,
            css_methodology: None,
        }
    }

    fn calculate_score(
        css: &CssAnalysis,
        js: &JsAnalysis,
        html: &HtmlAnalysis,
    ) -> AnalysisScore {
        let animation_complexity =
            (css.animations.keyframes.len() * 5) as u32 +
            (css.transitions.len() * 2) as u32 +
            (css.transforms.values.len()) as u32 +
            if js.webgl.detected { 20 } else { 0 };

        let interactivity_score =
            (css.pseudo_states.hover_count * 2) as u32 +
            (css.pseudo_states.focus_count * 2) as u32 +
            (js.observers.intersection) as u32 +
            (html.ui_components.buttons) as u32;

        let visual_effects_score =
            (css.shadows.len() * 2) as u32 +
            (css.filters.len() * 3) as u32 +
            if css.transforms.has_3d { 10 } else { 0 };

        let accessibility_score =
            (html.accessibility.aria_attributes.len() * 2) as u32 +
            (html.accessibility.roles.len()) as u32 +
            if html.accessibility.has_alt_texts { 10 } else { 0 };

        let total = animation_complexity + interactivity_score +
                    visual_effects_score + accessibility_score;

        AnalysisScore {
            animation_complexity,
            interactivity_score,
            visual_effects_score,
            accessibility_score,
            total,
        }
    }
}
