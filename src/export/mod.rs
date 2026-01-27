//! Project Export Module
//!
//! Exports captured web pages to a clean, reproducible structure:
//! - index.html (clean semantic HTML)
//! - styles.css (unified CSS)
//! - scripts/main.js (relevant JS only)
//! - project.toml (metadata and dependencies)

use serde::{Deserialize, Serialize};

/// Project configuration and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project metadata
    pub meta: ProjectMeta,
    /// Source information
    pub source: SourceInfo,
    /// Detected technologies
    pub technologies: Technologies,
    /// External dependencies to import
    pub dependencies: Dependencies,
    /// Export options
    pub export: ExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub captured_at: String,
    pub generator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub url: String,
    pub title: Option<String>,
    pub viewport: ViewportInfo,
    pub language: Option<String>,
    pub charset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportInfo {
    pub width: u32,
    pub height: u32,
    pub device_scale: f64,
    pub device_type: String, // desktop, tablet, mobile
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technologies {
    /// CSS framework detected
    pub css_framework: Option<CssFramework>,
    /// JS framework detected
    pub js_framework: Option<JsFramework>,
    /// Animation libraries
    pub animation_libs: Vec<String>,
    /// 3D/Graphics libraries
    pub graphics_libs: Vec<String>,
    /// UI component libraries
    pub ui_libs: Vec<String>,
    /// Icon libraries
    pub icon_libs: Vec<String>,
    /// Other detected features
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssFramework {
    pub name: String,
    pub version: Option<String>,
    pub cdn_url: Option<String>,
    /// If true, styles are inlined; if false, link to CDN
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsFramework {
    pub name: String,
    pub version: Option<String>,
    pub cdn_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependencies {
    /// CSS files to import (CDN links)
    pub css: Vec<DependencyLink>,
    /// JS files to import (CDN links)
    pub js: Vec<DependencyLink>,
    /// Font imports
    pub fonts: Vec<FontImport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyLink {
    pub name: String,
    pub url: String,
    pub version: Option<String>,
    pub integrity: Option<String>,
    pub crossorigin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontImport {
    pub family: String,
    pub weights: Vec<String>,
    pub source: String, // google, adobe, local
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Minify output files
    pub minify: bool,
    /// Include source maps
    pub sourcemaps: bool,
    /// Inline small assets (base64)
    pub inline_assets: bool,
    /// Max inline asset size in KB
    pub inline_threshold_kb: u32,
    /// Preserve CSS custom properties
    pub preserve_variables: bool,
    /// Output format
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Standard HTML/CSS/JS
    Standard,
    /// Single HTML file with embedded CSS/JS
    SingleFile,
    /// Component format (for frameworks)
    Component,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            minify: false,
            sourcemaps: false,
            inline_assets: true,
            inline_threshold_kb: 10,
            preserve_variables: true,
            format: OutputFormat::Standard,
        }
    }
}

impl Default for Technologies {
    fn default() -> Self {
        Self {
            css_framework: None,
            js_framework: None,
            animation_libs: Vec::new(),
            graphics_libs: Vec::new(),
            ui_libs: Vec::new(),
            icon_libs: Vec::new(),
            features: Vec::new(),
        }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Self {
            css: Vec::new(),
            js: Vec::new(),
            fonts: Vec::new(),
        }
    }
}

/// Known CDN URLs for common libraries
pub struct LibraryCDN;

impl LibraryCDN {
    pub fn get_css(name: &str, version: Option<&str>) -> Option<String> {
        let v = version.unwrap_or("latest");
        match name.to_lowercase().as_str() {
            "tailwind" | "tailwindcss" => Some(format!(
                "https://cdn.tailwindcss.com"
            )),
            "bootstrap" => Some(format!(
                "https://cdn.jsdelivr.net/npm/bootstrap@{}/dist/css/bootstrap.min.css",
                if v == "latest" { "5" } else { v }
            )),
            "bulma" => Some(format!(
                "https://cdn.jsdelivr.net/npm/bulma@{}/css/bulma.min.css",
                if v == "latest" { "0.9" } else { v }
            )),
            "materialize" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/materialize/{}/css/materialize.min.css",
                if v == "latest" { "1.0.0" } else { v }
            )),
            "foundation" => Some(format!(
                "https://cdn.jsdelivr.net/npm/foundation-sites@{}/dist/css/foundation.min.css",
                if v == "latest" { "6.7" } else { v }
            )),
            "animate.css" | "animate" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/animate.css/{}/animate.min.css",
                if v == "latest" { "4.1.1" } else { v }
            )),
            _ => None,
        }
    }

    pub fn get_js(name: &str, version: Option<&str>) -> Option<String> {
        let v = version.unwrap_or("latest");
        match name.to_lowercase().as_str() {
            "gsap" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/gsap/{}/gsap.min.js",
                if v == "latest" { "3.12.2" } else { v }
            )),
            "three" | "threejs" | "three.js" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/three.js/{}/three.min.js",
                if v == "latest" { "r128" } else { v }
            )),
            "anime" | "animejs" | "anime.js" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/animejs/{}/anime.min.js",
                if v == "latest" { "3.2.1" } else { v }
            )),
            "scrollmagic" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/ScrollMagic/{}/ScrollMagic.min.js",
                if v == "latest" { "2.0.8" } else { v }
            )),
            "lottie" | "lottie-web" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/lottie-web/{}/lottie.min.js",
                if v == "latest" { "5.12.2" } else { v }
            )),
            "swiper" => Some(format!(
                "https://cdn.jsdelivr.net/npm/swiper@{}/swiper-bundle.min.js",
                if v == "latest" { "10" } else { v }
            )),
            "aos" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/aos/{}/aos.js",
                if v == "latest" { "2.3.4" } else { v }
            )),
            "particles" | "particles.js" => Some(
                "https://cdn.jsdelivr.net/particles.js/2.0.0/particles.min.js".to_string()
            ),
            "typed" | "typed.js" => Some(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/typed.js/{}/typed.min.js",
                if v == "latest" { "2.0.16" } else { v }
            )),
            _ => None,
        }
    }

    pub fn get_font_url(family: &str, weights: &[String]) -> String {
        let weights_str = if weights.is_empty() {
            "400;700".to_string()
        } else {
            weights.join(";")
        };
        let family_encoded = family.replace(' ', "+");
        format!(
            "https://fonts.googleapis.com/css2?family={}:wght@{}&display=swap",
            family_encoded, weights_str
        )
    }
}

