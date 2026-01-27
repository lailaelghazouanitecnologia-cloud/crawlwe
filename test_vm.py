#!/usr/bin/env python3
"""
Test CrawlWe VM
"""

import sys
sys.path.insert(0, '.')

from vm import parse_rules, Compiler, VM
from vm.dsl import print_ast


def test_lexer_parser():
    """Test DSL lexer and parser"""
    print("=" * 60)
    print("TEST: DSL Lexer & Parser")
    print("=" * 60)

    source = '''
@css {
    rule "animations" {
        match /@keyframes\\s+(\\w+)/ -> emit CSS_KEYFRAME($1)
        match /animation:\\s*([^;]+);/ -> emit CSS_ANIMATION($1)
    }
}

@js {
    rule "libs" {
        match /THREE\\.(\\w+)/ -> emit LIB_THREEJS($1)
        if contains "WebGLRenderer" -> emit WEBGL_DETECTED
    }
}
'''

    ast = parse_rules(source)
    print("AST:")
    print_ast(ast)
    print()
    return ast


def test_compiler(ast):
    """Test bytecode compiler"""
    print("=" * 60)
    print("TEST: Bytecode Compiler")
    print("=" * 60)

    compiler = Compiler()
    programs = compiler.compile(ast)

    for name, program in programs.items():
        print(f"\n{program.disassemble()}")

    return programs


def test_vm(programs):
    """Test VM execution"""
    print("=" * 60)
    print("TEST: VM Execution")
    print("=" * 60)

    # Test CSS source
    css_source = '''
@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes slideUp {
    0% { transform: translateY(100px); }
    100% { transform: translateY(0); }
}

.element {
    animation: fadeIn 0.3s ease-out;
    transition: all 0.5s ease;
}
'''

    # Test JS source
    js_source = '''
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight);
const renderer = new THREE.WebGLRenderer({ antialias: true });

function animate() {
    requestAnimationFrame(animate);
    renderer.render(scene, camera);
}
'''

    vm = VM(debug=False)

    if 'css' in programs:
        print("\n--- Running CSS Parser ---")
        results = vm.run(programs['css'], css_source)
        print(f"Results: {results}")

    if 'js' in programs:
        print("\n--- Running JS Parser ---")
        results = vm.run(programs['js'], js_source)
        print(f"Results: {results}")


def test_full_rules():
    """Test with default rules file"""
    print("=" * 60)
    print("TEST: Full Rules File")
    print("=" * 60)

    try:
        with open('rules/default.crawl', 'r') as f:
            rules_source = f.read()

        ast = parse_rules(rules_source)
        print(f"Parsed {len(ast.sections)} sections")

        compiler = Compiler()
        programs = compiler.compile(ast)

        for name, program in programs.items():
            print(f"  {name}: {len(program)} instructions")

        # Test with real CSS
        css_source = '''
:root {
    --primary-color: #3b82f6;
    --spacing: 1rem;
}

@keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.05); }
    100% { transform: scale(1); }
}

.card {
    display: flex;
    box-shadow: 0 4px 6px rgba(0,0,0,0.1);
    transition: transform 0.3s ease;
    animation: pulse 2s infinite;
}

.card:hover {
    transform: translateY(-4px);
}

@media (prefers-reduced-motion: reduce) {
    .card { animation: none; }
}
'''

        vm = VM(debug=False)
        if 'css' in programs:
            print("\n--- CSS Analysis ---")
            results = vm.run(programs['css'], css_source)
            for tag, values in sorted(results.items()):
                if values:
                    print(f"  {tag}: {values[:3]}{'...' if len(values) > 3 else ''}")

    except FileNotFoundError:
        print("rules/default.crawl not found")


def main():
    print()
    print("CrawlWe VM Test Suite")
    print("=" * 60)
    print()

    # Run tests
    ast = test_lexer_parser()
    programs = test_compiler(ast)
    test_vm(programs)
    test_full_rules()

    print()
    print("=" * 60)
    print("All tests completed!")
    print("=" * 60)


if __name__ == "__main__":
    main()
