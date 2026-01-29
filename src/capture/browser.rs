//! Chrome DevTools Protocol client for capturing web pages
//!
//! Advanced CDP implementation with:
//! - Smart scrolling for lazy loading
//! - Full CSS extraction (computed + external + inline)
//! - Network interception for resource capture
//! - Intelligent wait strategies for JS-heavy pages
//! - Library detection from window objects

use crate::models::*;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    LaunchError(String),
    #[error("Navigation failed: {0}")]
    NavigationError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("CDP error: {0}")]
    CdpError(String),
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Detected library information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLibrary {
    pub name: String,
    pub version: Option<String>,
    pub category: LibraryCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LibraryCategory {
    Animation,
    Framework,
    Utility,
    ThreeD,
    Styling,
    State,
    Build,
    Other,
}

/// Captured CSS data
#[derive(Debug, Clone, Default)]
pub struct CapturedCss {
    /// All stylesheets (URL -> content)
    pub stylesheets: HashMap<String, String>,
    /// Inline styles from <style> tags
    pub inline_styles: Vec<String>,
    /// Computed styles for elements (selector -> properties)
    pub computed: HashMap<String, HashMap<String, String>>,
    /// CSS variables from :root
    pub variables: HashMap<String, String>,
    /// @font-face rules
    pub font_faces: Vec<String>,
    /// @keyframes animations
    pub keyframes: HashMap<String, String>,
}

/// Network request info
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: String,
    pub resource_type: String,
    pub status: u32,
    pub mime_type: String,
}

/// Advanced browser controller for capturing pages
pub struct BrowserController {
    browser: Browser,
    config: CaptureConfig,
}

impl BrowserController {
    /// Launch a new browser instance with advanced configuration
    pub async fn launch(config: CaptureConfig) -> Result<Self, BrowserError> {
        let viewport = config.viewports.first().cloned().unwrap_or_else(Viewport::desktop);

        let mut builder = BrowserConfig::builder()
            .with_head() // headless mode
            .viewport(chromiumoxide::handler::viewport::Viewport {
                width: viewport.width,
                height: viewport.height,
                device_scale_factor: Some(viewport.device_scale_factor),
                emulating_mobile: viewport.is_mobile,
                ..Default::default()
            })
            .arg("--disable-web-security")
            .arg("--disable-features=IsolateOrigins,site-per-process")
            .arg("--disable-blink-features=AutomationControlled");

        // Add user agent if specified
        if let Some(ref ua) = config.user_agent {
            builder = builder.arg(format!("--user-agent={}", ua));
        }

        let browser_config = builder
            .build()
            .map_err(|e| BrowserError::LaunchError(e.to_string()))?;

        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| BrowserError::LaunchError(e.to_string()))?;

