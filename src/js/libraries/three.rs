//! Three.js parser
//!
//! Analyzes Three.js code to understand 3D scene setup and rendering.

use super::{
    LibraryAnalysis, LibraryCategory, RenderElement,
    registry::{LibraryParser, LibraryEffect, EffectType},
};
use crate::js::analyzer::JsAnalysisResult;
use regex::Regex;
use std::collections::HashMap;

/// Parser for Three.js 3D rendering
pub struct ThreeJsParser {
    // Three.js patterns
    scene_creation: Regex,
    camera_creation: Regex,
    renderer_creation: Regex,
    mesh_creation: Regex,
    geometry_creation: Regex,
    material_creation: Regex,
    light_creation: Regex,
    animation_loop: Regex,
    orbit_controls: Regex,
    texture_loader: Regex,
}

impl ThreeJsParser {
    pub fn new() -> Self {
        Self {
            scene_creation: Regex::new(r#"new\s+THREE\.Scene\s*\(\)"#).unwrap(),
            camera_creation: Regex::new(r#"new\s+THREE\.(\w+)Camera\s*\(([^)]*)\)"#).unwrap(),
            renderer_creation: Regex::new(r#"new\s+THREE\.WebGLRenderer\s*\(([^)]*)\)"#).unwrap(),
            mesh_creation: Regex::new(r#"new\s+THREE\.Mesh\s*\(([^)]*)\)"#).unwrap(),
            geometry_creation: Regex::new(r#"new\s+THREE\.(\w+)Geometry\s*\(([^)]*)\)"#).unwrap(),
            material_creation: Regex::new(r#"new\s+THREE\.(\w+)Material\s*\(\s*\{([^}]*)\}"#).unwrap(),
            light_creation: Regex::new(r#"new\s+THREE\.(\w+)Light\s*\(([^)]*)\)"#).unwrap(),
            animation_loop: Regex::new(r#"requestAnimationFrame\s*\(\s*(\w+)\s*\)"#).unwrap(),
            orbit_controls: Regex::new(r#"new\s+OrbitControls\s*\("#).unwrap(),
            texture_loader: Regex::new(r#"new\s+THREE\.TextureLoader\s*\(\)"#).unwrap(),
        }
    }

    /// Extract color from Three.js color value
    fn parse_color(color_str: &str) -> Option<String> {
        // Hex color: 0xff0000 -> #ff0000
        if color_str.starts_with("0x") {
            return Some(format!("#{}", &color_str[2..]));
        }

        // String color
        if color_str.starts_with('"') || color_str.starts_with('\'') {
            return Some(color_str.trim_matches(|c| c == '"' || c == '\'').to_string());
        }

        None
    }
}

impl Default for ThreeJsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryParser for ThreeJsParser {
    fn name(&self) -> &'static str {
        "three"
    }

    fn category(&self) -> LibraryCategory {
        LibraryCategory::ThreeD
    }

    fn handles(&self, analysis: &JsAnalysisResult) -> bool {
        analysis.libraries.iter().any(|lib| lib == "three" || lib.contains("three")) ||
        analysis.raw.as_ref()
            .map(|r| r.imports.iter().any(|(_, src)| src.contains("three")))
            .unwrap_or(false)
    }

    fn analyze(&self, code: &str, _analysis: &JsAnalysisResult) -> LibraryAnalysis {
        let mut result = LibraryAnalysis::new();
        let mut scene_info = HashMap::new();

        // Detect scene creation
        if self.scene_creation.is_match(code) {
            scene_info.insert("has_scene".to_string(), "true".to_string());
        }

        // Detect camera type
        if let Some(cap) = self.camera_creation.captures(code) {
            let camera_type = cap.get(1).map(|m| m.as_str()).unwrap_or("Perspective");
            scene_info.insert("camera_type".to_string(), camera_type.to_string());
        }

        // Detect renderer
        if self.renderer_creation.is_match(code) {
            scene_info.insert("has_renderer".to_string(), "true".to_string());

            // Generate canvas CSS
            result.generated_css.push(r#"
/* Three.js Canvas Styles */
.three-canvas {
    display: block;
    width: 100%;
    height: 100%;
}

.three-container {
    position: relative;
    width: 100%;
    height: 100vh;
    overflow: hidden;
}
"#.to_string());
        }

        // Detect geometries
        let mut geometries = Vec::new();
        for cap in self.geometry_creation.captures_iter(code) {
            if let Some(geom_type) = cap.get(1) {
                geometries.push(geom_type.as_str().to_string());
            }
        }

        // Detect materials and extract colors
        let mut materials = Vec::new();
        for cap in self.material_creation.captures_iter(code) {
            let mat_type = cap.get(1).map(|m| m.as_str()).unwrap_or("Basic");
            let props = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            materials.push(mat_type.to_string());

            // Extract color
            let color_pattern = Regex::new(r#"color\s*:\s*([^,}]+)"#).unwrap();
            if let Some(color_cap) = color_pattern.captures(props) {
                if let Some(color_str) = color_cap.get(1) {
                    if let Some(color) = Self::parse_color(color_str.as_str().trim()) {
                        scene_info.insert("material_color".to_string(), color);
                    }
                }
            }
        }

        // Detect lights
        let mut lights = Vec::new();
        for cap in self.light_creation.captures_iter(code) {
            if let Some(light_type) = cap.get(1) {
                lights.push(light_type.as_str().to_string());
            }
        }

        // Detect animation loop
        if self.animation_loop.is_match(code) {
            scene_info.insert("has_animation".to_string(), "true".to_string());
        }

        // Detect controls
        if self.orbit_controls.is_match(code) {
            scene_info.insert("has_orbit_controls".to_string(), "true".to_string());
        }

        // Build effect description
        result.effects.push(LibraryEffect {
            effect_type: EffectType::ThreeDRender,
            target: "canvas".to_string(),
            css_properties: scene_info.clone(),
            timing: None,
        });

        // Create render element for the canvas
        result.elements_to_render.push(RenderElement {
            tag: "canvas".to_string(),
            classes: vec!["three-canvas".to_string()],
            styles: HashMap::new(),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("data-engine".to_string(), "three.js".to_string());
                if !geometries.is_empty() {
                    attrs.insert("data-geometries".to_string(), geometries.join(","));
                }
                if !materials.is_empty() {
                    attrs.insert("data-materials".to_string(), materials.join(","));
                }
                if !lights.is_empty() {
                    attrs.insert("data-lights".to_string(), lights.join(","));
                }
                attrs
            },
            content: None,
            children: Vec::new(),
        });

        // Add warnings about dynamic 3D content
        result.warnings.push("Three.js 3D scene detected - visual representation requires JavaScript execution".to_string());

        if self.texture_loader.is_match(code) {
            result.warnings.push("Texture loading detected - external image resources may be required".to_string());
        }

        result
    }

    fn generate_css(&self, analysis: &LibraryAnalysis) -> String {
        let mut css = String::new();

        css.push_str("/* Three.js Scene CSS */\n\n");

        for block in &analysis.generated_css {
            css.push_str(block);
            css.push_str("\n\n");
        }

        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_three_scene() {
        let parser = ThreeJsParser::new();
        let code = r#"
            const scene = new THREE.Scene();
            const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight);
            const renderer = new THREE.WebGLRenderer();
        "#;

        let mut analysis = JsAnalysisResult::default();
        analysis.libraries.push("three".to_string());

        let result = parser.analyze(code, &analysis);
        assert!(!result.effects.is_empty());
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(ThreeJsParser::parse_color("0xff0000"), Some("#ff0000".to_string()));
        assert_eq!(ThreeJsParser::parse_color("\"red\""), Some("red".to_string()));
    }
}
