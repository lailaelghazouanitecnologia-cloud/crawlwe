"""
CrawlWe Writer - Save mockups to disk

Handles writing HTML, CSS, and assets to the output directory.
"""

import base64
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Optional
from urllib.parse import urlparse

from crawlwe.pipeline import CaptureResult


@dataclass
class MockupWriter:
    """Configurable mockup writer."""

    # Output structure
    html_filename: str = "index.html"
    css_filename: str = "styles.css"
    screenshot_filename: str = "screenshot.png"
    metadata_filename: str = "metadata.json"

    # Options
    save_screenshot: bool = True
    save_metadata: bool = True
    create_subdirs: bool = True  # Create subdirectory per domain

    def write(self, result: CaptureResult, output_dir: Path) -> Path:
        """Write a capture result to disk."""
        return write_mockup(
            result,
            output_dir,
            html_filename=self.html_filename,
            css_filename=self.css_filename,
            screenshot_filename=self.screenshot_filename,
            metadata_filename=self.metadata_filename,
            save_screenshot=self.save_screenshot,
            save_metadata=self.save_metadata,
            create_subdirs=self.create_subdirs,
        )


def write_mockup(
    result: CaptureResult,
    output_dir: Path,
    html_filename: str = "index.html",
    css_filename: str = "styles.css",
    screenshot_filename: str = "screenshot.png",
    metadata_filename: str = "metadata.json",
    save_screenshot: bool = True,
    save_metadata: bool = True,
    create_subdirs: bool = True,
) -> Path:
    """
    Write a capture result to disk as a mockup.

    Args:
        result: The capture result to save
        output_dir: Base output directory
        html_filename: Name for HTML file
        css_filename: Name for CSS file
        screenshot_filename: Name for screenshot file
        metadata_filename: Name for metadata JSON file
        save_screenshot: Whether to save screenshot
        save_metadata: Whether to save metadata
        create_subdirs: Whether to create subdirectory per domain

    Returns:
        Path to the mockup directory
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Determine mockup directory
    if create_subdirs:
        domain = _url_to_dirname(result.url)
        mockup_dir = output_dir / domain
    else:
        mockup_dir = output_dir

    mockup_dir.mkdir(parents=True, exist_ok=True)

    # Write HTML
    html_path = mockup_dir / html_filename
    html_content = _prepare_html(result.html, css_filename)
    html_path.write_text(html_content, encoding="utf-8")

    # Write CSS
    css_path = mockup_dir / css_filename
    css_path.write_text(result.css, encoding="utf-8")

    # Write screenshot
    if save_screenshot and result.screenshot:
        screenshot_path = mockup_dir / screenshot_filename
        screenshot_data = base64.b64decode(result.screenshot)
        screenshot_path.write_bytes(screenshot_data)

    # Write metadata
    if save_metadata:
        metadata_path = mockup_dir / metadata_filename
        metadata = {
            "url": result.url,
            "title": result.title,
            "element_count": result.element_count,
            "rule_count": result.rule_count,
            "colors": result.colors,
            "fonts": result.fonts,
            "variables": result.variables,
            "duration_ms": result.duration_ms,
        }
        metadata_path.write_text(json.dumps(metadata, indent=2), encoding="utf-8")

    return mockup_dir


def _url_to_dirname(url: str) -> str:
    """Convert URL to a safe directory name."""
    parsed = urlparse(url)

    # Start with domain
    dirname = parsed.netloc or "unknown"

    # Add path (simplified)
    if parsed.path and parsed.path != "/":
        path_part = parsed.path.strip("/").replace("/", "_")
        # Limit length
        if len(path_part) > 50:
            path_part = path_part[:50]
        dirname = f"{dirname}_{path_part}"

    # Sanitize
    dirname = re.sub(r"[^\w\-_.]", "_", dirname)
    dirname = re.sub(r"_+", "_", dirname)
    dirname = dirname.strip("_")

    return dirname or "unknown"


def _prepare_html(html: str, css_filename: str) -> str:
    """Prepare HTML for writing (ensure stylesheet link)."""
    # Check if already has our stylesheet link
    if f'href="{css_filename}"' in html:
        return html

    # Try to insert in <head>
    if "<head>" in html:
        link_tag = f'<link rel="stylesheet" href="{css_filename}">'

        # Insert after <head> or after existing meta/title
        if "</title>" in html:
            html = html.replace("</title>", f"</title>\n  {link_tag}")
        elif "<head>" in html:
            html = html.replace("<head>", f"<head>\n  {link_tag}")

        return html

    # No head tag, create minimal structure
    return f"""<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="{css_filename}">
