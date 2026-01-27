#!/usr/bin/env python3
"""Quick scraper for becaneparis.com"""

import requests
from bs4 import BeautifulSoup
from urllib.parse import urljoin, urlparse
import json
import os
import re

URL = "https://www.becaneparis.com/"
OUTPUT_DIR = "output/becaneparis"

def main():
    os.makedirs(f"{OUTPUT_DIR}/data", exist_ok=True)
    os.makedirs(f"{OUTPUT_DIR}/assets", exist_ok=True)

    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    }

    print(f"Fetching {URL}...")
    response = requests.get(URL, headers=headers, timeout=30)
    response.raise_for_status()

    html = response.text
    soup = BeautifulSoup(html, 'lxml')

    print(f"HTML: {len(html)} bytes")

    # Get title
    title = soup.title.string if soup.title else "Bécane Paris"
    print(f"Title: {title}")

    # Collect all CSS
    css_parts = []
    css_urls = []

    # External stylesheets
    for link in soup.find_all('link', rel='stylesheet'):
        href = link.get('href')
        if href:
            full_url = urljoin(URL, href)
            css_urls.append(full_url)
            try:
                print(f"Fetching CSS: {full_url[:60]}...")
                css_response = requests.get(full_url, headers=headers, timeout=10)
                if css_response.ok:
                    css_parts.append(f"/* Source: {full_url} */\n{css_response.text}")
            except Exception as e:
                css_parts.append(f"/* Failed to fetch: {full_url} - {e} */")

    # Inline styles
    for style in soup.find_all('style'):
        if style.string:
            css_parts.append(f"/* Inline style */\n{style.string}")

    css = "\n\n".join(css_parts)
    print(f"CSS: {len(css)} bytes from {len(css_urls)} stylesheets")

    # Get images
    images = []
    for img in soup.find_all('img'):
        src = img.get('src') or img.get('data-src')
        if src:
            images.append(urljoin(URL, src))

    # Get srcset images
    for el in soup.find_all(srcset=True):
        srcset = el.get('srcset', '')
        for part in srcset.split(','):
            src = part.strip().split()[0]
            if src:
                images.append(urljoin(URL, src))

    images = list(set(images))
    print(f"Images: {len(images)} found")

    # Get links
    links = []
    for a in soup.find_all('a', href=True):
        href = a.get('href')
        if href and not href.startswith('#'):
            links.append(urljoin(URL, href))
    links = list(set(links))
    print(f"Links: {len(links)} found")

    # Get scripts
    scripts = []
    for script in soup.find_all('script'):
        src = script.get('src')
        if src:
            scripts.append({"src": urljoin(URL, src), "inline": False})
        elif script.string:
            scripts.append({"src": None, "inline": True, "length": len(script.string)})
    print(f"Scripts: {len(scripts)} found")

    # Clean HTML for output
    clean_soup = BeautifulSoup(html, 'lxml')

    # Remove scripts and styles (we have them separately)
    for tag in clean_soup.find_all(['script', 'style', 'noscript']):
        tag.decompose()

    # Add our stylesheet link
    if clean_soup.head:
        new_link = clean_soup.new_tag('link', rel='stylesheet', href='styles.css')
        clean_soup.head.insert(0, new_link)

    clean_html = str(clean_soup)

    # Save files
    print("\nSaving files...")

    with open(f"{OUTPUT_DIR}/index.html", 'w', encoding='utf-8') as f:
        f.write(clean_html)
    print(f"  ✓ index.html ({len(clean_html)} bytes)")

    with open(f"{OUTPUT_DIR}/styles.css", 'w', encoding='utf-8') as f:
        f.write(css)
    print(f"  ✓ styles.css ({len(css)} bytes)")

    with open(f"{OUTPUT_DIR}/data/raw.html", 'w', encoding='utf-8') as f:
        f.write(html)
    print(f"  ✓ data/raw.html")

    with open(f"{OUTPUT_DIR}/data/images.json", 'w', encoding='utf-8') as f:
        json.dump(images, f, indent=2)
    print(f"  ✓ data/images.json ({len(images)} images)")

    with open(f"{OUTPUT_DIR}/data/links.json", 'w', encoding='utf-8') as f:
        json.dump(links, f, indent=2)
    print(f"  ✓ data/links.json ({len(links)} links)")

    with open(f"{OUTPUT_DIR}/data/scripts.json", 'w', encoding='utf-8') as f:
        json.dump(scripts, f, indent=2)
    print(f"  ✓ data/scripts.json ({len(scripts)} scripts)")

    with open(f"{OUTPUT_DIR}/data/css_sources.json", 'w', encoding='utf-8') as f:
        json.dump(css_urls, f, indent=2)
    print(f"  ✓ data/css_sources.json")

    # Metadata
    metadata = {
        "url": URL,
        "title": title,
        "stats": {
            "html_bytes": len(html),
            "css_bytes": len(css),
            "css_sources": len(css_urls),
            "images": len(images),
            "links": len(links),
            "scripts": len(scripts)
        }
    }
    with open(f"{OUTPUT_DIR}/metadata.json", 'w', encoding='utf-8') as f:
        json.dump(metadata, f, indent=2)
    print(f"  ✓ metadata.json")

    print(f"\n✅ Done! Output in {OUTPUT_DIR}/")

if __name__ == "__main__":
    main()
