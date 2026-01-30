//! Asset Registry
//!
//! Comprehensive asset management for extracted web resources:
//! - SVG deduplication and categorization
//! - Image cataloging
//! - Font registry with metadata
//! - TOML generation for asset imports

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Asset Registry - catalogs all extracted assets
#[derive(Debug, Clone, Default)]
pub struct AssetRegistry {
    /// SVG assets with deduplication
    pub svgs: HashMap<String, SvgAsset>,
    /// Image assets
    pub images: HashMap<String, ImageAsset>,
    /// Font assets
    pub fonts: HashMap<String, FontAsset>,
    /// Statistics
    pub stats: AssetStats,
}

#[derive(Debug, Clone, Default)]
pub struct AssetStats {
    pub total_svgs_found: usize,
    pub unique_svgs: usize,
    pub duplicate_svgs: usize,
    pub icons_found: usize,
    pub logos_found: usize,
    pub illustrations_found: usize,
    pub images_found: usize,
    pub fonts_found: usize,
}

#[derive(Debug, Clone)]
pub struct SvgAsset {
    /// Unique ID (content hash)
    pub id: String,
    /// Original filename or generated name
    pub filename: String,
    /// Category
    pub category: SvgCategory,
    /// SVG content
    pub content: String,
    /// Size (width x height if available)
    pub size: Option<(u32, u32)>,
    /// CSS classes found
    pub classes: Vec<String>,
    /// ViewBox
    pub viewbox: Option<String>,
    /// Local path after extraction
    pub local_path: Option<String>,
    /// Usage count in HTML
    pub usage_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvgCategory {
    Icon,       // Small, simple, used multiple times
    Logo,       // Brand-related, medium size
    Illustration, // Large, complex, decorative
    Background, // Used in CSS backgrounds
    Unknown,
}

impl SvgCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SvgCategory::Icon => "icon",
            SvgCategory::Logo => "logo",
            SvgCategory::Illustration => "illustration",
            SvgCategory::Background => "background",
            SvgCategory::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub format: ImageFormat,
    pub local_path: Option<String>,
    pub size_bytes: Option<usize>,
    pub context: ImageContext,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpg,
    Gif,
    Webp,
    Avif,
    Svg,
    Ico,
    Unknown,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => ImageFormat::Jpg,
            "gif" => ImageFormat::Gif,
            "webp" => ImageFormat::Webp,
            "avif" => ImageFormat::Avif,
            "svg" => ImageFormat::Svg,
            "ico" => ImageFormat::Ico,
            _ => ImageFormat::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Webp => "webp",
            ImageFormat::Avif => "avif",
            ImageFormat::Svg => "svg",
            ImageFormat::Ico => "ico",
            ImageFormat::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageContext {
    Content,    // <img> in HTML
    Background, // CSS background
    Icon,       // Favicon, app icon
    OpenGraph,  // OG/meta images
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FontAsset {
    pub id: String,
    pub family: String,
    pub weight: Option<String>,
    pub style: Option<String>,
    pub format: FontFormat,
    pub url: String,
    pub local_path: Option<String>,
    pub size_bytes: Option<usize>,
    pub is_variable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontFormat {
    Woff2,
    Woff,
    Ttf,
    Otf,
    Eot,
    Unknown,
}

impl FontFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "woff2" => FontFormat::Woff2,
            "woff" => FontFormat::Woff,
            "ttf" => FontFormat::Ttf,
            "otf" => FontFormat::Otf,
            "eot" => FontFormat::Eot,
            _ => FontFormat::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FontFormat::Woff2 => "woff2",
            FontFormat::Woff => "woff",
            FontFormat::Ttf => "ttf",
            FontFormat::Otf => "otf",
            FontFormat::Eot => "eot",
            FontFormat::Unknown => "unknown",
        }
    }
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract and catalog all SVGs from HTML
    pub fn extract_svgs(&mut self, html: &str) {
        let svg_pattern = Regex::new(r#"<svg[^>]*>[\s\S]*?</svg>"#).unwrap();
        let class_pattern = Regex::new(r#"class="([^"]+)""#).unwrap();
        let viewbox_pattern = Regex::new(r#"viewBox="([^"]+)""#).unwrap();
        let size_pattern = Regex::new(r#"(?:width|height)="(\d+)""#).unwrap();

        let mut seen_hashes: HashMap<u64, String> = HashMap::new();

        for svg_match in svg_pattern.find_iter(html) {
            let svg_content = svg_match.as_str();
            self.stats.total_svgs_found += 1;

            // Calculate content hash for deduplication
            let content_hash = self.hash_content(svg_content);

            // Check if we've seen this SVG before
            if let Some(existing_id) = seen_hashes.get(&content_hash) {
                // Increment usage count
                if let Some(svg) = self.svgs.get_mut(existing_id) {
                    svg.usage_count += 1;
                }
                self.stats.duplicate_svgs += 1;
                continue;
            }

            // Extract metadata
            let classes: Vec<String> = class_pattern
                .captures_iter(svg_content)
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .collect();

            let viewbox = viewbox_pattern
                .captures(svg_content)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

            let sizes: Vec<u32> = size_pattern
                .captures_iter(svg_content)
                .filter_map(|c| c.get(1).and_then(|m| m.as_str().parse().ok()))
                .collect();

            let size = if sizes.len() >= 2 {
                Some((sizes[0], sizes[1]))
            } else if sizes.len() == 1 {
                Some((sizes[0], sizes[0]))
            } else {
                // Try to parse from viewBox
                viewbox.as_ref().and_then(|vb| {
                    let parts: Vec<&str> = vb.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let w: u32 = parts[2].parse().ok()?;
                        let h: u32 = parts[3].parse().ok()?;
                        Some((w, h))
                    } else {
                        None
                    }
                })
            };

            // Categorize SVG
            let category = self.categorize_svg(&classes, size, svg_content.len());

            // Generate ID and filename
            let id = format!("svg-{:016x}", content_hash);
            let filename = self.generate_svg_filename(&category, &classes, &id);

            seen_hashes.insert(content_hash, id.clone());

            // Update stats
            self.stats.unique_svgs += 1;
            match category {
                SvgCategory::Icon => self.stats.icons_found += 1,
                SvgCategory::Logo => self.stats.logos_found += 1,
                SvgCategory::Illustration => self.stats.illustrations_found += 1,
                _ => {}
            }

            self.svgs.insert(id.clone(), SvgAsset {
                id,
                filename,
                category,
                content: svg_content.to_string(),
                size,
                classes,
                viewbox,
                local_path: None,
                usage_count: 1,
            });
        }
    }

    /// Register an image asset
    pub fn register_image(&mut self, url: &str, local_path: Option<&str>, context: ImageContext, size_bytes: Option<usize>) {
        let id = format!("img-{:016x}", self.hash_content(url));

        let filename = url.split('/').last().unwrap_or("image.png");
        let ext = filename.split('.').last().unwrap_or("png");
        let format = ImageFormat::from_extension(ext);

        self.images.insert(id.clone(), ImageAsset {
            id,
            filename: filename.to_string(),
            url: url.to_string(),
            format,
            local_path: local_path.map(|s| s.to_string()),
            size_bytes,
            context,
        });

        self.stats.images_found += 1;
    }

    /// Register a font asset
    pub fn register_font(
        &mut self,
        family: &str,
        weight: Option<&str>,
        style: Option<&str>,
        url: &str,
        local_path: Option<&str>,
        size_bytes: Option<usize>,
    ) {
        let id = format!("font-{:016x}", self.hash_content(url));

        let filename = url.split('/').last().unwrap_or("font.woff2");
        let ext = filename.split('.').last().unwrap_or("woff2");
        let format = FontFormat::from_extension(ext);

        let is_variable = family.to_lowercase().contains("var") ||
                          url.to_lowercase().contains("variable");

        self.fonts.insert(id.clone(), FontAsset {
            id,
            family: family.to_string(),
            weight: weight.map(|s| s.to_string()),
            style: style.map(|s| s.to_string()),
            format,
            url: url.to_string(),
            local_path: local_path.map(|s| s.to_string()),
            size_bytes,
            is_variable,
        });

        self.stats.fonts_found += 1;
    }

    /// Generate project.toml content with asset registry
    pub fn generate_toml(
        &self,
        url: &str,
        title: &str,
        mode: &str,
        libraries: &[(String, String)], // (name, category)
        css_stats: &CssStats,
    ) -> String {
        let mut toml = String::new();

        // Meta section
        toml.push_str("[meta]\n");
        toml.push_str(&format!("name = \"{}\"\n", self.sanitize_name(title)));
        toml.push_str("version = \"1.0.0\"\n");
        toml.push_str("generator = \"crawlwe\"\n");
        toml.push_str(&format!("mode = \"{}\"\n", mode));
        toml.push_str(&format!("captured_at = \"{}\"\n", chrono::Utc::now().to_rfc3339()));
        toml.push('\n');

        // Source section
        toml.push_str("[source]\n");
        toml.push_str(&format!("url = \"{}\"\n", url));
        toml.push_str(&format!("title = \"{}\"\n", title.replace('"', "\\\"")));
        toml.push('\n');

        // Technologies section
        toml.push_str("[technologies]\n");
        for (name, category) in libraries {
            match category.as_str() {
                "css" => toml.push_str(&format!("css_framework = \"{}\"\n", name)),
                "ui" => toml.push_str(&format!("ui_framework = \"{}\"\n", name)),
                "animation" => toml.push_str(&format!("animation_lib = \"{}\"\n", name)),
                _ => {}
            }
        }
        toml.push('\n');

        // Assets section
        toml.push_str("[assets]\n");
        toml.push_str(&format!("svgs = {}\n", self.stats.unique_svgs));
        toml.push_str(&format!("images = {}\n", self.stats.images_found));
        toml.push_str(&format!("fonts = {}\n", self.stats.fonts_found));
        toml.push('\n');

        // SVGs by category
        if !self.svgs.is_empty() {
            toml.push_str("[assets.svgs]\n");

            // Group by category
            let mut icons: Vec<&SvgAsset> = Vec::new();
            let mut logos: Vec<&SvgAsset> = Vec::new();
            let mut illustrations: Vec<&SvgAsset> = Vec::new();

            for svg in self.svgs.values() {
                match svg.category {
                    SvgCategory::Icon => icons.push(svg),
                    SvgCategory::Logo => logos.push(svg),
                    SvgCategory::Illustration => illustrations.push(svg),
                    _ => {}
                }
            }

            if !icons.is_empty() {
                toml.push_str(&format!("icons = {} # Small reusable icons\n", icons.len()));
            }
            if !logos.is_empty() {
                toml.push_str(&format!("logos = {} # Brand logos\n", logos.len()));
            }
            if !illustrations.is_empty() {
                toml.push_str(&format!("illustrations = {} # Large decorative SVGs\n", illustrations.len()));
            }
            toml.push('\n');
        }

        // Fonts section with local paths
        if !self.fonts.is_empty() {
            toml.push_str("[assets.fonts]\n");

            // Group by family
            let mut families: HashMap<String, Vec<&FontAsset>> = HashMap::new();
            for font in self.fonts.values() {
                families.entry(font.family.clone()).or_default().push(font);
            }

            for (family, fonts) in &families {
                let weights: Vec<String> = fonts
                    .iter()
                    .filter_map(|f| f.weight.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();

                let local_paths: Vec<String> = fonts
                    .iter()
                    .filter_map(|f| f.local_path.clone())
                    .collect();

                if !local_paths.is_empty() {
                    toml.push_str(&format!(
                        "\"{}\" = {{ weights = {:?}, files = {:?} }}\n",
                        family, weights, local_paths
                    ));
                }
            }
            toml.push('\n');
        }

        // Images section
        if !self.images.is_empty() {
            toml.push_str("[assets.images]\n");

            // Group by context
            let content_imgs: Vec<_> = self.images.values()
                .filter(|i| i.context == ImageContext::Content)
                .collect();
            let bg_imgs: Vec<_> = self.images.values()
                .filter(|i| i.context == ImageContext::Background)
                .collect();

            if !content_imgs.is_empty() {
                toml.push_str(&format!("content = {} # Images in HTML\n", content_imgs.len()));
            }
            if !bg_imgs.is_empty() {
                toml.push_str(&format!("backgrounds = {} # CSS background images\n", bg_imgs.len()));
            }
            toml.push('\n');
        }

        // CSS stats
        toml.push_str("[css]\n");
        toml.push_str(&format!("variables = {}\n", css_stats.variables));
        toml.push_str(&format!("keyframes = {}\n", css_stats.keyframes));
        toml.push_str(&format!("colors = {}\n", css_stats.colors));
        toml.push('\n');

        // Import paths for developer use
        toml.push_str("# Import paths for use in your project:\n");
        toml.push_str("# import styles from './styles.css'\n");
        toml.push_str("# import { icons } from './assets/svgs/icons/'\n");
        toml.push_str("# import fonts from './assets/fonts/'\n");

        toml
    }

    fn hash_content(&self, content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Normalize whitespace for dedup
        let normalized: String = content.split_whitespace().collect::<Vec<_>>().join(" ");
        normalized.hash(&mut hasher);
        hasher.finish()
    }

    fn categorize_svg(&self, classes: &[String], size: Option<(u32, u32)>, content_len: usize) -> SvgCategory {
        // Check classes for hints
        let class_str = classes.join(" ").to_lowercase();

        if class_str.contains("icon") || class_str.contains("plus") || class_str.contains("arrow") ||
           class_str.contains("chevron") || class_str.contains("check") {
            return SvgCategory::Icon;
        }

        if class_str.contains("logo") || class_str.contains("brand") {
            return SvgCategory::Logo;
        }

        if class_str.contains("illustration") || class_str.contains("hero") || class_str.contains("art") {
            return SvgCategory::Illustration;
        }

        // Check size
        if let Some((w, h)) = size {
            if w <= 24 && h <= 24 {
                return SvgCategory::Icon;
            }
            if w >= 100 || h >= 100 {
                return SvgCategory::Illustration;
            }
            if w >= 40 && w <= 100 {
                return SvgCategory::Logo;
            }
        }

        // Check content length
        if content_len < 500 {
            return SvgCategory::Icon;
        }
        if content_len > 2000 {
            return SvgCategory::Illustration;
        }

        SvgCategory::Unknown
    }

    fn generate_svg_filename(&self, category: &SvgCategory, classes: &[String], id: &str) -> String {
        // Try to extract a meaningful name from classes
        if let Some(class) = classes.first() {
            let name = class
                .split('_')
                .next()
                .unwrap_or(class)
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();

            if !name.is_empty() && name.len() < 30 {
                return format!("{}-{}.svg", category.as_str(), name);
            }
        }

        // Fallback to hash-based name
        format!("{}-{}.svg", category.as_str(), &id[4..12])
    }

    fn sanitize_name(&self, name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    /// Get SVGs that should be extracted to files (large/important ones)
    pub fn get_extractable_svgs(&self) -> Vec<&SvgAsset> {
        self.svgs
            .values()
            .filter(|svg| {
                // Extract if:
                // - It's a logo
                // - It's an illustration
                // - It's used multiple times (deduplicate by extracting once)
                // - It's large (> 500 bytes)
                svg.category == SvgCategory::Logo ||
                svg.category == SvgCategory::Illustration ||
                svg.usage_count > 2 ||
                svg.content.len() > 500
            })
            .collect()
    }

    /// Replace inline SVGs with <img> references and return modified HTML
    pub fn replace_svgs_with_refs(&self, html: &str, base_path: &str) -> String {
        let mut result = html.to_string();

        for svg in self.get_extractable_svgs() {
            if let Some(local_path) = &svg.local_path {
                let img_tag = format!(
                    r#"<img src="{}/{}" alt="{}" class="{}">"#,
                    base_path,
                    local_path,
                    svg.category.as_str(),
                    svg.classes.join(" ")
                );
                result = result.replace(&svg.content, &img_tag);
            }
        }

        result
    }
}

#[derive(Debug, Clone, Default)]
pub struct CssStats {
    pub variables: usize,
    pub keyframes: usize,
    pub colors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_extraction() {
        let mut registry = AssetRegistry::new();

        let html = r#"
            <div>
                <svg class="Icon_plus__abc" width="8" height="8"><path d="M0 0"/></svg>
                <svg class="Icon_plus__abc" width="8" height="8"><path d="M0 0"/></svg>
                <svg class="Logo_brand__xyz" width="100" height="50"><path d="M1 1"/></svg>
            </div>
        "#;

        registry.extract_svgs(html);

        assert_eq!(registry.stats.total_svgs_found, 3);
        assert_eq!(registry.stats.unique_svgs, 2); // Two unique SVGs
        assert_eq!(registry.stats.duplicate_svgs, 1); // One duplicate
    }

    #[test]
    fn test_svg_categorization() {
        let mut registry = AssetRegistry::new();

        let html = r#"
            <svg class="PlusIcon_icon__abc" width="8" height="8"><path/></svg>
            <svg class="Logo_brand__xyz" width="80" height="40"><path/></svg>
            <svg class="Hero_illustration__123" width="500" height="300"><path/><path/><path/></svg>
        "#;

        registry.extract_svgs(html);

        let icons: Vec<_> = registry.svgs.values()
            .filter(|s| s.category == SvgCategory::Icon)
            .collect();
        let logos: Vec<_> = registry.svgs.values()
            .filter(|s| s.category == SvgCategory::Logo)
            .collect();

        assert!(!icons.is_empty(), "Should find icons");
        assert!(!logos.is_empty(), "Should find logos");
    }

    #[test]
    fn test_font_registration() {
        let mut registry = AssetRegistry::new();

        registry.register_font(
            "Inter",
            Some("400"),
            Some("normal"),
            "https://fonts.example.com/inter.woff2",
            Some("assets/fonts/inter.woff2"),
            Some(25000),
        );

        assert_eq!(registry.stats.fonts_found, 1);
        assert!(registry.fonts.values().next().unwrap().family == "Inter");
    }
}