/// Project exporter
pub struct ProjectExporter {
    config: ProjectConfig,
}

impl ProjectExporter {
    pub fn new(config: ProjectConfig) -> Self {
        Self { config }
    }

    /// Generate project.toml content
    pub fn generate_project_toml(&self) -> String {
        let mut toml = String::new();

        // Meta section
        toml.push_str("[meta]\n");
        toml.push_str(&format!("name = \"{}\"\n", self.config.meta.name));
        toml.push_str(&format!("version = \"{}\"\n", self.config.meta.version));
        if let Some(desc) = &self.config.meta.description {
            toml.push_str(&format!("description = \"{}\"\n", desc));
        }
        toml.push_str(&format!("captured_at = \"{}\"\n", self.config.meta.captured_at));
        toml.push_str(&format!("generator = \"{}\"\n", self.config.meta.generator));
        toml.push('\n');

        // Source section
        toml.push_str("[source]\n");
        toml.push_str(&format!("url = \"{}\"\n", self.config.source.url));
        if let Some(title) = &self.config.source.title {
            toml.push_str(&format!("title = \"{}\"\n", title));
        }
        toml.push_str(&format!("charset = \"{}\"\n", self.config.source.charset));
        if let Some(lang) = &self.config.source.language {
            toml.push_str(&format!("language = \"{}\"\n", lang));
        }
        toml.push('\n');

        // Viewport section
        toml.push_str("[source.viewport]\n");
        toml.push_str(&format!("width = {}\n", self.config.source.viewport.width));
        toml.push_str(&format!("height = {}\n", self.config.source.viewport.height));
        toml.push_str(&format!("device_scale = {}\n", self.config.source.viewport.device_scale));
        toml.push_str(&format!("device_type = \"{}\"\n", self.config.source.viewport.device_type));
        toml.push('\n');

        // Technologies section
        toml.push_str("[technologies]\n");
        if let Some(css_fw) = &self.config.technologies.css_framework {
            toml.push_str(&format!("css_framework = \"{}\"\n", css_fw.name));
        }
        if let Some(js_fw) = &self.config.technologies.js_framework {
            toml.push_str(&format!("js_framework = \"{}\"\n", js_fw.name));
        }
        if !self.config.technologies.animation_libs.is_empty() {
            toml.push_str(&format!(
                "animation_libs = [{}]\n",
                self.config.technologies.animation_libs
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.config.technologies.graphics_libs.is_empty() {
            toml.push_str(&format!(
                "graphics_libs = [{}]\n",
                self.config.technologies.graphics_libs
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.config.technologies.features.is_empty() {
            toml.push_str(&format!(
                "features = [{}]\n",
                self.config.technologies.features
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        toml.push('\n');

        // Dependencies section
        if !self.config.dependencies.css.is_empty() {
            toml.push_str("[[dependencies.css]]\n");
            for dep in &self.config.dependencies.css {
                toml.push_str(&format!("name = \"{}\"\n", dep.name));
                toml.push_str(&format!("url = \"{}\"\n", dep.url));
                if let Some(v) = &dep.version {
                    toml.push_str(&format!("version = \"{}\"\n", v));
                }
                toml.push('\n');
            }
        }

        if !self.config.dependencies.js.is_empty() {
            toml.push_str("[[dependencies.js]]\n");
            for dep in &self.config.dependencies.js {
                toml.push_str(&format!("name = \"{}\"\n", dep.name));
                toml.push_str(&format!("url = \"{}\"\n", dep.url));
                if let Some(v) = &dep.version {
                    toml.push_str(&format!("version = \"{}\"\n", v));
                }
                toml.push('\n');
            }
        }

        if !self.config.dependencies.fonts.is_empty() {
            toml.push_str("[[dependencies.fonts]]\n");
            for font in &self.config.dependencies.fonts {
                toml.push_str(&format!("family = \"{}\"\n", font.family));
                toml.push_str(&format!("source = \"{}\"\n", font.source));
                if !font.weights.is_empty() {
                    toml.push_str(&format!(
                        "weights = [{}]\n",
                        font.weights.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")
                    ));
                }
                toml.push('\n');
            }
        }

        // Export section
        toml.push_str("[export]\n");
        toml.push_str(&format!("minify = {}\n", self.config.export.minify));
        toml.push_str(&format!("sourcemaps = {}\n", self.config.export.sourcemaps));
        toml.push_str(&format!("inline_assets = {}\n", self.config.export.inline_assets));
        toml.push_str(&format!("preserve_variables = {}\n", self.config.export.preserve_variables));

        toml
    }

    /// Generate index.html with proper imports
    pub fn generate_html(&self, body_content: &str) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html");
        if let Some(lang) = &self.config.source.language {
            html.push_str(&format!(" lang=\"{}\"", lang));
        }
        html.push_str(">\n");
        html.push_str("<head>\n");
        html.push_str(&format!("  <meta charset=\"{}\">\n", self.config.source.charset));
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");

        if let Some(title) = &self.config.source.title {
            html.push_str(&format!("  <title>{}</title>\n", title));
        }

        // Font imports
        for font in &self.config.dependencies.fonts {
            if let Some(url) = &font.url {
                html.push_str(&format!("  <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n"));
                html.push_str(&format!("  <link href=\"{}\" rel=\"stylesheet\">\n", url));
            }
        }

        // CSS dependencies
        for dep in &self.config.dependencies.css {
            html.push_str(&format!("  <link rel=\"stylesheet\" href=\"{}\"", dep.url));
            if let Some(integrity) = &dep.integrity {
                html.push_str(&format!(" integrity=\"{}\"", integrity));
            }
            if let Some(crossorigin) = &dep.crossorigin {
                html.push_str(&format!(" crossorigin=\"{}\"", crossorigin));
            }
            html.push_str(">\n");
        }

        // Local stylesheet
        html.push_str("  <link rel=\"stylesheet\" href=\"styles.css\">\n");

        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str(body_content);
        html.push_str("\n");

        // JS dependencies
        for dep in &self.config.dependencies.js {
            html.push_str(&format!("  <script src=\"{}\"", dep.url));
            if let Some(integrity) = &dep.integrity {
                html.push_str(&format!(" integrity=\"{}\"", integrity));
            }
            if let Some(crossorigin) = &dep.crossorigin {
                html.push_str(&format!(" crossorigin=\"{}\"", crossorigin));
            }
            html.push_str("></script>\n");
        }

        // Local script
        html.push_str("  <script src=\"scripts/main.js\"></script>\n");

        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_cdn() {
        assert!(LibraryCDN::get_css("tailwind", None).is_some());
        assert!(LibraryCDN::get_js("gsap", None).is_some());
        assert!(LibraryCDN::get_js("three", None).is_some());
    }
}
