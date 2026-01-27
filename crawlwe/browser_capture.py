"""
Browser-based capture using Playwright

For JavaScript-heavy pages with animations, loading states, and dynamic content.
"""

import asyncio
import base64
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
from urllib.parse import urljoin, urlparse


@dataclass
class BrowserCaptureOptions:
    """Options for browser-based capture."""

    # Viewport
    viewport_width: int = 1440
    viewport_height: int = 900
    device_scale: float = 1.0

    # Waiting
    wait_for_load: bool = True
    wait_for_network_idle: bool = True
    wait_after_load_ms: int = 2000  # Extra wait for animations
    max_wait_ms: int = 30000

    # What to capture
    capture_screenshot: bool = True
    capture_full_page: bool = True

    # Extraction
    remove_scripts: bool = True
    remove_styles_tags: bool = True
    remove_data_attrs: bool = True
    remove_framework_attrs: bool = True

    # Output
    output_dir: Optional[Path] = None


@dataclass
class BrowserCaptureResult:
    """Result from browser capture."""

    url: str
    title: Optional[str] = None

    # Content
    html_raw: str = ""          # Original HTML
    html_clean: str = ""        # Cleaned HTML
    css_inline: str = ""        # Inline styles from page
    css_external: list = field(default_factory=list)  # External CSS URLs
    css_combined: str = ""      # All CSS combined

    # Scripts
    js_external: list = field(default_factory=list)   # External JS URLs
    js_inline: str = ""         # Inline scripts (for analysis)

    # Assets
    screenshot: Optional[str] = None  # Base64 PNG
    fonts: list = field(default_factory=list)
    images: list = field(default_factory=list)

    # Metadata
    element_count: int = 0
    detected_libs: list = field(default_factory=list)

    # Timing
    load_time_ms: float = 0
    error: Optional[str] = None


