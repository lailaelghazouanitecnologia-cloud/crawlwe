# CLAUDE.md - AI Assistant Guide for CrawlWe

## Project Overview

**CrawlWe** is a web page extractor and analyzer designed for ML training data collection. It captures clean, structured HTML and CSS from dynamic web pages to enable training of AI models that generate more natural, human-like designs.

- **Version:** 0.1.0 (Alpha)
- **License:** MIT
- **Python:** >=3.10
- **Architecture:** Hybrid Python/Rust with Python-first fallback

### Key Goals
- Extract fully-rendered HTML + CSS + metadata from JavaScript-heavy websites
- Analyze animations, transitions, UI patterns, CSS frameworks, and 3D/WebGL technologies
- Provide clean training data for ML models

## Directory Structure

```
crawlwe/
├── crawlwe/                 # Main Python package (primary entry point)
│   ├── __init__.py          # Package init, Rust core imports with fallback
│   ├── cli.py               # Click-based CLI (6 commands)
│   ├── pipeline.py          # Orchestration: capture() and batch_capture()
│   └── writer.py            # Output: mockups, datasets, merging
│
├── vm/                      # Virtual Machine for rule-based parsing
│   ├── __init__.py          # Module exports
│   ├── dsl.py               # Domain-specific language lexer/parser
│   ├── opcodes.py           # VM instruction definitions (50+ opcodes)
│   ├── compiler.py          # AST to bytecode compiler
│   └── vm.py                # VM execution engine
│
├── src/                     # Rust binary CLI
│   └── main.rs              # Chrome DevTools Protocol client
│
├── rust/                    # Rust library (Python bindings via maturin)
│   └── src/                 # Native extension module source
│
├── rules/                   # Rule definitions
│   └── default.crawl        # DSL rules for CSS/JS/HTML analysis
│
├── scripts/                 # Example and utility scripts
│   └── example_usage.py     # Library API usage examples
│
├── data/seeds/              # Seed URLs for batch processing
├── output/                  # Captured mockups (output directory)
│
├── crawlwe.py               # Legacy unified CLI dispatcher
├── scrape.py                # Raw scraper with WebGL detection
├── analyze.py               # Deep analyzer for patterns
└── test_vm.py               # VM test suite
```

## Quick Commands

### Installation
```bash
# Python-only (default)
pip install -e .

# With Rust acceleration (optional)
pip install maturin
maturin develop --release
```

### Running the CLI
```bash
# Primary CLI entry point
crawlwe fetch <url> -o output/           # Single page capture
crawlwe batch seeds.txt -o output/ -c 5  # Batch capture
crawlwe analyze <url> [--json]           # Quick analysis
crawlwe optimize <css_file> [-o out]     # CSS optimization
crawlwe info                             # System info

# Alternative invocation
python -m crawlwe <command>
```

### Running Tests
```bash
# Run VM tests
python test_vm.py

# Run with pytest (if installed)
pytest test_vm.py -v

# Test specific VM components
python -c "from test_vm import test_lexer_parser; test_lexer_parser()"
```

### Code Quality
```bash
# Format code
black . --line-length 100

# Lint
ruff check .

# Type check
mypy crawlwe/ vm/
```

## Architecture

### Fallback Pattern
The codebase uses a Rust-first-with-Python-fallback architecture:
```python
try:
    result = _capture_with_rust(url, options)
except ImportError:
    result = _capture_with_python(url, options)
```

### Key Components

1. **Pipeline (`crawlwe/pipeline.py`)** - Core orchestration
   - `CaptureOptions`: Configuration dataclass
   - `CaptureResult`: Output dataclass with HTML, CSS, metadata
   - `capture()`: Single URL capture
   - `batch_capture()`: Concurrent multi-URL capture

2. **Writer (`crawlwe/writer.py`)** - Output handling
   - `write_mockup()`: Save capture to directory
   - `merge_mockups()`: Combine multiple captures
   - `export_dataset()`: Export for ML training (JSONL/JSON/CSV)

3. **VM System (`vm/`)** - Rule-based parsing engine
   - DSL for defining parsing rules
   - Bytecode compiler for performance
   - 50+ domain-specific opcodes (CSS_FIND_KEYFRAMES, JS_FIND_CALL, etc.)

4. **Analyzers (`scrape.py`, `analyze.py`)** - Detection capabilities
   - CSS animations/transitions/transforms
   - 3D libraries (Three.js, Babylon.js, PixiJS, A-Frame)
   - WebGL/WebGPU usage patterns
   - CSS frameworks (Tailwind, CSS Modules)
   - Animation libraries (GSAP, Anime.js, Lottie)

