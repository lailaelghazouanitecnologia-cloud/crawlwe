"""
CrawlWe - Web Page Extractor for ML Training

A powerful tool to extract clean HTML + CSS from web pages
for training AI models to generate more natural designs.

Usage:
    from crawlwe import capture, batch_capture

    # Single page
    result = capture("https://example.com")
    print(result.html)
    print(result.css)

    # Multiple pages
    results = batch_capture(["url1", "url2", "url3"])
"""

__version__ = "0.1.0"
__author__ = "CrawlWe Team"

from crawlwe.pipeline import capture, batch_capture, CaptureOptions
from crawlwe.writer import write_mockup, MockupWriter

# Try to import Rust core (may not be compiled yet)
try:
    from crawlwe.crawlwe_core import (
        capture_page,
        parse_css,
        parse_html,
        extract_css_colors,
        extract_css_fonts,
        merge_css,
        calculate_specificity,
        analyze_html,
        PageResult,
        Viewport,
        CaptureConfig,
    )

    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False

__all__ = [
    # High-level API
    "capture",
    "batch_capture",
    "CaptureOptions",
    "write_mockup",
    "MockupWriter",
    # Rust bindings (when available)
    "capture_page",
    "parse_css",
    "parse_html",
    "extract_css_colors",
    "extract_css_fonts",
    "merge_css",
    "calculate_specificity",
    "analyze_html",
    "PageResult",
    "Viewport",
    "CaptureConfig",
    # Meta
    "RUST_AVAILABLE",
    "__version__",
]
