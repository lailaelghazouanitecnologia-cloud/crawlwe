//! External Resource Extractor
//!
//! Extracts external CSS, JS, and font resources from HTML
//! to ensure all dependencies are captured.

use regex::Regex;
use lazy_static::lazy_static;
use super::{DependencyLink, FontImport};

lazy_static! {
    // Match <script src="...">
    static ref SCRIPT_RE: Regex = Regex::new(
        r#"<script[^>]*\ssrc=["']([^"']+)["'][^>]*>"#
    ).unwrap();

    // Match <link rel="stylesheet" href="...">
    static ref STYLESHEET_RE: Regex = Regex::new(
        r#"<link[^>]*\srel=["']stylesheet["'][^>]*\shref=["']([^"']+)["'][^>]*>"#
    ).unwrap();

    // Match <link href="..." rel="stylesheet">
    static ref STYLESHEET_RE2: Regex = Regex::new(
        r#"<link[^>]*\shref=["']([^"']+)["'][^>]*\srel=["']stylesheet["'][^>]*>"#
    ).unwrap();

    // Match Google Fonts URLs
    static ref GOOGLE_FONTS_RE: Regex = Regex::new(
        r#"fonts\.googleapis\.com/css2?\?family=([^&"']+)"#
    ).unwrap();

    // Match integrity attribute
    static ref INTEGRITY_RE: Regex = Regex::new(
        r#"integrity=["']([^"']+)["']"#
    ).unwrap();

    // Match crossorigin attribute
    static ref CROSSORIGIN_RE: Regex = Regex::new(
        r#"crossorigin=["']?([^"'\s>]+)"#
    ).unwrap();
}

/// Known CDN patterns for library detection
const CDN_PATTERNS: &[(&str, &str)] = &[
    // Major CDNs
    ("cdnjs.cloudflare.com", "cdnjs"),
    ("cdn.jsdelivr.net", "jsdelivr"),
    ("unpkg.com", "unpkg"),
    ("cdn.skypack.dev", "skypack"),
    ("esm.sh", "esm.sh"),

    // Specific libraries
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
    ("particles.js", "Particles.js"),
    ("particles.min.js", "Particles.js"),
    ("typed.js", "Typed.js"),
    ("typed.min.js", "Typed.js"),
    ("chart.js", "Chart.js"),
    ("chart.min.js", "Chart.js"),
    ("d3.js", "D3.js"),
    ("d3.min.js", "D3.js"),
    ("jquery", "jQuery"),
    ("bootstrap", "Bootstrap"),
    ("tailwind", "Tailwind CSS"),
    ("bulma", "Bulma"),
    ("materialize", "Materialize"),
    ("foundation", "Foundation"),
    ("semantic", "Semantic UI"),
    ("uikit", "UIkit"),
    ("alpine", "Alpine.js"),
    ("vue", "Vue.js"),
    ("react", "React"),
    ("angular", "Angular"),
    ("svelte", "Svelte"),
    ("lodash", "Lodash"),
    ("moment", "Moment.js"),
    ("dayjs", "Day.js"),
    ("axios", "Axios"),
    ("popper", "Popper.js"),
    ("tippy", "Tippy.js"),
    ("plyr", "Plyr"),
    ("video.js", "Video.js"),
    ("howler", "Howler.js"),
    ("tone.js", "Tone.js"),
    ("p5.js", "p5.js"),
    ("paper.js", "Paper.js"),
    ("fabric", "Fabric.js"),
    ("konva", "Konva"),
    ("pixi", "PixiJS"),
    ("phaser", "Phaser"),
    ("matter.js", "Matter.js"),
    ("cannon", "Cannon.js"),
    ("oimo", "Oimo.js"),
    ("highlight", "Highlight.js"),
    ("prism", "Prism.js"),
    ("codemirror", "CodeMirror"),
    ("monaco", "Monaco Editor"),
    ("ace", "Ace Editor"),
    ("quill", "Quill"),
    ("tinymce", "TinyMCE"),
    ("ckeditor", "CKEditor"),
    ("flatpickr", "Flatpickr"),
    ("choices", "Choices.js"),
    ("select2", "Select2"),
    ("tom-select", "Tom Select"),
    ("sortable", "SortableJS"),
    ("draggable", "Draggable"),
    ("interact", "interact.js"),
    ("hammer", "Hammer.js"),
    ("socket.io", "Socket.IO"),
    ("pusher", "Pusher"),
    ("firebase", "Firebase"),
    ("supabase", "Supabase"),
    ("stripe", "Stripe"),
    ("paypal", "PayPal"),
    ("mapbox", "Mapbox"),
    ("leaflet", "Leaflet"),
    ("openlayers", "OpenLayers"),
    ("cesium", "CesiumJS"),
];

/// Extract all external resources from HTML
pub struct ResourceExtractor;

impl ResourceExtractor {
    /// Extract all external JavaScript files
    pub fn extract_scripts(html: &str) -> Vec<DependencyLink> {
        let mut scripts = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        for cap in SCRIPT_RE.captures_iter(html) {
            let url = cap.get(1).map_or("", |m| m.as_str());

            // Skip inline data URLs
            if url.starts_with("data:") || url.is_empty() {
                continue;
            }

            // Skip duplicates
            if seen_urls.contains(url) {
                continue;
            }
            seen_urls.insert(url.to_string());

            // Skip local scripts (relative paths without CDN)
            if !url.starts_with("http") && !url.starts_with("//") {
                continue;
            }

            let full_match = cap.get(0).map_or("", |m| m.as_str());
            let integrity = INTEGRITY_RE.captures(full_match)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());
            let crossorigin = CROSSORIGIN_RE.captures(full_match)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());

