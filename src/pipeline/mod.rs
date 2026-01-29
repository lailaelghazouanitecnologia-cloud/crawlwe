//! Pipeline - Unified processing pipeline
//!
//! Integrates microparsers, VM, and optimizer into a single pipeline:
//!
//! ```text
//! Input (HTML/CSS/JS)
//!       │
//!       ▼
//! ┌─────────────────┐
//! │  Microparsers   │  Extract structured data
//! │  CSS │ JS │ Lib │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │       VM        │  Execute parsing programs
//! │   (bytecode)    │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   Optimizer     │  Clean, deduplicate, format
//! └────────┬────────┘
//!          │
//!          ▼
//! Output (index.html, styles.css, project.toml)
//! ```

pub mod microparsers;
pub mod optimizer;

pub use microparsers::*;
pub use optimizer::*;

use serde::{Deserialize, Serialize};

use crate::vm::{VM, ParserBuilder};
use crate::export::LibraryCDN;

// ============================================================================
// PIPELINE
// ============================================================================

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub css_options: CssOptimizeOptions,
    pub html_options: HtmlOptimizeOptions,
    pub output_format: OutputFormat,
    pub detect_libraries: bool,
    pub generate_project_toml: bool,
    pub use_vm: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            css_options: CssOptimizeOptions::default(),
            html_options: HtmlOptimizeOptions::default(),
            output_format: OutputFormat::default(),
            detect_libraries: true,
            generate_project_toml: true,
            use_vm: true,
        }
    }
}

/// Pipeline input
#[derive(Debug, Clone)]
pub struct PipelineInput {
    pub url: String,
    pub html: String,
    pub css: String,
    pub js: String,
    pub title: Option<String>,
    pub external_scripts: Vec<String>,
    pub external_styles: Vec<String>,
}

/// Pipeline output - ready for writing to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub html: String,
    pub css: String,
    pub project_toml: String,
    pub metadata: PipelineMetadata,
}

/// Metadata about the processed page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    pub url: String,
    pub title: Option<String>,
    pub libraries: Vec<LibraryInfo>,
    pub css_stats: CssOptimizeStats,
    pub html_stats: HtmlOptimizeStats,
    pub parse_stats: ParseStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub name: String,
    pub category: String,
    pub version: Option<String>,
    pub cdn_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParseStats {
    pub css_variables: usize,
    pub css_keyframes: usize,
    pub css_colors: usize,
    pub js_imports: usize,
    pub js_exports: usize,
    pub detected_libraries: usize,
}

