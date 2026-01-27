"""
CrawlWe - Web Page Extractor for ML Training

A powerful tool to extract clean HTML + CSS from web pages
for training AI models to generate more natural designs.

Usage:
    from crawlwe import capture, batch_capture, analyze

    # Single page capture
    result = capture("https://example.com")
    print(result.html)
    print(result.css)

    # Analyze content
    analysis = analyze(result.html, result.css)
    print(analysis)

    # Batch capture
    results = batch_capture(["url1", "url2", "url3"])
"""

__version__ = "0.1.0"
__author__ = "CrawlWe Team"

from crawlwe.pipeline import capture, batch_capture, CaptureOptions, CaptureResult
from crawlwe.writer import write_mockup, MockupWriter, merge_mockups, export_dataset

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
        # Classes
        PageResult,
    )
    RUST_AVAILABLE = True
except ImportError:
    # Rust core not compiled - provide Python fallbacks
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
    PageResult = None


def analyze(html: str = "", css: str = "", js: str = "") -> dict:
    """
    Analyze web content and return detailed analysis.

    Uses Rust core if available, otherwise returns empty analysis.

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


__all__ = [
    # High-level Python API
    "capture",
    "batch_capture",
    "CaptureOptions",
    "CaptureResult",
    "write_mockup",
    "MockupWriter",
    "merge_mockups",
    "export_dataset",
    "analyze",
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
    "PageResult",
    # Meta
    "RUST_AVAILABLE",
    "__version__",
]
