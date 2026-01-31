//! CSS Asset Extractor
//!
//! Deep extraction of all assets referenced in CSS:
//! - Fonts (woff, woff2, ttf, otf, eot)
//! - Images (png, jpg, webp, gif, svg, avif)
//! - Background images
//! - Cursor images
//! - Border images
//! - Mask images
//! - Filter URLs (SVG filters)
//!
//! Handles:
//! - url() with quotes, without quotes, with spaces
//! - Data URIs (skipped)
//! - Relative paths
//! - Absolute paths
//! - Protocol-relative URLs (//example.com)
//! - @import statements
//! - @font-face src with format() hints
//! - Google Fonts, Adobe Fonts, custom CDNs

use regex::Regex;
use std::collections::{HashMap, HashSet};
use url::Url;

/// Result of CSS asset extraction
#[derive(Debug, Clone, Default)]
pub struct CssAssetExtractionResult {
    /// Font URLs found (URL -> local path after download)
    pub fonts: HashMap<String, FontInfo>,
    /// Image URLs found (URL -> local path after download)
    pub images: HashMap<String, ImageInfo>,
    /// @import URLs found
    pub imports: Vec<String>,
    /// CSS variables that reference assets
    pub asset_variables: HashMap<String, String>,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Statistics
    pub stats: ExtractionStats,
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    pub url: String,
    pub original_url: String,  // The original URL as it appears in CSS (for replacement)
    pub format: Option<String>,
    pub font_family: Option<String>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub url: String,
    pub original_url: String,  // The original URL as it appears in CSS (for replacement)
    pub context: ImageContext,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageContext {
    BackgroundImage,
    BorderImage,
    MaskImage,
    ListStyleImage,
    Cursor,
    Content,
    Filter,
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractionStats {
    pub total_urls_found: usize,
    pub fonts_found: usize,
    pub images_found: usize,
    pub data_uris_skipped: usize,
    pub imports_found: usize,
    pub invalid_urls: usize,
}

/// CSS Asset Extractor
pub struct CssAssetExtractor {
    // Compiled regex patterns for performance
    url_pattern: Regex,
    import_pattern: Regex,
    font_face_pattern: Regex,
    font_src_pattern: Regex,
    css_var_pattern: Regex,
}

impl CssAssetExtractor {
    pub fn new() -> Self {
        Self {
            // Match url() with optional quotes, handling escaped chars
            url_pattern: Regex::new(
                r#"url\(\s*["']?([^"')]+)["']?\s*\)"#
            ).unwrap(),

            // Match @import with url() or direct string
            import_pattern: Regex::new(
                r#"@import\s+(?:url\(\s*)?["']?([^"';\)]+)["']?\s*\)?[^;]*;"#
            ).unwrap(),

            // Match @font-face blocks
            font_face_pattern: Regex::new(
                r#"@font-face\s*\{([^}]+)\}"#
            ).unwrap(),

            // Match src in @font-face (handles multiple sources)
            font_src_pattern: Regex::new(
                r#"src\s*:\s*([^;]+);"#
            ).unwrap(),

            // Match CSS variables with url()
            css_var_pattern: Regex::new(
                r#"--([a-zA-Z0-9_-]+)\s*:\s*url\(\s*["']?([^"')]+)["']?\s*\)"#
            ).unwrap(),
        }
    }

    /// Extract all assets from CSS content
    pub fn extract(&self, css: &str, base_url: &Url) -> CssAssetExtractionResult {
        let mut result = CssAssetExtractionResult::default();

        // 1. Extract @imports first (they might contain more CSS)
        self.extract_imports(css, base_url, &mut result);

        // 2. Extract @font-face blocks (need special handling for src)
        self.extract_font_faces(css, base_url, &mut result);

        // 3. Extract all other url() references
        self.extract_urls(css, base_url, &mut result);

        // 4. Extract CSS variables that reference assets
        self.extract_css_var_assets(css, base_url, &mut result);

        result
    }

    /// Extract @import statements
    fn extract_imports(&self, css: &str, base_url: &Url, result: &mut CssAssetExtractionResult) {
        for cap in self.import_pattern.captures_iter(css) {
            let import_url = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");

            if import_url.is_empty() || import_url.starts_with("data:") {
                continue;
            }

            match self.resolve_url(import_url, base_url) {
                Some(resolved) => {
                    result.imports.push(resolved);
                    result.stats.imports_found += 1;
                }
                None => {
                    result.errors.push(format!("Invalid @import URL: {}", import_url));
                    result.stats.invalid_urls += 1;
                }
            }
        }
    }

    /// Extract @font-face blocks with detailed info
    fn extract_font_faces(&self, css: &str, base_url: &Url, result: &mut CssAssetExtractionResult) {
        for cap in self.font_face_pattern.captures_iter(css) {
            let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");

            // Extract font-family
            let font_family = self.extract_property(block, "font-family");
            let font_weight = self.extract_property(block, "font-weight");
            let font_style = self.extract_property(block, "font-style");

            // Extract src (may have multiple url() entries)
            if let Some(src_match) = self.font_src_pattern.captures(block) {
                let src_value = src_match.get(1).map(|m| m.as_str()).unwrap_or("");

                // Parse each url() in src
                self.extract_font_urls(
                    src_value,
                    base_url,
                    &font_family,
                    &font_weight,
                    &font_style,
                    result,
                );
            }
        }
    }

    /// Extract font URLs from src value
    fn extract_font_urls(
        &self,
        src: &str,
        base_url: &Url,
        font_family: &Option<String>,
        font_weight: &Option<String>,
        font_style: &Option<String>,
        result: &mut CssAssetExtractionResult,
    ) {
        // Pattern to match url() with optional format()
        let url_format_re = Regex::new(
            r#"url\(\s*["']?([^"')]+)["']?\s*\)(?:\s*format\(\s*["']?([^"')]+)["']?\s*\))?"#
        ).unwrap();

        // Also handle local() references
        let local_re = Regex::new(r#"local\(\s*["']?([^"')]+)["']?\s*\)"#).unwrap();

        for cap in url_format_re.captures_iter(src) {
            let font_url = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let format = cap.get(2).map(|m| m.as_str().to_string());

            // Skip data URIs and local() references
            if font_url.is_empty() || font_url.starts_with("data:") {
                if font_url.starts_with("data:") {
                    result.stats.data_uris_skipped += 1;
                }
                continue;
            }

            result.stats.total_urls_found += 1;

            match self.resolve_url(font_url, base_url) {
                Some(resolved) => {
                    // Check if it's actually a font file
                    if Self::is_font_url(&resolved) {
                        result.fonts.insert(resolved.clone(), FontInfo {
                            url: resolved,
                            original_url: font_url.to_string(),  // Keep original for CSS replacement
                            format,
                            font_family: font_family.clone(),
                            font_weight: font_weight.clone(),
                            font_style: font_style.clone(),
                            local_path: None,
                        });
                        result.stats.fonts_found += 1;
                    }
                }
                None => {
                    result.errors.push(format!("Invalid font URL: {}", font_url));
                    result.stats.invalid_urls += 1;
                }
            }
        }

        // Track local() references (for documentation)
        for cap in local_re.captures_iter(src) {
            let local_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !local_name.is_empty() {
                // Local fonts don't need downloading, but track them
                let local_url = format!("local({})", local_name);
                result.fonts.entry(format!("local:{}", local_name)).or_insert(FontInfo {
                    url: local_url.clone(),
                    original_url: local_url,
                    format: None,
                    font_family: font_family.clone(),
                    font_weight: font_weight.clone(),
                    font_style: font_style.clone(),
                    local_path: None, // N/A for local fonts
                });
            }
        }
    }

    /// Extract all url() references (images, cursors, etc.)
    fn extract_urls(&self, css: &str, base_url: &Url, result: &mut CssAssetExtractionResult) {
        // Process CSS line by line to understand context
        for line in css.lines() {
            let line_lower = line.to_lowercase();

            // Determine context from property name
            let context = if line_lower.contains("background") {
                ImageContext::BackgroundImage
            } else if line_lower.contains("border-image") {
                ImageContext::BorderImage
            } else if line_lower.contains("mask") {
                ImageContext::MaskImage
            } else if line_lower.contains("list-style") {
                ImageContext::ListStyleImage
            } else if line_lower.contains("cursor") {
                ImageContext::Cursor
            } else if line_lower.contains("content") {
                ImageContext::Content
            } else if line_lower.contains("filter") {
                ImageContext::Filter
            } else {
                ImageContext::Other
            };

            // Skip @font-face blocks (already handled)
            if line_lower.contains("@font-face") || line_lower.contains("src:") {
                continue;
            }

            for cap in self.url_pattern.captures_iter(line) {
                let url_str = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");

                // Skip empty, data URIs, and fragment-only references
                if url_str.is_empty() ||
                   url_str.starts_with("data:") ||
                   url_str.starts_with("#") {
                    if url_str.starts_with("data:") {
                        result.stats.data_uris_skipped += 1;
                    }
                    continue;
                }

                result.stats.total_urls_found += 1;

                match self.resolve_url(url_str, base_url) {
                    Some(resolved) => {
                        // Skip if already in fonts
                        if result.fonts.contains_key(&resolved) {
                            continue;
                        }

                        // Check if it's a font (might be outside @font-face)
                        if Self::is_font_url(&resolved) {
                            result.fonts.insert(resolved.clone(), FontInfo {
                                url: resolved,
                                original_url: url_str.to_string(),  // Keep original for CSS replacement
                                format: None,
                                font_family: None,
                                font_weight: None,
                                font_style: None,
                                local_path: None,
                            });
                            result.stats.fonts_found += 1;
                        } else if Self::is_image_url(&resolved) || context != ImageContext::Other {
                            // It's an image or used in an image context
                            result.images.insert(resolved.clone(), ImageInfo {
                                url: resolved,
                                original_url: url_str.to_string(),  // Keep original for CSS replacement
                                context,
                                local_path: None,
                            });
                            result.stats.images_found += 1;
                        }
                    }
                    None => {
                        result.errors.push(format!("Invalid URL: {}", url_str));
                        result.stats.invalid_urls += 1;
                    }
                }
            }
        }
    }

    /// Extract CSS variables that reference assets
    fn extract_css_var_assets(&self, css: &str, base_url: &Url, result: &mut CssAssetExtractionResult) {
        for cap in self.css_var_pattern.captures_iter(css) {
            let var_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let url_str = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            if url_str.is_empty() || url_str.starts_with("data:") {
                continue;
            }

            if let Some(resolved) = self.resolve_url(url_str, base_url) {
                result.asset_variables.insert(
                    format!("--{}", var_name),
                    resolved,
                );
            }
        }
    }

    /// Resolve a URL relative to base
    fn resolve_url(&self, url_str: &str, base_url: &Url) -> Option<String> {
        let url_str = url_str.trim();

        // Handle protocol-relative URLs
        if url_str.starts_with("//") {
            return Some(format!("https:{}", url_str));
        }

        // Handle absolute URLs
        if url_str.starts_with("http://") || url_str.starts_with("https://") {
            return Some(url_str.to_string());
        }

        // Handle relative URLs
        base_url.join(url_str).ok().map(|u| u.to_string())
    }

    /// Extract a CSS property value from a block
    fn extract_property(&self, block: &str, property: &str) -> Option<String> {
        let pattern = format!(r#"{}:\s*["']?([^;"']+)["']?\s*[;}}]"#, regex::escape(property));
        let re = Regex::new(&pattern).ok()?;
        re.captures(block)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().trim_matches('"').trim_matches('\'').to_string())
    }

    /// Check if URL looks like a font file
    fn is_font_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.ends_with(".woff2") ||
        lower.ends_with(".woff") ||
        lower.ends_with(".ttf") ||
        lower.ends_with(".otf") ||
        lower.ends_with(".eot") ||
        lower.contains("fonts.googleapis.com") ||
        lower.contains("fonts.gstatic.com") ||
        lower.contains("use.typekit.net") ||
        lower.contains("fast.fonts.net")
    }

    /// Check if URL looks like an image file
    fn is_image_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.ends_with(".png") ||
        lower.ends_with(".jpg") ||
        lower.ends_with(".jpeg") ||
        lower.ends_with(".gif") ||
        lower.ends_with(".svg") ||
        lower.ends_with(".webp") ||
        lower.ends_with(".avif") ||
        lower.ends_with(".ico") ||
        lower.ends_with(".bmp")
    }

    /// Replace URLs in CSS with local paths
    pub fn replace_urls(&self, css: &str, font_map: &HashMap<String, String>, image_map: &HashMap<String, String>) -> String {
        let mut result = css.to_string();

        // Replace font URLs
        for (original_url, local_path) in font_map {
            result = result.replace(original_url, local_path);
        }

        // Replace image URLs
        for (original_url, local_path) in image_map {
            result = result.replace(original_url, local_path);
        }

        result
    }
}

impl Default for CssAssetExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_font_face() {
        let extractor = CssAssetExtractor::new();
        let css = r#"
            @font-face {
                font-family: 'Open Sans';
                font-weight: 400;
                src: url('https://fonts.example.com/opensans.woff2') format('woff2'),
                     url('https://fonts.example.com/opensans.woff') format('woff');
            }
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let result = extractor.extract(css, &base_url);

