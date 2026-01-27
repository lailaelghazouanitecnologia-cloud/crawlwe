#!/usr/bin/env python3
"""
CrawlWe Scraper - Extracts formatted HTML, CSS, and JavaScript from web pages.
Detects WebGL/WebGPU code.
"""

import requests
from bs4 import BeautifulSoup
from urllib.parse import urljoin, urlparse
import json
import os
import re
import cssbeautifier

URL = "https://www.becaneparis.com/"
OUTPUT_DIR = "output/becaneparis"

# WebGL/WebGPU detection patterns
WEBGL_PATTERNS = [
    r'getContext\s*\(\s*["\']webgl',
    r'getContext\s*\(\s*["\']experimental-webgl',
    r'WebGLRenderingContext',
    r'WebGL2RenderingContext',
    r'gl\.(bindBuffer|bindTexture|createShader|createProgram|drawArrays|drawElements)',
    r'gl\.(uniform|attribute|varying)',
    r'THREE\.',  # Three.js
    r'PIXI\.',   # PixiJS
    r'Babylon\.',  # BabylonJS
    r'createShader|createProgram|shaderSource|compileShader',
    r'vertexAttribPointer|enableVertexAttribArray',
    r'uniformMatrix|uniform[1234][fi]v?',
]

WEBGPU_PATTERNS = [
    r'navigator\.gpu',
    r'GPUDevice',
    r'GPUAdapter',
    r'GPUBuffer',
    r'GPUTexture',
    r'GPUShaderModule',
    r'GPURenderPipeline',
    r'GPUComputePipeline',
    r'createShaderModule',
    r'createRenderPipeline',
    r'createComputePipeline',
    r'@vertex|@fragment|@compute',  # WGSL
]

def format_css(css_text):
    """Format CSS to be readable."""
    try:
        opts = cssbeautifier.default_options()
        opts.indent_size = 2
        opts.selector_separator_newline = True
        opts.newline_between_rules = True
        return cssbeautifier.beautify(css_text, opts)
    except Exception as e:
        print(f"   Warning: CSS formatting failed: {e}")
        return css_text

def format_html(soup):
    """Format HTML to be readable."""
    return soup.prettify(formatter="html5")

def detect_webgl_webgpu(code):
    """Detect WebGL/WebGPU patterns in code."""
    findings = {
        "webgl": [],
        "webgpu": [],
        "has_webgl": False,
        "has_webgpu": False
    }

    for pattern in WEBGL_PATTERNS:
        matches = re.findall(pattern, code, re.IGNORECASE)
        if matches:
            findings["webgl"].extend(matches[:5])  # Limit to 5 examples
            findings["has_webgl"] = True

    for pattern in WEBGPU_PATTERNS:
        matches = re.findall(pattern, code, re.IGNORECASE)
        if matches:
            findings["webgpu"].extend(matches[:5])
            findings["has_webgpu"] = True

    # Deduplicate
    findings["webgl"] = list(set(findings["webgl"]))
    findings["webgpu"] = list(set(findings["webgpu"]))

    return findings

