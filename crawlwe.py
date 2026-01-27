#!/usr/bin/env python3
"""
CrawlWe - Web Page Extractor for ML Training

A comprehensive tool for extracting and analyzing web page content:
- Scrapes HTML, CSS, JavaScript from web pages
- Analyzes animations, transitions, UI patterns
- Detects CSS frameworks (Tailwind, CSS Modules, CSS-in-JS)
- Detects 3D/WebGL libraries (Three.js, PIXI, etc.)
- Uses VM-based rules engine for extensible parsing

Usage:
    crawlwe scrape <url> [-o output_dir]
    crawlwe analyze <output_dir>
    crawlwe vm-test
"""

import argparse
import sys
import json
from pathlib import Path


def cmd_scrape(args):
    """Scrape a web page"""
    from scrape import main as scrape_main

    # Modify sys.argv for the scraper
    sys.argv = ['scrape.py', args.url]
    if args.output:
        sys.argv.extend(['-o', args.output])

    scrape_main()


def cmd_analyze(args):
    """Analyze scraped content"""
    from analyze import main as analyze_main

    sys.argv = ['analyze.py', args.directory]
    analyze_main()


def cmd_vm_analyze(args):
    """Analyze using VM-based rules"""
    from vm import parse_rules, Compiler, VM

    output_dir = Path(args.directory)

    if not output_dir.exists():
        print(f"Error: Directory {output_dir} does not exist")
        sys.exit(1)

    # Load rules
    rules_file = Path(args.rules) if args.rules else Path('rules/default.crawl')
    if not rules_file.exists():
        print(f"Error: Rules file {rules_file} not found")
        sys.exit(1)

    print(f"🔧 Loading rules from {rules_file}...")
    rules_source = rules_file.read_text()
    ast = parse_rules(rules_source)

    print(f"⚙️  Compiling rules...")
    compiler = Compiler()
    programs = compiler.compile(ast)

    for name, program in programs.items():
        print(f"   {name}: {len(program)} instructions")

    # Load content
    css_content = ""
    css_file = output_dir / "styles.css"
    if css_file.exists():
        css_content = css_file.read_text(encoding='utf-8', errors='ignore')

    css_dir = output_dir / "css"
    if css_dir.exists():
        for css_path in css_dir.glob("*.css"):
            css_content += "\n" + css_path.read_text(encoding='utf-8', errors='ignore')

    js_content = ""
    scripts_dir = output_dir / "scripts"
    if scripts_dir.exists():
        for js_path in scripts_dir.glob("*.js"):
            js_content += "\n" + js_path.read_text(encoding='utf-8', errors='ignore')

    html_content = ""
    raw_html = output_dir / "data" / "raw.html"
    if raw_html.exists():
        html_content = raw_html.read_text(encoding='utf-8', errors='ignore')
    else:
        html_file = output_dir / "index.html"
        if html_file.exists():
            html_content = html_file.read_text(encoding='utf-8', errors='ignore')

    print(f"\n📄 Loaded: CSS={len(css_content)} JS={len(js_content)} HTML={len(html_content)} bytes")

    # Run VM analysis
    vm = VM(debug=args.debug)
    all_results = {}

    if 'css' in programs and css_content:
        print("\n🔬 Running CSS VM...")
        results = vm.run(programs['css'], css_content)
        all_results['css'] = results
        print(f"   Found {len(results)} categories")

    if 'js' in programs and js_content:
        print("\n🔬 Running JS VM...")
        results = vm.run(programs['js'], js_content)
        all_results['js'] = results
        print(f"   Found {len(results)} categories")

    if 'html' in programs and html_content:
        print("\n🔬 Running HTML VM...")
        results = vm.run(programs['html'], html_content)
        all_results['html'] = results
        print(f"   Found {len(results)} categories")

    # Print results
    print("\n" + "=" * 60)
    print("VM ANALYSIS RESULTS")
    print("=" * 60)

    for section, results in all_results.items():
        print(f"\n## {section.upper()}")
        print("-" * 40)
        for tag, values in sorted(results.items()):
            if values:
                count = len(values)
                preview = str(values[:3])[:80]
                print(f"  {tag}: {count} matches")
                if args.verbose:
                    print(f"    {preview}...")

    # Save results
    results_file = output_dir / "data" / "vm_analysis.json"
    results_file.parent.mkdir(parents=True, exist_ok=True)
    with open(results_file, 'w') as f:
        json.dump(all_results, f, indent=2, default=str)
    print(f"\n💾 Saved to {results_file}")


