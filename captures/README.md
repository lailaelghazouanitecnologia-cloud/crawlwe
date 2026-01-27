# Captures Directory

This directory contains captured web pages. Each capture creates a subdirectory with the following structure:

```
captures/
├── example.com/
│   ├── index.html      # Clean HTML (scripts/styles removed)
│   ├── styles.css      # Combined CSS (inline + external)
│   ├── scripts/
│   │   └── main.js     # Placeholder script
│   ├── data/
│   │   ├── raw.html    # Original HTML
│   │   ├── metadata.json
│   │   └── analysis.json
│   ├── project.toml    # Project configuration
│   └── screenshot.png  # Full page screenshot
```

## Usage

```bash
# Capture with Python/Playwright (JS-heavy pages)
python scripts/capture_page.py https://example.com -o captures/example

# Capture with CLI
crawlwe fetch https://example.com -o captures/example

# Analyze captured content
crawlwe analyze captures/example
```

## Project Configuration

Each capture generates a `project.toml` with detected technologies:

```toml
[meta]
name = "example-site"
version = "1.0.0"

[source]
url = "https://example.com"

[technologies]
css_framework = "Tailwind"
js_libraries = ["GSAP", "Three.js"]

[dependencies.scripts]
gsap = { cdn = "https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.2/gsap.min.js" }
```
