#!/usr/bin/env python3
"""
Deep Analyzer for Web Page Content
Extracts and classifies: animations, transitions, UI patterns, interactions, etc.
"""

import re
import json
import sys
from pathlib import Path
from collections import defaultdict
from bs4 import BeautifulSoup

# ============================================================================
# CSS ANALYSIS PATTERNS
# ============================================================================

CSS_PATTERNS = {
    "animations": {
        "@keyframes": r'@keyframes\s+([a-zA-Z0-9_-]+)\s*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}',
        "animation_property": r'animation\s*:\s*([^;]+);',
        "animation_name": r'animation-name\s*:\s*([^;]+);',
        "animation_duration": r'animation-duration\s*:\s*([^;]+);',
        "animation_timing": r'animation-timing-function\s*:\s*([^;]+);',
        "animation_delay": r'animation-delay\s*:\s*([^;]+);',
        "animation_iteration": r'animation-iteration-count\s*:\s*([^;]+);',
    },
    "transitions": {
        "transition": r'transition\s*:\s*([^;]+);',
        "transition_property": r'transition-property\s*:\s*([^;]+);',
        "transition_duration": r'transition-duration\s*:\s*([^;]+);',
        "transition_timing": r'transition-timing-function\s*:\s*([^;]+);',
    },
    "transforms": {
        "transform": r'transform\s*:\s*([^;]+);',
        "transform_origin": r'transform-origin\s*:\s*([^;]+);',
        "perspective": r'perspective\s*:\s*([^;]+);',
        "transform_style": r'transform-style\s*:\s*([^;]+);',
    },
    "filters": {
        "filter": r'filter\s*:\s*([^;]+);',
        "backdrop_filter": r'backdrop-filter\s*:\s*([^;]+);',
    },
    "layout": {
        "display_flex": r'display\s*:\s*flex',
        "display_grid": r'display\s*:\s*grid',
        "position_fixed": r'position\s*:\s*fixed',
        "position_sticky": r'position\s*:\s*sticky',
        "position_absolute": r'position\s*:\s*absolute',
    },
    "pseudo_states": {
        "hover": r':hover\s*\{([^}]+)\}',
        "focus": r':focus\s*\{([^}]+)\}',
        "active": r':active\s*\{([^}]+)\}',
        "focus_within": r':focus-within\s*\{([^}]+)\}',
        "focus_visible": r':focus-visible\s*\{([^}]+)\}',
        "visited": r':visited\s*\{([^}]+)\}',
    },
    "pseudo_elements": {
        "before": r'::before\s*\{([^}]+)\}',
        "after": r'::after\s*\{([^}]+)\}',
        "placeholder": r'::placeholder\s*\{([^}]+)\}',
        "selection": r'::selection\s*\{([^}]+)\}',
    },
    "media_queries": {
        "media": r'@media\s*([^{]+)\{',
        "prefers_reduced_motion": r'prefers-reduced-motion',
        "prefers_color_scheme": r'prefers-color-scheme',
    },
    "custom_properties": {
        "css_variables": r'--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);',
        "var_usage": r'var\s*\(\s*--([a-zA-Z0-9_-]+)',
    },
    "visual_effects": {
        "box_shadow": r'box-shadow\s*:\s*([^;]+);',
        "text_shadow": r'text-shadow\s*:\s*([^;]+);',
        "opacity": r'opacity\s*:\s*([^;]+);',
        "mix_blend_mode": r'mix-blend-mode\s*:\s*([^;]+);',
        "clip_path": r'clip-path\s*:\s*([^;]+);',
        "mask": r'mask\s*:\s*([^;]+);',
    },
    "scroll": {
        "scroll_behavior": r'scroll-behavior\s*:\s*([^;]+);',
        "scroll_snap": r'scroll-snap-\w+\s*:\s*([^;]+);',
        "overflow": r'overflow\s*:\s*([^;]+);',
        "overscroll": r'overscroll-behavior\s*:\s*([^;]+);',
    },
}

# ============================================================================
# JAVASCRIPT ANALYSIS PATTERNS
# ============================================================================

