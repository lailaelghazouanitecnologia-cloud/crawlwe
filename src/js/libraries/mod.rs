//! Library-specific parsers
//!
//! Each library has its own parser that understands its specific patterns
//! for rendering, animations, and styling.

pub mod gsap;
pub mod framer;
pub mod styled;
pub mod react;
pub mod three;
pub mod tailwind;
pub mod anime;
pub mod registry;

pub use registry::{LibraryRegistry, LibraryParser, LibraryEffect};
pub use gsap::GsapParser;
pub use framer::FramerMotionParser;
pub use styled::StyledComponentsParser;
pub use react::ReactParser;
pub use three::ThreeJsParser;
pub use tailwind::TailwindParser;
pub use anime::AnimeJsParser;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detected library with its version and usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLibrary {
    /// Library name
    pub name: String,
    /// Library category
    pub category: LibraryCategory,
    /// Detected version (if available)
    pub version: Option<String>,
    /// Import sources found
    pub imports: Vec<String>,
    /// Usage patterns detected
    pub patterns: Vec<String>,
}

/// Library categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryCategory {
    /// Animation libraries (GSAP, Framer Motion, anime.js)
    Animation,
    /// 3D rendering (Three.js, Babylon)
    ThreeD,
    /// 2D canvas/SVG (Paper.js, Fabric.js, Lottie)
    TwoD,
    /// UI frameworks (React, Vue, Svelte)
    UIFramework,
    /// CSS-in-JS (styled-components, Emotion)
    CssInJs,
    /// CSS utilities (Tailwind, Bootstrap)
    CssUtility,
    /// State management (Redux, MobX)
    StateManagement,
    /// Scroll effects (Locomotive Scroll, ScrollMagic)
    Scroll,
    /// Data visualization (D3, Chart.js)
    DataViz,
    /// Other
    Other,
}

impl LibraryCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Animation => "animation",
            Self::ThreeD => "3d",
            Self::TwoD => "2d",
            Self::UIFramework => "ui-framework",
            Self::CssInJs => "css-in-js",
            Self::CssUtility => "css-utility",
            Self::StateManagement => "state",
            Self::Scroll => "scroll",
            Self::DataViz => "dataviz",
            Self::Other => "other",
        }
    }
}

/// Result from library-specific analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryAnalysis {
    /// Detected libraries
    pub libraries: Vec<DetectedLibrary>,

    /// CSS that should be generated
    pub generated_css: Vec<String>,

    /// Animations detected (name -> keyframes CSS)
    pub animations: HashMap<String, AnimationDefinition>,

    /// Elements that should be rendered
    pub elements_to_render: Vec<RenderElement>,

    /// JavaScript effects to simulate
    pub effects: Vec<LibraryEffect>,

    /// Warnings about unsupported features
    pub warnings: Vec<String>,
}

/// Animation definition extracted from JS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDefinition {
    /// Animation name/identifier
    pub name: String,
    /// Target selector
    pub target: String,
    /// Duration in seconds
    pub duration: f64,
    /// CSS properties being animated
    pub properties: HashMap<String, AnimationProperty>,
    /// Easing function
    pub easing: String,
    /// Generated CSS keyframes
    pub keyframes_css: Option<String>,
}

/// Single property animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationProperty {
    /// Starting value
    pub from: String,
    /// Ending value
    pub to: String,
    /// CSS property name
    pub css_property: String,
}

/// Element to render from JS analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderElement {
    /// HTML tag
    pub tag: String,
    /// CSS classes
    pub classes: Vec<String>,
    /// Inline styles
    pub styles: HashMap<String, String>,
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Inner HTML content
    pub content: Option<String>,
    /// Child elements
    pub children: Vec<RenderElement>,
}

impl LibraryAnalysis {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another analysis into this one
    pub fn merge(&mut self, other: LibraryAnalysis) {
        self.libraries.extend(other.libraries);
        self.generated_css.extend(other.generated_css);
        self.animations.extend(other.animations);
        self.elements_to_render.extend(other.elements_to_render);
        self.effects.extend(other.effects);
        self.warnings.extend(other.warnings);
    }

    /// Generate all CSS from the analysis
    pub fn to_css(&self) -> String {
        let mut css = String::new();

        // Generated CSS
        for block in &self.generated_css {
            css.push_str(block);
            css.push_str("\n\n");
        }

        // Animation keyframes
        for (name, anim) in &self.animations {
            if let Some(keyframes) = &anim.keyframes_css {
                css.push_str(&format!("/* Animation: {} */\n", name));
                css.push_str(keyframes);
                css.push_str("\n\n");
            }
        }

        css
    }
}