async def capture_with_playwright(
    url: str,
    options: Optional[BrowserCaptureOptions] = None
) -> BrowserCaptureResult:
    """
    Capture a page using Playwright browser automation.

    This handles:
    - JavaScript execution
    - Loading animations
    - Dynamic content
    - External resources
    """
    from playwright.async_api import async_playwright
    import time

    if options is None:
        options = BrowserCaptureOptions()

    result = BrowserCaptureResult(url=url)
    start_time = time.time()

    async with async_playwright() as p:
        # Launch browser - try different paths
        import os
        chrome_paths = [
            '/root/.cache/ms-playwright/chromium-1194/chrome-linux/chrome',
            '/root/.cache/ms-playwright/chromium-1200/chrome-linux/chrome',
            None  # Use default
        ]

        browser = None
        for chrome_path in chrome_paths:
            try:
                if chrome_path and os.path.exists(chrome_path):
                    browser = await p.chromium.launch(
                        headless=True,
                        executable_path=chrome_path
                    )
                else:
                    browser = await p.chromium.launch(headless=True)
                break
            except Exception as e:
                continue

        if browser is None:
            # Try firefox as last resort
            try:
                browser = await p.firefox.launch(headless=True)
            except Exception:
                raise RuntimeError("No browser available. Run: playwright install")

        context = await browser.new_context(
            viewport={
                'width': options.viewport_width,
                'height': options.viewport_height
            },
            device_scale_factor=options.device_scale,
            user_agent='Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
        )

        page = await context.new_page()

        # Collect external resources
        external_css = []
        external_js = []
        fonts = []

        async def handle_response(response):
            url = response.url
            content_type = response.headers.get('content-type', '')

            if 'text/css' in content_type or url.endswith('.css'):
                external_css.append(url)
            elif 'javascript' in content_type or url.endswith('.js'):
                external_js.append(url)
            elif 'font' in content_type or any(url.endswith(ext) for ext in ['.woff', '.woff2', '.ttf', '.otf', '.eot']):
                fonts.append(url)

        page.on('response', handle_response)

        try:
            # Navigate to page
            await page.goto(url, wait_until='domcontentloaded', timeout=options.max_wait_ms)

            # Wait for network to be idle (most resources loaded)
            if options.wait_for_network_idle:
                try:
                    await page.wait_for_load_state('networkidle', timeout=options.max_wait_ms)
                except:
                    pass  # Continue even if timeout

            # Extra wait for animations to complete
            if options.wait_after_load_ms > 0:
                await asyncio.sleep(options.wait_after_load_ms / 1000)

            # Get title
            result.title = await page.title()

            # Get raw HTML
            result.html_raw = await page.content()

            # Extract inline styles
            result.css_inline = await page.evaluate('''() => {
                const styles = [];
                document.querySelectorAll('style').forEach(style => {
                    if (style.textContent) {
                        styles.push(style.textContent);
                    }
                });
                return styles.join('\\n\\n');
            }''')

            # Extract inline scripts (for analysis)
            result.js_inline = await page.evaluate('''() => {
                const scripts = [];
                document.querySelectorAll('script:not([src])').forEach(script => {
                    if (script.textContent && script.textContent.trim()) {
                        scripts.push(script.textContent);
                    }
                });
                return scripts.join('\\n\\n');
            }''')

            # Get computed styles for body (to detect fonts)
            body_styles = await page.evaluate('''() => {
                const computed = getComputedStyle(document.body);
                return {
                    fontFamily: computed.fontFamily,
                    backgroundColor: computed.backgroundColor,
                    color: computed.color
                };
            }''')

            # Extract font families
            if body_styles.get('fontFamily'):
                font_families = [f.strip().strip('"\'') for f in body_styles['fontFamily'].split(',')]
                result.fonts = [f for f in font_families if f and f.lower() not in ('sans-serif', 'serif', 'monospace', 'system-ui')]

            # Get element count
            result.element_count = await page.evaluate('() => document.querySelectorAll("*").length')

            # Generate clean HTML
            result.html_clean = await page.evaluate('''() => {
                const clone = document.documentElement.cloneNode(true);

                // Remove scripts
                clone.querySelectorAll('script, noscript').forEach(el => el.remove());

                // Remove style tags (we'll link to external CSS)
                clone.querySelectorAll('style').forEach(el => el.remove());

                // Remove link[rel=stylesheet] (we'll add them back)
                clone.querySelectorAll('link[rel="stylesheet"]').forEach(el => el.remove());

                // Remove meta tags except charset and viewport
                clone.querySelectorAll('meta').forEach(el => {
                    const name = el.getAttribute('name');
                    const charset = el.getAttribute('charset');
                    const httpEquiv = el.getAttribute('http-equiv');
                    if (!charset && name !== 'viewport' && !httpEquiv) {
                        el.remove();
                    }
                });

                // Remove data-* and framework attributes
                clone.querySelectorAll('*').forEach(el => {
                    [...el.attributes].forEach(attr => {
                        if (attr.name.startsWith('data-') ||
                            attr.name.startsWith('ng-') ||
                            attr.name.startsWith('v-') ||
                            attr.name.startsWith('x-') ||
                            attr.name.startsWith('_') ||
                            attr.name.startsWith('@') ||
                            attr.name.startsWith(':')) {
                            el.removeAttribute(attr.name);
                        }
                    });
                });

                // Remove empty attributes
                clone.querySelectorAll('*').forEach(el => {
                    [...el.attributes].forEach(attr => {
                        if (attr.value === '' && attr.name !== 'alt') {
                            el.removeAttribute(attr.name);
                        }
                    });
                });

                return clone.outerHTML;
            }''')

            # Take screenshot
            if options.capture_screenshot:
                screenshot_bytes = await page.screenshot(
                    full_page=options.capture_full_page,
                    type='png'
                )
                result.screenshot = base64.b64encode(screenshot_bytes).decode('utf-8')

            # Store external resources
            result.css_external = list(set(external_css))
            result.js_external = list(set(external_js))

            # Fetch and combine CSS
            css_parts = [result.css_inline] if result.css_inline else []

            for css_url in result.css_external[:20]:  # Limit to 20 stylesheets
                try:
                    css_response = await page.request.get(css_url, timeout=5000)
                    if css_response.ok:
                        css_text = await css_response.text()
                        css_parts.append(f"/* Source: {css_url} */\n{css_text}")
                except:
                    css_parts.append(f"/* Failed to fetch: {css_url} */")

            result.css_combined = '\n\n'.join(css_parts)

            # Detect libraries
            result.detected_libs = await page.evaluate('''() => {
                const libs = [];

                // Check for common global variables
                if (window.gsap || window.TweenMax || window.TweenLite) libs.push('GSAP');
                if (window.THREE) libs.push('Three.js');
                if (window.anime) libs.push('Anime.js');
                if (window.ScrollMagic) libs.push('ScrollMagic');
                if (window.Lottie || window.lottie) libs.push('Lottie');
                if (window.Swiper) libs.push('Swiper');
                if (window.AOS) libs.push('AOS');
                if (window.particlesJS || window.Particles) libs.push('Particles.js');
                if (window.Typed) libs.push('Typed.js');
                if (window.Chart) libs.push('Chart.js');
                if (window.d3) libs.push('D3.js');
                if (window.jQuery || window.$) libs.push('jQuery');
                if (window.React) libs.push('React');
                if (window.Vue) libs.push('Vue.js');
                if (window.angular) libs.push('Angular');
                if (window.Alpine) libs.push('Alpine.js');
                if (window.Locomotive) libs.push('Locomotive Scroll');
                if (window.LocomotiveScroll) libs.push('Locomotive Scroll');
                if (window.Barba) libs.push('Barba.js');
                if (window.barba) libs.push('Barba.js');
                if (window.Plyr) libs.push('Plyr');
                if (window.Glide) libs.push('Glide.js');
                if (window.Splide) libs.push('Splide');
                if (window.Flickity) libs.push('Flickity');
                if (window.Rellax) libs.push('Rellax');
                if (window.ScrollTrigger) libs.push('ScrollTrigger');
                if (window.SmoothScroll) libs.push('Smooth Scroll');
                if (window.Isotope) libs.push('Isotope');
                if (window.Masonry) libs.push('Masonry');
                if (window.imagesLoaded) libs.push('imagesLoaded');
                if (window.Lightbox || window.lightbox) libs.push('Lightbox');
                if (window.GLightbox) libs.push('GLightbox');
                if (window.Fancybox || window.fancybox) libs.push('Fancybox');

                return libs;
            }''')

        except Exception as e:
            result.error = str(e)

        finally:
            result.load_time_ms = (time.time() - start_time) * 1000
            await browser.close()

    return result