            let name = Self::detect_library_name(url);
            let version = Self::extract_version(url);

            scripts.push(DependencyLink {
                name,
                url: Self::normalize_url(url),
                version,
                integrity,
                crossorigin,
            });
        }

        scripts
    }

    /// Extract all external stylesheets
    pub fn extract_stylesheets(html: &str) -> Vec<DependencyLink> {
        let mut stylesheets = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        // Try both regex patterns
        for re in [&*STYLESHEET_RE, &*STYLESHEET_RE2] {
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map_or("", |m| m.as_str());

                if url.starts_with("data:") || url.is_empty() {
                    continue;
                }

                if seen_urls.contains(url) {
                    continue;
                }
                seen_urls.insert(url.to_string());

                // Skip local stylesheets
                if !url.starts_with("http") && !url.starts_with("//") {
                    continue;
                }

                let full_match = cap.get(0).map_or("", |m| m.as_str());
                let integrity = INTEGRITY_RE.captures(full_match)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());
                let crossorigin = CROSSORIGIN_RE.captures(full_match)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());

                let name = Self::detect_library_name(url);
                let version = Self::extract_version(url);

                stylesheets.push(DependencyLink {
                    name,
                    url: Self::normalize_url(url),
                    version,
                    integrity,
                    crossorigin,
                });
            }
        }

        stylesheets
    }

    /// Extract Google Fonts
    pub fn extract_fonts(html: &str) -> Vec<FontImport> {
        let mut fonts = Vec::new();
        let mut seen_families = std::collections::HashSet::new();

        for cap in GOOGLE_FONTS_RE.captures_iter(html) {
            let family_param = cap.get(1).map_or("", |m| m.as_str());

            // Parse family name (format: "Family+Name:wght@400;700")
            let family = family_param
                .split(':')
                .next()
                .unwrap_or(family_param)
                .replace('+', " ");

            if seen_families.contains(&family) {
                continue;
            }
            seen_families.insert(family.clone());

            // Extract weights if present
            let weights: Vec<String> = if family_param.contains("wght@") {
                family_param
                    .split("wght@")
                    .nth(1)
                    .unwrap_or("400")
                    .split(';')
                    .map(|w| w.to_string())
                    .collect()
            } else {
                vec!["400".to_string(), "700".to_string()]
            };

            let url = format!(
                "https://fonts.googleapis.com/css2?family={}:wght@{}&display=swap",
                family.replace(' ', "+"),
                weights.join(";")
            );

            fonts.push(FontImport {
                family,
                weights,
                source: "google".to_string(),
                url: Some(url),
            });
        }

        fonts
    }

    /// Detect library name from URL
    fn detect_library_name(url: &str) -> String {
        let url_lower = url.to_lowercase();

        // Check known patterns
        for (pattern, name) in CDN_PATTERNS {
            if url_lower.contains(pattern) {
                return name.to_string();
            }
        }

        // Try to extract from filename
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

            // Capitalize first letter of each word
            return name
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(c) => c.to_uppercase().chain(chars).collect(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }

        "Unknown Library".to_string()
    }

    /// Extract version from URL
    fn extract_version(url: &str) -> Option<String> {
        // Common patterns: @1.2.3, /1.2.3/, -1.2.3, _1.2.3
        let version_patterns = [
            r"@(\d+\.\d+(?:\.\d+)?)",
            r"/(\d+\.\d+(?:\.\d+)?)/",
            r"-(\d+\.\d+(?:\.\d+)?)[./]",
            r"_(\d+\.\d+(?:\.\d+)?)[./]",
            r"v(\d+\.\d+(?:\.\d+)?)",
        ];

        for pattern in version_patterns {
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

    /// Normalize URL (add https: if protocol-relative)
    fn normalize_url(url: &str) -> String {
        if url.starts_with("//") {
            format!("https:{}", url)
        } else {
            url.to_string()
        }
    }
}

/// Merge extracted dependencies with detected ones, avoiding duplicates
pub fn merge_dependencies(
    detected: &mut Vec<DependencyLink>,
    extracted: Vec<DependencyLink>,
) {
    let existing_urls: std::collections::HashSet<_> =
        detected.iter().map(|d| d.url.to_lowercase()).collect();

    for dep in extracted {
        let url_lower = dep.url.to_lowercase();
        if !existing_urls.contains(&url_lower) {
            detected.push(dep);
        }
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
        assert!(scripts.iter().any(|s| s.name == "GSAP"));
        assert!(scripts.iter().any(|s| s.name == "Three.js"));
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(
            ResourceExtractor::extract_version("https://cdn.com/lib@1.2.3/file.js"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            ResourceExtractor::extract_version("https://cdn.com/lib/1.2.3/file.js"),
            Some("1.2.3".to_string())
        );
    }

    #[test]
    fn test_detect_library() {
        assert_eq!(
            ResourceExtractor::detect_library_name("https://cdn.com/gsap/gsap.min.js"),
            "GSAP"
        );
        assert_eq!(
            ResourceExtractor::detect_library_name("https://cdn.com/three.js"),
            "Three.js"
        );
    }
}
