//! Library parser registry
//!
//! Automatically detects and loads appropriate parsers for each library.

use super::{
    AnimationDefinition, DetectedLibrary, LibraryAnalysis, LibraryCategory,
    GsapParser, FramerMotionParser, StyledComponentsParser, ReactParser,
    ThreeJsParser, TailwindParser, AnimeJsParser,
};
use crate::js::analyzer::JsAnalysisResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect that a library produces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEffect {
    /// Effect type
    pub effect_type: EffectType,
    /// Target selector
    pub target: String,
    /// CSS properties affected
    pub css_properties: HashMap<String, String>,
    /// Timing information
    pub timing: Option<EffectTiming>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    /// Animation effect
    Animation,
    /// Style change
    StyleChange,
    /// Class manipulation
    ClassChange,
    /// DOM insertion
    DomInsertion,
    /// 3D rendering
    ThreeDRender,
    /// Canvas drawing
    CanvasDraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectTiming {
    /// Start time in ms
    pub start: f64,
    /// Duration in ms
    pub duration: f64,
    /// Easing function
    pub easing: String,
}

/// Trait for library-specific parsers
pub trait LibraryParser: Send + Sync {
    /// Library name
    fn name(&self) -> &'static str;

    /// Library category
    fn category(&self) -> LibraryCategory;

    /// Check if this parser handles the given library
    fn handles(&self, analysis: &JsAnalysisResult) -> bool;

    /// Analyze JavaScript code specific to this library
    fn analyze(&self, code: &str, analysis: &JsAnalysisResult) -> LibraryAnalysis;

    /// Generate CSS from analysis results
    fn generate_css(&self, analysis: &LibraryAnalysis) -> String;
}

/// Registry of all library parsers
pub struct LibraryRegistry {
    parsers: Vec<Box<dyn LibraryParser>>,
}

impl Default for LibraryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryRegistry {
    /// Create a new registry with all built-in parsers
    pub fn new() -> Self {
        let parsers: Vec<Box<dyn LibraryParser>> = vec![
            Box::new(GsapParser::new()),
            Box::new(FramerMotionParser::new()),
            Box::new(StyledComponentsParser::new()),
            Box::new(ReactParser::new()),
            Box::new(ThreeJsParser::new()),
            Box::new(TailwindParser::new()),
            Box::new(AnimeJsParser::new()),
        ];

        Self { parsers }
    }

    /// Detect which libraries are used in the code
    pub fn detect_libraries(&self, analysis: &JsAnalysisResult) -> Vec<DetectedLibrary> {
        let mut detected = Vec::new();

        for parser in &self.parsers {
            if parser.handles(analysis) {
                detected.push(DetectedLibrary {
                    name: parser.name().to_string(),
                    category: parser.category(),
                    version: None,
                    imports: analysis.libraries.clone(),
                    patterns: Vec::new(),
                });
            }
        }

        detected
    }

    /// Analyze code with all applicable parsers
    pub fn analyze(&self, code: &str, analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut combined = LibraryAnalysis::new();

        for parser in &self.parsers {
            if parser.handles(analysis) {
                let lib_analysis = parser.analyze(code, analysis);
                combined.merge(lib_analysis);
            }
        }

        combined
    }

    /// Generate CSS from all libraries
    pub fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        // Use the analysis to generate CSS
        css.push_str(&analysis.to_css());

        css
    }

    /// Get a specific parser by name
    pub fn get_parser(&self, name: &str) -> Option<&dyn LibraryParser> {
        self.parsers.iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// List all registered parsers
    pub fn list_parsers(&self) -> Vec<&str> {
        self.parsers.iter().map(|p| p.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = LibraryRegistry::new();
        let parsers = registry.list_parsers();

        assert!(parsers.contains(&"gsap"));
        assert!(parsers.contains(&"framer-motion"));
        assert!(parsers.contains(&"styled-components"));
    }
}