JS_PATTERNS = {
    "animation_libraries": {
        "gsap": [r'gsap\.', r'TweenMax', r'TweenLite', r'TimelineMax', r'TimelineLite', r'ScrollTrigger'],
        "anime_js": [r'anime\s*\(', r'anime\.timeline'],
        "framer_motion": [r'motion\.', r'useAnimation', r'AnimatePresence'],
        "lottie": [r'lottie\.', r'bodymovin', r'lottie-web'],
        "velocity": [r'Velocity\s*\(', r'\.velocity\s*\('],
        "popmotion": [r'popmotion', r'@popmotion'],
        "motion_one": [r'motion\s*\(', r'@motionone'],
    },
    "3d_libraries": {
        "three_js": [r'THREE\.', r'WebGLRenderer', r'Scene\s*\(\s*\)', r'PerspectiveCamera'],
        "babylon": [r'BABYLON\.', r'BabylonJS'],
        "aframe": [r'AFRAME\.', r'a-scene', r'a-entity'],
        "pixi": [r'PIXI\.', r'PixiJS'],
        "p5": [r'p5\.', r'createCanvas'],
        "ogl": [r'ogl', r'OGL\.'],
        "regl": [r'regl\s*\('],
    },
    "scroll_libraries": {
        "locomotive": [r'LocomotiveScroll', r'locomotive-scroll'],
        "scroll_magic": [r'ScrollMagic', r'ScrollScene'],
        "aos": [r'AOS\.init', r'data-aos'],
        "scrollreveal": [r'ScrollReveal'],
        "lenis": [r'Lenis', r'@studio-freight/lenis'],
        "smooth_scrollbar": [r'Scrollbar\.init'],
    },
    "native_animation": {
        "request_animation_frame": [r'requestAnimationFrame\s*\('],
        "cancel_animation_frame": [r'cancelAnimationFrame\s*\('],
        "web_animations_api": [r'\.animate\s*\(\s*\[', r'Animation\s*\(', r'KeyframeEffect'],
        "set_timeout": [r'setTimeout\s*\('],
        "set_interval": [r'setInterval\s*\('],
    },
    "observers": {
        "intersection_observer": [r'IntersectionObserver'],
        "mutation_observer": [r'MutationObserver'],
        "resize_observer": [r'ResizeObserver'],
        "performance_observer": [r'PerformanceObserver'],
    },
    "event_handlers": {
        "scroll_events": [r'addEventListener\s*\(\s*[\'"]scroll[\'"]', r'onscroll'],
        "mouse_events": [r'mousemove', r'mouseenter', r'mouseleave', r'mousedown', r'mouseup'],
        "touch_events": [r'touchstart', r'touchmove', r'touchend', r'gesture'],
        "pointer_events": [r'pointerdown', r'pointermove', r'pointerup'],
        "wheel_events": [r'wheel', r'mousewheel', r'DOMMouseScroll'],
    },
    "state_management": {
        "react_hooks": [r'useState', r'useEffect', r'useRef', r'useCallback', r'useMemo'],
        "vue_reactivity": [r'ref\s*\(', r'reactive\s*\(', r'computed\s*\(', r'watch\s*\('],
        "svelte_stores": [r'writable\s*\(', r'readable\s*\(', r'\$:'],
    },
    "ui_frameworks": {
        "react": [r'React\.', r'createElement', r'jsx', r'__jsx'],
        "vue": [r'Vue\.', r'createApp', r'defineComponent'],
        "svelte": [r'svelte', r'SvelteComponent'],
        "angular": [r'@angular', r'ngModule'],
        "next_js": [r'next/', r'_next', r'__NEXT'],
        "nuxt": [r'nuxt', r'__NUXT'],
    },
    "webgl_webgpu": {
        "webgl_context": [r'getContext\s*\(\s*[\'"]webgl', r'getContext\s*\(\s*[\'"]experimental-webgl'],
        "webgl2_context": [r'getContext\s*\(\s*[\'"]webgl2'],
        "webgpu_context": [r'navigator\.gpu', r'GPUDevice', r'GPUBuffer', r'GPUTexture'],
        "shader_code": [r'createShader', r'shaderSource', r'compileShader', r'createProgram'],
        "gl_calls": [r'gl\.bindBuffer', r'gl\.bindTexture', r'gl\.drawArrays', r'gl\.drawElements'],
    },
}

# ============================================================================
# HTML/UI PATTERNS
# ============================================================================

UI_PATTERNS = {
    "components": {
        "modal": ["modal", "dialog", "popup", "overlay", "lightbox"],
        "dropdown": ["dropdown", "select", "combobox", "autocomplete", "menu"],
        "carousel": ["carousel", "slider", "swiper", "slideshow", "gallery"],
        "tabs": ["tab", "tablist", "tabpanel"],
        "accordion": ["accordion", "collapse", "expandable", "collapsible"],
        "navigation": ["nav", "navbar", "sidebar", "drawer", "header", "footer"],
        "form": ["form", "input", "textarea", "checkbox", "radio", "button", "submit"],
        "card": ["card", "tile", "panel", "box"],
        "tooltip": ["tooltip", "popover", "hint"],
        "notification": ["toast", "notification", "alert", "snackbar", "banner"],
        "progress": ["progress", "loader", "spinner", "skeleton", "loading"],
        "pagination": ["pagination", "pager", "page-"],
    },
    "interactive": {
        "toggle": ["toggle", "switch", "checkbox"],
        "drag_drop": ["draggable", "droppable", "sortable", "drag", "drop"],
        "resize": ["resizable", "resize"],
        "scroll_area": ["scroll", "overflow", "scrollbar"],
        "parallax": ["parallax", "rellax"],
    },
    "media": {
        "video": ["video", "player", "youtube", "vimeo"],
        "audio": ["audio", "sound", "music"],
        "canvas": ["canvas"],
        "svg": ["svg"],
        "webgl": ["webgl", "three", "gl-"],
    },
}