        // Spawn handler in background
        tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        Ok(Self { browser, config })
    }

    /// Capture a page with full fidelity
    pub async fn capture_page(&self, url: &str) -> Result<CapturedPage, BrowserError> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::NavigationError(e.to_string()))?;

        // Wait for initial navigation
        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::NavigationError(e.to_string()))?;

        // Wait for network to be idle
        self.wait_for_network_idle(&page, 2000).await?;

        // Scroll to trigger lazy loading
        self.scroll_page(&page).await?;

        // Wait again after scroll
        self.wait_for_network_idle(&page, 1000).await?;

        let viewport = self.config.viewports.first().cloned().unwrap_or_else(Viewport::desktop);
        let mut captured = CapturedPage::new(url.to_string(), viewport);

        // Get page title
        captured.title = self.get_title(&page).await.ok();

        // Get raw HTML (before cleaning)
        let raw_html = self.get_raw_html(&page).await?;

        // Extract all CSS
        let css_data = self.extract_all_css(&page, &raw_html).await?;
        captured.css = self.compile_css(&css_data);

        // Get clean HTML
        captured.html = self.get_clean_html(&page).await?;

        // Detect libraries
        let libraries = self.detect_libraries(&page).await?;

        // Take screenshot if configured
        if self.config.take_screenshots {
            captured.screenshot = self.take_full_screenshot(&page).await.ok();
        }

        // Store detected libraries in metadata
        // (We could add this to CapturedPage struct later)
        eprintln!("Detected {} libraries", libraries.len());
        for lib in &libraries {
            eprintln!("  - {} {:?}", lib.name, lib.version);
        }

        Ok(captured)
    }

    /// Wait for network to be idle (no requests for given ms)
    async fn wait_for_network_idle(&self, page: &Page, idle_time_ms: u64) -> Result<(), BrowserError> {
        let script = format!(
            r#"
            new Promise((resolve) => {{
                let timeout;
                const checkIdle = () => {{
                    clearTimeout(timeout);
                    timeout = setTimeout(resolve, {});
                }};

                // Check if document is ready
                if (document.readyState === 'complete') {{
                    checkIdle();
                }} else {{
                    window.addEventListener('load', checkIdle);
                }}

                // Also listen for any pending requests
                const observer = new PerformanceObserver((list) => {{
                    checkIdle();
                }});
                observer.observe({{ entryTypes: ['resource'] }});

                // Fallback timeout
                setTimeout(resolve, 10000);
            }})
            "#,
            idle_time_ms
        );

        page.evaluate(script)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?;

        Ok(())
    }

    /// Scroll page to trigger lazy loading
    async fn scroll_page(&self, page: &Page) -> Result<(), BrowserError> {
        let scroll_script = r#"
            async function scrollPage() {
                const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

                // Get total scroll height
                const totalHeight = Math.max(
                    document.body.scrollHeight,
                    document.documentElement.scrollHeight
                );
                const viewportHeight = window.innerHeight;

                // Scroll in steps
                const steps = Math.ceil(totalHeight / viewportHeight);
                const maxSteps = Math.min(steps, 20); // Limit to 20 scrolls

                for (let i = 0; i <= maxSteps; i++) {
                    const scrollTo = Math.min(i * viewportHeight, totalHeight);
                    window.scrollTo({ top: scrollTo, behavior: 'instant' });
                    await delay(100); // Wait for lazy content to load
                }

                // Scroll back to top
                window.scrollTo({ top: 0, behavior: 'instant' });
                await delay(200);

                return { totalHeight, steps: maxSteps };
            }
            scrollPage()
        "#;

        page.evaluate(scroll_script)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?;

        Ok(())
    }

    /// Get page title
    async fn get_title(&self, page: &Page) -> Result<String, BrowserError> {
        page.evaluate("document.title")
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))
    }

    /// Get raw HTML content
    async fn get_raw_html(&self, page: &Page) -> Result<String, BrowserError> {
        page.evaluate("document.documentElement.outerHTML")
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))
    }

    /// Extract all CSS from the page
    async fn extract_all_css(&self, page: &Page, _raw_html: &str) -> Result<CapturedCss, BrowserError> {
        let css_script = r#"
            (function() {
                const result = {
                    stylesheets: {},
                    inlineStyles: [],
                    variables: {},
                    fontFaces: [],
                    keyframes: {}
                };

                // Get all stylesheets
                for (const sheet of document.styleSheets) {
                    try {
                        const href = sheet.href || `inline-${result.inlineStyles.length}`;
                        let cssText = '';

                        for (const rule of sheet.cssRules) {
                            cssText += rule.cssText + '\n';

                            // Extract CSS variables from :root
                            if (rule.selectorText === ':root' && rule.style) {
                                for (let i = 0; i < rule.style.length; i++) {
                                    const prop = rule.style[i];
                                    if (prop.startsWith('--')) {
                                        result.variables[prop] = rule.style.getPropertyValue(prop);
                                    }
                                }
                            }

                            // Extract @font-face
                            if (rule.type === CSSRule.FONT_FACE_RULE) {
                                result.fontFaces.push(rule.cssText);
                            }

                            // Extract @keyframes
                            if (rule.type === CSSRule.KEYFRAMES_RULE) {
                                result.keyframes[rule.name] = rule.cssText;
                            }
                        }

                        if (sheet.href) {
                            result.stylesheets[href] = cssText;
                        } else {
                            result.inlineStyles.push(cssText);
                        }
                    } catch (e) {
                        // CORS - can't access stylesheet
                        if (sheet.href) {
                            result.stylesheets[sheet.href] = '/* CORS: could not access */';
                        }
                    }
                }

                return result;
            })()
        "#;

        let css_result: serde_json::Value = page
            .evaluate(css_script)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))?;

        let mut css_data = CapturedCss::default();

        // Parse stylesheets
        if let Some(sheets) = css_result.get("stylesheets").and_then(|v| v.as_object()) {
            for (url, content) in sheets {
                if let Some(css) = content.as_str() {
                    css_data.stylesheets.insert(url.clone(), css.to_string());
                }
            }
        }

        // Parse inline styles
        if let Some(inline) = css_result.get("inlineStyles").and_then(|v| v.as_array()) {
            for style in inline {
                if let Some(css) = style.as_str() {
                    css_data.inline_styles.push(css.to_string());
                }
            }
        }

        // Parse variables
        if let Some(vars) = css_result.get("variables").and_then(|v| v.as_object()) {
            for (name, value) in vars {
                if let Some(v) = value.as_str() {
                    css_data.variables.insert(name.clone(), v.to_string());
                }
            }
        }

        // Parse font faces
        if let Some(fonts) = css_result.get("fontFaces").and_then(|v| v.as_array()) {
            for font in fonts {
                if let Some(css) = font.as_str() {
                    css_data.font_faces.push(css.to_string());
                }
            }
        }

        // Parse keyframes
        if let Some(kf) = css_result.get("keyframes").and_then(|v| v.as_object()) {
            for (name, content) in kf {
                if let Some(css) = content.as_str() {
                    css_data.keyframes.insert(name.clone(), css.to_string());
                }
            }
        }

        Ok(css_data)
    }

    /// Compile all CSS into a single stylesheet
    fn compile_css(&self, css_data: &CapturedCss) -> String {
        let mut output = String::new();

        // Header
        output.push_str("/* CrawlWe Extracted Styles */\n\n");

        // CSS Variables
        if !css_data.variables.is_empty() {
            output.push_str(":root {\n");
            for (name, value) in &css_data.variables {
                output.push_str(&format!("  {}: {};\n", name, value));
            }
            output.push_str("}\n\n");
        }

        // Font faces
        for font_face in &css_data.font_faces {
            output.push_str(font_face);
            output.push_str("\n\n");
        }

        // Keyframes
        for keyframe in css_data.keyframes.values() {
            output.push_str(keyframe);
            output.push_str("\n\n");
        }

        // External stylesheets
        for (url, css) in &css_data.stylesheets {
            if !css.contains("CORS") {
                output.push_str(&format!("/* Source: {} */\n", url));
                output.push_str(css);
                output.push_str("\n\n");
            }
        }

        // Inline styles
        for (i, css) in css_data.inline_styles.iter().enumerate() {
            output.push_str(&format!("/* Inline style #{} */\n", i + 1));
            output.push_str(css);
            output.push_str("\n\n");
        }

        output
    }

    /// Get clean HTML from page (scripts removed, attributes cleaned)
    async fn get_clean_html(&self, page: &Page) -> Result<String, BrowserError> {
        let clean_script = r#"
            (function() {
                const clone = document.documentElement.cloneNode(true);

                // Remove script and style elements
                clone.querySelectorAll('script, noscript').forEach(el => el.remove());

                // Remove style elements (we have CSS separately)
                clone.querySelectorAll('style').forEach(el => el.remove());

                // Remove stylesheet links (we have CSS separately)
                clone.querySelectorAll('link[rel="stylesheet"]').forEach(el => el.remove());

                // Clean attributes
                clone.querySelectorAll('*').forEach(el => {
                    const attrsToRemove = [];
                    for (const attr of el.attributes) {
                        // Remove data attributes
                        if (attr.name.startsWith('data-')) {
                            attrsToRemove.push(attr.name);
                        }
                        // Remove framework attributes
                        if (attr.name.startsWith('ng-') ||
                            attr.name.startsWith('v-') ||
                            attr.name.startsWith('x-') ||
                            attr.name.startsWith('@') ||
                            attr.name.startsWith(':') ||
                            attr.name.startsWith('_')) {
                            attrsToRemove.push(attr.name);
                        }
                        // Remove event handlers
                        if (attr.name.startsWith('on')) {
                            attrsToRemove.push(attr.name);
                        }
                    }
                    attrsToRemove.forEach(attr => el.removeAttribute(attr));
                });

                // Add our stylesheet link
                const head = clone.querySelector('head');
                if (head) {
                    const link = document.createElement('link');
                    link.rel = 'stylesheet';
                    link.href = 'styles.css';
                    head.appendChild(link);
                }

                return clone.outerHTML;
            })()
        "#;

        page.evaluate(clean_script)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))
    }

    /// Detect JavaScript libraries from window object
    async fn detect_libraries(&self, page: &Page) -> Result<Vec<DetectedLibrary>, BrowserError> {
        let detect_script = r#"
            (function() {
                const libs = [];

                // GSAP
                if (window.gsap || window.TweenMax || window.TweenLite) {
                    libs.push({
                        name: 'GSAP',
                        version: window.gsap?.version || null,
                        category: 'Animation'
                    });
                }

                // Three.js
                if (window.THREE) {
                    libs.push({
                        name: 'Three.js',
                        version: window.THREE.REVISION || null,
                        category: 'ThreeD'
                    });
                }

                // React
                if (window.React || document.querySelector('[data-reactroot]') ||
                    document.querySelector('[data-react-helmet]')) {
                    libs.push({
                        name: 'React',
                        version: window.React?.version || null,
                        category: 'Framework'
                    });
                }

                // Vue
                if (window.Vue || window.__VUE__) {
                    libs.push({
                        name: 'Vue.js',
                        version: window.Vue?.version || null,
                        category: 'Framework'
                    });
                }

                // Angular
                if (window.ng || document.querySelector('[ng-app]') ||
                    document.querySelector('[data-ng-app]')) {
                    libs.push({
                        name: 'Angular',
                        version: window.ng?.version?.full || null,
                        category: 'Framework'
                    });
                }

                // jQuery
                if (window.jQuery || window.$?.fn?.jquery) {
                    libs.push({
                        name: 'jQuery',
                        version: window.jQuery?.fn?.jquery || window.$?.fn?.jquery || null,
                        category: 'Utility'
                    });
                }

                // Lodash
                if (window._ && window._.VERSION) {
                    libs.push({
                        name: 'Lodash',
                        version: window._.VERSION,
                        category: 'Utility'
                    });
                }

                // Lottie
                if (window.lottie || window.bodymovin) {
                    libs.push({
                        name: 'Lottie',
                        version: null,
                        category: 'Animation'
                    });
                }

                // Anime.js
                if (window.anime) {
                    libs.push({
                        name: 'Anime.js',
                        version: null,
                        category: 'Animation'
                    });
                }

                // ScrollMagic
                if (window.ScrollMagic) {
                    libs.push({
                        name: 'ScrollMagic',
                        version: window.ScrollMagic?.version || null,
                        category: 'Animation'
                    });
                }

                // Locomotive Scroll
                if (window.LocomotiveScroll) {
                    libs.push({
                        name: 'Locomotive Scroll',
                        version: null,
                        category: 'Animation'
                    });
                }

                // Swiper
                if (window.Swiper) {
                    libs.push({
                        name: 'Swiper',
                        version: null,
                        category: 'Utility'
                    });
                }

                // Barba.js
                if (window.barba) {
                    libs.push({
                        name: 'Barba.js',
                        version: window.barba?.version || null,
                        category: 'Animation'
                    });
                }

                // Splitting.js
                if (window.Splitting) {
                    libs.push({
                        name: 'Splitting',
                        version: null,
                        category: 'Animation'
                    });
                }

                // Motion One
                if (window.Motion) {
                    libs.push({
                        name: 'Motion One',
                        version: null,
                        category: 'Animation'
                    });
                }

                // Framer Motion (check for motion elements)
                if (document.querySelector('[data-framer-component-type]')) {
                    libs.push({
                        name: 'Framer Motion',
                        version: null,
                        category: 'Animation'
                    });
                }

                // Tailwind (check for utility classes)
                const hasTailwind = document.querySelector('[class*="flex"]') &&
                    document.querySelector('[class*="text-"]') &&
                    document.querySelector('[class*="bg-"]');
                if (hasTailwind) {
                    libs.push({
                        name: 'Tailwind CSS',
                        version: null,
                        category: 'Styling'
                    });
                }

                // Next.js
                if (window.__NEXT_DATA__ || document.querySelector('#__next')) {
                    libs.push({
                        name: 'Next.js',
                        version: window.__NEXT_DATA__?.buildId?.substring(0, 8) || null,
                        category: 'Framework'
                    });
                }

                // Nuxt.js
                if (window.__NUXT__ || document.querySelector('#__nuxt')) {
                    libs.push({
                        name: 'Nuxt.js',
                        version: null,
                        category: 'Framework'
                    });
                }

                // Svelte
                if (document.querySelector('[class*="svelte-"]')) {
                    libs.push({
                        name: 'Svelte',
                        version: null,
                        category: 'Framework'
                    });
                }

                // PIXI.js
                if (window.PIXI) {
                    libs.push({
                        name: 'PixiJS',
                        version: window.PIXI.VERSION || null,
                        category: 'ThreeD'
                    });
                }

                // ScrollTrigger (GSAP plugin)
                if (window.ScrollTrigger) {
                    libs.push({
                        name: 'ScrollTrigger',
                        version: null,
                        category: 'Animation'
                    });
                }

                // AOS (Animate On Scroll)
                if (window.AOS) {
                    libs.push({
                        name: 'AOS',
                        version: null,
                        category: 'Animation'
                    });
                }

                // Typed.js
                if (window.Typed) {
                    libs.push({
                        name: 'Typed.js',
                        version: null,
                        category: 'Animation'
                    });
                }

                return libs;
            })()
        "#;

        let libs_result: Vec<serde_json::Value> = page
            .evaluate(detect_script)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))?;

        let mut libraries = Vec::new();
        for lib in libs_result {
            let name = lib.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let version = lib.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
            let category_str = lib.get("category").and_then(|v| v.as_str()).unwrap_or("Other");

            let category = match category_str {
                "Animation" => LibraryCategory::Animation,
                "Framework" => LibraryCategory::Framework,
                "Utility" => LibraryCategory::Utility,
                "ThreeD" => LibraryCategory::ThreeD,
                "Styling" => LibraryCategory::Styling,
                "State" => LibraryCategory::State,
                "Build" => LibraryCategory::Build,
                _ => LibraryCategory::Other,
            };

            libraries.push(DetectedLibrary { name, version, category });
        }

        Ok(libraries)
    }

    /// Take a full page screenshot
    async fn take_full_screenshot(&self, page: &Page) -> Result<String, BrowserError> {
        // First, get the full page dimensions
        let dimensions: serde_json::Value = page
            .evaluate(r#"
                ({
                    width: Math.max(
                        document.body.scrollWidth,
                        document.documentElement.scrollWidth
                    ),
                    height: Math.max(
                        document.body.scrollHeight,
                        document.documentElement.scrollHeight
                    )
                })
            "#)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))?;

        let _width = dimensions.get("width").and_then(|v| v.as_u64()).unwrap_or(1440) as u32;
        let _height = dimensions.get("height").and_then(|v| v.as_u64()).unwrap_or(900) as u32;

        // Take full page screenshot
        let screenshot = page
            .execute(
                CaptureScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .capture_beyond_viewport(true)
                    .build(),
            )
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        // Convert to base64
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(&screenshot.data))
    }

    /// Get computed styles for a specific element
    pub async fn get_computed_styles(
        &self,
        page: &Page,
        selector: &str,
    ) -> Result<HashMap<String, String>, BrowserError> {
        let script = format!(
            r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return null;

                const styles = window.getComputedStyle(el);
                const result = {{}};

                for (let i = 0; i < styles.length; i++) {{
                    const prop = styles[i];
                    result[prop] = styles.getPropertyValue(prop);
                }}

                return result;
            }})()
            "#,
            selector.replace('\'', "\\'").replace('"', "\\\"")
        );

        let result: Option<HashMap<String, String>> = page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::JsError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::JsError(e.to_string()))?;

        result.ok_or_else(|| BrowserError::ElementNotFound(selector.to_string()))
    }

    /// Close the browser
    pub async fn close(mut self) -> Result<(), BrowserError> {
        self.browser
            .close()
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;
        Ok(())
    }
}

