//! Deep CSS Extraction from JavaScript
//!
//! Comprehensive extraction of CSS-related patterns from JS bundles:
//!
//! ## CSS Variables
//! - style.setProperty("--var", value) - Direct assignments
//! - style.setProperty("--".concat(name), value) - Dynamic names
//! - getPropertyValue("--var") || fallback - With defaults
//! - var(--name, default) - CSS var() functions
//!
//! ## CSS Module Patterns
//! - module().className - CSS Module class references
//! - Component_class__hash - Hashed class names
//! - Mapping between semantic and hashed names
//!
//! ## Style Objects (React/JSX)
//! - style={{ color: 'red', padding: '10px' }}
//! - style: { backgroundColor: value }
//! - Inline style prop patterns
//!
//! ## Canvas/Context Styling
//! - ctx.strokeStyle, fillStyle, font, lineWidth
//! - Canvas 2D rendering context properties
//!
//! ## Dynamic Values
//! - HSL/RGB color construction via .concat()
//! - Gradient building with template literals
//! - Computed dimensions and positions
//!
//! ## Animation Patterns
//! - requestAnimationFrame with style updates
//! - Transition/animation property changes
//! - Keyframe-like value sequences

use regex::Regex;
use std::collections::{HashMap, HashSet, BTreeMap};

/// Comprehensive result of CSS extraction from JavaScript
#[derive(Debug, Clone, Default)]
pub struct JsCssExtractionResult {
    /// CSS variables with full context
    pub css_variables: HashMap<String, CssVariableInfo>,

    /// CSS Module class mappings (semantic -> hashed)
    pub css_module_classes: HashMap<String, CssModuleInfo>,

    /// Extracted style objects
    pub style_objects: Vec<StyleObject>,

    /// Canvas/context styling
    pub canvas_styles: Vec<CanvasStyle>,

    /// Dynamic color constructions
    pub dynamic_colors: Vec<DynamicColor>,

    /// Gradient patterns
    pub gradients: Vec<GradientInfo>,

    /// Animation frame patterns
    pub animations: Vec<AnimationPattern>,

    /// Theme/config objects
    pub theme_objects: Vec<ThemeObject>,

    /// All colors found (static)
    pub colors: HashSet<String>,

    /// All font definitions
    pub fonts: Vec<FontDefinition>,

    /// Statistics
    pub stats: ExtractionStats,
}

#[derive(Debug, Clone)]
pub struct CssVariableInfo {
    /// Variable name (without --)
    pub name: String,
    /// All values found for this variable
    pub values: Vec<CssVariableValue>,
    /// Default/fallback value
    pub default_value: Option<String>,
    /// Is the name dynamically constructed?
    pub is_dynamic_name: bool,
    /// Context where found
    pub contexts: Vec<VariableContext>,
}

