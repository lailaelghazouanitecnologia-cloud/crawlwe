"""
CrawlWe - Web Page Extractor for ML Training

A powerful tool to extract clean HTML + CSS from web pages
for training AI models to generate more natural designs.

The main CLI is now in Rust (`crawlwe` binary).
This Python package provides:
  - PyO3 bindings to the Rust core
  - High-level Python API for library usage

Usage:
    # Install Rust core
    maturin develop

    # Use from Python
    from crawlwe import analyze, detect_tailwind

    analysis = analyze(html, css, js)
    tailwind = detect_tailwind(html, css)

CLI Usage (Rust):
    crawlwe fetch <url>
    crawlwe batch urls.txt
    crawlwe analyze ./output
    crawlwe export ./output
"""

__version__ = "0.1.0"
__author__ = "CrawlWe Team"

# Try to import Rust core (compiled with maturin)
RUST_AVAILABLE = False

try:
    from crawlwe_core import (
        # Analysis functions
        analyze_content,
        analyze_css,
        analyze_js,
        analyze_html,
        # Extraction functions
        extract_css_colors,
        extract_css_fonts,
        extract_css_variables,
        # Detection functions
        detect_tailwind,
        detect_webgl,
        detect_webgpu,
        detect_animation_libraries,
        detect_ui_frameworks,
        # Export functions
        get_css_cdn,
        get_js_cdn,
        get_font_url,
        generate_project_config,
        # Classes
        PageResult,
    )
    RUST_AVAILABLE = True
except ImportError:
    # Rust core not compiled - provide None placeholders
    analyze_content = None
    analyze_css = None
    analyze_js = None
    analyze_html = None
    extract_css_colors = None
    extract_css_fonts = None
    extract_css_variables = None
    detect_tailwind = None
    detect_webgl = None
    detect_webgpu = None
    detect_animation_libraries = None
    detect_ui_frameworks = None
    get_css_cdn = None
    get_js_cdn = None
    get_font_url = None
    generate_project_config = None
    PageResult = None


def analyze(html: str = "", css: str = "", js: str = "") -> dict:
    """
    Analyze web content and return detailed analysis.

    Uses Rust core if available.

    Args:
        html: HTML content to analyze
        css: CSS content to analyze
        js: JavaScript content to analyze

    Returns:
        Dictionary with analysis results
    """
    if RUST_AVAILABLE and analyze_content:
        import json
        return json.loads(analyze_content(html, css, js))
    return {
        "error": "Rust core not available. Build with: maturin develop",
        "css": {},
        "js": {},
        "html": {},
    }


def generate_config(url: str, html: str, css: str, js: str = "", title: str = None) -> str:
    """
    Generate project.toml configuration for captured content.

    Args:
        url: Source URL
        html: HTML content
        css: CSS content
        js: JavaScript content (optional)
        title: Page title (optional)

    Returns:
        TOML configuration string
    """
    if RUST_AVAILABLE and generate_project_config:
        return generate_project_config(url, html, css, js, title)
    return "# Rust core not available"


__all__ = [
    # High-level Python API
    "analyze",
    "generate_config",
    # Rust bindings (when available)
    "analyze_content",
    "analyze_css",
    "analyze_js",
    "analyze_html",
    "extract_css_colors",
    "extract_css_fonts",
    "extract_css_variables",
    "detect_tailwind",
    "detect_webgl",
    "detect_webgpu",
    "detect_animation_libraries",
    "detect_ui_frameworks",
    "get_css_cdn",
    "get_js_cdn",
    "get_font_url",
    "generate_project_config",
    "PageResult",
    # Meta
    "RUST_AVAILABLE",
    "__version__",
]