/// Calculate CSS specificity for a selector
pub fn calculate_specificity(selector: &str) -> Specificity {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;

    for part in selector.split_whitespace() {
        for segment in part.split(|c| c == '>' || c == '+' || c == '~') {
            let segment = segment.trim();

            ids += segment.matches('#').count() as u32;
            classes += segment.matches('.').count() as u32;
            classes += segment.matches('[').count() as u32;
            classes += segment
                .matches(':')
                .count()
                .saturating_sub(segment.matches("::").count()) as u32;

            if !segment.is_empty()
                && !segment.starts_with('#')
                && !segment.starts_with('.')
                && !segment.starts_with('[')
                && !segment.starts_with(':')
            {
                elements += 1;
            }
            elements += segment.matches("::").count() as u32;
        }
    }

    Specificity { ids, classes, elements }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity() {
        assert_eq!(
            calculate_specificity("div"),
            Specificity { ids: 0, classes: 0, elements: 1 }
        );
        assert_eq!(
            calculate_specificity(".class"),
            Specificity { ids: 0, classes: 1, elements: 0 }
        );
        assert_eq!(
            calculate_specificity("#id"),
            Specificity { ids: 1, classes: 0, elements: 0 }
        );
        assert_eq!(
            calculate_specificity("#id .class div"),
            Specificity { ids: 1, classes: 1, elements: 1 }
        );
        assert_eq!(
            calculate_specificity("div > p + span"),
            Specificity { ids: 0, classes: 0, elements: 3 }
        );
    }
}
