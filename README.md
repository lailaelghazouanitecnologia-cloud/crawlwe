# CrawlWe

**Web Page Extractor for ML Training** - Captures clean HTML + CSS from web pages to train AI models that generate more natural designs.

## Installation (Windows/Linux/Mac)

```bash
# Just install it!
pip install -e .

# Or install from the directory
cd crawlwe
pip install .
```

That's it! No Rust required.

## Usage

### Command Line

```bash
# Capture a single page
python -m crawlwe fetch https://example.com -o output/

# Batch capture multiple URLs
python -m crawlwe batch seeds.txt -o output/ --concurrent 5

# Analyze a page (without saving)
python -m crawlwe analyze https://stripe.com

# Show system info
python -m crawlwe info
```

### Python API

```python
from crawlwe import capture, batch_capture, CaptureOptions
from crawlwe.writer import write_mockup
from pathlib import Path

# Single capture
result = capture("https://example.com")
print(f"Elements: {result.element_count}")
print(f"CSS Rules: {result.rule_count}")
print(f"Colors: {result.colors}")

# Save to disk
write_mockup(result, Path("output"))

# Batch capture
urls = ["https://example.com", "https://example.org"]
for result in batch_capture(urls, concurrent=3):
    if not result.error:
        write_mockup(result, Path("output"))
```

## Output Structure

```
output/example.com/
├── index.html      # Clean HTML (no scripts, cleaned attributes)
├── styles.css      # Unified CSS (all sources merged)
├── screenshot.png  # Visual reference
└── metadata.json   # Colors, fonts, CSS variables
```

## Windows Instructions

On Windows, after `pip install`, you may need to use `python -m` prefix:

```powershell
# Instead of: crawlwe fetch ...
python -m crawlwe fetch https://example.com -o output/

# Or add Python Scripts to PATH:
# %APPDATA%\Python\Python3X\Scripts
```

## Features

| Feature | Python-only | With Rust |
|---------|:-----------:|:---------:|
| Basic HTML/CSS extraction | ✅ | ✅ |
| CSS deduplication | ❌ | ✅ |
| Capture :hover/:focus states | ❌ | ✅ |
| Chrome headless | ❌ | ✅ |
| Speed | Slow | Fast |

## Project Structure

```
crawlwe/
├── crawlwe/           # Python package
│   ├── cli.py         # Command-line interface
│   ├── pipeline.py    # Capture orchestration
│   └── writer.py      # Output handling
├── rust/src/          # Rust core (optional)
│   ├── browser.rs     # Chrome CDP client
│   ├── css_parser.rs  # CSS optimization
│   └── html_parser.rs # HTML cleaning
├── setup.py           # Python-only install
└── pyproject.toml     # Full install with Rust
```

## License

MIT
