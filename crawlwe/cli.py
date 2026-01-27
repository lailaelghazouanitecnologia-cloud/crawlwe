"""
CrawlWe CLI - Command Line Interface

Usage:
    crawlwe fetch <url> -o output/
    crawlwe batch seeds.txt -o output/ --concurrent 10
    crawlwe analyze <directory>
    crawlwe export <directory> -o dataset.jsonl
    crawlwe info
"""

import json
import sys
from pathlib import Path

import click
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn
from rich.table import Table
from rich.panel import Panel

console = Console()


@click.group()
@click.version_option(version="0.1.0", prog_name="crawlwe")
def main():
    """CrawlWe - Web Page Extractor for ML Training

    Extract clean HTML + CSS from web pages to train AI models
    that generate more natural, human-like designs.
    """
    pass


@main.command()
@click.argument("url")
@click.option("-o", "--output", default="output", help="Output directory")
@click.option("--no-screenshot", is_flag=True, help="Skip taking screenshots")
@click.option("--width", default=1440, help="Viewport width")
@click.option("--height", default=900, help="Viewport height")
def fetch(url: str, output: str, no_screenshot: bool, width: int, height: int):
    """Fetch a single URL and extract HTML + CSS."""
    from crawlwe.pipeline import capture, CaptureOptions
    from crawlwe.writer import write_mockup

    console.print(Panel(f"[bold blue]CrawlWe[/] - Capturing: {url}"))

    options = CaptureOptions(
        capture_screenshot=not no_screenshot,
        viewport_width=width,
        viewport_height=height,
    )

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        transient=True,
        console=console,
    ) as progress:
        task = progress.add_task("Capturing page...", total=None)

        try:
            result = capture(url, options)
            progress.update(task, description="Writing files...")

            output_path = Path(output)
            mockup_path = write_mockup(result, output_path)

        except Exception as e:
            console.print(f"[red]Error:[/] {e}")
            sys.exit(1)

    # Show results
    table = Table(title="Capture Results")
    table.add_column("Property", style="cyan")
    table.add_column("Value", style="green")

    table.add_row("URL", result.url)
    table.add_row("Title", result.title or "N/A")
    table.add_row("Elements", str(result.element_count))
    table.add_row("CSS Rules", str(result.rule_count))
    table.add_row("Colors", str(len(result.colors)))
    table.add_row("Fonts", str(len(result.fonts)))
    table.add_row("Output", str(mockup_path))

    console.print(table)
    console.print(f"\n[green]OK[/] Saved to: [bold]{mockup_path}[/]")


@main.command()
@click.argument("seeds_file", type=click.Path(exists=True))
@click.option("-o", "--output", default="output", help="Output directory")
@click.option("-c", "--concurrent", default=5, help="Concurrent captures")
@click.option("--no-screenshot", is_flag=True, help="Skip taking screenshots")
def batch(seeds_file: str, output: str, concurrent: int, no_screenshot: bool):
    """Batch capture multiple URLs from a seeds file."""
    from crawlwe.pipeline import batch_capture, CaptureOptions
    from crawlwe.writer import write_mockup

    with open(seeds_file) as f:
        urls = [line.strip() for line in f if line.strip() and not line.startswith("#")]

    if not urls:
        console.print("[yellow]No URLs found in seeds file[/]")
        sys.exit(1)

    console.print(Panel(f"[bold blue]CrawlWe[/] - Batch Capture: {len(urls)} URLs"))

    options = CaptureOptions(capture_screenshot=not no_screenshot)
    output_path = Path(output)
    output_path.mkdir(parents=True, exist_ok=True)

    success_count = 0
    error_count = 0

    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TaskProgressColumn(),
        console=console,
    ) as progress:
        task = progress.add_task("Capturing...", total=len(urls))

        for result in batch_capture(urls, options, concurrent=concurrent):
            if result.error:
                error_count += 1
                console.print(f"[red]X[/] {result.url}: {result.error}")
            else:
                success_count += 1
                write_mockup(result, output_path)
                console.print(f"[green]OK[/] {result.url}")

            progress.advance(task)

    console.print()
    console.print(Panel(f"[green]Success: {success_count}[/] | [red]Errors: {error_count}[/]"))


@main.command()
@click.argument("directory", type=click.Path(exists=True))
@click.option("--json", "as_json", is_flag=True, help="Output as JSON")
def analyze(directory: str, as_json: bool):
    """Analyze scraped content in a directory."""
    from crawlwe import analyze as run_analysis, RUST_AVAILABLE

    dir_path = Path(directory)

    # Load content
    css_content = ""
    css_file = dir_path / "styles.css"
    if css_file.exists():
        css_content = css_file.read_text(encoding='utf-8', errors='ignore')

    css_dir = dir_path / "css"
    if css_dir.exists():
        for css_path in css_dir.glob("*.css"):
            css_content += "\n" + css_path.read_text(encoding='utf-8', errors='ignore')

    js_content = ""
    scripts_dir = dir_path / "scripts"
    if scripts_dir.exists():
        for js_path in scripts_dir.glob("*.js"):
            js_content += "\n" + js_path.read_text(encoding='utf-8', errors='ignore')

    html_content = ""
    html_file = dir_path / "index.html"
    if html_file.exists():
        html_content = html_file.read_text(encoding='utf-8', errors='ignore')
    raw_html = dir_path / "data" / "raw.html"
    if raw_html.exists():
        html_content = raw_html.read_text(encoding='utf-8', errors='ignore')

    if not RUST_AVAILABLE:
        console.print("[yellow]Warning:[/] Rust core not available. Build with: maturin develop")

    console.print(f"[dim]Loaded: CSS={len(css_content)} JS={len(js_content)} HTML={len(html_content)} bytes[/]")

    with console.status("Analyzing..."):
        result = run_analysis(html_content, css_content, js_content)

    if as_json:
        console.print(json.dumps(result, indent=2))
    else:
        _print_analysis_report(result)

    # Save analysis
    analysis_file = dir_path / "data" / "analysis.json"
    analysis_file.parent.mkdir(parents=True, exist_ok=True)
    with open(analysis_file, 'w') as f:
        json.dump(result, f, indent=2)
    console.print(f"\n[dim]Saved to {analysis_file}[/]")