def cmd_full(args):
    """Full pipeline: scrape + analyze"""
    from scrape import main as scrape_main
    from analyze import main as analyze_main
    from urllib.parse import urlparse

    # Determine output directory
    if args.output:
        output_dir = args.output
    else:
        parsed = urlparse(args.url)
        domain = parsed.netloc.replace('www.', '').replace('.', '_')
        output_dir = f"output/{domain}"

    print("=" * 60)
    print("CRAWLWE FULL PIPELINE")
    print("=" * 60)
    print(f"URL: {args.url}")
    print(f"Output: {output_dir}")
    print()

    # Step 1: Scrape
    print("📥 STEP 1: Scraping...")
    print("-" * 40)
    sys.argv = ['scrape.py', args.url, '-o', output_dir]
    try:
        scrape_main()
    except Exception as e:
        print(f"Scrape error: {e}")
        return

    # Step 2: Analyze
    print("\n📊 STEP 2: Analyzing...")
    print("-" * 40)
    sys.argv = ['analyze.py', output_dir]
    try:
        analyze_main()
    except Exception as e:
        print(f"Analysis error: {e}")

    # Step 3: VM Analysis (optional)
    if args.vm:
        print("\n🔧 STEP 3: VM Analysis...")
        print("-" * 40)
        try:
            # Create a namespace for vm_analyze
            class VMArgs:
                directory = output_dir
                rules = args.rules
                debug = False
                verbose = args.verbose
            cmd_vm_analyze(VMArgs())
        except Exception as e:
            print(f"VM analysis error: {e}")

    print("\n" + "=" * 60)
    print("PIPELINE COMPLETE")
    print("=" * 60)
    print(f"Output: {output_dir}/")


def cmd_vm_test(args):
    """Run VM tests"""
    import test_vm
    test_vm.main()


def cmd_list_rules(args):
    """List available rules"""
    from vm import parse_rules
    from vm.dsl import print_ast

    rules_dir = Path('rules')
    if not rules_dir.exists():
        print("No rules directory found")
        return

    for rules_file in rules_dir.glob('*.crawl'):
        print(f"\n📄 {rules_file}")
        print("-" * 40)

        source = rules_file.read_text()
        ast = parse_rules(source)

        for section in ast.sections:
            print(f"  @{section.type}:")
            for rule in section.rules:
                stmt_count = len(rule.statements)
                print(f"    rule \"{rule.name}\" ({stmt_count} statements)")


def main():
    parser = argparse.ArgumentParser(
        description='CrawlWe - Web Page Extractor for ML Training',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  crawlwe scrape https://example.com
  crawlwe analyze output/example_com
  crawlwe full https://example.com --vm
  crawlwe vm-analyze output/example_com
  crawlwe list-rules
        """
    )

    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # Scrape command
    scrape_parser = subparsers.add_parser('scrape', help='Scrape a web page')
    scrape_parser.add_argument('url', help='URL to scrape')
    scrape_parser.add_argument('-o', '--output', help='Output directory')

    # Analyze command
    analyze_parser = subparsers.add_parser('analyze', help='Analyze scraped content')
    analyze_parser.add_argument('directory', help='Directory with scraped content')

    # VM Analyze command
    vm_parser = subparsers.add_parser('vm-analyze', help='Analyze using VM rules')
    vm_parser.add_argument('directory', help='Directory with scraped content')
    vm_parser.add_argument('-r', '--rules', help='Rules file (default: rules/default.crawl)')
    vm_parser.add_argument('-d', '--debug', action='store_true', help='Debug mode')
    vm_parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')

    # Full pipeline command
    full_parser = subparsers.add_parser('full', help='Full pipeline: scrape + analyze')
    full_parser.add_argument('url', help='URL to process')
    full_parser.add_argument('-o', '--output', help='Output directory')
    full_parser.add_argument('--vm', action='store_true', help='Include VM analysis')
    full_parser.add_argument('-r', '--rules', help='Rules file for VM')
    full_parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')

    # VM Test command
    subparsers.add_parser('vm-test', help='Run VM tests')

    # List rules command
    subparsers.add_parser('list-rules', help='List available rules')

    args = parser.parse_args()

    if args.command == 'scrape':
        cmd_scrape(args)
    elif args.command == 'analyze':
        cmd_analyze(args)
    elif args.command == 'vm-analyze':
        cmd_vm_analyze(args)
    elif args.command == 'full':
        cmd_full(args)
    elif args.command == 'vm-test':
        cmd_vm_test(args)
    elif args.command == 'list-rules':
        cmd_list_rules(args)
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