## Code Conventions

### Style
- **Line length:** 100 characters
- **Formatter:** Black
- **Linter:** Ruff (rules: E, F, W, I, N, UP, B, C4)
- **Type checking:** mypy strict mode

### Naming
- **Functions/variables:** `snake_case`
- **Classes:** `PascalCase`
- **Constants:** `UPPER_CASE`
- **Private functions:** `_leading_underscore`
- **CLI commands:** lowercase verbs (fetch, batch, analyze)

### Patterns
- **Dataclasses** for configuration and results
- **Type hints** on all functions (Python 3.10+ syntax)
- **Docstrings** on modules and public functions
- **Try/except** with graceful degradation

### Import Order
```python
# Standard library
import os
from pathlib import Path

# Third-party
import click
from rich.console import Console

# Local
from crawlwe.pipeline import capture
from vm import VM, Compiler
```

## Dependencies

### Core (always installed)
- `click>=8.1.0` - CLI framework
- `rich>=13.0.0` - Terminal UI/formatting
- `beautifulsoup4>=4.12.0` - HTML parsing
- `lxml>=5.0.0` - Fast XML parsing
- `cssbeautifier>=1.14.0` - CSS formatting
- `requests>=2.31.0` - HTTP client

### Optional (pip install -e ".[full]")
- `aiohttp>=3.9.0` - Async HTTP
- `aiofiles>=23.0.0` - Async file I/O
- `pydantic>=2.5.0` - Data validation

### Development (pip install -e ".[dev]")
- `pytest>=7.0.0` - Testing
- `pytest-asyncio>=0.23.0` - Async tests
- `black>=24.0.0` - Formatter
- `ruff>=0.1.0` - Linter
- `mypy>=1.8.0` - Type checker

## Key Files Reference

| File | Purpose |
|------|---------|
| `crawlwe/cli.py` | Main CLI entry point (Click commands) |
| `crawlwe/pipeline.py` | Core capture logic and orchestration |
| `crawlwe/writer.py` | Output formatting and file writing |
| `vm/dsl.py` | DSL lexer and parser implementation |
| `vm/opcodes.py` | VM instruction definitions |
| `vm/compiler.py` | DSL to bytecode compiler |
| `vm/vm.py` | Bytecode execution engine |
| `rules/default.crawl` | Default parsing rules |
| `scrape.py` | Raw HTML/CSS extraction with WebGL detection |
| `analyze.py` | Deep pattern analysis |
| `test_vm.py` | VM component tests |
| `pyproject.toml` | Build configuration and dependencies |

## Common Tasks for AI Assistants

### Adding a New CLI Command
1. Edit `crawlwe/cli.py`
2. Add a new `@cli.command()` decorated function
3. Use Click decorators for options/arguments
4. Follow existing command patterns (fetch, analyze, etc.)

### Adding Detection for a New Technology
1. Edit `scrape.py` for pattern detection
2. Add patterns to relevant detection arrays (e.g., `THREE_JS_PATTERNS`)
3. Update `analyze.py` if deep analysis is needed
4. Consider adding VM rules in `rules/default.crawl`

### Extending the VM
1. Add new opcodes in `vm/opcodes.py`
2. Implement handlers in `vm/vm.py` `_execute_instruction()`
3. Add compiler support in `vm/compiler.py`
4. Update DSL grammar in `vm/dsl.py` if new syntax needed

### Adding Output Formats
1. Edit `crawlwe/writer.py`
2. Add new export function following `export_dataset()` pattern
3. Update CLI if command-line option needed

## Output Structure

Captures are saved as mockup directories:
```
output/<domain>/
├── index.html      # Clean HTML (no scripts, sanitized)
├── styles.css      # Unified CSS (all sources merged)
├── screenshot.png  # Visual reference (base64 decoded)
└── metadata.json   # Colors, fonts, CSS variables, stats
```

## Testing

### VM Tests (`test_vm.py`)
- `test_lexer_parser()` - DSL tokenization and AST generation
- `test_compiler()` - Bytecode compilation
- `test_vm()` - Instruction execution
- `test_full_rules()` - Real rules file parsing

### Running Tests
```bash
# All tests
python test_vm.py

# Specific test
python -c "from test_vm import test_vm; test_vm()"
```

## Notes for AI Assistants

1. **Rust is optional** - Always ensure Python fallback works
2. **Chrome required** - For full dynamic page rendering (CDP)
3. **VM system is self-contained** - `vm/` can be tested independently
4. **Rich library** - Used for all terminal output formatting
5. **Type hints are strict** - mypy strict mode is enabled
6. **Line length 100** - Enforced by Black and Ruff