def _print_analysis_report(result: dict):
    """Print a formatted analysis report."""
    console.print(Panel("[bold]Analysis Report[/]"))

    css = result.get("css", {})
    if css:
        console.print("\n[bold cyan]CSS Analysis[/]")

        animations = css.get("animations", {})
        if animations.get("keyframes"):
            console.print(f"  Keyframes: {len(animations['keyframes'])}")

        if css.get("transitions"):
            console.print(f"  Transitions: {len(css['transitions'])}")

        transforms = css.get("transforms", {})
        if transforms.get("values"):
            console.print(f"  Transforms: {len(transforms['values'])}")
            types = []
            if transforms.get("has_3d"): types.append("3D")
            if transforms.get("has_translate"): types.append("translate")
            if transforms.get("has_rotate"): types.append("rotate")
            if transforms.get("has_scale"): types.append("scale")
            if types:
                console.print(f"    Types: {', '.join(types)}")

        layout = css.get("layout", {})
        if layout:
            console.print(f"  Flexbox: {layout.get('flexbox_count', 0)}, Grid: {layout.get('grid_count', 0)}")

        if css.get("colors"):
            console.print(f"  Colors: {len(css['colors'])}")

        if css.get("fonts"):
            console.print(f"  Fonts: {', '.join(css['fonts'][:5])}")

    js = result.get("js", {})
    if js:
        console.print("\n[bold cyan]JavaScript Analysis[/]")

        for lib in js.get("animation_libraries", []):
            if isinstance(lib, dict):
                console.print(f"  Animation: {lib.get('name')} ({lib.get('occurrences', 0)}x)")

        for lib in js.get("graphics_libraries", []):
            if isinstance(lib, dict):
                console.print(f"  Graphics: {lib.get('name')}")

        webgl = js.get("webgl", {})
        if webgl.get("detected"):
            console.print(f"  [green]WebGL[/] ({webgl.get('version', '?')})")

        webgpu = js.get("webgpu", {})
        if webgpu.get("detected"):
            console.print("  [green]WebGPU[/]")

        for fw in js.get("ui_frameworks", []):
            if isinstance(fw, dict):
                console.print(f"  Framework: {fw.get('name')}")

    html = result.get("html", {})
    if html:
        console.print("\n[bold cyan]HTML Analysis[/]")
        console.print(f"  Elements: {html.get('element_count', 0)}")

        media = html.get("media_elements", {})
        if media:
            console.print(f"  Media: {media.get('images', 0)} images, {media.get('canvas', 0)} canvas, {media.get('svg', 0)} svg")

    score = result.get("score", {})
    if score:
        console.print(f"\n[bold]Complexity Score: {score.get('total', 0)}[/]")


@main.command()
@click.argument("directory", type=click.Path(exists=True))
@click.option("-o", "--output", required=True, help="Output file path")
@click.option("-f", "--format", "fmt", default="jsonl", type=click.Choice(["jsonl", "json", "csv"]))
def export(directory: str, output: str, fmt: str):
    """Export mockups as a dataset for ML training."""
    from crawlwe.writer import export_dataset

    dir_path = Path(directory)
    output_path = Path(output)

    with console.status(f"Exporting to {fmt}..."):
        result_path = export_dataset(dir_path, output_path, format=fmt)

    console.print(f"[green]OK[/] Exported to: {result_path}")


@main.command()
def info():
    """Show system information and capabilities."""
    import platform
    import shutil
    from crawlwe import RUST_AVAILABLE, __version__

    table = Table(title="CrawlWe System Info")
    table.add_column("Component", style="cyan")
    table.add_column("Status", style="green")

    table.add_row("Version", __version__)
    table.add_row("Python", platform.python_version())
    table.add_row("Platform", platform.system())
    table.add_row("Rust Core", "[green]Available[/]" if RUST_AVAILABLE else "[red]Not compiled[/]")

    chrome_paths = ["google-chrome", "chromium", "chromium-browser"]
    chrome_found = any(shutil.which(p) for p in chrome_paths)
    table.add_row("Chrome", "[green]Found[/]" if chrome_found else "[yellow]Not in PATH[/]")

    console.print(table)

    if not RUST_AVAILABLE:
        console.print("\n[yellow]Build Rust core:[/] maturin develop")


if __name__ == "__main__":
    main()
