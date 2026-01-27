"""
CrawlWe CLI - Command Line Interface

Usage:
    crawlwe fetch <url> -o output/
    crawlwe batch seeds.txt -o output/ --concurrent 10
    crawlwe analyze <url>
"""

import sys
from pathlib import Path
from typing import Optional

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
@click.option("--no-states", is_flag=True, help="Skip capturing hover/focus states")
@click.option("--no-screenshot", is_flag=True, help="Skip taking screenshots")
@click.option("--width", default=1440, help="Viewport width")
@click.option("--height", default=900, help="Viewport height")
@click.option("--minify", is_flag=True, help="Minify output CSS")
def fetch(
    url: str,
    output: str,
    no_states: bool,
    no_screenshot: bool,
    width: int,
    height: int,
    minify: bool,
):
    """Fetch a single URL and extract HTML + CSS."""
    from crawlwe.pipeline import capture, CaptureOptions
    from crawlwe.writer import write_mockup

    console.print(Panel(f"[bold blue]CrawlWe[/] - Capturing: {url}"))

    options = CaptureOptions(
        capture_states=not no_states,
        capture_screenshot=not no_screenshot,
        viewport_width=width,
        viewport_height=height,
        minify_css=minify,
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

            progress.update(task, description="Done!")

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
    console.print(f"\n[green]✓[/] Mockup saved to: [bold]{mockup_path}[/]")


@main.command()
@click.argument("seeds_file", type=click.Path(exists=True))
@click.option("-o", "--output", default="output", help="Output directory")
@click.option("-c", "--concurrent", default=5, help="Concurrent captures")
@click.option("--no-states", is_flag=True, help="Skip capturing hover/focus states")
@click.option("--no-screenshot", is_flag=True, help="Skip taking screenshots")
def batch(
    seeds_file: str,
    output: str,
    concurrent: int,
    no_states: bool,
    no_screenshot: bool,
):
    """Batch capture multiple URLs from a seeds file.

    Seeds file should have one URL per line.
    """
    from crawlwe.pipeline import batch_capture, CaptureOptions
    from crawlwe.writer import write_mockup

    # Read URLs
    with open(seeds_file) as f:
        urls = [line.strip() for line in f if line.strip() and not line.startswith("#")]

    if not urls:
        console.print("[yellow]No URLs found in seeds file[/]")
        sys.exit(1)

    console.print(Panel(f"[bold blue]CrawlWe[/] - Batch Capture: {len(urls)} URLs"))

    options = CaptureOptions(
        capture_states=not no_states,
        capture_screenshot=not no_screenshot,
    )

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
                console.print(f"[red]✗[/] {result.url}: {result.error}")
            else:
                success_count += 1
                mockup_path = write_mockup(result, output_path)
                console.print(f"[green]✓[/] {result.url} → {mockup_path}")

            progress.advance(task)

    # Summary
    console.print()
    console.print(Panel(f"[green]Success: {success_count}[/] | [red]Errors: {error_count}[/]"))


@main.command()
@click.argument("url")
@click.option("--json", "as_json", is_flag=True, help="Output as JSON")
def analyze(url: str, as_json: bool):
    """Analyze a URL without saving (quick inspection)."""
    from crawlwe.pipeline import capture, CaptureOptions

    options = CaptureOptions(
        capture_states=False,
        capture_screenshot=False,
    )

    with console.status("Analyzing..."):
        try:
            result = capture(url, options)
        except Exception as e:
            console.print(f"[red]Error:[/] {e}")
            sys.exit(1)

    if as_json:
        import json

        data = {
            "url": result.url,
            "title": result.title,
            "element_count": result.element_count,
            "rule_count": result.rule_count,
            "colors": result.colors,
            "fonts": result.fonts,
            "variables": result.variables,
        }
        console.print(json.dumps(data, indent=2))
    else:
        # Rich output
        console.print(Panel(f"[bold]Analysis: {url}[/]"))

        table = Table(show_header=False)
        table.add_column("Property", style="cyan")
        table.add_column("Value")

        table.add_row("Title", result.title or "N/A")
        table.add_row("Elements", str(result.element_count))
        table.add_row("CSS Rules", str(result.rule_count))

        console.print(table)

        # Colors
        if result.colors:
            console.print("\n[bold]Colors:[/]")
            color_str = " ".join(
                [f"[on {c}]  [/] {c}" if c.startswith("#") else c for c in result.colors[:20]]
            )
            console.print(color_str)

        # Fonts
        if result.fonts:
            console.print("\n[bold]Fonts:[/]")
            console.print(", ".join(result.fonts))

        # CSS Variables
        if result.variables:
            console.print("\n[bold]CSS Variables:[/]")
            var_table = Table(show_header=True)
            var_table.add_column("Variable", style="cyan")
            var_table.add_column("Value")

            for name, value in list(result.variables.items())[:10]:
                var_table.add_row(name, value)

            console.print(var_table)


@main.command()
@click.argument("css_file", type=click.Path(exists=True))
@click.option("-o", "--output", help="Output file (default: stdout)")
@click.option("--minify", is_flag=True, help="Minify output")
def optimize(css_file: str, output: Optional[str], minify: bool):
    """Optimize and deduplicate a CSS file."""
    try:
        from crawlwe.crawlwe_core import parse_css
    except ImportError:
        console.print("[red]Error:[/] Rust core not available. Please build first.")
        sys.exit(1)

    with open(css_file) as f:
        css_text = f.read()

    original_size = len(css_text)

    with console.status("Optimizing..."):
        optimized = parse_css(css_text, deduplicate=True, minify=minify)

    new_size = len(optimized)
    reduction = ((original_size - new_size) / original_size) * 100

    if output:
        with open(output, "w") as f:
            f.write(optimized)
        console.print(f"[green]✓[/] Saved to: {output}")
    else:
        console.print(optimized)

    console.print(
        f"\n[dim]Size: {original_size:,} → {new_size:,} bytes ({reduction:.1f}% reduction)[/]"
    )


@main.command()
def info():
    """Show system information and capabilities."""
    import platform

    try:
        from crawlwe import RUST_AVAILABLE, __version__
    except ImportError:
        RUST_AVAILABLE = False
        __version__ = "unknown"

    table = Table(title="CrawlWe System Info")
    table.add_column("Component", style="cyan")
    table.add_column("Status", style="green")

    table.add_row("Version", __version__)
    table.add_row("Python", platform.python_version())
    table.add_row("Platform", platform.system())
    table.add_row(
        "Rust Core",
        "[green]Available[/]" if RUST_AVAILABLE else "[red]Not compiled[/]",
    )

    # Check for Chrome
    import shutil

    chrome_paths = [
        "google-chrome",
        "chromium",
        "chromium-browser",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ]
    chrome_found = any(shutil.which(p) for p in chrome_paths)
    table.add_row(
        "Chrome/Chromium",
        "[green]Found[/]" if chrome_found else "[yellow]Not in PATH[/]",
    )

    console.print(table)

    if not RUST_AVAILABLE:
        console.print("\n[yellow]Tip:[/] Build Rust core with: [bold]maturin develop[/]")


if __name__ == "__main__":
    main()