def analyze_css(css_content):
    """Deep analysis of CSS content."""
    results = {
        "animations": {
            "keyframes": [],
            "keyframes_code": {},
            "animated_elements": [],
            "total_animations": 0,
        },
        "transitions": {
            "values": [],
            "properties": [],
            "durations": [],
            "timing_functions": [],
            "total_transitions": 0,
        },
        "transforms": {
            "values": [],
            "types": set(),
            "total_transforms": 0,
        },
        "pseudo_states": {
            "hover": [],
            "focus": [],
            "active": [],
            "other": [],
        },
        "pseudo_elements": {
            "before": 0,
            "after": 0,
            "other": [],
        },
        "layout": {
            "flexbox_count": 0,
            "grid_count": 0,
            "position_fixed": 0,
            "position_sticky": 0,
        },
        "visual_effects": {
            "shadows": [],
            "filters": [],
            "blend_modes": [],
            "clip_paths": [],
        },
        "css_variables": {
            "definitions": {},
            "usage_count": 0,
        },
        "media_queries": {
            "breakpoints": [],
            "features": [],
        },
        "colors": {
            "palette": set(),
            "gradients": [],
        },
        "typography": {
            "font_families": set(),
            "font_sizes": set(),
        },
    }

    # Extract @keyframes with their code
    keyframe_matches = re.findall(
        r'@keyframes\s+([a-zA-Z0-9_-]+)\s*\{((?:[^{}]+|\{[^{}]*\})*)\}',
        css_content, re.DOTALL
    )
    results["animations"]["keyframes"] = list(set([k[0] for k in keyframe_matches]))
    results["animations"]["keyframes_code"] = {k[0]: k[1].strip() for k in keyframe_matches}

    # Extract animation properties (clean)
    animation_pattern = r'animation\s*:\s*([^;{}]+);'
    animations = re.findall(animation_pattern, css_content)
    clean_animations = []
    for a in animations:
        a = a.strip()
        if a and a != 'none' and len(a) < 200:  # Filter out malformed extractions
            clean_animations.append(a)
    results["animations"]["animated_elements"] = list(set(clean_animations))
    results["animations"]["total_animations"] = len(results["animations"]["animated_elements"])

    # Extract transitions (clean values only)
    transition_pattern = r'transition\s*:\s*([^;{}]+);'
    transitions = re.findall(transition_pattern, css_content)
    clean_transitions = []
    for t in transitions:
        t = t.strip()
        if t and t != 'none' and len(t) < 150:  # Filter out malformed extractions
            clean_transitions.append(t)
    results["transitions"]["values"] = list(set(clean_transitions))
    results["transitions"]["total_transitions"] = len(results["transitions"]["values"])

    # Extract transition details
    for trans in results["transitions"]["values"]:
        # Extract timing functions
        timing = re.findall(r'(ease|ease-in|ease-out|ease-in-out|linear|cubic-bezier\([^)]+\))', trans)
        results["transitions"]["timing_functions"].extend(timing)
        # Extract durations
        durations = re.findall(r'(\d+\.?\d*m?s)', trans)
        results["transitions"]["durations"].extend(durations)
        # Extract properties being transitioned
        props = re.findall(r'^([a-z-]+)', trans)
        results["transitions"]["properties"].extend(props)

    results["transitions"]["timing_functions"] = list(set(results["transitions"]["timing_functions"]))
    results["transitions"]["durations"] = list(set(results["transitions"]["durations"]))
    results["transitions"]["properties"] = list(set(results["transitions"]["properties"]))

    # Extract transforms (clean values only)
    transform_pattern = r'transform\s*:\s*([^;{}]+);'
    transforms = re.findall(transform_pattern, css_content)
    clean_transforms = []
    for t in transforms:
        t = t.strip()
        if t and t != 'none' and len(t) < 200:
            clean_transforms.append(t)
    results["transforms"]["values"] = list(set(clean_transforms))
    results["transforms"]["total_transforms"] = len(results["transforms"]["values"])

    # Identify transform types
    transform_types = set()
    for t in results["transforms"]["values"]:
        if 'translate' in t: transform_types.add('translate')
        if 'rotate' in t: transform_types.add('rotate')
        if 'scale' in t: transform_types.add('scale')
        if 'skew' in t: transform_types.add('skew')
        if 'matrix' in t: transform_types.add('matrix')
        if 'perspective' in t: transform_types.add('perspective')
        if '3d' in t: transform_types.add('3d')
    results["transforms"]["types"] = list(transform_types)

    # Extract pseudo-states (hover, focus, active)
    hover_rules = re.findall(r'([^{]+):hover\s*\{([^}]+)\}', css_content)
    results["pseudo_states"]["hover"] = [{"selector": h[0].strip(), "properties": h[1].strip()} for h in hover_rules[:20]]

    focus_rules = re.findall(r'([^{]+):focus\s*\{([^}]+)\}', css_content)
    results["pseudo_states"]["focus"] = [{"selector": f[0].strip(), "properties": f[1].strip()} for f in focus_rules[:20]]

    active_rules = re.findall(r'([^{]+):active\s*\{([^}]+)\}', css_content)
    results["pseudo_states"]["active"] = [{"selector": a[0].strip(), "properties": a[1].strip()} for a in active_rules[:20]]

    # Pseudo elements
    results["pseudo_elements"]["before"] = len(re.findall(r'::before', css_content))
    results["pseudo_elements"]["after"] = len(re.findall(r'::after', css_content))

    # Layout analysis
    results["layout"]["flexbox_count"] = len(re.findall(r'display\s*:\s*flex', css_content))
    results["layout"]["grid_count"] = len(re.findall(r'display\s*:\s*grid', css_content))
    results["layout"]["position_fixed"] = len(re.findall(r'position\s*:\s*fixed', css_content))
    results["layout"]["position_sticky"] = len(re.findall(r'position\s*:\s*sticky', css_content))

    # Visual effects
    shadows = re.findall(r'box-shadow\s*:\s*([^;]+);', css_content)
    results["visual_effects"]["shadows"] = list(set([s.strip() for s in shadows if s.strip() != 'none']))[:10]

    filters = re.findall(r'filter\s*:\s*([^;]+);', css_content)
    filters += re.findall(r'backdrop-filter\s*:\s*([^;]+);', css_content)
    results["visual_effects"]["filters"] = list(set([f.strip() for f in filters if f.strip() != 'none']))[:10]

    blend_modes = re.findall(r'mix-blend-mode\s*:\s*([^;]+);', css_content)
    results["visual_effects"]["blend_modes"] = list(set([b.strip() for b in blend_modes]))

    clip_paths = re.findall(r'clip-path\s*:\s*([^;]+);', css_content)
    results["visual_effects"]["clip_paths"] = list(set([c.strip() for c in clip_paths if c.strip() != 'none']))[:10]

    # CSS Variables
    var_defs = re.findall(r'--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);', css_content)
    results["css_variables"]["definitions"] = {v[0]: v[1].strip() for v in var_defs[:50]}
    results["css_variables"]["usage_count"] = len(re.findall(r'var\s*\(\s*--', css_content))

    # Media queries
    media_queries = re.findall(r'@media\s*\(([^)]+)\)', css_content)
    results["media_queries"]["breakpoints"] = list(set([m.strip() for m in media_queries]))[:20]

    if 'prefers-reduced-motion' in css_content:
        results["media_queries"]["features"].append("prefers-reduced-motion")
    if 'prefers-color-scheme' in css_content:
        results["media_queries"]["features"].append("prefers-color-scheme")

    # Colors
    hex_colors = re.findall(r'#([0-9a-fA-F]{3,8})\b', css_content)
    rgb_colors = re.findall(r'rgba?\s*\([^)]+\)', css_content)
    hsl_colors = re.findall(r'hsla?\s*\([^)]+\)', css_content)
    results["colors"]["palette"] = list(set(['#' + c for c in hex_colors[:30]] + rgb_colors[:20] + hsl_colors[:10]))

    gradients = re.findall(r'(linear-gradient|radial-gradient|conic-gradient)\s*\([^)]+\)', css_content)
    results["colors"]["gradients"] = list(set(gradients))[:10]

    # Typography
    font_families = re.findall(r'font-family\s*:\s*([^;]+);', css_content)
    results["typography"]["font_families"] = list(set([f.strip() for f in font_families]))[:15]

    font_sizes = re.findall(r'font-size\s*:\s*([^;]+);', css_content)
    results["typography"]["font_sizes"] = list(set([f.strip() for f in font_sizes]))[:20]

    return results


