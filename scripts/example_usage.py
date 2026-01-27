#!/usr/bin/env python3
"""
Example usage of CrawlWe library.

This script demonstrates how to use CrawlWe programmatically
to capture web pages and extract clean HTML + CSS.
"""

from pathlib import Path

# Import CrawlWe
from crawlwe import capture, batch_capture, CaptureOptions
from crawlwe.writer import write_mockup, export_dataset


def example_single_capture():
    """Capture a single page."""
    print("=" * 50)
    print("Example 1: Single Page Capture")
    print("=" * 50)

    # Configure capture options
    options = CaptureOptions(
        capture_states=True,  # Capture :hover, :focus states
        capture_screenshot=True,
        viewport_width=1440,
        viewport_height=900,
    )

    # Capture the page
    result = capture("https://example.com", options)

    # Print results
    print(f"URL: {result.url}")
    print(f"Title: {result.title}")
    print(f"Elements: {result.element_count}")
    print(f"CSS Rules: {result.rule_count}")
    print(f"Colors found: {result.colors}")
    print(f"Fonts found: {result.fonts}")
    print(f"Duration: {result.duration_ms:.0f}ms")

    # Save to disk
    output_path = write_mockup(result, Path("output"))
    print(f"Saved to: {output_path}")

    return result


def example_batch_capture():
    """Capture multiple pages."""
    print("\n" + "=" * 50)
    print("Example 2: Batch Capture")
    print("=" * 50)

    urls = [
        "https://example.com",
        "https://example.org",
        "https://example.net",
    ]

    options = CaptureOptions(
        capture_states=False,  # Faster without states
        capture_screenshot=False,
    )

    output_dir = Path("output/batch")

    success = 0
    errors = 0

    for result in batch_capture(urls, options, concurrent=3):
        if result.error:
            print(f"❌ {result.url}: {result.error}")
            errors += 1
        else:
            write_mockup(result, output_dir)
            print(f"✓ {result.url}")
            success += 1

    print(f"\nCompleted: {success} success, {errors} errors")


def example_export_dataset():
    """Export captured mockups as ML training data."""
    print("\n" + "=" * 50)
    print("Example 3: Export Dataset")
    print("=" * 50)

    mockups_dir = Path("output")
    output_file = Path("output/dataset.jsonl")

    if mockups_dir.exists():
        export_dataset(mockups_dir, output_file, format="jsonl")
        print(f"Dataset exported to: {output_file}")
    else:
        print("No mockups found. Run examples 1 or 2 first.")


def example_rust_functions():
    """Use Rust functions directly."""
    print("\n" + "=" * 50)
    print("Example 4: Direct Rust Functions")
    print("=" * 50)

    try:
        from crawlwe import (
            parse_css,
            parse_html,
            extract_css_colors,
            calculate_specificity,
            RUST_AVAILABLE,
        )

        if not RUST_AVAILABLE:
            print("Rust core not available. Build with: maturin develop")
            return

        # Parse and optimize CSS
        css = """
        .button {
            background: #3498db;
            padding: 10px 20px;
        }
        .button {
            border-radius: 5px;
        }
        .btn {
            background: #3498db;
            padding: 10px 20px;
        }
        """

        optimized = parse_css(css, deduplicate=True, minify=False)
        print("Optimized CSS:")
        print(optimized)

        # Extract colors
        colors = extract_css_colors(css)
        print(f"\nColors found: {colors}")

        # Calculate specificity
        spec = calculate_specificity("div.class#id")
        print(f"\nSpecificity of 'div.class#id': {spec}")

    except ImportError as e:
        print(f"Rust core not available: {e}")
        print("Build with: maturin develop")


def main():
    """Run all examples."""
    print("CrawlWe Usage Examples")
    print("=" * 50)

    # Check if Rust core is available
    from crawlwe import RUST_AVAILABLE

    print(f"Rust core available: {RUST_AVAILABLE}")

    if not RUST_AVAILABLE:
        print("\nNote: Running in Python-only mode (slower).")
        print("For full performance, build Rust core:")
        print("  pip install maturin")
        print("  maturin develop")

    # Run examples
    try:
        example_single_capture()
    except Exception as e:
        print(f"Example 1 failed: {e}")

    try:
        example_batch_capture()
    except Exception as e:
        print(f"Example 2 failed: {e}")

    try:
        example_export_dataset()
    except Exception as e:
        print(f"Example 3 failed: {e}")

    try:
        example_rust_functions()
    except Exception as e:
        print(f"Example 4 failed: {e}")


if __name__ == "__main__":
    main()
