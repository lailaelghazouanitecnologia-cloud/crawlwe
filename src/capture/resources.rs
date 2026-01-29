//! Resource Extraction Module
//!
//! Extracts external resources (CSS, JS, fonts) from HTML content.

use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

lazy_static! {
    // Script tags
    static ref SCRIPT_SRC_RE: Regex = Regex::new(
        r#"<script[^>]*\ssrc=["']([^"']+)["'][^>]*>"#
    ).unwrap();

    // Stylesheet links
    static ref LINK_STYLESHEET_RE: Regex = Regex::new(
        r#"<link[^>]*\shref=["']([^"']+)["'][^>]*\srel=["']stylesheet["'][^>]*>"#
    ).unwrap();
    static ref LINK_STYLESHEET_RE2: Regex = Regex::new(
        r#"<link[^>]*\srel=["']stylesheet["'][^>]*\shref=["']([^"']+)["'][^>]*>"#
    ).unwrap();

    // Google Fonts
    static ref GOOGLE_FONTS_RE: Regex = Regex::new(
        r#"fonts\.googleapis\.com/css2?\?family=([^&"']+)"#
    ).unwrap();

    // Inline styles
    static ref STYLE_TAG_RE: Regex = Regex::new(
        r#"(?s)<style[^>]*>(.*?)</style>"#
    ).unwrap();

    // Inline scripts
    static ref SCRIPT_INLINE_RE: Regex = Regex::new(
        r#"(?s)<script(?![^>]*\ssrc=)[^>]*>(.*?)</script>"#
    ).unwrap();

    // Image sources
    static ref IMG_SRC_RE: Regex = Regex::new(
        r#"<img[^>]*\ssrc=["']([^"']+)["'][^>]*>"#
    ).unwrap();

    // CSS url() references
    static ref CSS_URL_RE: Regex = Regex::new(
        r#"url\(["']?([^"')]+)["']?\)"#
    ).unwrap();
}

/// Extracted resource from HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedResource {
    pub url: String,
    pub resource_type: ResourceType,
    pub name: Option<String>,
    pub version: Option<String>,
    pub integrity: Option<String>,
    pub crossorigin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Script,
    Stylesheet,
    Font,
    Image,
    Other,
}

/// Resource extractor
pub struct ResourceExtractor;

impl ResourceExtractor {
    /// Extract all external scripts
    pub fn extract_scripts(html: &str) -> Vec<ExtractedResource> {
        let mut scripts = Vec::new();
        let mut seen = HashSet::new();

        for cap in SCRIPT_SRC_RE.captures_iter(html) {
            let url = cap.get(1).map_or("", |m| m.as_str());

            if url.is_empty() || url.starts_with("data:") {
                continue;
            }

            if !url.starts_with("http") && !url.starts_with("//") {
                continue; // Skip local scripts
            }

            let normalized = Self::normalize_url(url);
            if seen.contains(&normalized) {
                continue;
            }
            seen.insert(normalized.clone());

            scripts.push(ExtractedResource {
                url: normalized.clone(),
                resource_type: ResourceType::Script,
                name: Some(Self::detect_library_name(&normalized)),
                version: Self::extract_version(&normalized),
                integrity: Self::extract_attribute(cap.get(0).map_or("", |m| m.as_str()), "integrity"),
                crossorigin: Self::extract_attribute(cap.get(0).map_or("", |m| m.as_str()), "crossorigin"),
            });
        }

        scripts
    }

    /// Extract all external stylesheets
    pub fn extract_stylesheets(html: &str) -> Vec<ExtractedResource> {
        let mut stylesheets = Vec::new();
        let mut seen = HashSet::new();

        for re in [&*LINK_STYLESHEET_RE, &*LINK_STYLESHEET_RE2] {
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map_or("", |m| m.as_str());

                if url.is_empty() || url.starts_with("data:") {
                    continue;
                }

                if !url.starts_with("http") && !url.starts_with("//") {
                    continue;
                }

                let normalized = Self::normalize_url(url);
                if seen.contains(&normalized) {
                    continue;
                }
                seen.insert(normalized.clone());

                stylesheets.push(ExtractedResource {
                    url: normalized.clone(),
                    resource_type: ResourceType::Stylesheet,
                    name: Some(Self::detect_library_name(&normalized)),
                    version: Self::extract_version(&normalized),
                    integrity: Self::extract_attribute(cap.get(0).map_or("", |m| m.as_str()), "integrity"),
                    crossorigin: Self::extract_attribute(cap.get(0).map_or("", |m| m.as_str()), "crossorigin"),
                });
            }
        }