def analyze_js(js_content):
    """Deep analysis of JavaScript content."""
    results = {
        "animation_libraries": {},
        "3d_libraries": {},
        "scroll_libraries": {},
        "native_animation": {},
        "observers": {},
        "event_handlers": {},
        "ui_frameworks": {},
        "webgl_webgpu": {},
        "custom_animations": [],
        "easing_functions": [],
        "shaders": {
            "vertex_shaders": [],
            "fragment_shaders": [],
            "shader_uniforms": [],
            "shader_attributes": [],
        },
        "three_js_objects": {
            "geometries": [],
            "materials": [],
            "lights": [],
            "controls": [],
        },
    }

    # Check each category
    for category, patterns in JS_PATTERNS.items():
        if category not in results:
            results[category] = {}
        for name, pattern_list in patterns.items():
            matches = []
            for pattern in pattern_list:
                found = re.findall(pattern, js_content, re.IGNORECASE)
                if found:
                    matches.extend(found if isinstance(found[0], str) else [str(f) for f in found])
            if matches:
                results[category][name] = {
                    "detected": True,
                    "occurrences": len(matches),
                    "samples": list(set(matches))[:5]
                }

    # Find custom animation functions
    custom_anim = re.findall(r'function\s+(\w*anim\w*)\s*\(', js_content, re.IGNORECASE)
    custom_anim += re.findall(r'const\s+(\w*anim\w*)\s*=', js_content, re.IGNORECASE)
    custom_anim += re.findall(r'let\s+(\w*anim\w*)\s*=', js_content, re.IGNORECASE)
    results["custom_animations"] = list(set(custom_anim))[:20]

    # Find easing functions
    easing_patterns = [
        'easeIn', 'easeOut', 'easeInOut', 'linear', 'ease',
        'cubic-bezier', 'spring', 'bounce', 'elastic'
    ]
    found_easing = []
    for easing in easing_patterns:
        if easing.lower() in js_content.lower():
            found_easing.append(easing)
    results["easing_functions"] = found_easing

    # Extract shader-related code
    # Find GLSL shader code (often in template strings or comments)
    vertex_patterns = [
        r'gl_Position\s*=',
        r'attribute\s+vec\d+\s+(\w+)',
        r'varying\s+vec\d+\s+(\w+)',
    ]
    fragment_patterns = [
        r'gl_FragColor\s*=',
        r'precision\s+(highp|mediump|lowp)\s+float',
        r'uniform\s+sampler2D\s+(\w+)',
    ]

    for pattern in vertex_patterns:
        matches = re.findall(pattern, js_content)
        if matches:
            results["shaders"]["vertex_shaders"].extend(matches if isinstance(matches[0], str) else [m[0] for m in matches])

    for pattern in fragment_patterns:
        matches = re.findall(pattern, js_content)
        if matches:
            results["shaders"]["fragment_shaders"].extend(matches if isinstance(matches[0], str) else [m[0] for m in matches])

    # Extract shader uniforms
    uniforms = re.findall(r'uniform\s+(\w+)\s+(\w+)', js_content)
    results["shaders"]["shader_uniforms"] = list(set([f"{u[0]} {u[1]}" for u in uniforms]))[:20]

    # Extract shader attributes
    attributes = re.findall(r'attribute\s+(\w+)\s+(\w+)', js_content)
    results["shaders"]["shader_attributes"] = list(set([f"{a[0]} {a[1]}" for a in attributes]))[:20]

    # Three.js specific objects
    # Geometries
    geom_patterns = [
        r'(BoxGeometry|SphereGeometry|PlaneGeometry|CylinderGeometry|TorusGeometry|BufferGeometry|CircleGeometry|RingGeometry|ConeGeometry|TetrahedronGeometry)',
    ]
    for pattern in geom_patterns:
        matches = re.findall(pattern, js_content)
        results["three_js_objects"]["geometries"].extend(matches)
    results["three_js_objects"]["geometries"] = list(set(results["three_js_objects"]["geometries"]))

    # Materials
    mat_patterns = [
        r'(MeshBasicMaterial|MeshStandardMaterial|MeshPhongMaterial|MeshLambertMaterial|ShaderMaterial|RawShaderMaterial|PointsMaterial|LineBasicMaterial|SpriteMaterial)',
    ]
    for pattern in mat_patterns:
        matches = re.findall(pattern, js_content)
        results["three_js_objects"]["materials"].extend(matches)
    results["three_js_objects"]["materials"] = list(set(results["three_js_objects"]["materials"]))

    # Lights
    light_patterns = [
        r'(AmbientLight|DirectionalLight|PointLight|SpotLight|HemisphereLight|RectAreaLight)',
    ]
    for pattern in light_patterns:
        matches = re.findall(pattern, js_content)
        results["three_js_objects"]["lights"].extend(matches)
    results["three_js_objects"]["lights"] = list(set(results["three_js_objects"]["lights"]))

    # Controls
    control_patterns = [
        r'(OrbitControls|TrackballControls|FlyControls|FirstPersonControls|PointerLockControls|DragControls)',
    ]
    for pattern in control_patterns:
        matches = re.findall(pattern, js_content)
        results["three_js_objects"]["controls"].extend(matches)
    results["three_js_objects"]["controls"] = list(set(results["three_js_objects"]["controls"]))

    return results


