#!/usr/bin/env python3
"""
Example usage of CrawlWe library.

This script demonstrates how to use CrawlWe programmatically
to capture web pages, analyze content, and extract clean HTML + CSS.
"""

from pathlib import Path

# Import CrawlWe
from crawlwe import capture, batch_capture, analyze, CaptureOptions, RUST_AVAILABLE
from crawlwe.writer import write_mockup, export_dataset


def example_single_capture():
    """Capture a single page."""
    print("=" * 50)
    print("Example 1: Single Page Capture")
    print("=" * 50)

    options = CaptureOptions(
        capture_screenshot=True,
        viewport_width=1440,
        viewport_height=900,
    )

    result = capture("https://example.com", options)

    print(f"URL: {result.url}")
    print(f"Title: {result.title}")
    print(f"Elements: {result.element_count}")
    print(f"CSS Rules: {result.rule_count}")
    print(f"Colors found: {result.colors}")
    print(f"Fonts found: {result.fonts}")
    print(f"Duration: {result.duration_ms:.0f}ms")

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

    options = CaptureOptions(capture_screenshot=False)
    output_dir = Path("output/batch")

    success = 0
    errors = 0

    for result in batch_capture(urls, options, concurrent=3):
        if result.error:
            print(f"X {result.url}: {result.error}")
            errors += 1
        else:
            write_mockup(result, output_dir)
            print(f"OK {result.url}")
            success += 1

    print(f"\nCompleted: {success} success, {errors} errors")


def example_analyze_content():
    """Analyze web content for animations, frameworks, etc."""
    print("\n" + "=" * 50)
    print("Example 3: Content Analysis")
    print("=" * 50)

    # Example CSS with animations
    css = """
    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    .button {
        display: flex;
        background: #3498db;
        transition: all 0.3s ease;
        transform: translateY(0);
    }

    .button:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    }

    .grid {
        display: grid;
        gap: 1rem;
    }
    """

    # Example JS with Three.js
    js = """
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });

    gsap.to('.element', { duration: 1, x: 100 });

    const observer = new IntersectionObserver(callback);
    """

    # Example HTML
    html = """
    <html>
    <body>
        <header role="banner">
            <nav aria-label="Main">
                <button class="btn">Menu</button>
            </nav>
        </header>
        <main>
            <canvas id="webgl"></canvas>
            <img src="hero.jpg" alt="Hero image">
            <svg viewBox="0 0 100 100"></svg>
        </main>
        <footer>Footer</footer>
    </body>
    </html>
    """

    # Run analysis
    result = analyze(html, css, js)

    if "error" in result:
        print(f"Note: {result['error']}")
        return

    # Print CSS analysis
    css_result = result.get("css", {})
    print("\nCSS Analysis:")
    print(f"  Keyframes: {len(css_result.get('animations', {}).get('keyframes', []))}")
    print(f"  Transitions: {len(css_result.get('transitions', []))}")
    print(f"  Flexbox: {css_result.get('layout', {}).get('flexbox_count', 0)}")
    print(f"  Grid: {css_result.get('layout', {}).get('grid_count', 0)}")

    # Print JS analysis
    js_result = result.get("js", {})
    print("\nJavaScript Analysis:")
    for lib in js_result.get("animation_libraries", []):
        print(f"  Animation lib: {lib.get('name')}")
    for lib in js_result.get("graphics_libraries", []):
        print(f"  Graphics lib: {lib.get('name')}")
    if js_result.get("webgl", {}).get("detected"):
        print("  WebGL: Detected")

    # Print HTML analysis
    html_result = result.get("html", {})
    print("\nHTML Analysis:")
    print(f"  Elements: {html_result.get('element_count', 0)}")
    media = html_result.get("media_elements", {})
    print(f"  Canvas: {media.get('canvas', 0)}, SVG: {media.get('svg', 0)}")

    # Print score
    score = result.get("score", {})
    print(f"\nComplexity Score: {score.get('total', 0)}")


def example_export_dataset():
    """Export captured mockups as ML training data."""
    print("\n" + "=" * 50)
    print("Example 4: Export Dataset")
    print("=" * 50)

    mockups_dir = Path("output")
    output_file = Path("output/dataset.jsonl")

    if mockups_dir.exists() and any(mockups_dir.iterdir()):
        export_dataset(mockups_dir, output_file, format="jsonl")
        print(f"Dataset exported to: {output_file}")
    else:
        print("No mockups found. Run example 1 first.")


def main():
    """Run all examples."""
    print("CrawlWe Usage Examples")
    print("=" * 50)
    print(f"Rust core available: {RUST_AVAILABLE}")

    if not RUST_AVAILABLE:
        print("\nNote: Running in Python-only mode.")
        print("For full analysis, build Rust core:")
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
        example_analyze_content()
    except Exception as e:
        print(f"Example 3 failed: {e}")

    try:
        example_export_dataset()
    except Exception as e:
        print(f"Example 4 failed: {e}")


if __name__ == "__main__":
    main()