def main():
    # Create directories
    os.makedirs(f"{OUTPUT_DIR}/data", exist_ok=True)
    os.makedirs(f"{OUTPUT_DIR}/scripts", exist_ok=True)
    os.makedirs(f"{OUTPUT_DIR}/css", exist_ok=True)

    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0"
    }

    print(f"🕷️  CrawlWe Scraper")
    print(f"==================")
    print(f"URL: {URL}")
    print()

    # Fetch page
    print("📄 Fetching page...")
    response = requests.get(URL, headers=headers, timeout=30)
    response.raise_for_status()

    html = response.text
    soup = BeautifulSoup(html, 'lxml')

    title = soup.title.string if soup.title else "Bécane Paris"
    print(f"   Title: {title}")
    print(f"   HTML: {len(html):,} bytes")

    # ========================================
    # EXTRACT CSS
    # ========================================
    print("\n📦 Extracting CSS...")
    css_files = []
    all_css = []

    for i, link in enumerate(soup.find_all('link', rel='stylesheet')):
        href = link.get('href')
        if not href:
            continue

        full_url = urljoin(URL, href)
        filename = f"stylesheet_{i}.css"

        try:
            print(f"   Fetching: {urlparse(full_url).path[:50]}...")
            css_response = requests.get(full_url, headers=headers, timeout=10)
            if css_response.ok:
                raw_css = css_response.text
                formatted_css = format_css(raw_css)

                # Save individual CSS file
                with open(f"{OUTPUT_DIR}/css/{filename}", 'w', encoding='utf-8') as f:
                    f.write(f"/* Source: {full_url} */\n\n")
                    f.write(formatted_css)

                css_files.append({
                    "url": full_url,
                    "file": f"css/{filename}",
                    "size_raw": len(raw_css),
                    "size_formatted": len(formatted_css)
                })

                all_css.append(f"/* ========================================\n   Source: {full_url}\n   ======================================== */\n\n{formatted_css}")
                print(f"   ✓ {filename} ({len(formatted_css):,} bytes)")
        except Exception as e:
            print(f"   ✗ Failed: {e}")

    # Inline styles
    for i, style in enumerate(soup.find_all('style')):
        if style.string:
            formatted = format_css(style.string)
            filename = f"inline_{i}.css"

            with open(f"{OUTPUT_DIR}/css/{filename}", 'w', encoding='utf-8') as f:
                f.write(f"/* Inline style #{i} */\n\n")
                f.write(formatted)

            css_files.append({
                "url": None,
                "file": f"css/{filename}",
                "size_formatted": len(formatted),
                "inline": True
            })
            all_css.append(f"/* Inline style #{i} */\n\n{formatted}")
            print(f"   ✓ {filename} (inline)")

    # Combined CSS
    combined_css = "\n\n".join(all_css)
    with open(f"{OUTPUT_DIR}/styles.css", 'w', encoding='utf-8') as f:
        f.write(combined_css)
    print(f"\n   Combined: styles.css ({len(combined_css):,} bytes)")

    # ========================================
    # EXTRACT JAVASCRIPT
    # ========================================
    print("\n📦 Extracting JavaScript...")
    js_files = []
    all_js_code = []
    webgl_findings = {"webgl": [], "webgpu": [], "has_webgl": False, "has_webgpu": False}

    # External scripts
    for i, script in enumerate(soup.find_all('script', src=True)):
        src = script.get('src')
        if not src:
            continue

        full_url = urljoin(URL, src)
        filename = f"external_{i}.js"

        try:
            # Only fetch same-origin scripts (avoid third-party)
            if urlparse(full_url).netloc == urlparse(URL).netloc:
                print(f"   Fetching: {urlparse(full_url).path[:50]}...")
                js_response = requests.get(full_url, headers=headers, timeout=10)
                if js_response.ok:
                    js_code = js_response.text

                    # Save JS file
                    with open(f"{OUTPUT_DIR}/scripts/{filename}", 'w', encoding='utf-8') as f:
                        f.write(f"// Source: {full_url}\n\n")
                        f.write(js_code)

                    # Detect WebGL/WebGPU
                    findings = detect_webgl_webgpu(js_code)

                    js_files.append({
                        "url": full_url,
                        "file": f"scripts/{filename}",
                        "size": len(js_code),
                        "has_webgl": findings["has_webgl"],
                        "has_webgpu": findings["has_webgpu"]
                    })

                    all_js_code.append(js_code)

                    if findings["has_webgl"]:
                        webgl_findings["has_webgl"] = True
                        webgl_findings["webgl"].extend(findings["webgl"])
                        print(f"   ✓ {filename} ({len(js_code):,} bytes) [WebGL detected!]")
                    elif findings["has_webgpu"]:
                        webgl_findings["has_webgpu"] = True
                        webgl_findings["webgpu"].extend(findings["webgpu"])
                        print(f"   ✓ {filename} ({len(js_code):,} bytes) [WebGPU detected!]")
                    else:
                        print(f"   ✓ {filename} ({len(js_code):,} bytes)")
            else:
                js_files.append({
                    "url": full_url,
                    "file": None,
                    "external": True,
                    "note": "Third-party script (not downloaded)"
                })
                print(f"   → {urlparse(full_url).netloc} (third-party, skipped)")
        except Exception as e:
            print(f"   ✗ Failed: {e}")

    # Inline scripts
    inline_count = 0
    for i, script in enumerate(soup.find_all('script')):
        if script.get('src'):
            continue
        if not script.string or len(script.string.strip()) < 50:
            continue

        js_code = script.string
        filename = f"inline_{inline_count}.js"

        # Detect WebGL/WebGPU
        findings = detect_webgl_webgpu(js_code)

        # Save inline script
        with open(f"{OUTPUT_DIR}/scripts/{filename}", 'w', encoding='utf-8') as f:
            f.write(f"// Inline script #{inline_count}\n\n")
            f.write(js_code)

        js_files.append({
            "url": None,
            "file": f"scripts/{filename}",
            "size": len(js_code),
            "inline": True,
            "has_webgl": findings["has_webgl"],
            "has_webgpu": findings["has_webgpu"]
        })

        all_js_code.append(js_code)

        if findings["has_webgl"]:
            webgl_findings["has_webgl"] = True
            webgl_findings["webgl"].extend(findings["webgl"])
            print(f"   ✓ {filename} ({len(js_code):,} bytes) [WebGL detected!]")
        elif findings["has_webgpu"]:
            webgl_findings["has_webgpu"] = True
            webgl_findings["webgpu"].extend(findings["webgpu"])
            print(f"   ✓ {filename} ({len(js_code):,} bytes) [WebGPU detected!]")
        else:
            print(f"   ✓ {filename} ({len(js_code):,} bytes)")

        inline_count += 1

    # Deduplicate WebGL/WebGPU findings
    webgl_findings["webgl"] = list(set(webgl_findings["webgl"]))[:20]
    webgl_findings["webgpu"] = list(set(webgl_findings["webgpu"]))[:20]

    # ========================================
    # EXTRACT OTHER DATA
    # ========================================
    print("\n📦 Extracting metadata...")

    # Images
    images = []
    for img in soup.find_all('img'):
        src = img.get('src') or img.get('data-src')
        if src:
            images.append(urljoin(URL, src))
    for el in soup.find_all(srcset=True):
        srcset = el.get('srcset', '')
        for part in srcset.split(','):
            src = part.strip().split()[0]
            if src:
                images.append(urljoin(URL, src))
    images = list(set(images))
    print(f"   Images: {len(images)}")

    # Links
    links = []
    for a in soup.find_all('a', href=True):
        href = a.get('href')
        if href and not href.startswith('#') and not href.startswith('javascript:'):
            links.append(urljoin(URL, href))
    links = list(set(links))
    print(f"   Links: {len(links)}")

    # ========================================
    # FORMAT AND SAVE HTML
    # ========================================
    print("\n📄 Formatting HTML...")

    # Clean soup for output
    clean_soup = BeautifulSoup(html, 'lxml')

    # Remove scripts and styles (we have them separately)
    for tag in clean_soup.find_all(['script', 'noscript']):
        tag.decompose()
    for tag in clean_soup.find_all('style'):
        tag.decompose()
    for tag in clean_soup.find_all('link', rel='stylesheet'):
        tag.decompose()

    # Add our stylesheet link
    if clean_soup.head:
        # Add meta charset
        meta_charset = clean_soup.new_tag('meta', charset='utf-8')
        clean_soup.head.insert(0, meta_charset)

        # Add viewport
        meta_viewport = clean_soup.new_tag('meta')
        meta_viewport['name'] = 'viewport'
        meta_viewport['content'] = 'width=device-width, initial-scale=1'
        clean_soup.head.insert(1, meta_viewport)

        # Add stylesheet
        link_css = clean_soup.new_tag('link', rel='stylesheet', href='styles.css')
        clean_soup.head.insert(2, link_css)

    formatted_html = format_html(clean_soup)

    with open(f"{OUTPUT_DIR}/index.html", 'w', encoding='utf-8') as f:
        f.write(formatted_html)
    print(f"   ✓ index.html ({len(formatted_html):,} bytes)")

    # Save raw HTML
    with open(f"{OUTPUT_DIR}/data/raw.html", 'w', encoding='utf-8') as f:
        f.write(html)
    print(f"   ✓ data/raw.html ({len(html):,} bytes)")

    # ========================================
    # SAVE DATA FILES
    # ========================================
    print("\n💾 Saving data files...")

    with open(f"{OUTPUT_DIR}/data/css_files.json", 'w', encoding='utf-8') as f:
        json.dump(css_files, f, indent=2)
    print(f"   ✓ data/css_files.json")

    with open(f"{OUTPUT_DIR}/data/js_files.json", 'w', encoding='utf-8') as f:
        json.dump(js_files, f, indent=2)
    print(f"   ✓ data/js_files.json")

    with open(f"{OUTPUT_DIR}/data/images.json", 'w', encoding='utf-8') as f:
        json.dump(images, f, indent=2)
    print(f"   ✓ data/images.json")

    with open(f"{OUTPUT_DIR}/data/links.json", 'w', encoding='utf-8') as f:
        json.dump(links, f, indent=2)
    print(f"   ✓ data/links.json")

    with open(f"{OUTPUT_DIR}/data/webgl_webgpu.json", 'w', encoding='utf-8') as f:
        json.dump(webgl_findings, f, indent=2)
    print(f"   ✓ data/webgl_webgpu.json")

    # Metadata
    metadata = {
        "url": URL,
        "title": title,
        "stats": {
            "html_bytes": len(html),
            "html_formatted_bytes": len(formatted_html),
            "css_total_bytes": len(combined_css),
            "css_files_count": len(css_files),
            "js_files_count": len(js_files),
            "images_count": len(images),
            "links_count": len(links)
        },
        "webgl_webgpu": {
            "has_webgl": webgl_findings["has_webgl"],
            "has_webgpu": webgl_findings["has_webgpu"],
            "webgl_patterns_found": len(webgl_findings["webgl"]),
            "webgpu_patterns_found": len(webgl_findings["webgpu"])
        }
    }
    with open(f"{OUTPUT_DIR}/metadata.json", 'w', encoding='utf-8') as f:
        json.dump(metadata, f, indent=2)
    print(f"   ✓ metadata.json")

    # ========================================
    # SUMMARY
    # ========================================
    print(f"\n{'='*50}")
    print(f"✅ DONE!")
    print(f"{'='*50}")
    print(f"\n📁 Output: {OUTPUT_DIR}/")
    print(f"   ├── index.html      ({len(formatted_html):,} bytes, formatted)")
    print(f"   ├── styles.css      ({len(combined_css):,} bytes, formatted)")
    print(f"   ├── css/            ({len(css_files)} files)")
    print(f"   ├── scripts/        ({len(js_files)} files)")
    print(f"   ├── data/")
    print(f"   │   ├── raw.html")
    print(f"   │   ├── css_files.json")
    print(f"   │   ├── js_files.json")
    print(f"   │   ├── images.json ({len(images)} images)")
    print(f"   │   ├── links.json  ({len(links)} links)")
    print(f"   │   └── webgl_webgpu.json")
    print(f"   └── metadata.json")

    if webgl_findings["has_webgl"]:
        print(f"\n🎮 WebGL DETECTED!")
        print(f"   Patterns: {webgl_findings['webgl'][:5]}")

    if webgl_findings["has_webgpu"]:
        print(f"\n🎮 WebGPU DETECTED!")
        print(f"   Patterns: {webgl_findings['webgpu'][:5]}")

if __name__ == "__main__":
    main()