/// Main processing pipeline
pub struct Pipeline {
    config: PipelineConfig,
    vm: VM,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            vm: VM::new(),
        }
    }

    pub fn with_default() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// Process input through the full pipeline
    pub fn process(&mut self, input: PipelineInput) -> PipelineOutput {
        let mut parse_stats = ParseStats::default();

        // ============================================
        // PHASE 1: Microparse
        // ============================================

        // Parse CSS
        let css_parsed = CssMicroparser::parse(&input.css);
        parse_stats.css_variables = css_parsed.variables.len();
        parse_stats.css_keyframes = css_parsed.keyframes.len();
        parse_stats.css_colors = css_parsed.colors.len();

        // Parse JS
        let js_parsed = JsMicroparser::parse(&input.js);
        parse_stats.js_imports = js_parsed.imports.len();
        parse_stats.js_exports = js_parsed.exports.len();

        // Parse HTML for resources
        let html_parsed = HtmlMicroparser::parse(&input.html);

        // Detect libraries
        let mut all_script_urls: Vec<String> = input.external_scripts.clone();
        for script in &html_parsed.scripts {
            if let Some(src) = &script.src {
                all_script_urls.push(src.clone());
            }
        }

        let lib_result = if self.config.detect_libraries {
            LibMicroparser::parse(&all_script_urls, &input.js)
        } else {
            LibParseResult::default()
        };
        parse_stats.detected_libraries = lib_result.libraries.len();

        // ============================================
        // PHASE 2: VM Processing (optional)
        // ============================================

        if self.config.use_vm {
            // Run CSS analyzer
            let css_program = ParserBuilder::css_analyzer();
            let _ = self.vm.execute(&css_program, &input.css);

            // Run HTML extractor
            let html_program = ParserBuilder::html_extractor();
            let _ = self.vm.execute(&html_program, &input.html);
        }

        // ============================================
        // PHASE 3: Optimize
        // ============================================

        // Optimize CSS
        let css_optimized = CssOptimizer::optimize(&input.css, &self.config.css_options);

        // Optimize HTML
        let html_optimized = HtmlOptimizer::optimize(&input.html, &self.config.html_options);

        // ============================================
        // PHASE 4: Format Output
        // ============================================

        let js_libs: Vec<String> = lib_result.libraries.iter()
            .filter_map(|lib| {
                if let LibrarySource::Url(url) = &lib.source {
                    Some(url.clone())
                } else {
                    LibraryCDN::get_js(&lib.name, lib.version.as_deref())
                }
            })
            .collect();

        let formatted = OutputFormatter::format_package(
            &html_optimized.html,
            &css_optimized.css,
            &js_libs,
            &self.config.output_format,
        );

        // ============================================
        // PHASE 5: Generate Project Config
        // ============================================

        let libraries: Vec<LibraryInfo> = lib_result.libraries.iter()
            .map(|lib| LibraryInfo {
                name: lib.name.clone(),
                category: lib.category.clone(),
                version: lib.version.clone(),
                cdn_url: match &lib.source {
                    LibrarySource::Url(url) => Some(url.clone()),
                    _ => LibraryCDN::get_js(&lib.name, lib.version.as_deref()),
                },
            })
            .collect();

        let project_toml = if self.config.generate_project_toml {
            self.generate_project_toml(&input, &libraries, &css_parsed)
        } else {
            String::new()
        };

        // Build output
        PipelineOutput {
            html: formatted.html,
            css: formatted.css,
            project_toml,
            metadata: PipelineMetadata {
                url: input.url,
                title: input.title,
                libraries,
                css_stats: css_optimized.stats,
                html_stats: html_optimized.stats,
                parse_stats,
            },
        }
    }

    fn generate_project_toml(
        &self,
        input: &PipelineInput,
        libraries: &[LibraryInfo],
        css_parsed: &CssParseResult,
    ) -> String {
        use chrono::Utc;
        use crate::export::*;

        let mut config = ProjectConfig {
            meta: ProjectMeta {
                name: sanitize_name(input.title.as_deref().unwrap_or("captured-page")),
                version: "1.0.0".to_string(),
                description: input.title.clone(),
                captured_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                generator: "crawlwe".to_string(),
            },
            source: SourceInfo {
                url: input.url.clone(),
                title: input.title.clone(),
                viewport: ViewportInfo::default(),
                language: detect_language(&input.html),
                charset: "UTF-8".to_string(),
            },
            technologies: Technologies {
                css_framework: detect_css_framework(libraries),
                js_framework: None,
                animation_libs: libraries.iter()
                    .filter(|l| l.category == "animation")
                    .map(|l| l.name.clone())
                    .collect(),
                graphics_libs: libraries.iter()
                    .filter(|l| l.category == "3d" || l.category == "2d")
                    .map(|l| l.name.clone())
                    .collect(),
                ui_libs: libraries.iter()
                    .filter(|l| l.category == "ui")
                    .map(|l| l.name.clone())
                    .collect(),
                icon_libs: vec![],
                features: {
                    let mut f = vec![];
                    if input.js.contains("getContext('webgl") || input.js.contains("getContext(\"webgl") {
                        f.push("webgl".to_string());
                    }
                    if input.js.contains("navigator.gpu") {
                        f.push("webgpu".to_string());
                    }
                    f
                },
            },
            dependencies: Dependencies {
                js: libraries.iter()
                    .filter_map(|l| {
                        l.cdn_url.as_ref().map(|url| DependencyLink {
                            name: l.name.clone(),
                            url: url.clone(),
                            version: l.version.clone(),
                            integrity: None,
                            crossorigin: None,
                        })
                    })
                    .collect(),
                css: vec![],
                fonts: css_parsed.fonts.iter()
                    .map(|f| FontImport {
                        family: f.clone(),
                        weights: vec!["400".to_string(), "700".to_string()],
                        source: "google".to_string(),
                        url: Some(LibraryCDN::get_font_url(f, &["400".to_string(), "700".to_string()])),
                    })
                    .collect(),
            },
            export: ExportOptions::default(),
        };

        let exporter = ProjectExporter::new(config);
        exporter.generate_project_toml()
    }
}

// Helper functions

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn detect_language(html: &str) -> Option<String> {
    let patterns = [r#"lang="([^"]+)""#, r#"lang='([^']+)'"#];
    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(cap) = re.captures(html) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
    }
    None
}

fn detect_css_framework(libraries: &[LibraryInfo]) -> Option<crate::export::CssFramework> {
    for lib in libraries {
        if lib.category == "css" {
            return Some(crate::export::CssFramework {
                name: lib.name.clone(),
                version: lib.version.clone(),
                cdn_url: lib.cdn_url.clone(),
                inline: false,
            });
        }
    }
    None
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Quick process function for simple use cases
pub fn process(
    url: &str,
    html: &str,
    css: &str,
    js: &str,
) -> PipelineOutput {
    let mut pipeline = Pipeline::with_default();
    pipeline.process(PipelineInput {
        url: url.to_string(),
        html: html.to_string(),
        css: css.to_string(),
        js: js.to_string(),
        title: None,
        external_scripts: vec![],
        external_styles: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head><title>Test</title></head>
            <body>
                <div class="container">
                    <h1>Hello World</h1>
                </div>
                <script src="https://cdn.jsdelivr.net/npm/gsap@3.12.2/dist/gsap.min.js"></script>
            </body>
            </html>
        "#;

        let css = r#"
            :root { --primary: #3b82f6; }
            .container { max-width: 1200px; color: var(--primary); }
        "#;

        let result = process("https://example.com", html, css, "");

        assert!(!result.html.is_empty());
        assert!(!result.css.is_empty());
        assert!(result.metadata.parse_stats.css_variables > 0);
    }
}
