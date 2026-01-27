# CrawlWe

**Web Page Extractor for ML Training** - Captures clean HTML + CSS from web pages to train AI models that generate more natural designs.

## Features

- **Full Page Capture**: Renders JavaScript-heavy pages with Chromium
- **CSS Extraction**: Captures all computed styles, variables, keyframes
- **Library Detection**: Detects 50+ libraries (GSAP, Three.js, React, etc.)
- **VM Parser**: Extensible bytecode VM for custom parsing
- **Project Export**: Generates `project.toml` with dependencies

## Installation

### From Source (Rust)

```bash
# Build the CLI
cargo build --release

# Install globally
cargo install --path .
```

### Python Bindings

```bash
# Install with maturin
pip install maturin
maturin develop

# Use from Python
python -c "from crawlwe import analyze; print(analyze('<div>test</div>', '', ''))"
```

## CLI Usage

```bash
# Capture a single page
crawlwe fetch https://example.com -o output/

# Batch capture from file
crawlwe batch urls.txt -o captures/

# Analyze captured content
crawlwe analyze ./output

# Generate project.toml
crawlwe export ./output

# Parse with VM (advanced)
crawlwe parse styles.css --program css_analyzer
```

## Output Structure

```
output/
├── index.html       # Clean HTML (scripts removed)
├── styles.css       # Unified CSS
├── project.toml     # Dependencies and metadata
├── screenshot.png   # Full page screenshot
└── data/
    ├── raw.html     # Original rendered HTML
    └── metadata.json
```

## Python API

```python
from crawlwe import analyze, detect_tailwind, generate_config

# Analyze content
result = analyze(html, css, js)
print(result['css']['variables'])
print(result['libraries'])

# Detect frameworks
tailwind = detect_tailwind(html, css)

# Generate project config
toml = generate_config("https://example.com", html, css, js, "My Page")
```

## Architecture

```
Input (URL)
    │
    ▼
┌─────────────────┐
│  Browser Capture │  chromiumoxide (Rust)
│  HTML + CSS + JS │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Microparsers   │  CSS / JS / Library detection
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│       VM        │  Bytecode execution (optional)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Optimizer     │  Clean, deduplicate, format
└────────┬────────┘
         │
         ▼
Output (index.html, styles.css, project.toml)
```

## Library Detection

Detects 50+ libraries including:

- **Animation**: GSAP, Anime.js, Motion One, Framer Motion
- **3D/Graphics**: Three.js, Babylon.js, PixiJS, P5.js
- **UI**: React, Vue, Svelte, Alpine.js
- **CSS**: Tailwind, Bootstrap, Bulma
- **Scroll**: Locomotive Scroll, Lenis, ScrollMagic

## Project Structure

```
crawlwe/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # PyO3 bindings
│   ├── commands/         # CLI commands
│   │   ├── fetch.rs      # Single page capture
│   │   ├── batch.rs      # Batch capture
│   │   ├── analyze.rs    # Content analysis
│   │   ├── export.rs     # Project.toml generation
│   │   └── parse.rs      # VM parser
│   ├── pipeline/
│   │   ├── microparsers.rs  # CSS/JS/Lib parsers
│   │   └── optimizer.rs     # Content optimization
│   ├── vm/
│   │   ├── opcodes.rs    # Bytecode instructions
│   │   ├── vm.rs         # Virtual machine
│   │   └── compiler.rs   # DSL compiler
│   └── export/           # Project config generation
├── crawlwe/              # Python wrapper (legacy)
├── captures/             # Example captures
└── Cargo.toml
```

## License

MIT
