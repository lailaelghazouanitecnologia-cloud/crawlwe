#!/usr/bin/env python3
"""
Capture a single page using Playwright.

Usage:
    python scripts/capture_page.py https://www.adaline.ai/
    python scripts/capture_page.py https://example.com -o output/example
"""

import argparse
import asyncio
import sys
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from crawlwe.browser_capture import (
    BrowserCaptureOptions,
    capture_with_playwright,
    save_capture
)


async def main():
    parser = argparse.ArgumentParser(description='Capture a web page')
    parser.add_argument('url', help='URL to capture')
    parser.add_argument('-o', '--output', help='Output directory')
    parser.add_argument('--width', type=int, default=1440, help='Viewport width')
    parser.add_argument('--height', type=int, default=900, help='Viewport height')
    parser.add_argument('--wait', type=int, default=3000, help='Wait after load (ms)')
    parser.add_argument('--no-screenshot', action='store_true', help='Skip screenshot')

    args = parser.parse_args()

    # Determine output directory
    if args.output:
        output_dir = Path(args.output)
    else:
        from urllib.parse import urlparse
        domain = urlparse(args.url).netloc.replace('www.', '')
        output_dir = Path('captures') / domain

    print(f"🔄 Capturing: {args.url}")
    print(f"📁 Output: {output_dir}")

    options = BrowserCaptureOptions(
        viewport_width=args.width,
        viewport_height=args.height,
        wait_after_load_ms=args.wait,
        capture_screenshot=not args.no_screenshot
    )

    try:
        # Capture
        print("⏳ Loading page...")
        result = await capture_with_playwright(args.url, options)

        if result.error:
            print(f"❌ Error: {result.error}")
            sys.exit(1)

        print(f"✅ Loaded in {result.load_time_ms:.0f}ms")
        print(f"   Title: {result.title}")
        print(f"   Elements: {result.element_count}")
        print(f"   CSS files: {len(result.css_external)}")
        print(f"   JS files: {len(result.js_external)}")

        if result.detected_libs:
            print(f"   Libraries: {', '.join(result.detected_libs)}")

        # Save
        print("\n💾 Saving files...")
        saved_path = await save_capture(result, output_dir)

        print(f"\n✅ Saved to: {saved_path}")
        print("\nFiles created:")
        for f in sorted(saved_path.rglob('*')):
            if f.is_file():
                size = f.stat().st_size
                if size > 1024:
                    size_str = f"{size/1024:.1f}KB"
                else:
                    size_str = f"{size}B"
                print(f"   {f.relative_to(saved_path)} ({size_str})")

    except Exception as e:
        print(f"❌ Failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    asyncio.run(main())