        stylesheets
    }

    /// Extract Google Fonts
    pub fn extract_fonts(html: &str) -> Vec<ExtractedResource> {
        let mut fonts = Vec::new();
        let mut seen = HashSet::new();

        for cap in GOOGLE_FONTS_RE.captures_iter(html) {
            let family_param = cap.get(1).map_or("", |m| m.as_str());
            let family = family_param
                .split(':')
                .next()
                .unwrap_or(family_param)
                .replace('+', " ");

            if seen.contains(&family) {
                continue;
            }
            seen.insert(family.clone());

            let url = format!(
                "https://fonts.googleapis.com/css2?family={}&display=swap",
                family_param
            );

            fonts.push(ExtractedResource {
                url,
                resource_type: ResourceType::Font,
                name: Some(family),
                version: None,
                integrity: None,
                crossorigin: None,
            });
        }

        fonts
    }

    /// Extract inline styles
    pub fn extract_inline_styles(html: &str) -> String {
        let mut styles = Vec::new();

        for cap in STYLE_TAG_RE.captures_iter(html) {
            if let Some(content) = cap.get(1) {
                let style = content.as_str().trim();
                if !style.is_empty() {
                    styles.push(style.to_string());
                }
            }
        }

        styles.join("\n\n")
    }

    /// Extract inline scripts
    pub fn extract_inline_scripts(html: &str) -> String {
        let mut scripts = Vec::new();

        for cap in SCRIPT_INLINE_RE.captures_iter(html) {
            if let Some(content) = cap.get(1) {
                let script = content.as_str().trim();
                if !script.is_empty() {
                    scripts.push(script.to_string());
                }
            }
        }

        scripts.join("\n\n")
    }

    /// Extract images
    pub fn extract_images(html: &str) -> Vec<String> {
        let mut images = Vec::new();
        let mut seen = HashSet::new();

        for cap in IMG_SRC_RE.captures_iter(html) {
            let url = cap.get(1).map_or("", |m| m.as_str());

            if url.is_empty() || url.starts_with("data:") {
                continue;
            }

            if seen.contains(url) {
                continue;
            }
            seen.insert(url.to_string());

            images.push(Self::normalize_url(url));
        }

        images
    }

    /// Normalize URL (add https: if protocol-relative)
    fn normalize_url(url: &str) -> String {
        if url.starts_with("//") {
            format!("https:{}", url)
        } else {
            url.to_string()
        }
    }

    /// Detect library name from URL
    fn detect_library_name(url: &str) -> String {
        let known_libs = [
            ("gsap", "GSAP"),
            ("three.js", "Three.js"),
            ("three.min.js", "Three.js"),
            ("anime.js", "Anime.js"),
            ("anime.min.js", "Anime.js"),
            ("scrollmagic", "ScrollMagic"),
            ("lottie", "Lottie"),
            ("swiper", "Swiper"),
            ("aos.js", "AOS"),
            ("aos.min.js", "AOS"),
            ("particles", "Particles.js"),
            ("typed", "Typed.js"),
            ("chart", "Chart.js"),
            ("d3", "D3.js"),
            ("jquery", "jQuery"),
            ("bootstrap", "Bootstrap"),
            ("tailwind", "Tailwind CSS"),
            ("bulma", "Bulma"),
            ("alpine", "Alpine.js"),
            ("vue", "Vue.js"),
            ("react", "React"),
            ("angular", "Angular"),
            ("locomotive", "Locomotive Scroll"),
            ("barba", "Barba.js"),
            ("plyr", "Plyr"),
            ("glide", "Glide.js"),
            ("splide", "Splide"),
            ("flickity", "Flickity"),
            ("rellax", "Rellax"),
            ("isotope", "Isotope"),
            ("masonry", "Masonry"),
        ];

        let url_lower = url.to_lowercase();
        for (pattern, name) in known_libs {
            if url_lower.contains(pattern) {
                return name.to_string();
            }
        }

        // Extract from filename
        if let Some(filename) = url.split('/').last() {
            let name = filename
                .split('?')
                .next()
                .unwrap_or(filename)
                .trim_end_matches(".js")
                .trim_end_matches(".min")
                .trim_end_matches(".css")
                .replace('-', " ")
                .replace('_', " ");

            return Self::capitalize_words(&name);
        }

        "Unknown".to_string()
    }

    /// Extract version from URL
    fn extract_version(url: &str) -> Option<String> {
        let patterns = [
            r"@(\d+\.\d+(?:\.\d+)?)",
            r"/(\d+\.\d+(?:\.\d+)?)/",
            r"-(\d+\.\d+(?:\.\d+)?)[./]",
            r"v(\d+\.\d+(?:\.\d+)?)",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(cap) = re.captures(url) {
                    if let Some(version) = cap.get(1) {
                        return Some(version.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract attribute value from tag
    fn extract_attribute(tag: &str, attr: &str) -> Option<String> {
        let pattern = format!(r#"{}=["']([^"']+)["']"#, attr);
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(cap) = re.captures(tag) {
                return cap.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    /// Capitalize first letter of each word
    fn capitalize_words(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scripts() {
        let html = r#"
            <script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.2/gsap.min.js"></script>
            <script src="https://unpkg.com/three@0.150.0/build/three.min.js"></script>
        "#;

        let scripts = ResourceExtractor::extract_scripts(html);
        assert_eq!(scripts.len(), 2);
    }

    #[test]
    fn test_detect_library() {
        assert_eq!(
            ResourceExtractor::detect_library_name("https://cdn.com/gsap.min.js"),
            "GSAP"
        );
    }
}
