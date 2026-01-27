#!/usr/bin/env python3
"""
Test capture script for multiple sites.
Uses Playwright for JS-heavy pages and processes through the pipeline.
"""

import asyncio
import sys
import os
import json
from pathlib import Path
from datetime import datetime

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from crawlwe.browser_capture import capture_with_playwright, BrowserCaptureOptions, save_capture

# Test URLs
TEST_URLS = [
    "https://www.eduardbodak.com/",
    "https://stripe.dev/",
    "https://www.adaline.ai/",
]


async def capture_site(url: str, output_dir: Path) -> dict:
    """Capture a single site."""
    print(f"\n{'='*60}")
    print(f"Capturing: {url}")
    print('='*60)

    # Create site-specific directory
    from urllib.parse import urlparse
    domain = urlparse(url).netloc.replace("www.", "")
    site_dir = output_dir / domain
    site_dir.mkdir(parents=True, exist_ok=True)

    options = BrowserCaptureOptions(
        wait_for_network_idle=True,
        wait_after_load_ms=5000,  # Extra wait for animations
        max_wait_ms=30000,
        capture_screenshot=True,
        viewport_width=1440,
        viewport_height=900,
    )

    try:
        result = await capture_with_playwright(url, options)

        # Save capture
        await save_capture(result, str(site_dir))

        # Print summary
        print(f"\n✓ Captured successfully!")
        print(f"  Title: {result.title}")
        print(f"  HTML size: {len(result.html_raw):,} bytes")
        print(f"  CSS size: {len(result.css_combined):,} bytes")
        print(f"  External scripts: {len(result.js_external)}")
        print(f"  External styles: {len(result.css_external)}")

        if result.detected_libs:
            print(f"  Libraries detected:")
            for lib in result.detected_libs[:10]:  # Show first 10
                print(f"    - {lib}")

        return {
            "url": url,
            "domain": domain,
            "success": True,
            "title": result.title,
            "html_size": len(result.html_raw),
            "css_size": len(result.css_combined),
            "scripts": len(result.js_external),
            "stylesheets": len(result.css_external),
            "libraries": result.detected_libs,
        }

    except Exception as e:
        print(f"\n✗ Failed to capture: {e}")
        return {
            "url": url,
            "domain": domain,
            "success": False,
            "error": str(e),
        }


async def main():
    """Run captures for all test URLs."""
    print("\n" + "="*60)
    print("CrawlWe Test Captures")
    print("="*60)
    print(f"Date: {datetime.now().isoformat()}")
    print(f"Sites to capture: {len(TEST_URLS)}")

    # Output directory
    output_dir = Path(__file__).parent.parent / "captures" / "test_run"
    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"Output directory: {output_dir}")

    # Capture all sites
    results = []
    for url in TEST_URLS:
        result = await capture_site(url, output_dir)
        results.append(result)

    # Summary
    print("\n" + "="*60)
    print("CAPTURE SUMMARY")
    print("="*60)

    successful = [r for r in results if r.get("success")]
    failed = [r for r in results if not r.get("success")]

    print(f"\nSuccessful: {len(successful)}/{len(results)}")
    for r in successful:
        libs = r.get("libraries", [])
        print(f"  ✓ {r['domain']}")
        print(f"    HTML: {r['html_size']:,}b | CSS: {r['css_size']:,}b | Libs: {len(libs)}")

    if failed:
        print(f"\nFailed: {len(failed)}")
        for r in failed:
            print(f"  ✗ {r['domain']}: {r.get('error', 'Unknown error')}")

    # Save summary
    summary_path = output_dir / "summary.json"
    with open(summary_path, "w") as f:
        json.dump({
            "timestamp": datetime.now().isoformat(),
            "total": len(results),
            "successful": len(successful),
            "failed": len(failed),
            "results": results,
        }, f, indent=2)

    print(f"\nSummary saved to: {summary_path}")
    print("\nDone!")


if __name__ == "__main__":
    asyncio.run(main())
