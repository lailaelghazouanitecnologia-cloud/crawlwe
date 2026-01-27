# CrawlWe

Fast web page scraper using Chrome DevTools Protocol. Extracts fully rendered HTML + CSS + Assets from dynamic JavaScript pages.

## Requirements

- [Rust](https://rustup.rs/)
- Chrome or Chromium browser

## Installation

```bash
git clone <repo>
cd crawlwe
cargo build --release
```

## Usage

```bash
# Basic capture
./target/release/crawlwe --url "https://example.com" --output output/example

# Wait longer for heavy JS sites
./target/release/crawlwe --url "https://www.becaneparis.com/" --output output/becane --wait 15

# Download images and fonts
./target/release/crawlwe --url "https://stripe.dev" --output output/stripe --download-images --download-fonts

# Custom viewport
./target/release/crawlwe --url "https://example.com" --output output/mobile --width 375 --height 812
```

## Output Structure

```
output/
├── index.html          # Clean HTML (scripts removed)
├── styles.css          # All CSS merged
├── project.toml        # Dependencies and metadata
├── metadata.json       # Capture stats
├── screenshot.png      # Viewport screenshot
├── screenshot_full.png # Full page screenshot
├── data/
│   ├── raw.html        # Original rendered HTML
│   ├── images.json     # Image URLs
│   ├── fonts.json      # Font info
│   └── links.json      # All links
├── scripts/
│   ├── scripts.json    # Script info
│   └── inline_*.js     # Inline scripts
└── assets/
    ├── images/         # Downloaded images
    └── fonts/          # Downloaded fonts
```

## Features

- **Chrome DevTools Protocol**: Full JavaScript rendering
- **CSS Extraction**: Inline styles + external stylesheets
- **Library Detection**: 30+ libraries (GSAP, Three.js, React, etc.)
- **Asset Extraction**: Images, fonts, scripts, links
- **Screenshots**: Viewport and full page
- **Lazy Loading**: Scrolls page to trigger lazy content
- **Project Config**: Generates project.toml with dependencies

## Detected Libraries

- **Animation**: GSAP, Anime.js, Framer Motion, Lottie
- **3D/Graphics**: Three.js, Babylon.js, PixiJS, P5.js
- **Scroll**: Locomotive Scroll, Lenis, ScrollMagic
- **UI**: React, Vue, Svelte, Alpine.js
- **Frameworks**: Next.js, Nuxt
- **CSS**: Tailwind, Bootstrap, Bulma
- **Platforms**: Shopify, WordPress, Sanity

## License

MIT