def analyze_html(html_content):
    """Deep analysis of HTML structure."""
    soup = BeautifulSoup(html_content, 'lxml')

    results = {
        "structure": {
            "total_elements": 0,
            "max_depth": 0,
            "semantic_elements": {},
        },
        "ui_components": {},
        "interactive_elements": {},
        "media_elements": {},
        "accessibility": {
            "aria_attributes": [],
            "roles": [],
            "landmarks": [],
        },
        "data_attributes": {},
        "svg_analysis": {
            "count": 0,
            "animated": False,
            "paths": 0,
            "filters": 0,
        },
        "canvas_elements": 0,
    }

    # Count total elements
    all_elements = soup.find_all()
    results["structure"]["total_elements"] = len(all_elements)

    # Calculate max depth
    def get_depth(element, depth=0):
        children = element.find_all(recursive=False)
        if not children:
            return depth
        return max(get_depth(child, depth + 1) for child in children)

    results["structure"]["max_depth"] = get_depth(soup)

    # Semantic elements
    semantic_tags = ['header', 'nav', 'main', 'article', 'section', 'aside', 'footer', 'figure', 'figcaption']
    for tag in semantic_tags:
        count = len(soup.find_all(tag))
        if count > 0:
            results["structure"]["semantic_elements"][tag] = count

    # UI Components detection
    for component_type, keywords in UI_PATTERNS["components"].items():
        found = []
        for keyword in keywords:
            # Check classes
            elements = soup.find_all(class_=re.compile(keyword, re.IGNORECASE))
            found.extend([el.get('class', []) for el in elements])
            # Check IDs
            elements = soup.find_all(id=re.compile(keyword, re.IGNORECASE))
            found.extend([el.get('id', '') for el in elements])
            # Check data attributes
            elements = soup.find_all(attrs={f'data-{keyword}': True})
            found.extend([str(el.attrs) for el in elements])

        if found:
            results["ui_components"][component_type] = {
                "count": len(found),
                "samples": [str(f)[:100] for f in found[:5]]
            }

    # Interactive elements
    buttons = soup.find_all('button')
    links = soup.find_all('a')
    inputs = soup.find_all(['input', 'textarea', 'select'])

    results["interactive_elements"] = {
        "buttons": len(buttons),
        "links": len(links),
        "form_inputs": len(inputs),
        "clickable_divs": len(soup.find_all('div', onclick=True)),
    }

    # Media elements
    results["media_elements"] = {
        "images": len(soup.find_all('img')),
        "videos": len(soup.find_all('video')),
        "audio": len(soup.find_all('audio')),
        "iframes": len(soup.find_all('iframe')),
        "picture": len(soup.find_all('picture')),
    }

    # Accessibility
    aria_elements = soup.find_all(attrs=lambda x: x and any(k.startswith('aria-') for k in x.keys()) if isinstance(x, dict) else False)
    results["accessibility"]["aria_attributes"] = list(set([
        k for el in aria_elements
        for k in el.attrs.keys()
        if k.startswith('aria-')
    ]))[:20]

    role_elements = soup.find_all(attrs={"role": True})
    results["accessibility"]["roles"] = list(set([el.get('role') for el in role_elements]))[:20]

    # Data attributes
    data_attrs = defaultdict(int)
    for el in all_elements:
        for attr in el.attrs.keys():
            if attr.startswith('data-'):
                data_attrs[attr] += 1
    results["data_attributes"] = dict(sorted(data_attrs.items(), key=lambda x: -x[1])[:30])

    # SVG Analysis
    svgs = soup.find_all('svg')
    results["svg_analysis"]["count"] = len(svgs)

    svg_paths = 0
    svg_filters = 0
    svg_animated = False
    for svg in svgs:
        svg_paths += len(svg.find_all('path'))
        svg_filters += len(svg.find_all('filter'))
        if svg.find_all(['animate', 'animateTransform', 'animateMotion']):
            svg_animated = True

    results["svg_analysis"]["paths"] = svg_paths
    results["svg_analysis"]["filters"] = svg_filters
    results["svg_analysis"]["animated"] = svg_animated

    # Canvas
    results["canvas_elements"] = len(soup.find_all('canvas'))

    return results