#[derive(Debug, Clone)]
pub struct CssVariableValue {
    /// The raw value or expression
    pub raw: String,
    /// Resolved value if possible
    pub resolved: Option<String>,
    /// Is this a dynamic/computed value?
    pub is_dynamic: bool,
    /// Source expression
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableContext {
    SetProperty,          // style.setProperty("--var", value)
    SetPropertyDynamic,   // style.setProperty("--".concat(name), value)
    GetPropertyFallback,  // getPropertyValue("--var") || default
    CssVarFunction,       // var(--name, default)
    ThemeAccess,          // theme.colors.primary
    ComputedStyle,        // getComputedStyle().getPropertyValue()
}

#[derive(Debug, Clone)]
pub struct CssModuleInfo {
    /// The module variable name (e.g., 'a', 'styles')
    pub module_var: String,
    /// The class property name (e.g., 'button', 'container')
    pub class_name: String,
    /// Inferred component name
    pub component_hint: Option<String>,
    /// Usage count
    pub usage_count: usize,
}

#[derive(Debug, Clone)]
pub struct StyleObject {
    /// Element or context hint
    pub element_hint: String,
    /// CSS properties (camelCase -> value)
    pub properties: BTreeMap<String, StyleValue>,
    /// Is this a spread/merged style?
    pub is_spread: bool,
    /// Source pattern
    pub source: StyleSource,
}

#[derive(Debug, Clone)]
pub struct StyleValue {
    /// Raw value from JS
    pub raw: String,
    /// Converted CSS value
    pub css_value: Option<String>,
    /// Is dynamic/computed
    pub is_dynamic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StyleSource {
    JsxStyleProp,      // style={{ ... }}
    StyleAssignment,   // element.style = { ... }
    ObjectLiteral,     // const styles = { ... }
    SpreadMerge,       // { ...baseStyle, color: 'red' }
}

#[derive(Debug, Clone)]
pub struct CanvasStyle {
    /// Property name (strokeStyle, fillStyle, etc.)
    pub property: String,
    /// Value or expression
    pub value: String,
    /// Resolved color if possible
    pub resolved_color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DynamicColor {
    /// Color function (hsl, rgb, rgba)
    pub function: String,
    /// Parameters (may include variables)
    pub params: Vec<String>,
    /// Reconstructed CSS if possible
    pub css_color: Option<String>,
    /// Source expression
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct GradientInfo {
    /// Gradient type (linear, radial, conic)
    pub gradient_type: String,
    /// Direction/angle
    pub direction: Option<String>,
    /// Color stops
    pub color_stops: Vec<String>,
    /// Full CSS if reconstructable
    pub css_gradient: Option<String>,
    /// Source expression
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct AnimationPattern {
    /// Animation identifier
    pub name: String,
    /// Properties being animated
    pub properties: Vec<String>,
    /// CSS variables being modified
    pub css_vars_modified: Vec<String>,
    /// Is RAF-based animation
    pub uses_raf: bool,
    /// Extracted values at different states
    pub states: Vec<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct ThemeObject {
    /// Theme/config name
    pub name: String,
    /// Nested color values
    pub colors: HashMap<String, String>,
    /// Spacing values
    pub spacing: HashMap<String, String>,
    /// Font definitions
    pub fonts: HashMap<String, String>,
    /// Breakpoints
    pub breakpoints: HashMap<String, String>,
    /// Other values
    pub other: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct FontDefinition {
    /// Font family
    pub family: String,
    /// Size if specified
    pub size: Option<String>,
    /// Weight if specified
    pub weight: Option<String>,
    /// Full font shorthand
    pub shorthand: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractionStats {
    pub css_variables_found: usize,
    pub set_property_calls: usize,
    pub get_property_calls: usize,
    pub dynamic_variables: usize,
    pub css_module_classes: usize,
    pub style_objects_found: usize,
    pub canvas_styles_found: usize,
    pub colors_found: usize,
    pub gradients_found: usize,
    pub animations_found: usize,
    pub theme_objects_found: usize,
}

/// Deep CSS extractor from JavaScript code
pub struct JsCssExtractor {
    // CSS Variable patterns
    set_property_pattern: Regex,
    set_property_dynamic_pattern: Regex,
    get_property_fallback_pattern: Regex,
    get_property_simple_pattern: Regex,
    css_var_pattern: Regex,

    // CSS Module patterns
    css_module_access_pattern: Regex,
    css_module_hashed_class_pattern: Regex,

    // Style object patterns
    jsx_style_prop_pattern: Regex,
    style_object_pattern: Regex,
    style_property_pattern: Regex,

    // Canvas patterns
    canvas_style_pattern: Regex,
    canvas_font_pattern: Regex,

    // Color patterns
    hsl_concat_pattern: Regex,
    hsl_static_pattern: Regex,
    rgb_pattern: Regex,
    rgba_pattern: Regex,
    hex_pattern: Regex,

    // Gradient patterns
    linear_gradient_pattern: Regex,
    linear_gradient_concat_pattern: Regex,
    radial_gradient_pattern: Regex,

    // Animation patterns
    raf_pattern: Regex,
    raf_callback_pattern: Regex,
    transition_pattern: Regex,

    // Theme patterns
    theme_object_pattern: Regex,
    colors_object_pattern: Regex,

    // Property name mapping (camelCase -> kebab-case)
    property_map: HashMap<&'static str, &'static str>,
}

impl JsCssExtractor {
    pub fn new() -> Self {
        let mut property_map = HashMap::new();
        // Common CSS property mappings
        property_map.insert("backgroundColor", "background-color");
        property_map.insert("backgroundImage", "background-image");
        property_map.insert("backgroundSize", "background-size");
        property_map.insert("backgroundPosition", "background-position");
        property_map.insert("borderRadius", "border-radius");
        property_map.insert("borderColor", "border-color");
        property_map.insert("borderWidth", "border-width");
        property_map.insert("borderStyle", "border-style");
        property_map.insert("boxShadow", "box-shadow");
        property_map.insert("fontSize", "font-size");
        property_map.insert("fontWeight", "font-weight");
        property_map.insert("fontFamily", "font-family");
        property_map.insert("fontStyle", "font-style");
        property_map.insert("lineHeight", "line-height");
        property_map.insert("letterSpacing", "letter-spacing");
        property_map.insert("textAlign", "text-align");
        property_map.insert("textDecoration", "text-decoration");
        property_map.insert("textTransform", "text-transform");
        property_map.insert("marginTop", "margin-top");
        property_map.insert("marginRight", "margin-right");
        property_map.insert("marginBottom", "margin-bottom");
        property_map.insert("marginLeft", "margin-left");
        property_map.insert("paddingTop", "padding-top");
        property_map.insert("paddingRight", "padding-right");
        property_map.insert("paddingBottom", "padding-bottom");
        property_map.insert("paddingLeft", "padding-left");
        property_map.insert("maxWidth", "max-width");
        property_map.insert("maxHeight", "max-height");
        property_map.insert("minWidth", "min-width");
        property_map.insert("minHeight", "min-height");
        property_map.insert("flexDirection", "flex-direction");
        property_map.insert("flexWrap", "flex-wrap");
        property_map.insert("justifyContent", "justify-content");
        property_map.insert("alignItems", "align-items");
        property_map.insert("alignSelf", "align-self");
        property_map.insert("gridTemplateColumns", "grid-template-columns");
        property_map.insert("gridTemplateRows", "grid-template-rows");
        property_map.insert("gridGap", "grid-gap");
        property_map.insert("zIndex", "z-index");
        property_map.insert("pointerEvents", "pointer-events");
        property_map.insert("objectFit", "object-fit");
        property_map.insert("objectPosition", "object-position");
        property_map.insert("overflowX", "overflow-x");
        property_map.insert("overflowY", "overflow-y");
        property_map.insert("whiteSpace", "white-space");
        property_map.insert("wordBreak", "word-break");
        property_map.insert("strokeStyle", "stroke");
        property_map.insert("fillStyle", "fill");
        property_map.insert("lineWidth", "stroke-width");

        Self {
            // CSS Variable patterns
            set_property_pattern: Regex::new(
                r#"\.setProperty\s*\(\s*["']--([a-zA-Z][a-zA-Z0-9_-]*)["']\s*,\s*([^)]+)\)"#
            ).unwrap(),

            set_property_dynamic_pattern: Regex::new(
                r#"\.setProperty\s*\(\s*["']--["']\s*\.concat\s*\(\s*([^)]+)\s*\)\s*,\s*([^)]+)\)"#
            ).unwrap(),

            get_property_fallback_pattern: Regex::new(
                r#"\.getPropertyValue\s*\(\s*["']--([a-zA-Z][a-zA-Z0-9_-]*)["']\s*\)\s*\|\|\s*["']([^"']+)["']"#
            ).unwrap(),

            get_property_simple_pattern: Regex::new(
                r#"\.getPropertyValue\s*\(\s*["']--([a-zA-Z][a-zA-Z0-9_-]*)["']\s*\)"#
            ).unwrap(),

            css_var_pattern: Regex::new(
                r#"var\s*\(\s*--([a-zA-Z][a-zA-Z0-9_-]*)(?:\s*,\s*([^)]+))?\s*\)"#
            ).unwrap(),

            // CSS Module patterns - matches a().className, styles().btn, etc.
            css_module_access_pattern: Regex::new(
                r#"([a-z_$][a-zA-Z0-9_$]*)\s*\(\s*\)\s*\.([a-zA-Z_][a-zA-Z0-9_]*)"#
            ).unwrap(),

            css_module_hashed_class_pattern: Regex::new(
                r#"([A-Z][a-zA-Z0-9]*)_([a-zA-Z][a-zA-Z0-9]*)__[a-zA-Z0-9]{5,}"#
            ).unwrap(),

            // Style object patterns
            jsx_style_prop_pattern: Regex::new(
                r#"style\s*=\s*\{\s*\{([^}]+)\}\s*\}"#
            ).unwrap(),

            style_object_pattern: Regex::new(
                r#"style\s*:\s*\{([^}]+)\}"#
            ).unwrap(),

            style_property_pattern: Regex::new(
                r#"([a-zA-Z][a-zA-Z0-9]*)\s*:\s*["']?([^,"'}]+)["']?"#
            ).unwrap(),

            // Canvas patterns
            canvas_style_pattern: Regex::new(
                r#"\.(strokeStyle|fillStyle|globalAlpha|globalCompositeOperation)\s*=\s*([^;,\n]+)"#
            ).unwrap(),

            canvas_font_pattern: Regex::new(
                r#"\.font\s*=\s*["'`]([^"'`]+)["'`]"#
            ).unwrap(),

            // Color patterns - handles .concat() patterns
            hsl_concat_pattern: Regex::new(
                r#"hsl\s*\(\s*["']?\s*\.concat\s*\(\s*([^)]+)\s*\)\s*(?:,\s*["']([^"']+)["'])?"#
            ).unwrap(),

            hsl_static_pattern: Regex::new(
                r#"hsl\s*\(\s*([0-9.]+)\s*(?:deg)?\s*,\s*([0-9.]+)\s*%?\s*,\s*([0-9.]+)\s*%?\s*\)"#
            ).unwrap(),

            rgb_pattern: Regex::new(
                r#"rgb\s*\(\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*\)"#
            ).unwrap(),

            rgba_pattern: Regex::new(
                r#"rgba\s*\(\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*\)"#
            ).unwrap(),

            hex_pattern: Regex::new(
                r##"["']#([0-9a-fA-F]{3,8})["']"##
            ).unwrap(),

            // Gradient patterns
            linear_gradient_pattern: Regex::new(
                r#"linear-gradient\s*\(\s*([^)]+)\s*\)"#
            ).unwrap(),

            linear_gradient_concat_pattern: Regex::new(
                r#"linear-gradient\s*\(\s*([0-9]+)deg\s*,\s*["']\s*\.concat\s*\("#
            ).unwrap(),

            radial_gradient_pattern: Regex::new(
                r#"radial-gradient\s*\(\s*([^)]+)\s*\)"#
            ).unwrap(),

            // Animation patterns
            raf_pattern: Regex::new(
                r#"requestAnimationFrame\s*\(\s*([a-zA-Z_$][a-zA-Z0-9_$]*)\s*\)"#
            ).unwrap(),

            raf_callback_pattern: Regex::new(
                r#"requestAnimationFrame\s*\(\s*(?:function|\([^)]*\)\s*=>)"#
            ).unwrap(),

            transition_pattern: Regex::new(
                r#"transition\s*[=:]\s*["']([^"']+)["']"#
            ).unwrap(),

            // Theme patterns
            theme_object_pattern: Regex::new(
                r#"(?:const|let|var)\s+(theme|Theme|THEME)\s*=\s*\{"#
            ).unwrap(),

            colors_object_pattern: Regex::new(
                r#"colors?\s*:\s*\{([^}]+)\}"#
            ).unwrap(),

            property_map,
        }
    }

    /// Extract all CSS information from JavaScript code
    pub fn extract(&self, js_code: &str) -> JsCssExtractionResult {
        let mut result = JsCssExtractionResult::default();

        // Extract CSS variables (setProperty, getPropertyValue)
        self.extract_css_variables(js_code, &mut result);

        // Extract CSS Module class patterns
        self.extract_css_modules(js_code, &mut result);

        // Extract style objects
        self.extract_style_objects(js_code, &mut result);

        // Extract canvas/context styling
        self.extract_canvas_styles(js_code, &mut result);

        // Extract colors (static and dynamic)
        self.extract_colors(js_code, &mut result);

        // Extract gradients
        self.extract_gradients(js_code, &mut result);

        // Extract animation patterns
        self.extract_animations(js_code, &mut result);

        // Extract theme objects
        self.extract_themes(js_code, &mut result);

        // Update stats
        self.update_stats(&mut result);

        result
    }

    fn extract_css_variables(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Static setProperty calls
        for cap in self.set_property_pattern.captures_iter(js_code) {
            if let (Some(name), Some(value)) = (cap.get(1), cap.get(2)) {
                let var_name = name.as_str().to_string();
                let raw_value = value.as_str().trim().to_string();

                result.stats.set_property_calls += 1;

                let entry = result.css_variables
                    .entry(var_name.clone())
                    .or_insert_with(|| CssVariableInfo {
                        name: var_name,
                        values: Vec::new(),
                        default_value: None,
                        is_dynamic_name: false,
                        contexts: Vec::new(),
                    });

                let (resolved, is_dynamic) = self.resolve_value(&raw_value);
                entry.values.push(CssVariableValue {
                    raw: raw_value.clone(),
                    resolved,
                    is_dynamic,
                    source: "setProperty".to_string(),
                });

                if !entry.contexts.contains(&VariableContext::SetProperty) {
                    entry.contexts.push(VariableContext::SetProperty);
                }
            }
        }

        // Dynamic setProperty calls (--".concat(name))
        for cap in self.set_property_dynamic_pattern.captures_iter(js_code) {
            if let Some(name_expr) = cap.get(1) {
                result.stats.set_property_calls += 1;
                result.stats.dynamic_variables += 1;

                // Create a placeholder for dynamic variable
                let placeholder_name = format!("_dynamic_{}", result.stats.dynamic_variables);
                result.css_variables.insert(placeholder_name.clone(), CssVariableInfo {
                    name: placeholder_name,
                    values: vec![CssVariableValue {
                        raw: name_expr.as_str().to_string(),
                        resolved: None,
                        is_dynamic: true,
                        source: "setProperty.concat".to_string(),
                    }],
                    default_value: None,
                    is_dynamic_name: true,
                    contexts: vec![VariableContext::SetPropertyDynamic],
                });
            }
        }

        // getPropertyValue with fallback
        for cap in self.get_property_fallback_pattern.captures_iter(js_code) {
            if let (Some(name), Some(fallback)) = (cap.get(1), cap.get(2)) {
                let var_name = name.as_str().to_string();
                let fallback_value = fallback.as_str().to_string();

                result.stats.get_property_calls += 1;

                let entry = result.css_variables
                    .entry(var_name.clone())
                    .or_insert_with(|| CssVariableInfo {
                        name: var_name,
                        values: Vec::new(),
                        default_value: None,
                        is_dynamic_name: false,
                        contexts: Vec::new(),
                    });

                entry.default_value = Some(fallback_value.clone());
                entry.values.push(CssVariableValue {
                    raw: fallback_value,
                    resolved: Some(fallback.as_str().to_string()),
                    is_dynamic: false,
                    source: "getPropertyValue||fallback".to_string(),
                });

                if !entry.contexts.contains(&VariableContext::GetPropertyFallback) {
                    entry.contexts.push(VariableContext::GetPropertyFallback);
                }
            }
        }

        // Simple getPropertyValue (no fallback)
        for cap in self.get_property_simple_pattern.captures_iter(js_code) {
            if let Some(name) = cap.get(1) {
                let var_name = name.as_str().to_string();

                if !result.css_variables.contains_key(&var_name) {
                    result.stats.get_property_calls += 1;
                    result.css_variables.insert(var_name.clone(), CssVariableInfo {
                        name: var_name,
                        values: Vec::new(),
                        default_value: None,
                        is_dynamic_name: false,
                        contexts: vec![VariableContext::GetPropertyFallback],
                    });
                }
            }
        }
    }

    fn extract_css_modules(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // JS methods and APIs to ignore - these are NOT CSS Module classes
        let js_methods_to_ignore: HashSet<&str> = [
            // Promise methods
            "then", "catch", "finally", "resolve", "reject",
            // Array methods
            "map", "filter", "reduce", "forEach", "find", "findIndex", "indexOf", "includes",
            "concat", "slice", "splice", "push", "pop", "shift", "unshift", "join", "split",
            "sort", "reverse", "every", "some", "flat", "flatMap", "fill", "keys", "values", "entries",
            // String methods
            "trim", "replace", "replaceAll", "match", "search", "toLowerCase", "toUpperCase",
            "charAt", "charCodeAt", "substring", "substr", "startsWith", "endsWith", "padStart", "padEnd",
            "repeat", "normalize", "localeCompare",
            // Object methods
            "bind", "call", "apply", "toString", "valueOf", "hasOwnProperty", "isPrototypeOf",
            "propertyIsEnumerable", "toLocaleString", "getPrototypeOf", "setPrototypeOf",
            // DOM API methods
            "getBoundingClientRect", "querySelector", "querySelectorAll", "getElementById",
            "getElementsByClassName", "getElementsByTagName", "createElement", "createTextNode",
            "appendChild", "removeChild", "insertBefore", "replaceChild", "cloneNode",
            "getAttribute", "setAttribute", "removeAttribute", "hasAttribute", "addEventListener",
            "removeEventListener", "dispatchEvent", "focus", "blur", "click", "submit",
            "getComputedStyle", "getPropertyValue", "setProperty", "requestAnimationFrame",
            "cancelAnimationFrame", "setTimeout", "clearTimeout", "setInterval", "clearInterval",
            // DOM position/size
            "top", "left", "right", "bottom", "width", "height", "x", "y",
            // Number/Math
            "toFixed", "toPrecision", "toExponential", "isNaN", "isFinite", "parseInt", "parseFloat",
            // Date methods
            "getTime", "getDate", "getMonth", "getFullYear", "getHours", "getMinutes", "getSeconds",
            "setTime", "setDate", "setMonth", "setFullYear", "getDay", "getTimezoneOffset",
            // JSON
            "parse", "stringify",
            // Media
            "play", "pause", "load", "canPlayType",
            // Storage
            "getItem", "setItem", "removeItem", "clear",
            // Fetch/XHR
            "fetch", "abort", "json", "text", "blob", "arrayBuffer", "formData",
            // Canvas
            "getContext", "toDataURL", "toBlob", "drawImage", "fillRect", "strokeRect", "clearRect",
            "beginPath", "closePath", "moveTo", "lineTo", "arc", "arcTo", "bezierCurveTo", "quadraticCurveTo",
            // Misc common patterns
            "next", "done", "value", "memoizedState", "createScriptURL", "onBeforeLayoutMeasure",
            "onUpdate", "transformTemplate", "whileTap",
        ].iter().cloned().collect();

        // Function names that are definitely NOT CSS module imports
        let non_module_functions: HashSet<&str> = [
            "text", "random", "resolve", "reject", "resume", "suspend", "blob", "entries",
            "toLowerCase", "toUpperCase", "trim", "shift", "slice", "reverse", "valueOf",
            "getDate", "getFullYear", "getBoundingClientRect", "getProps", "udio",
            "createScriptURL", "_getSoundIds", "tt",
        ].iter().cloned().collect();

        // Extract module().className patterns
        let mut class_counts: HashMap<String, usize> = HashMap::new();

        for cap in self.css_module_access_pattern.captures_iter(js_code) {
            if let (Some(module_var), Some(class_name)) = (cap.get(1), cap.get(2)) {
                let module = module_var.as_str();
                let class = class_name.as_str();

                // Skip if this is a known JS method (false positive)
                if js_methods_to_ignore.contains(class) {
                    continue;
                }

                // Skip if the "module" name is actually a function call
                if non_module_functions.contains(module) {
                    continue;
                }

                // Skip single-letter module names that are common variable names used for non-CSS purposes
                // Only accept single letters that are commonly used for CSS modules: a, c, m, s, u, etc.
                // with valid CSS-like class names (not JS method names)
                if module.len() == 1 && !class.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                    continue;
                }

                let key = format!("{}().{}", module, class);

                *class_counts.entry(key.clone()).or_insert(0) += 1;

                if !result.css_module_classes.contains_key(&key) {
                    result.css_module_classes.insert(key.clone(), CssModuleInfo {
                        module_var: module.to_string(),
                        class_name: class.to_string(),
                        component_hint: None,
                        usage_count: 0,
                    });
                }
            }
        }

        // Update usage counts
        for (key, count) in class_counts {
            if let Some(info) = result.css_module_classes.get_mut(&key) {
                info.usage_count = count;
            }
        }

        // Extract hashed class names to infer component hints
        for cap in self.css_module_hashed_class_pattern.captures_iter(js_code) {
            if let (Some(component), Some(class)) = (cap.get(1), cap.get(2)) {
                let component_name = component.as_str().to_string();
                let class_name = class.as_str().to_string();

                // Try to match with extracted module classes
                for info in result.css_module_classes.values_mut() {
                    if info.class_name == class_name && info.component_hint.is_none() {
                        info.component_hint = Some(component_name.clone());
                    }
                }
            }
        }

        result.stats.css_module_classes = result.css_module_classes.len();
    }

    fn extract_style_objects(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // JSX style={{ ... }} patterns
        for cap in self.jsx_style_prop_pattern.captures_iter(js_code) {
            if let Some(content) = cap.get(1) {
                if let Some(style_obj) = self.parse_style_content(content.as_str(), StyleSource::JsxStyleProp) {
                    result.style_objects.push(style_obj);
                    result.stats.style_objects_found += 1;
                }
            }
        }

        // style: { ... } patterns
        for cap in self.style_object_pattern.captures_iter(js_code) {
            if let Some(content) = cap.get(1) {
                if let Some(style_obj) = self.parse_style_content(content.as_str(), StyleSource::ObjectLiteral) {
                    result.style_objects.push(style_obj);
                    result.stats.style_objects_found += 1;
                }
            }
        }
    }

    fn parse_style_content(&self, content: &str, source: StyleSource) -> Option<StyleObject> {
        let mut properties = BTreeMap::new();

        for cap in self.style_property_pattern.captures_iter(content) {
            if let (Some(prop), Some(value)) = (cap.get(1), cap.get(2)) {
                let prop_name = prop.as_str().to_string();
                let raw_value = value.as_str().trim().to_string();

                // Skip JS expressions
                if self.is_js_expression(&raw_value) {
                    continue;
                }

                let (resolved, is_dynamic) = self.resolve_value(&raw_value);

                properties.insert(prop_name, StyleValue {
                    raw: raw_value,
                    css_value: resolved,
                    is_dynamic,
                });
            }
        }

        if properties.is_empty() {
            return None;
        }

        Some(StyleObject {
            element_hint: "extracted".to_string(),
            properties,
            is_spread: content.contains("..."),
            source,
        })
    }

    fn extract_canvas_styles(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Canvas style properties
        for cap in self.canvas_style_pattern.captures_iter(js_code) {
            if let (Some(prop), Some(value)) = (cap.get(1), cap.get(2)) {
                let property = prop.as_str().to_string();
                let raw_value = value.as_str().trim().to_string();

                let resolved = self.try_resolve_color(&raw_value);

                result.canvas_styles.push(CanvasStyle {
                    property,
                    value: raw_value,
                    resolved_color: resolved,
                });
                result.stats.canvas_styles_found += 1;
            }
        }

        // Canvas font
        for cap in self.canvas_font_pattern.captures_iter(js_code) {
            if let Some(font) = cap.get(1) {
                let font_str = font.as_str().to_string();

                // Parse font shorthand
                if let Some(font_def) = self.parse_font_shorthand(&font_str) {
                    result.fonts.push(font_def);
                }
            }
        }
    }

    fn extract_colors(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Static HSL colors
        for cap in self.hsl_static_pattern.captures_iter(js_code) {
            if let (Some(h), Some(s), Some(l)) = (cap.get(1), cap.get(2), cap.get(3)) {
                let color = format!("hsl({}deg, {}%, {}%)", h.as_str(), s.as_str(), l.as_str());
                result.colors.insert(color);
            }
        }

        // Dynamic HSL with concat - try to extract static parts
        for cap in self.hsl_concat_pattern.captures_iter(js_code) {
            let source = cap.get(0).map(|m| m.as_str()).unwrap_or("").to_string();

            // Try to extract any static color values nearby
            if let Some(saturation) = cap.get(2) {
                result.dynamic_colors.push(DynamicColor {
                    function: "hsl".to_string(),
                    params: vec![
                        "dynamic".to_string(),
                        saturation.as_str().to_string(),
                    ],
                    css_color: None,
                    source,
                });
            }
        }

        // RGB colors
        for cap in self.rgb_pattern.captures_iter(js_code) {
            if let (Some(r), Some(g), Some(b)) = (cap.get(1), cap.get(2), cap.get(3)) {
                let color = format!("rgb({}, {}, {})", r.as_str(), g.as_str(), b.as_str());
                result.colors.insert(color);
            }
        }

        // RGBA colors
        for cap in self.rgba_pattern.captures_iter(js_code) {
            if let (Some(r), Some(g), Some(b), Some(a)) = (cap.get(1), cap.get(2), cap.get(3), cap.get(4)) {
                let color = format!("rgba({}, {}, {}, {})", r.as_str(), g.as_str(), b.as_str(), a.as_str());
                result.colors.insert(color);
            }
        }

        // Hex colors
        for cap in self.hex_pattern.captures_iter(js_code) {
            if let Some(hex) = cap.get(1) {
                let color = format!("#{}", hex.as_str());
                result.colors.insert(color);
            }
        }

        result.stats.colors_found = result.colors.len();
    }

    fn extract_gradients(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Linear gradients
        for cap in self.linear_gradient_pattern.captures_iter(js_code) {
            if let Some(content) = cap.get(1) {
                let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");

                result.gradients.push(GradientInfo {
                    gradient_type: "linear".to_string(),
                    direction: self.extract_gradient_direction(content.as_str()),
                    color_stops: self.extract_color_stops(content.as_str()),
                    css_gradient: Some(full_match.to_string()),
                    source: full_match.to_string(),
                });
            }
        }

        // Linear gradients with concat (partial)
        for cap in self.linear_gradient_concat_pattern.captures_iter(js_code) {
            if let Some(angle) = cap.get(1) {
                result.gradients.push(GradientInfo {
                    gradient_type: "linear".to_string(),
                    direction: Some(format!("{}deg", angle.as_str())),
                    color_stops: vec!["dynamic".to_string()],
                    css_gradient: None,
                    source: cap.get(0).map(|m| m.as_str()).unwrap_or("").to_string(),
                });
            }
        }

        // Radial gradients
        for cap in self.radial_gradient_pattern.captures_iter(js_code) {
            if let Some(content) = cap.get(1) {
                let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");

                result.gradients.push(GradientInfo {
                    gradient_type: "radial".to_string(),
                    direction: None,
                    color_stops: self.extract_color_stops(content.as_str()),
                    css_gradient: Some(full_match.to_string()),
                    source: full_match.to_string(),
                });
            }
        }

        result.stats.gradients_found = result.gradients.len();
    }

    fn extract_animations(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Find RAF patterns and associated style changes
        let mut raf_callbacks: HashSet<String> = HashSet::new();

        for cap in self.raf_pattern.captures_iter(js_code) {
            if let Some(callback) = cap.get(1) {
                raf_callbacks.insert(callback.as_str().to_string());
            }
        }

        // Count RAF usage
        let raf_count = self.raf_pattern.find_iter(js_code).count() +
                        self.raf_callback_pattern.find_iter(js_code).count();

        if raf_count > 0 {
            // Look for style modifications near RAF
            let mut animated_properties: Vec<String> = Vec::new();
            let mut css_vars_modified: Vec<String> = Vec::new();

            // Find setProperty calls that might be in animation loops
            for cap in self.set_property_pattern.captures_iter(js_code) {
                if let Some(name) = cap.get(1) {
                    css_vars_modified.push(name.as_str().to_string());
                }
            }

            // Look for canvas style properties that might be animated
            for cap in self.canvas_style_pattern.captures_iter(js_code) {
                if let Some(prop) = cap.get(1) {
                    animated_properties.push(prop.as_str().to_string());
                }
            }

            if !animated_properties.is_empty() || !css_vars_modified.is_empty() {
                result.animations.push(AnimationPattern {
                    name: "raf_animation".to_string(),
                    properties: animated_properties,
                    css_vars_modified,
                    uses_raf: true,
                    states: Vec::new(),
                });
                result.stats.animations_found += 1;
            }
        }

        // Extract transition properties
        for cap in self.transition_pattern.captures_iter(js_code) {
            if let Some(value) = cap.get(1) {
                result.animations.push(AnimationPattern {
                    name: "css_transition".to_string(),
                    properties: vec![value.as_str().to_string()],
                    css_vars_modified: Vec::new(),
                    uses_raf: false,
                    states: Vec::new(),
                });
            }
        }
    }

    fn extract_themes(&self, js_code: &str, result: &mut JsCssExtractionResult) {
        // Look for theme objects
        if self.theme_object_pattern.is_match(js_code) {
            let mut theme = ThemeObject {
                name: "theme".to_string(),
                colors: HashMap::new(),
                spacing: HashMap::new(),
                fonts: HashMap::new(),
                breakpoints: HashMap::new(),
                other: HashMap::new(),
            };

            // Extract colors sub-object
            for cap in self.colors_object_pattern.captures_iter(js_code) {
                if let Some(content) = cap.get(1) {
                    for prop_cap in self.style_property_pattern.captures_iter(content.as_str()) {
                        if let (Some(name), Some(value)) = (prop_cap.get(1), prop_cap.get(2)) {
                            let color_name = name.as_str().to_string();
                            let color_value = value.as_str().trim().trim_matches('"').trim_matches('\'').to_string();

                            if color_value.starts_with('#') ||
                               color_value.starts_with("rgb") ||
                               color_value.starts_with("hsl") {
                                theme.colors.insert(color_name, color_value);
                            }
                        }
                    }
                }
            }

            if !theme.colors.is_empty() {
                result.theme_objects.push(theme);
                result.stats.theme_objects_found += 1;
            }
        }
    }

    fn resolve_value(&self, raw: &str) -> (Option<String>, bool) {
        let trimmed = raw.trim().trim_matches('"').trim_matches('\'');

        // Check if it's a simple static value
        if trimmed.starts_with('#') ||
           trimmed.starts_with("rgb") ||
           trimmed.starts_with("hsl") ||
           trimmed.ends_with("px") ||
           trimmed.ends_with("em") ||
           trimmed.ends_with("rem") ||
           trimmed.ends_with("%") ||
           trimmed.parse::<f64>().is_ok() {
            return (Some(trimmed.to_string()), false);
        }

        // Check for var() references
        if trimmed.starts_with("var(") {
            return (Some(trimmed.to_string()), false);
        }

        // Check for template literal or concat
        if trimmed.contains("concat") || trimmed.contains("${") || trimmed.contains("`") {
            return (None, true);
        }

        // Check for variable reference
        if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') &&
           !trimmed.is_empty() &&
           trimmed.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
            return (None, true);
        }

        (Some(trimmed.to_string()), false)
    }

    fn try_resolve_color(&self, value: &str) -> Option<String> {
        let trimmed = value.trim();

        // Direct color values
        if trimmed.starts_with('#') ||
           trimmed.starts_with("rgb") ||
           trimmed.starts_with("hsl") {
            return Some(trimmed.to_string());
        }

        // getPropertyValue fallback pattern
        if let Some(cap) = self.get_property_fallback_pattern.captures(trimmed) {
            return cap.get(2).map(|m| m.as_str().to_string());
        }

        None
    }

    fn parse_font_shorthand(&self, font: &str) -> Option<FontDefinition> {
        let parts: Vec<&str> = font.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let mut size = None;
        let mut weight = None;
        let mut family = None;

        for part in &parts {
            if part.ends_with("px") || part.ends_with("em") || part.ends_with("rem") || part.ends_with("pt") {
                size = Some(part.to_string());
            } else if part.parse::<u32>().is_ok() || *part == "bold" || *part == "normal" || *part == "lighter" {
                weight = Some(part.to_string());
            } else if !part.is_empty() {
                family = Some(part.to_string());
            }
        }

        Some(FontDefinition {
            family: family.unwrap_or_else(|| "sans-serif".to_string()),
            size,
            weight,
            shorthand: Some(font.to_string()),
        })
    }

    fn extract_gradient_direction(&self, content: &str) -> Option<String> {
        let parts: Vec<&str> = content.split(',').collect();
        if let Some(first) = parts.first() {
            let trimmed = first.trim();
            if trimmed.ends_with("deg") ||
               trimmed.starts_with("to ") ||
               trimmed == "circle" ||
               trimmed == "ellipse" {
                return Some(trimmed.to_string());
            }
        }
        None
    }

    fn extract_color_stops(&self, content: &str) -> Vec<String> {
        let mut stops = Vec::new();

        for part in content.split(',').skip(1) {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                stops.push(trimmed.to_string());
            }
        }

        stops
    }

    fn is_js_expression(&self, value: &str) -> bool {
        value.contains("concat(") ||
        value.contains("function") ||
        value.contains("=>") ||
        value.contains("${") ||
        value.starts_with("e.") ||
        value.starts_with("t.") ||
        value.starts_with("r.") ||
        value.starts_with("n.") ||
        value.starts_with("o.") ||
        value.starts_with("this.") ||
        value.contains("?") ||  // ternary
        value.contains("&&") ||
        value.contains("||") ||
        (value.len() <= 2 && value.chars().all(|c| c.is_alphabetic()))
    }

    fn update_stats(&self, result: &mut JsCssExtractionResult) {
        result.stats.css_variables_found = result.css_variables.len();
    }

    /// Convert camelCase to kebab-case
    pub fn camel_to_kebab(&self, s: &str) -> String {
        if let Some(mapped) = self.property_map.get(s) {
            return mapped.to_string();
        }

        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Generate useful CSS from extraction result
    ///
    /// Only generates CSS that provides real value:
    /// - CSS variables with resolved values (for runtime theme switching)
    /// - Theme objects with color definitions
    ///
    /// Does NOT generate:
    /// - CSS Module comments (these classes already exist in the original CSS)
    /// - Orphan color palettes (--js-color-N variables nobody uses)
    /// - Malformed gradients from dynamic concatenation
    /// - Generic .canvas-context classes that don't match HTML
    /// - Style objects as .js-object-N (these don't match HTML classes)
    pub fn generate_css(&self, result: &JsCssExtractionResult) -> String {
        let mut css = String::new();

        // Only generate CSS variables that have REAL resolved values
        // These are useful for runtime theme switching via JS
        let useful_vars: Vec<_> = result.css_variables.iter()
            .filter(|(_, info)| {
                // Skip dynamic names (placeholders)
                if info.is_dynamic_name {
                    return false;
                }
                // Must have a real value (not inherit, not a JS expression)
                let value = info.default_value.as_ref()
                    .or_else(|| {
                        info.values.iter()
                            .find(|v| v.resolved.is_some() && !v.is_dynamic)
                            .and_then(|v| v.resolved.as_ref())
                    });

                if let Some(v) = value {
                    !self.is_js_expression(v) && v != "inherit" && !v.is_empty()
                } else {
                    false
                }
            })
            .collect();

        if !useful_vars.is_empty() {
            css.push_str("/* ====================================\n");
            css.push_str("   CSS Variables from JavaScript\n");
            css.push_str("   (Runtime theme values)\n");
            css.push_str("   ==================================== */\n");
            css.push_str(":root {\n");

            for (name, info) in useful_vars {
                let value = info.default_value.as_ref()
                    .or_else(|| {
                        info.values.iter()
                            .find(|v| v.resolved.is_some() && !v.is_dynamic)
                            .and_then(|v| v.resolved.as_ref())
                    })
                    .cloned()
                    .unwrap_or_default();

                // Only add context comment if it's meaningful
                let context_str = if info.contexts.contains(&VariableContext::GetPropertyFallback) {
                    " /* fallback value */"
                } else {
                    ""
                };

                css.push_str(&format!("  --{}: {};{}\n", name, value, context_str));
            }

            css.push_str("}\n\n");
        }

        // Generate theme variables if they have actual color values
        for theme in &result.theme_objects {
            let useful_colors: Vec<_> = theme.colors.iter()
                .filter(|(_, v)| !self.is_js_expression(v) && !v.is_empty())
                .collect();

            if !useful_colors.is_empty() {
                css.push_str("/* ====================================\n");
                css.push_str(&format!("   Theme: {}\n", theme.name));
                css.push_str("   ==================================== */\n");
                css.push_str(":root {\n");

                for (name, value) in useful_colors {
                    css.push_str(&format!("  --theme-{}: {};\n", name, value));
                }

                css.push_str("}\n\n");
            }
        }

        // NOTE: We intentionally DO NOT generate:
        //
        // 1. CSS Module class comments - These classes already exist in the
        //    original CSS with their hashed names. Comments like
        //    "u().rect → .HeroDev_rect__[hash]" provide no value.
        //
        // 2. Color palette as --js-color-N - These are orphan variables that
        //    nothing references. The actual colors are used inline in the
        //    original CSS where needed.
        //
        // 3. Gradients from .concat() - These produce malformed CSS like
        //    "linear-gradient(20deg, ".concat(l,", ");" which is invalid.
        //
        // 4. .canvas-context class - Canvas styling is imperative via JS,
        //    not CSS classes. This class doesn't appear in HTML.
        //
        // 5. .js-object-N classes - Style objects in JS are applied inline
        //    or via existing CSS classes. Generic numbered classes don't
        //    match any HTML structure.
        //
        // 6. Animation comments - Just metadata, not usable CSS.

        css
    }
}

impl Default for JsCssExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_property_extraction() {
        let extractor = JsCssExtractor::new();
        let js_code = r##"
            g.style.setProperty("--fontColor", d);
            g.style.setProperty("--highlightColor", "#ff0000");
        "##;

        let result = extractor.extract(js_code);
        assert!(result.css_variables.contains_key("fontColor"));
        assert!(result.css_variables.contains_key("highlightColor"));
    }

    #[test]
    fn test_get_property_fallback() {
        let extractor = JsCssExtractor::new();
        let js_code = r##"
            e.strokeStyle = document.body.style.getPropertyValue("--artStroke") || "#1d1d1d";
        "##;

        let result = extractor.extract(js_code);
        assert!(result.css_variables.contains_key("artStroke"));
        assert_eq!(
            result.css_variables.get("artStroke").unwrap().default_value,
            Some("#1d1d1d".to_string())
        );
    }

    #[test]
    fn test_css_module_extraction() {
        let extractor = JsCssExtractor::new();
        let js_code = r##"
            className: r()(a().button, a().primary)
            className: c().icon
            className: styles().container
        "##;

        let result = extractor.extract(js_code);
        assert!(result.css_module_classes.contains_key("a().button"));
        assert!(result.css_module_classes.contains_key("c().icon"));
        assert!(result.css_module_classes.contains_key("styles().container"));
    }

    #[test]
    fn test_canvas_style_extraction() {
        let extractor = JsCssExtractor::new();
        let js_code = r##"
            ctx.strokeStyle = "#ff0000";
            ctx.fillStyle = "rgba(0, 0, 0, 0.5)";
            ctx.lineWidth = 2;
        "##;

        let result = extractor.extract(js_code);
        assert_eq!(result.canvas_styles.len(), 2);
        assert!(result.canvas_styles.iter().any(|s| s.property == "strokeStyle"));
    }
}