</head>
{html}
</html>"""


def merge_mockups(mockup_dirs: list[Path], output_dir: Path) -> Path:
    """
    Merge multiple mockups into a single combined CSS file.

    Useful for creating a unified stylesheet from multiple captures.

    Args:
        mockup_dirs: List of mockup directories to merge
        output_dir: Where to save the merged result

    Returns:
        Path to the merged CSS file
    """
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    combined_css_parts = []
    all_colors = set()
    all_fonts = set()
    all_variables = {}

    for mockup_dir in mockup_dirs:
        mockup_dir = Path(mockup_dir)

        # Read CSS
        css_path = mockup_dir / "styles.css"
        if css_path.exists():
            css = css_path.read_text(encoding="utf-8")
            combined_css_parts.append(f"/* Source: {mockup_dir.name} */\n{css}")

        # Read metadata
        metadata_path = mockup_dir / "metadata.json"
        if metadata_path.exists():
            try:
                metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
                all_colors.update(metadata.get("colors", []))
                all_fonts.update(metadata.get("fonts", []))
                all_variables.update(metadata.get("variables", {}))
            except json.JSONDecodeError:
                pass

    # Combine CSS
    combined_css = "\n\n".join(combined_css_parts)

    # Try to optimize with Rust core
    try:
        from crawlwe.crawlwe_core import merge_css

        combined_css = merge_css([combined_css])
    except ImportError:
        pass

    # Write combined CSS
    css_output = output_dir / "combined.css"
    css_output.write_text(combined_css, encoding="utf-8")

    # Write combined metadata
    metadata_output = output_dir / "combined_metadata.json"
    metadata = {
        "source_count": len(mockup_dirs),
        "colors": sorted(all_colors),
        "fonts": sorted(all_fonts),
        "variables": all_variables,
    }
    metadata_output.write_text(json.dumps(metadata, indent=2), encoding="utf-8")

    return css_output


def export_dataset(
    mockups_dir: Path,
    output_file: Path,
    format: str = "jsonl",
) -> Path:
    """
    Export mockups as a dataset for ML training.

    Args:
        mockups_dir: Directory containing mockups
        output_file: Output file path
        format: Export format ('jsonl', 'json', 'csv')

    Returns:
        Path to the exported file
    """
    mockups_dir = Path(mockups_dir)
    output_file = Path(output_file)

    records = []

    for mockup_path in mockups_dir.iterdir():
        if not mockup_path.is_dir():
            continue

        html_path = mockup_path / "index.html"
        css_path = mockup_path / "styles.css"
        metadata_path = mockup_path / "metadata.json"

        if not html_path.exists() or not css_path.exists():
            continue

        html = html_path.read_text(encoding="utf-8")
        css = css_path.read_text(encoding="utf-8")

        record = {
            "id": mockup_path.name,
            "html": html,
            "css": css,
        }

        if metadata_path.exists():
            try:
                metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
                record.update(
                    {
                        "url": metadata.get("url"),
                        "title": metadata.get("title"),
                        "colors": metadata.get("colors", []),
                        "fonts": metadata.get("fonts", []),
                    }
                )
            except json.JSONDecodeError:
                pass

        records.append(record)

    # Write output
    if format == "jsonl":
        with open(output_file, "w", encoding="utf-8") as f:
            for record in records:
                f.write(json.dumps(record, ensure_ascii=False) + "\n")

    elif format == "json":
        with open(output_file, "w", encoding="utf-8") as f:
            json.dump(records, f, indent=2, ensure_ascii=False)

    elif format == "csv":
        import csv

        with open(output_file, "w", newline="", encoding="utf-8") as f:
            if records:
                writer = csv.DictWriter(f, fieldnames=records[0].keys())
                writer.writeheader()
                writer.writerows(records)

    return output_file