def generate_classification_report(css_analysis, js_analysis, html_analysis):
    """Generate a human-readable classification report."""

    report = []
    report.append("=" * 80)
    report.append("DEEP ANALYSIS REPORT")
    report.append("=" * 80)
    report.append("")

    # Animation Classification
    report.append("## ANIMATIONS & MOTION")
    report.append("-" * 40)

    anim_score = 0

    if css_analysis["animations"]["keyframes"]:
        report.append(f"✓ CSS Keyframe Animations: {len(css_analysis['animations']['keyframes'])}")
        report.append(f"  Names: {', '.join(css_analysis['animations']['keyframes'][:10])}")
        # Show keyframe code samples
        for name in css_analysis['animations']['keyframes'][:3]:
            code = css_analysis['animations'].get('keyframes_code', {}).get(name, '')
            if code:
                code_preview = code[:100].replace('\n', ' ')
                report.append(f"  @keyframes {name}: {code_preview}...")
        anim_score += len(css_analysis['animations']['keyframes']) * 5

    if css_analysis["transitions"]["total_transitions"] > 0:
        report.append(f"✓ CSS Transitions: {css_analysis['transitions']['total_transitions']}")
        if css_analysis["transitions"]["values"]:
            for trans in css_analysis["transitions"]["values"][:5]:
                report.append(f"  • {trans}")
        if css_analysis["transitions"]["timing_functions"]:
            report.append(f"  Timing: {', '.join(css_analysis['transitions']['timing_functions'][:5])}")
        if css_analysis["transitions"]["durations"]:
            report.append(f"  Durations: {', '.join(css_analysis['transitions']['durations'][:5])}")
        anim_score += css_analysis["transitions"]["total_transitions"] * 2

    if css_analysis["transforms"]["total_transforms"] > 0:
        report.append(f"✓ CSS Transforms: {css_analysis['transforms']['total_transforms']}")
        if css_analysis["transforms"]["types"]:
            report.append(f"  Types: {', '.join(css_analysis['transforms']['types'])}")
        if css_analysis["transforms"]["values"]:
            for trans in css_analysis["transforms"]["values"][:5]:
                report.append(f"  • {trans}")
        anim_score += css_analysis["transforms"]["total_transforms"]

    # JS Animation Libraries
    for lib, data in js_analysis.get("animation_libraries", {}).items():
        if isinstance(data, dict) and data.get("detected"):
            report.append(f"✓ JS Animation Library: {lib} ({data['occurrences']} occurrences)")
            anim_score += 10

    for lib, data in js_analysis.get("3d_libraries", {}).items():
        if isinstance(data, dict) and data.get("detected"):
            report.append(f"✓ 3D Graphics Library: {lib} ({data['occurrences']} occurrences)")
            anim_score += 15

    for lib, data in js_analysis.get("scroll_libraries", {}).items():
        if isinstance(data, dict) and data.get("detected"):
            report.append(f"✓ Scroll Animation Library: {lib} ({data['occurrences']} occurrences)")
            anim_score += 10

    native_anim = js_analysis.get("native_animation", {})
    if native_anim.get("request_animation_frame", {}).get("detected"):
        report.append(f"✓ requestAnimationFrame: {native_anim['request_animation_frame']['occurrences']} calls")
        anim_score += 5

    report.append(f"\n→ Animation Complexity Score: {anim_score}")
    report.append("")

    # Interaction Classification
    report.append("## INTERACTIONS & STATES")
    report.append("-" * 40)

    if css_analysis["pseudo_states"]["hover"]:
        report.append(f"✓ Hover Effects: {len(css_analysis['pseudo_states']['hover'])}")

    if css_analysis["pseudo_states"]["focus"]:
        report.append(f"✓ Focus States: {len(css_analysis['pseudo_states']['focus'])}")

    if css_analysis["pseudo_states"]["active"]:
        report.append(f"✓ Active States: {len(css_analysis['pseudo_states']['active'])}")

    observers = js_analysis.get("observers", {})
    for obs, data in observers.items():
        if isinstance(data, dict) and data.get("detected"):
            report.append(f"✓ {obs}: {data['occurrences']} instances")

    events = js_analysis.get("event_handlers", {})
    for event, data in events.items():
        if isinstance(data, dict) and data.get("detected"):
            report.append(f"✓ {event}: {data['occurrences']} handlers")

    report.append("")

    # UI Components
    report.append("## UI COMPONENTS DETECTED")
    report.append("-" * 40)

    for comp, data in html_analysis["ui_components"].items():
        report.append(f"✓ {comp.upper()}: {data['count']} instances")

    report.append("")

    # Visual Effects
    report.append("## VISUAL EFFECTS")
    report.append("-" * 40)

    if css_analysis["visual_effects"]["shadows"]:
        report.append(f"✓ Box Shadows: {len(css_analysis['visual_effects']['shadows'])}")

    if css_analysis["visual_effects"]["filters"]:
        report.append(f"✓ CSS Filters: {len(css_analysis['visual_effects']['filters'])}")
        report.append(f"  Values: {', '.join(css_analysis['visual_effects']['filters'][:5])}")

    if css_analysis["visual_effects"]["blend_modes"]:
        report.append(f"✓ Blend Modes: {', '.join(css_analysis['visual_effects']['blend_modes'])}")

    if css_analysis["visual_effects"]["clip_paths"]:
        report.append(f"✓ Clip Paths: {len(css_analysis['visual_effects']['clip_paths'])}")

    report.append("")

    # WebGL/WebGPU
    webgl = js_analysis.get("webgl_webgpu", {})
    if any(isinstance(v, dict) and v.get("detected") for v in webgl.values()):
        report.append("## 3D GRAPHICS (WebGL/WebGPU)")
        report.append("-" * 40)
        for feature, data in webgl.items():
            if isinstance(data, dict) and data.get("detected"):
                report.append(f"✓ {feature}: {data['occurrences']} occurrences")

    # Three.js Objects
    three_objects = js_analysis.get("three_js_objects", {})
    if any(three_objects.get(k) for k in three_objects):
        report.append("")
        report.append("Three.js Components:")
        if three_objects.get("geometries"):
            report.append(f"  Geometries: {', '.join(three_objects['geometries'])}")
        if three_objects.get("materials"):
            report.append(f"  Materials: {', '.join(three_objects['materials'])}")
        if three_objects.get("lights"):
            report.append(f"  Lights: {', '.join(three_objects['lights'])}")
        if three_objects.get("controls"):
            report.append(f"  Controls: {', '.join(three_objects['controls'])}")

    # Shaders
    shaders = js_analysis.get("shaders", {})
    if any(shaders.get(k) for k in shaders):
        report.append("")
        report.append("GLSL Shaders:")
        if shaders.get("shader_uniforms"):
            report.append(f"  Uniforms: {', '.join(shaders['shader_uniforms'][:10])}")
        if shaders.get("shader_attributes"):
            report.append(f"  Attributes: {', '.join(shaders['shader_attributes'][:10])}")

    report.append("")

    # Layout
    report.append("## LAYOUT TECHNIQUES")
    report.append("-" * 40)

    if css_analysis["layout"]["flexbox_count"]:
        report.append(f"✓ Flexbox: {css_analysis['layout']['flexbox_count']} instances")

    if css_analysis["layout"]["grid_count"]:
        report.append(f"✓ CSS Grid: {css_analysis['layout']['grid_count']} instances")

    if css_analysis["layout"]["position_fixed"]:
        report.append(f"✓ Fixed Position: {css_analysis['layout']['position_fixed']} elements")

    if css_analysis["layout"]["position_sticky"]:
        report.append(f"✓ Sticky Position: {css_analysis['layout']['position_sticky']} elements")

    report.append("")

    # Design System
    report.append("## DESIGN SYSTEM")
    report.append("-" * 40)

    if css_analysis["css_variables"]["definitions"]:
        report.append(f"✓ CSS Variables: {len(css_analysis['css_variables']['definitions'])} defined")
        report.append(f"  Usage: {css_analysis['css_variables']['usage_count']} var() calls")
        # Show some variable names
        vars_preview = list(css_analysis['css_variables']['definitions'].keys())[:10]
        report.append(f"  Examples: --{', --'.join(vars_preview)}")

    if css_analysis["colors"]["palette"]:
        report.append(f"✓ Color Palette: {len(css_analysis['colors']['palette'])} colors")

    if css_analysis["colors"]["gradients"]:
        report.append(f"✓ Gradients: {len(css_analysis['colors']['gradients'])}")

    if css_analysis["typography"]["font_families"]:
        report.append(f"✓ Font Families: {len(css_analysis['typography']['font_families'])}")
        report.append(f"  Fonts: {', '.join(list(css_analysis['typography']['font_families'])[:5])}")

    report.append("")

    # Media & SVG
    report.append("## MEDIA & GRAPHICS")
    report.append("-" * 40)

    media = html_analysis["media_elements"]
    report.append(f"Images: {media['images']}")
    report.append(f"Videos: {media['videos']}")
    report.append(f"Iframes: {media['iframes']}")

    svg = html_analysis["svg_analysis"]
    if svg["count"]:
        report.append(f"SVGs: {svg['count']} (paths: {svg['paths']}, filters: {svg['filters']})")
        if svg["animated"]:
            report.append("  → Contains SMIL animations!")

    if html_analysis["canvas_elements"]:
        report.append(f"Canvas Elements: {html_analysis['canvas_elements']}")

    report.append("")

    # Accessibility
    report.append("## ACCESSIBILITY")
    report.append("-" * 40)

    a11y = html_analysis["accessibility"]
    if a11y["aria_attributes"]:
        report.append(f"✓ ARIA Attributes: {len(a11y['aria_attributes'])}")
        report.append(f"  Used: {', '.join(a11y['aria_attributes'][:10])}")

    if a11y["roles"]:
        report.append(f"✓ ARIA Roles: {', '.join(a11y['roles'][:10])}")

    report.append("")

    # Frameworks
    frameworks = js_analysis.get("ui_frameworks", {})
    detected_frameworks = [f for f, d in frameworks.items() if isinstance(d, dict) and d.get("detected")]
    if detected_frameworks:
        report.append("## FRAMEWORKS DETECTED")
        report.append("-" * 40)
        for fw in detected_frameworks:
            report.append(f"✓ {fw}")
        report.append("")

    report.append("=" * 80)

    return "\n".join(report)