        assert_eq!(result.stats.fonts_found, 2);
        assert!(result.fonts.contains_key("https://fonts.example.com/opensans.woff2"));
        assert!(result.fonts.contains_key("https://fonts.example.com/opensans.woff"));
    }

    #[test]
    fn test_extract_background_image() {
        let extractor = CssAssetExtractor::new();
        let css = r#"
            .hero {
                background-image: url('/images/hero.jpg');
            }
            .icon {
                background: url("icons/star.svg") no-repeat;
            }
        "#;

        let base_url = Url::parse("https://example.com/css/").unwrap();
        let result = extractor.extract(css, &base_url);

        assert_eq!(result.stats.images_found, 2);
        assert!(result.images.contains_key("https://example.com/images/hero.jpg"));
        assert!(result.images.contains_key("https://example.com/css/icons/star.svg"));
    }

    #[test]
    fn test_skip_data_uri() {
        let extractor = CssAssetExtractor::new();
        let css = r#"
            .icon {
                background: url('data:image/svg+xml,<svg>...</svg>');
            }
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let result = extractor.extract(css, &base_url);

        assert_eq!(result.stats.images_found, 0);
        assert_eq!(result.stats.data_uris_skipped, 1);
    }

    #[test]
    fn test_extract_imports() {
        let extractor = CssAssetExtractor::new();
        let css = r#"
            @import url('reset.css');
            @import "variables.css";
            @import url(https://example.com/theme.css);
        "#;

        let base_url = Url::parse("https://example.com/styles/").unwrap();
        let result = extractor.extract(css, &base_url);

        assert_eq!(result.stats.imports_found, 3);
        assert!(result.imports.contains(&"https://example.com/styles/reset.css".to_string()));
    }

    #[test]
    fn test_local_font() {
        let extractor = CssAssetExtractor::new();
        let css = r#"
            @font-face {
                font-family: 'System Font';
                src: local('Arial'), local('Helvetica');
            }
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let result = extractor.extract(css, &base_url);

        // Local fonts should be tracked but not counted as downloadable
        assert!(result.fonts.contains_key("local:Arial"));
        assert!(result.fonts.contains_key("local:Helvetica"));
    }

    #[test]
    fn test_protocol_relative_url() {
        let extractor = CssAssetExtractor::new();
        let css = r#"
            .bg {
                background: url(//cdn.example.com/image.png);
            }
        "#;

        let base_url = Url::parse("https://example.com/").unwrap();
        let result = extractor.extract(css, &base_url);

        assert!(result.images.contains_key("https://cdn.example.com/image.png"));
    }
}
