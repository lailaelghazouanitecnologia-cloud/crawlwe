//! Chrome DevTools Protocol client for capturing web pages
//!
//! Uses chromiumoxide to control Chrome/Chromium headless browser
//! and extract computed styles, DOM structure, and CSS rules.
//!
//! Note: This is a simplified implementation. Full browser automation
//! is handled by the Python layer using Playwright.

use crate::models::*;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams;
use chromiumoxide::page::Page;
use futures::StreamExt;
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
}

/// Browser controller for capturing pages
pub struct BrowserController {
    browser: Browser,
    config: CaptureConfig,
}

impl BrowserController {
    /// Launch a new browser instance
    pub async fn launch(config: CaptureConfig) -> Result<Self, BrowserError> {
        let browser_config = BrowserConfig::builder()
            .with_head() // headless mode
            .viewport(chromiumoxide::handler::viewport::Viewport {
                width: config.viewports.first().map(|v| v.width).unwrap_or(1440),
                height: config.viewports.first().map(|v| v.height).unwrap_or(900),
                device_scale_factor: Some(1.0),
                ..Default::default()
            })
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

    /// Capture a single page (simplified)
    pub async fn capture_page(&self, url: &str) -> Result<CapturedPage, BrowserError> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::NavigationError(e.to_string()))?;

        // Wait for page to load
        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::NavigationError(e.to_string()))?;

        let viewport = self.config.viewports.first().cloned().unwrap_or(Viewport::desktop());
        let mut captured = CapturedPage::new(url.to_string(), viewport);

        // Get page title
        captured.title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.into_value().ok());

        // Get HTML
        captured.html = self.get_clean_html(&page).await.unwrap_or_default();

        // Take screenshot if configured
        if self.config.take_screenshots {
            captured.screenshot = self.take_screenshot(&page).await.ok();
        }

        Ok(captured)
    }

    /// Get clean HTML from page
    async fn get_clean_html(&self, page: &Page) -> Result<String, BrowserError> {
        let html: String = page
            .evaluate(
                r#"
                (function() {
                    const clone = document.documentElement.cloneNode(true);
                    clone.querySelectorAll('script, noscript, style, link[rel="stylesheet"]')
                        .forEach(el => el.remove());
                    clone.querySelectorAll('*').forEach(el => {
                        [...el.attributes].forEach(attr => {
                            if (attr.name.startsWith('data-') ||
                                attr.name.startsWith('ng-') ||
                                attr.name.startsWith('v-') ||
                                attr.name.startsWith('_')) {
                                el.removeAttribute(attr.name);
                            }
                        });
                    });
                    return clone.outerHTML;
                })()
            "#,
            )
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        Ok(html)
    }

    /// Take a screenshot of the page
    async fn take_screenshot(&self, page: &Page) -> Result<String, BrowserError> {
        let screenshot = page
            .execute(
                CaptureScreenshotParams::builder()
                    .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
                    .build(),
            )
            .await
            .map_err(|e| BrowserError::CdpError(e.to_string()))?;

        // Convert binary data to base64 string
        Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &screenshot.data))
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
    }
}