def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze.py <output_directory>")
        print("Example: python analyze.py output/becaneparis")
        sys.exit(1)

    output_dir = Path(sys.argv[1])

    if not output_dir.exists():
        print(f"Error: Directory {output_dir} does not exist")
        sys.exit(1)

    print(f"🔍 Analyzing {output_dir}...")
    print()

    # Load CSS
    css_content = ""
    css_file = output_dir / "styles.css"
    if css_file.exists():
        css_content = css_file.read_text(encoding='utf-8', errors='ignore')

    # Load additional CSS files
    css_dir = output_dir / "css"
    if css_dir.exists():
        for css_path in css_dir.glob("*.css"):
            css_content += "\n" + css_path.read_text(encoding='utf-8', errors='ignore')

    print(f"📄 Loaded {len(css_content)} bytes of CSS")

    # Load JavaScript
    js_content = ""
    scripts_dir = output_dir / "scripts"
    if scripts_dir.exists():
        for js_path in scripts_dir.glob("*.js"):
            js_content += "\n" + js_path.read_text(encoding='utf-8', errors='ignore')

    print(f"📄 Loaded {len(js_content)} bytes of JavaScript")

    # Load HTML
    html_content = ""
    html_file = output_dir / "index.html"
    if html_file.exists():
        html_content = html_file.read_text(encoding='utf-8', errors='ignore')

    # Also try raw HTML
    raw_html = output_dir / "data" / "raw.html"
    if raw_html.exists():
        html_content = raw_html.read_text(encoding='utf-8', errors='ignore')

    print(f"📄 Loaded {len(html_content)} bytes of HTML")
    print()

    # Run analyses
    print("🔬 Analyzing CSS...")
    css_analysis = analyze_css(css_content)

    print("🔬 Analyzing JavaScript...")
    js_analysis = analyze_js(js_content)

    print("🔬 Analyzing HTML...")
    html_analysis = analyze_html(html_content)

    # Generate report
    print()
    report = generate_classification_report(css_analysis, js_analysis, html_analysis)
    print(report)

    # Save detailed JSON
    analysis_data = {
        "css": css_analysis,
        "javascript": js_analysis,
        "html": html_analysis,
    }

    # Clean up sets for JSON serialization
    def clean_for_json(obj):
        if isinstance(obj, set):
            return list(obj)
        elif isinstance(obj, dict):
            return {k: clean_for_json(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [clean_for_json(v) for v in obj]
        return obj

    analysis_data = clean_for_json(analysis_data)

    analysis_file = output_dir / "data" / "deep_analysis.json"
    analysis_file.parent.mkdir(parents=True, exist_ok=True)
    with open(analysis_file, 'w', encoding='utf-8') as f:
        json.dump(analysis_data, f, indent=2, ensure_ascii=False)

    print(f"\n💾 Saved detailed analysis to {analysis_file}")

    # Save report
    report_file = output_dir / "analysis_report.txt"
    with open(report_file, 'w', encoding='utf-8') as f:
        f.write(report)

    print(f"💾 Saved report to {report_file}")


if __name__ == "__main__":
    main()
