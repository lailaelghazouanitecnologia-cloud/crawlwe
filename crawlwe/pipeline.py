"""
CrawlWe Pipeline - Orchestrates the capture process

This module provides the high-level API for capturing web pages.
It coordinates between the Rust core (heavy lifting) and Python (logic).
"""

from dataclasses import dataclass, field
from typing import Iterator, Optional
from concurrent.futures import ThreadPoolExecutor, as_completed
import time


@dataclass
class CaptureOptions:
    """Configuration options for page capture."""

    # What to capture
    capture_states: bool = True  # hover, focus, active
    capture_screenshot: bool = True
    capture_fonts: bool = False
    capture_images: bool = False

    # Viewport
    viewport_width: int = 1440
    viewport_height: int = 900

    # Output
    minify_css: bool = False
    minify_html: bool = False

    # Limits
    max_elements: int = 2000
    timeout_ms: int = 30000

    # Browser
    user_agent: Optional[str] = None


@dataclass
class CaptureResult:
    """Result of a page capture."""

    url: str
    title: Optional[str] = None
    html: str = ""
    css: str = ""
    screenshot: Optional[str] = None  # Base64 PNG
    element_count: int = 0
    rule_count: int = 0
    colors: list = field(default_factory=list)
    fonts: list = field(default_factory=list)
    variables: dict = field(default_factory=dict)
    error: Optional[str] = None
    duration_ms: float = 0


def capture(url: str, options: Optional[CaptureOptions] = None) -> CaptureResult:
    """
    Capture a single web page.

    Args:
        url: The URL to capture
        options: Capture configuration options

    Returns:
        CaptureResult with HTML, CSS, and metadata

    Example:
        result = capture("https://example.com")
        print(result.html)
        print(result.css)
    """
    if options is None:
        options = CaptureOptions()

    start_time = time.time()

    try:
        # Try Rust core first (fast path)
        result = _capture_with_rust(url, options)
    except ImportError:
        # Fallback to Python implementation
        result = _capture_with_python(url, options)

    result.duration_ms = (time.time() - start_time) * 1000
    return result


def batch_capture(
    urls: list[str],
    options: Optional[CaptureOptions] = None,
    concurrent: int = 5,
) -> Iterator[CaptureResult]:
    """
    Capture multiple pages concurrently.

    Args:
        urls: List of URLs to capture
        options: Capture configuration options
        concurrent: Number of concurrent captures

    Yields:
        CaptureResult for each URL

    Example:
        for result in batch_capture(["url1", "url2"]):
            if result.error:
                print(f"Failed: {result.url}")
            else:
                save(result)
    """
    if options is None:
        options = CaptureOptions()

    with ThreadPoolExecutor(max_workers=concurrent) as executor:
        futures = {executor.submit(capture, url, options): url for url in urls}

        for future in as_completed(futures):
            url = futures[future]
            try:
                yield future.result()
            except Exception as e:
                yield CaptureResult(url=url, error=str(e))


def _capture_with_rust(url: str, options: CaptureOptions) -> CaptureResult:
    """Capture using Rust core (preferred, fast)."""
    from crawlwe.crawlwe_core import capture_page

    page_result = capture_page(
        url,
        capture_states=options.capture_states,
        capture_screenshot=options.capture_screenshot,
        viewport_width=options.viewport_width,
        viewport_height=options.viewport_height,
    )

    return CaptureResult(
        url=page_result.url,
        title=page_result.title,
        html=page_result.html,
        css=page_result.css,
        screenshot=page_result.screenshot,
        element_count=page_result.element_count,
        rule_count=page_result.rule_count,
        colors=list(page_result.colors),
        fonts=list(page_result.fonts),
        variables=dict(page_result.variables),
    )


def _capture_with_python(url: str, options: CaptureOptions) -> CaptureResult:
    """
    Capture using pure Python (fallback, slower).

    Uses requests + BeautifulSoup for basic extraction.
    Does not support states or full CSS extraction.
    """
    import requests
    from bs4 import BeautifulSoup
    import cssbeautifier

    # Fetch page
    headers = {"User-Agent": options.user_agent or "CrawlWe/0.1"}
    response = requests.get(url, headers=headers, timeout=options.timeout_ms / 1000)
    response.raise_for_status()

    soup = BeautifulSoup(response.text, "lxml")

    # Extract title
    title = soup.title.string if soup.title else None

    # Collect all CSS
    css_parts = []

    # External stylesheets
    for link in soup.find_all("link", rel="stylesheet"):
        href = link.get("href")
        if href:
            try:
                if href.startswith("//"):
                    href = "https:" + href
                elif href.startswith("/"):
                    from urllib.parse import urljoin

                    href = urljoin(url, href)

                css_response = requests.get(href, headers=headers, timeout=10)
                if css_response.ok:
                    css_parts.append(f"/* Source: {href} */\n{css_response.text}")
            except Exception:
                pass

    # Inline styles
    for style in soup.find_all("style"):
        if style.string:
            css_parts.append(f"/* Inline style */\n{style.string}")

    # Combine and format CSS
    combined_css = "\n\n".join(css_parts)
    if combined_css and not options.minify_css:
        try:
            combined_css = cssbeautifier.beautify(combined_css)
        except Exception:
            pass

    # Clean HTML
    # Remove script, style, link tags
    for tag in soup.find_all(["script", "style", "link", "noscript"]):
        tag.decompose()

    # Remove data-* attributes
    for tag in soup.find_all(True):
        attrs_to_remove = [attr for attr in tag.attrs if attr.startswith("data-")]
        for attr in attrs_to_remove:
            del tag[attr]

    # Count elements
    element_count = len(soup.find_all(True))

    # Extract colors from CSS
    colors = _extract_colors_python(combined_css)
    fonts = _extract_fonts_python(combined_css)

    # Generate clean HTML
    if soup.html:
        # Add stylesheet link
        if soup.head is None:
            soup.html.insert(0, soup.new_tag("head"))

        link_tag = soup.new_tag("link", rel="stylesheet", href="styles.css")
        soup.head.append(link_tag)

        html = str(soup)
    else:
        html = response.text

    return CaptureResult(
        url=url,
        title=title,
        html=html,
        css=combined_css,
        element_count=element_count,
        rule_count=combined_css.count("{"),
        colors=colors,
        fonts=fonts,
    )


def _extract_colors_python(css: str) -> list[str]:
    """Extract color values from CSS text."""
    import re

    colors = set()

    # Hex colors
    for match in re.findall(r"#[0-9a-fA-F]{3,8}\b", css):
        colors.add(match.lower())

    # rgb/rgba
    for match in re.findall(r"rgba?\([^)]+\)", css):
        colors.add(match)

    # hsl/hsla
    for match in re.findall(r"hsla?\([^)]+\)", css):
        colors.add(match)

    return sorted(colors)


def _extract_fonts_python(css: str) -> list[str]:
    """Extract font families from CSS text."""
    import re

    fonts = set()

    for match in re.findall(r'font-family:\s*([^;]+)', css):
        for family in match.split(","):
            family = family.strip().strip("\"'")
            if family and family.lower() not in (
                "sans-serif",
                "serif",
                "monospace",
                "cursive",
                "fantasy",
                "system-ui",
                "inherit",
            ):
                fonts.add(family)

    return sorted(fonts)