def capture_sync(url: str, options: Optional[BrowserCaptureOptions] = None) -> BrowserCaptureResult:
    """Synchronous wrapper for capture_with_playwright."""
    return asyncio.run(capture_with_playwright(url, options))


async def save_capture(result: BrowserCaptureResult, output_dir: Path) -> Path:
    """
    Save captured content to disk.

    Creates:
        output_dir/
            project.toml
            index.html
            styles.css
            scripts/
                main.js
            data/
                raw.html
                analysis.json
            screenshot.png
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Create subdirectories
    (output_dir / 'scripts').mkdir(exist_ok=True)
    (output_dir / 'data').mkdir(exist_ok=True)

    # Save raw HTML
    (output_dir / 'data' / 'raw.html').write_text(result.html_raw, encoding='utf-8')

    # Save clean HTML with stylesheet link
    clean_html = result.html_clean
    if '</head>' in clean_html:
        clean_html = clean_html.replace(
            '</head>',
            '  <link rel="stylesheet" href="styles.css">\n</head>'
        )
    if '</body>' in clean_html:
        clean_html = clean_html.replace(
            '</body>',
            '  <script src="scripts/main.js"></script>\n</body>'
        )
    (output_dir / 'index.html').write_text(clean_html, encoding='utf-8')

    # Save CSS
    (output_dir / 'styles.css').write_text(result.css_combined, encoding='utf-8')

    # Save placeholder JS
    js_content = f'''// Generated by CrawlWe
// Source: {result.url}
// Detected libraries: {', '.join(result.detected_libs) or 'none'}

document.addEventListener('DOMContentLoaded', function() {{
    console.log('Page loaded');
}});
'''
    (output_dir / 'scripts' / 'main.js').write_text(js_content, encoding='utf-8')

    # Save screenshot
    if result.screenshot:
        screenshot_data = base64.b64decode(result.screenshot)
        (output_dir / 'screenshot.png').write_bytes(screenshot_data)

    # Generate project.toml
    try:
        from crawlwe import generate_project_config
        project_toml = generate_project_config(
            result.url,
            result.html_raw,
            result.css_combined,
            result.js_inline,
            result.title
        )
        (output_dir / 'project.toml').write_text(project_toml, encoding='utf-8')
    except ImportError:
        # Fallback if Rust core not available
        project_toml = f'''[meta]
name = "{_sanitize_name(result.title or 'captured-page')}"
version = "1.0.0"
generator = "crawlwe"

[source]
url = "{result.url}"
title = "{result.title or ''}"

[technologies]
detected_libs = {result.detected_libs}
'''
        (output_dir / 'project.toml').write_text(project_toml, encoding='utf-8')

    # Save metadata
    import json
    metadata = {
        'url': result.url,
        'title': result.title,
        'element_count': result.element_count,
        'detected_libs': result.detected_libs,
        'css_external': result.css_external,
        'js_external': result.js_external,
        'fonts': result.fonts,
        'load_time_ms': result.load_time_ms,
        'error': result.error
    }
    (output_dir / 'data' / 'metadata.json').write_text(
        json.dumps(metadata, indent=2),
        encoding='utf-8'
    )

    return output_dir


def _sanitize_name(name: str) -> str:
    """Sanitize name for use in filenames/identifiers."""
    return re.sub(r'[^a-zA-Z0-9]+', '-', name.lower()).strip('-')[:50]
