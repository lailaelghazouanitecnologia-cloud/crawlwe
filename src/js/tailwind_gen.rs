//! Tailwind CSS Generator - Complete Coverage
//!
//! Generates CSS rules for ALL Tailwind utility classes.
//! Comprehensive hardcoded mappings for maximum compatibility.

use std::collections::{HashMap, HashSet};

/// Tailwind generation result
#[derive(Debug, Clone, Default)]
pub struct TailwindGenResult {
    pub css: String,
    pub classes_processed: usize,
    pub classes_generated: usize,
    pub unknown_classes: Vec<String>,
}

/// Complete Tailwind CSS Generator
pub struct TailwindGenerator {
    /// All utility mappings
    utilities: HashMap<&'static str, &'static str>,
    /// Spacing scale
    spacing: HashMap<&'static str, &'static str>,
    /// Colors (full palette)
    colors: HashMap<&'static str, &'static str>,
    /// Breakpoints
    breakpoints: HashMap<&'static str, &'static str>,
}

impl TailwindGenerator {
    pub fn new() -> Self {
        let mut gen = Self {
            utilities: HashMap::new(),
            spacing: HashMap::new(),
            colors: HashMap::new(),
            breakpoints: HashMap::new(),
        };
        gen.init_spacing();
        gen.init_colors();
        gen.init_breakpoints();
        gen.init_utilities();
        gen
    }

    fn init_spacing(&mut self) {
        let values = [
            ("0", "0px"), ("px", "1px"), ("0.5", "0.125rem"), ("1", "0.25rem"),
            ("1.5", "0.375rem"), ("2", "0.5rem"), ("2.5", "0.625rem"), ("3", "0.75rem"),
            ("3.5", "0.875rem"), ("4", "1rem"), ("5", "1.25rem"), ("6", "1.5rem"),
            ("7", "1.75rem"), ("8", "2rem"), ("9", "2.25rem"), ("10", "2.5rem"),
            ("11", "2.75rem"), ("12", "3rem"), ("14", "3.5rem"), ("16", "4rem"),
            ("20", "5rem"), ("24", "6rem"), ("28", "7rem"), ("32", "8rem"),
            ("36", "9rem"), ("40", "10rem"), ("44", "11rem"), ("48", "12rem"),
            ("52", "13rem"), ("56", "14rem"), ("60", "15rem"), ("64", "16rem"),
            ("72", "18rem"), ("80", "20rem"), ("96", "24rem"),
            ("auto", "auto"), ("full", "100%"), ("screen", "100vw"), ("min", "min-content"),
            ("max", "max-content"), ("fit", "fit-content"),
        ];
        for (k, v) in values { self.spacing.insert(k, v); }
    }

    fn init_colors(&mut self) {
        let colors = [
            // Base
            ("transparent", "transparent"), ("current", "currentColor"),
            ("black", "#000000"), ("white", "#ffffff"),
            // Slate
            ("slate-50", "#f8fafc"), ("slate-100", "#f1f5f9"), ("slate-200", "#e2e8f0"),
            ("slate-300", "#cbd5e1"), ("slate-400", "#94a3b8"), ("slate-500", "#64748b"),
            ("slate-600", "#475569"), ("slate-700", "#334155"), ("slate-800", "#1e293b"),
            ("slate-900", "#0f172a"), ("slate-950", "#020617"),
            // Gray
            ("gray-50", "#f9fafb"), ("gray-100", "#f3f4f6"), ("gray-200", "#e5e7eb"),
            ("gray-300", "#d1d5db"), ("gray-400", "#9ca3af"), ("gray-500", "#6b7280"),
            ("gray-600", "#4b5563"), ("gray-700", "#374151"), ("gray-800", "#1f2937"),
            ("gray-900", "#111827"), ("gray-950", "#030712"),
            // Zinc
            ("zinc-50", "#fafafa"), ("zinc-100", "#f4f4f5"), ("zinc-200", "#e4e4e7"),
            ("zinc-300", "#d4d4d8"), ("zinc-400", "#a1a1aa"), ("zinc-500", "#71717a"),
            ("zinc-600", "#52525b"), ("zinc-700", "#3f3f46"), ("zinc-800", "#27272a"),
            ("zinc-900", "#18181b"), ("zinc-950", "#09090b"),
            // Neutral
            ("neutral-50", "#fafafa"), ("neutral-100", "#f5f5f5"), ("neutral-200", "#e5e5e5"),
            ("neutral-300", "#d4d4d4"), ("neutral-400", "#a3a3a3"), ("neutral-500", "#737373"),
            ("neutral-600", "#525252"), ("neutral-700", "#404040"), ("neutral-800", "#262626"),
            ("neutral-900", "#171717"), ("neutral-950", "#0a0a0a"),
            // Stone
            ("stone-50", "#fafaf9"), ("stone-100", "#f5f5f4"), ("stone-200", "#e7e5e4"),
            ("stone-300", "#d6d3d1"), ("stone-400", "#a8a29e"), ("stone-500", "#78716c"),
            ("stone-600", "#57534e"), ("stone-700", "#44403c"), ("stone-800", "#292524"),
            ("stone-900", "#1c1917"), ("stone-950", "#0c0a09"),
            // Red
            ("red-50", "#fef2f2"), ("red-100", "#fee2e2"), ("red-200", "#fecaca"),
            ("red-300", "#fca5a5"), ("red-400", "#f87171"), ("red-500", "#ef4444"),
            ("red-600", "#dc2626"), ("red-700", "#b91c1c"), ("red-800", "#991b1b"),
            ("red-900", "#7f1d1d"), ("red-950", "#450a0a"),
            // Orange
            ("orange-50", "#fff7ed"), ("orange-100", "#ffedd5"), ("orange-200", "#fed7aa"),
            ("orange-300", "#fdba74"), ("orange-400", "#fb923c"), ("orange-500", "#f97316"),
            ("orange-600", "#ea580c"), ("orange-700", "#c2410c"), ("orange-800", "#9a3412"),
            ("orange-900", "#7c2d12"), ("orange-950", "#431407"),
            // Amber
            ("amber-50", "#fffbeb"), ("amber-100", "#fef3c7"), ("amber-200", "#fde68a"),
            ("amber-300", "#fcd34d"), ("amber-400", "#fbbf24"), ("amber-500", "#f59e0b"),
            ("amber-600", "#d97706"), ("amber-700", "#b45309"), ("amber-800", "#92400e"),
            ("amber-900", "#78350f"), ("amber-950", "#451a03"),
            // Yellow
            ("yellow-50", "#fefce8"), ("yellow-100", "#fef9c3"), ("yellow-200", "#fef08a"),
            ("yellow-300", "#fde047"), ("yellow-400", "#facc15"), ("yellow-500", "#eab308"),
            ("yellow-600", "#ca8a04"), ("yellow-700", "#a16207"), ("yellow-800", "#854d0e"),
            ("yellow-900", "#713f12"), ("yellow-950", "#422006"),
            // Lime
            ("lime-50", "#f7fee7"), ("lime-100", "#ecfccb"), ("lime-200", "#d9f99d"),
            ("lime-300", "#bef264"), ("lime-400", "#a3e635"), ("lime-500", "#84cc16"),
            ("lime-600", "#65a30d"), ("lime-700", "#4d7c0f"), ("lime-800", "#3f6212"),
            ("lime-900", "#365314"), ("lime-950", "#1a2e05"),
            // Green
            ("green-50", "#f0fdf4"), ("green-100", "#dcfce7"), ("green-200", "#bbf7d0"),
            ("green-300", "#86efac"), ("green-400", "#4ade80"), ("green-500", "#22c55e"),
            ("green-600", "#16a34a"), ("green-700", "#15803d"), ("green-800", "#166534"),
            ("green-900", "#14532d"), ("green-950", "#052e16"),
            // Emerald
            ("emerald-50", "#ecfdf5"), ("emerald-100", "#d1fae5"), ("emerald-200", "#a7f3d0"),
            ("emerald-300", "#6ee7b7"), ("emerald-400", "#34d399"), ("emerald-500", "#10b981"),
            ("emerald-600", "#059669"), ("emerald-700", "#047857"), ("emerald-800", "#065f46"),
            ("emerald-900", "#064e3b"), ("emerald-950", "#022c22"),
            // Teal
            ("teal-50", "#f0fdfa"), ("teal-100", "#ccfbf1"), ("teal-200", "#99f6e4"),
            ("teal-300", "#5eead4"), ("teal-400", "#2dd4bf"), ("teal-500", "#14b8a6"),
            ("teal-600", "#0d9488"), ("teal-700", "#0f766e"), ("teal-800", "#115e59"),
            ("teal-900", "#134e4a"), ("teal-950", "#042f2e"),
            // Cyan
            ("cyan-50", "#ecfeff"), ("cyan-100", "#cffafe"), ("cyan-200", "#a5f3fc"),
            ("cyan-300", "#67e8f9"), ("cyan-400", "#22d3ee"), ("cyan-500", "#06b6d4"),
            ("cyan-600", "#0891b2"), ("cyan-700", "#0e7490"), ("cyan-800", "#155e75"),
            ("cyan-900", "#164e63"), ("cyan-950", "#083344"),
            // Sky
            ("sky-50", "#f0f9ff"), ("sky-100", "#e0f2fe"), ("sky-200", "#bae6fd"),
            ("sky-300", "#7dd3fc"), ("sky-400", "#38bdf8"), ("sky-500", "#0ea5e9"),
            ("sky-600", "#0284c7"), ("sky-700", "#0369a1"), ("sky-800", "#075985"),
            ("sky-900", "#0c4a6e"), ("sky-950", "#082f49"),
            // Blue
            ("blue-50", "#eff6ff"), ("blue-100", "#dbeafe"), ("blue-200", "#bfdbfe"),
            ("blue-300", "#93c5fd"), ("blue-400", "#60a5fa"), ("blue-500", "#3b82f6"),
            ("blue-600", "#2563eb"), ("blue-700", "#1d4ed8"), ("blue-800", "#1e40af"),
            ("blue-900", "#1e3a8a"), ("blue-950", "#172554"),
            // Indigo
            ("indigo-50", "#eef2ff"), ("indigo-100", "#e0e7ff"), ("indigo-200", "#c7d2fe"),
            ("indigo-300", "#a5b4fc"), ("indigo-400", "#818cf8"), ("indigo-500", "#6366f1"),
            ("indigo-600", "#4f46e5"), ("indigo-700", "#4338ca"), ("indigo-800", "#3730a3"),
            ("indigo-900", "#312e81"), ("indigo-950", "#1e1b4b"),
            // Violet
            ("violet-50", "#f5f3ff"), ("violet-100", "#ede9fe"), ("violet-200", "#ddd6fe"),
            ("violet-300", "#c4b5fd"), ("violet-400", "#a78bfa"), ("violet-500", "#8b5cf6"),
            ("violet-600", "#7c3aed"), ("violet-700", "#6d28d9"), ("violet-800", "#5b21b6"),
            ("violet-900", "#4c1d95"), ("violet-950", "#2e1065"),
            // Purple
            ("purple-50", "#faf5ff"), ("purple-100", "#f3e8ff"), ("purple-200", "#e9d5ff"),
            ("purple-300", "#d8b4fe"), ("purple-400", "#c084fc"), ("purple-500", "#a855f7"),
            ("purple-600", "#9333ea"), ("purple-700", "#7e22ce"), ("purple-800", "#6b21a8"),
            ("purple-900", "#581c87"), ("purple-950", "#3b0764"),
            // Fuchsia
            ("fuchsia-50", "#fdf4ff"), ("fuchsia-100", "#fae8ff"), ("fuchsia-200", "#f5d0fe"),
            ("fuchsia-300", "#f0abfc"), ("fuchsia-400", "#e879f9"), ("fuchsia-500", "#d946ef"),
            ("fuchsia-600", "#c026d3"), ("fuchsia-700", "#a21caf"), ("fuchsia-800", "#86198f"),
            ("fuchsia-900", "#701a75"), ("fuchsia-950", "#4a044e"),
            // Pink
            ("pink-50", "#fdf2f8"), ("pink-100", "#fce7f3"), ("pink-200", "#fbcfe8"),
            ("pink-300", "#f9a8d4"), ("pink-400", "#f472b6"), ("pink-500", "#ec4899"),
            ("pink-600", "#db2777"), ("pink-700", "#be185d"), ("pink-800", "#9d174d"),
            ("pink-900", "#831843"), ("pink-950", "#500724"),
            // Rose
            ("rose-50", "#fff1f2"), ("rose-100", "#ffe4e6"), ("rose-200", "#fecdd3"),
            ("rose-300", "#fda4af"), ("rose-400", "#fb7185"), ("rose-500", "#f43f5e"),
            ("rose-600", "#e11d48"), ("rose-700", "#be123c"), ("rose-800", "#9f1239"),
            ("rose-900", "#881337"), ("rose-950", "#4c0519"),
        ];
        for (k, v) in colors { self.colors.insert(k, v); }
    }

    fn init_breakpoints(&mut self) {
        self.breakpoints.insert("sm", "640px");
        self.breakpoints.insert("md", "768px");
        self.breakpoints.insert("lg", "1024px");
        self.breakpoints.insert("xl", "1280px");
        self.breakpoints.insert("2xl", "1536px");
    }

    fn init_utilities(&mut self) {
        // === LAYOUT ===
        // Aspect Ratio
        self.utilities.insert("aspect-auto", "aspect-ratio: auto;");
        self.utilities.insert("aspect-square", "aspect-ratio: 1 / 1;");
        self.utilities.insert("aspect-video", "aspect-ratio: 16 / 9;");

        // Container
        self.utilities.insert("container", "width: 100%;");

        // Columns
        for i in 1..=12 {
            let class = Box::leak(format!("columns-{}", i).into_boxed_str());
            let css = Box::leak(format!("columns: {};", i).into_boxed_str());
            self.utilities.insert(class, css);
        }
        self.utilities.insert("columns-auto", "columns: auto;");

        // Break
        self.utilities.insert("break-after-auto", "break-after: auto;");
        self.utilities.insert("break-after-avoid", "break-after: avoid;");
        self.utilities.insert("break-after-all", "break-after: all;");
        self.utilities.insert("break-after-page", "break-after: page;");
        self.utilities.insert("break-before-auto", "break-before: auto;");
        self.utilities.insert("break-before-avoid", "break-before: avoid;");
        self.utilities.insert("break-inside-auto", "break-inside: auto;");
        self.utilities.insert("break-inside-avoid", "break-inside: avoid;");

        // Box Decoration
        self.utilities.insert("box-decoration-clone", "box-decoration-break: clone;");
        self.utilities.insert("box-decoration-slice", "box-decoration-break: slice;");

        // Box Sizing
        self.utilities.insert("box-border", "box-sizing: border-box;");
        self.utilities.insert("box-content", "box-sizing: content-box;");

        // Display
        self.utilities.insert("block", "display: block;");
        self.utilities.insert("inline-block", "display: inline-block;");
        self.utilities.insert("inline", "display: inline;");
        self.utilities.insert("flex", "display: flex;");
        self.utilities.insert("inline-flex", "display: inline-flex;");
        self.utilities.insert("table", "display: table;");
        self.utilities.insert("inline-table", "display: inline-table;");
        self.utilities.insert("table-caption", "display: table-caption;");
        self.utilities.insert("table-cell", "display: table-cell;");
        self.utilities.insert("table-column", "display: table-column;");
        self.utilities.insert("table-column-group", "display: table-column-group;");
        self.utilities.insert("table-footer-group", "display: table-footer-group;");
        self.utilities.insert("table-header-group", "display: table-header-group;");
        self.utilities.insert("table-row-group", "display: table-row-group;");
        self.utilities.insert("table-row", "display: table-row;");
        self.utilities.insert("flow-root", "display: flow-root;");
        self.utilities.insert("grid", "display: grid;");
        self.utilities.insert("inline-grid", "display: inline-grid;");
        self.utilities.insert("contents", "display: contents;");
        self.utilities.insert("list-item", "display: list-item;");
        self.utilities.insert("hidden", "display: none;");

        // Float
        self.utilities.insert("float-start", "float: inline-start;");
        self.utilities.insert("float-end", "float: inline-end;");
        self.utilities.insert("float-right", "float: right;");
        self.utilities.insert("float-left", "float: left;");
        self.utilities.insert("float-none", "float: none;");

        // Clear
        self.utilities.insert("clear-start", "clear: inline-start;");
        self.utilities.insert("clear-end", "clear: inline-end;");
        self.utilities.insert("clear-left", "clear: left;");
        self.utilities.insert("clear-right", "clear: right;");
        self.utilities.insert("clear-both", "clear: both;");
        self.utilities.insert("clear-none", "clear: none;");

        // Isolation
        self.utilities.insert("isolate", "isolation: isolate;");
        self.utilities.insert("isolation-auto", "isolation: auto;");

        // Object Fit
        self.utilities.insert("object-contain", "object-fit: contain;");
        self.utilities.insert("object-cover", "object-fit: cover;");
        self.utilities.insert("object-fill", "object-fit: fill;");
        self.utilities.insert("object-none", "object-fit: none;");
        self.utilities.insert("object-scale-down", "object-fit: scale-down;");

        // Object Position
        self.utilities.insert("object-bottom", "object-position: bottom;");
        self.utilities.insert("object-center", "object-position: center;");
        self.utilities.insert("object-left", "object-position: left;");
        self.utilities.insert("object-left-bottom", "object-position: left bottom;");
        self.utilities.insert("object-left-top", "object-position: left top;");
        self.utilities.insert("object-right", "object-position: right;");
        self.utilities.insert("object-right-bottom", "object-position: right bottom;");
        self.utilities.insert("object-right-top", "object-position: right top;");
        self.utilities.insert("object-top", "object-position: top;");

        // Overflow
        self.utilities.insert("overflow-auto", "overflow: auto;");
        self.utilities.insert("overflow-hidden", "overflow: hidden;");
        self.utilities.insert("overflow-clip", "overflow: clip;");
        self.utilities.insert("overflow-visible", "overflow: visible;");
        self.utilities.insert("overflow-scroll", "overflow: scroll;");
        self.utilities.insert("overflow-x-auto", "overflow-x: auto;");
        self.utilities.insert("overflow-y-auto", "overflow-y: auto;");
        self.utilities.insert("overflow-x-hidden", "overflow-x: hidden;");
        self.utilities.insert("overflow-y-hidden", "overflow-y: hidden;");
        self.utilities.insert("overflow-x-clip", "overflow-x: clip;");
        self.utilities.insert("overflow-y-clip", "overflow-y: clip;");
        self.utilities.insert("overflow-x-visible", "overflow-x: visible;");
        self.utilities.insert("overflow-y-visible", "overflow-y: visible;");
        self.utilities.insert("overflow-x-scroll", "overflow-x: scroll;");
        self.utilities.insert("overflow-y-scroll", "overflow-y: scroll;");

        // Overscroll
        self.utilities.insert("overscroll-auto", "overscroll-behavior: auto;");
        self.utilities.insert("overscroll-contain", "overscroll-behavior: contain;");
        self.utilities.insert("overscroll-none", "overscroll-behavior: none;");
        self.utilities.insert("overscroll-y-auto", "overscroll-behavior-y: auto;");
        self.utilities.insert("overscroll-y-contain", "overscroll-behavior-y: contain;");
        self.utilities.insert("overscroll-y-none", "overscroll-behavior-y: none;");
        self.utilities.insert("overscroll-x-auto", "overscroll-behavior-x: auto;");
        self.utilities.insert("overscroll-x-contain", "overscroll-behavior-x: contain;");
        self.utilities.insert("overscroll-x-none", "overscroll-behavior-x: none;");

        // Position
        self.utilities.insert("static", "position: static;");
        self.utilities.insert("fixed", "position: fixed;");
        self.utilities.insert("absolute", "position: absolute;");
        self.utilities.insert("relative", "position: relative;");
        self.utilities.insert("sticky", "position: sticky;");

        // Visibility
        self.utilities.insert("visible", "visibility: visible;");
        self.utilities.insert("invisible", "visibility: hidden;");
        self.utilities.insert("collapse", "visibility: collapse;");

        // Z-Index
        self.utilities.insert("z-0", "z-index: 0;");
        self.utilities.insert("z-10", "z-index: 10;");
        self.utilities.insert("z-20", "z-index: 20;");
        self.utilities.insert("z-30", "z-index: 30;");
        self.utilities.insert("z-40", "z-index: 40;");
        self.utilities.insert("z-50", "z-index: 50;");
        self.utilities.insert("z-auto", "z-index: auto;");

        // === FLEXBOX & GRID ===
        // Flex Basis
        self.utilities.insert("basis-auto", "flex-basis: auto;");
        self.utilities.insert("basis-full", "flex-basis: 100%;");

        // Flex Direction
        self.utilities.insert("flex-row", "flex-direction: row;");
        self.utilities.insert("flex-row-reverse", "flex-direction: row-reverse;");
        self.utilities.insert("flex-col", "flex-direction: column;");
        self.utilities.insert("flex-col-reverse", "flex-direction: column-reverse;");

        // Flex Wrap
        self.utilities.insert("flex-wrap", "flex-wrap: wrap;");
        self.utilities.insert("flex-wrap-reverse", "flex-wrap: wrap-reverse;");
        self.utilities.insert("flex-nowrap", "flex-wrap: nowrap;");

        // Flex
        self.utilities.insert("flex-1", "flex: 1 1 0%;");
        self.utilities.insert("flex-auto", "flex: 1 1 auto;");
        self.utilities.insert("flex-initial", "flex: 0 1 auto;");
        self.utilities.insert("flex-none", "flex: none;");

        // Flex Grow
        self.utilities.insert("grow", "flex-grow: 1;");
        self.utilities.insert("grow-0", "flex-grow: 0;");

        // Flex Shrink
        self.utilities.insert("shrink", "flex-shrink: 1;");
        self.utilities.insert("shrink-0", "flex-shrink: 0;");

        // Order
        self.utilities.insert("order-first", "order: -9999;");
        self.utilities.insert("order-last", "order: 9999;");
        self.utilities.insert("order-none", "order: 0;");
        for i in 1..=12 {
            let class = Box::leak(format!("order-{}", i).into_boxed_str());
            let css = Box::leak(format!("order: {};", i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Grid Template Columns
        self.utilities.insert("grid-cols-none", "grid-template-columns: none;");
        self.utilities.insert("grid-cols-subgrid", "grid-template-columns: subgrid;");
        for i in 1..=12 {
            let class = Box::leak(format!("grid-cols-{}", i).into_boxed_str());
            let css = Box::leak(format!("grid-template-columns: repeat({}, minmax(0, 1fr));", i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Grid Column Span
        self.utilities.insert("col-auto", "grid-column: auto;");
        self.utilities.insert("col-span-full", "grid-column: 1 / -1;");
        for i in 1..=12 {
            let class = Box::leak(format!("col-span-{}", i).into_boxed_str());
            let css = Box::leak(format!("grid-column: span {} / span {};", i, i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Grid Column Start/End
        self.utilities.insert("col-start-auto", "grid-column-start: auto;");
        self.utilities.insert("col-end-auto", "grid-column-end: auto;");
        for i in 1..=13 {
            let class = Box::leak(format!("col-start-{}", i).into_boxed_str());
            let css = Box::leak(format!("grid-column-start: {};", i).into_boxed_str());
            self.utilities.insert(class, css);
            let class = Box::leak(format!("col-end-{}", i).into_boxed_str());
            let css = Box::leak(format!("grid-column-end: {};", i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Grid Template Rows
        self.utilities.insert("grid-rows-none", "grid-template-rows: none;");
        self.utilities.insert("grid-rows-subgrid", "grid-template-rows: subgrid;");
        for i in 1..=12 {
            let class = Box::leak(format!("grid-rows-{}", i).into_boxed_str());
            let css = Box::leak(format!("grid-template-rows: repeat({}, minmax(0, 1fr));", i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Grid Row Span
        self.utilities.insert("row-auto", "grid-row: auto;");
        self.utilities.insert("row-span-full", "grid-row: 1 / -1;");
        for i in 1..=12 {
            let class = Box::leak(format!("row-span-{}", i).into_boxed_str());
            let css = Box::leak(format!("grid-row: span {} / span {};", i, i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Grid Flow
        self.utilities.insert("grid-flow-row", "grid-auto-flow: row;");
        self.utilities.insert("grid-flow-col", "grid-auto-flow: column;");
        self.utilities.insert("grid-flow-dense", "grid-auto-flow: dense;");
        self.utilities.insert("grid-flow-row-dense", "grid-auto-flow: row dense;");
        self.utilities.insert("grid-flow-col-dense", "grid-auto-flow: column dense;");

        // Auto Columns/Rows
        self.utilities.insert("auto-cols-auto", "grid-auto-columns: auto;");
        self.utilities.insert("auto-cols-min", "grid-auto-columns: min-content;");
        self.utilities.insert("auto-cols-max", "grid-auto-columns: max-content;");
        self.utilities.insert("auto-cols-fr", "grid-auto-columns: minmax(0, 1fr);");
        self.utilities.insert("auto-rows-auto", "grid-auto-rows: auto;");
        self.utilities.insert("auto-rows-min", "grid-auto-rows: min-content;");
        self.utilities.insert("auto-rows-max", "grid-auto-rows: max-content;");
        self.utilities.insert("auto-rows-fr", "grid-auto-rows: minmax(0, 1fr);");

        // Justify Content
        self.utilities.insert("justify-normal", "justify-content: normal;");
        self.utilities.insert("justify-start", "justify-content: flex-start;");
        self.utilities.insert("justify-end", "justify-content: flex-end;");
        self.utilities.insert("justify-center", "justify-content: center;");
        self.utilities.insert("justify-between", "justify-content: space-between;");
        self.utilities.insert("justify-around", "justify-content: space-around;");
        self.utilities.insert("justify-evenly", "justify-content: space-evenly;");
        self.utilities.insert("justify-stretch", "justify-content: stretch;");

        // Justify Items
        self.utilities.insert("justify-items-start", "justify-items: start;");
        self.utilities.insert("justify-items-end", "justify-items: end;");
        self.utilities.insert("justify-items-center", "justify-items: center;");
        self.utilities.insert("justify-items-stretch", "justify-items: stretch;");

        // Justify Self
        self.utilities.insert("justify-self-auto", "justify-self: auto;");
        self.utilities.insert("justify-self-start", "justify-self: start;");
        self.utilities.insert("justify-self-end", "justify-self: end;");
        self.utilities.insert("justify-self-center", "justify-self: center;");
        self.utilities.insert("justify-self-stretch", "justify-self: stretch;");

        // Align Content
        self.utilities.insert("content-normal", "align-content: normal;");
        self.utilities.insert("content-center", "align-content: center;");
        self.utilities.insert("content-start", "align-content: flex-start;");
        self.utilities.insert("content-end", "align-content: flex-end;");
        self.utilities.insert("content-between", "align-content: space-between;");
        self.utilities.insert("content-around", "align-content: space-around;");
        self.utilities.insert("content-evenly", "align-content: space-evenly;");
        self.utilities.insert("content-baseline", "align-content: baseline;");
        self.utilities.insert("content-stretch", "align-content: stretch;");

        // Align Items
        self.utilities.insert("items-start", "align-items: flex-start;");
        self.utilities.insert("items-end", "align-items: flex-end;");
        self.utilities.insert("items-center", "align-items: center;");
        self.utilities.insert("items-baseline", "align-items: baseline;");
        self.utilities.insert("items-stretch", "align-items: stretch;");

        // Align Self
        self.utilities.insert("self-auto", "align-self: auto;");
        self.utilities.insert("self-start", "align-self: flex-start;");
        self.utilities.insert("self-end", "align-self: flex-end;");
        self.utilities.insert("self-center", "align-self: center;");
        self.utilities.insert("self-stretch", "align-self: stretch;");
        self.utilities.insert("self-baseline", "align-self: baseline;");

        // Place Content
        self.utilities.insert("place-content-center", "place-content: center;");
        self.utilities.insert("place-content-start", "place-content: start;");
        self.utilities.insert("place-content-end", "place-content: end;");
        self.utilities.insert("place-content-between", "place-content: space-between;");
        self.utilities.insert("place-content-around", "place-content: space-around;");
        self.utilities.insert("place-content-evenly", "place-content: space-evenly;");
        self.utilities.insert("place-content-baseline", "place-content: baseline;");
        self.utilities.insert("place-content-stretch", "place-content: stretch;");

        // Place Items
        self.utilities.insert("place-items-start", "place-items: start;");
        self.utilities.insert("place-items-end", "place-items: end;");
        self.utilities.insert("place-items-center", "place-items: center;");
        self.utilities.insert("place-items-baseline", "place-items: baseline;");
        self.utilities.insert("place-items-stretch", "place-items: stretch;");

        // Place Self
        self.utilities.insert("place-self-auto", "place-self: auto;");
        self.utilities.insert("place-self-start", "place-self: start;");
        self.utilities.insert("place-self-end", "place-self: end;");
        self.utilities.insert("place-self-center", "place-self: center;");
        self.utilities.insert("place-self-stretch", "place-self: stretch;");

        // === SPACING ===
        // Inset (top, right, bottom, left)
        self.utilities.insert("inset-0", "inset: 0px;");
        self.utilities.insert("inset-auto", "inset: auto;");
        self.utilities.insert("inset-x-0", "left: 0px; right: 0px;");
        self.utilities.insert("inset-y-0", "top: 0px; bottom: 0px;");
        self.utilities.insert("inset-x-auto", "left: auto; right: auto;");
        self.utilities.insert("inset-y-auto", "top: auto; bottom: auto;");
        self.utilities.insert("start-0", "inset-inline-start: 0px;");
        self.utilities.insert("start-auto", "inset-inline-start: auto;");
        self.utilities.insert("end-0", "inset-inline-end: 0px;");
        self.utilities.insert("end-auto", "inset-inline-end: auto;");
        self.utilities.insert("top-0", "top: 0px;");
        self.utilities.insert("top-auto", "top: auto;");
        self.utilities.insert("right-0", "right: 0px;");
        self.utilities.insert("right-auto", "right: auto;");
        self.utilities.insert("bottom-0", "bottom: 0px;");
        self.utilities.insert("bottom-auto", "bottom: auto;");
        self.utilities.insert("left-0", "left: 0px;");
        self.utilities.insert("left-auto", "left: auto;");

        // === SIZING ===
        self.utilities.insert("w-auto", "width: auto;");
        self.utilities.insert("w-full", "width: 100%;");
        self.utilities.insert("w-screen", "width: 100vw;");
        self.utilities.insert("w-svw", "width: 100svw;");
        self.utilities.insert("w-lvw", "width: 100lvw;");
        self.utilities.insert("w-dvw", "width: 100dvw;");
        self.utilities.insert("w-min", "width: min-content;");
        self.utilities.insert("w-max", "width: max-content;");
        self.utilities.insert("w-fit", "width: fit-content;");

        self.utilities.insert("min-w-0", "min-width: 0px;");
        self.utilities.insert("min-w-full", "min-width: 100%;");
        self.utilities.insert("min-w-min", "min-width: min-content;");
        self.utilities.insert("min-w-max", "min-width: max-content;");
        self.utilities.insert("min-w-fit", "min-width: fit-content;");

        self.utilities.insert("max-w-none", "max-width: none;");
        self.utilities.insert("max-w-full", "max-width: 100%;");
        self.utilities.insert("max-w-min", "max-width: min-content;");
        self.utilities.insert("max-w-max", "max-width: max-content;");
        self.utilities.insert("max-w-fit", "max-width: fit-content;");
        self.utilities.insert("max-w-prose", "max-width: 65ch;");
        self.utilities.insert("max-w-screen-sm", "max-width: 640px;");
        self.utilities.insert("max-w-screen-md", "max-width: 768px;");
        self.utilities.insert("max-w-screen-lg", "max-width: 1024px;");
        self.utilities.insert("max-w-screen-xl", "max-width: 1280px;");
        self.utilities.insert("max-w-screen-2xl", "max-width: 1536px;");
        self.utilities.insert("max-w-xs", "max-width: 20rem;");
        self.utilities.insert("max-w-sm", "max-width: 24rem;");
        self.utilities.insert("max-w-md", "max-width: 28rem;");
        self.utilities.insert("max-w-lg", "max-width: 32rem;");
        self.utilities.insert("max-w-xl", "max-width: 36rem;");
        self.utilities.insert("max-w-2xl", "max-width: 42rem;");
        self.utilities.insert("max-w-3xl", "max-width: 48rem;");
        self.utilities.insert("max-w-4xl", "max-width: 56rem;");
        self.utilities.insert("max-w-5xl", "max-width: 64rem;");
        self.utilities.insert("max-w-6xl", "max-width: 72rem;");
        self.utilities.insert("max-w-7xl", "max-width: 80rem;");

        self.utilities.insert("h-auto", "height: auto;");
        self.utilities.insert("h-full", "height: 100%;");
        self.utilities.insert("h-screen", "height: 100vh;");
        self.utilities.insert("h-svh", "height: 100svh;");
        self.utilities.insert("h-lvh", "height: 100lvh;");
        self.utilities.insert("h-dvh", "height: 100dvh;");
        self.utilities.insert("h-min", "height: min-content;");
        self.utilities.insert("h-max", "height: max-content;");
        self.utilities.insert("h-fit", "height: fit-content;");

        self.utilities.insert("min-h-0", "min-height: 0px;");
        self.utilities.insert("min-h-full", "min-height: 100%;");
        self.utilities.insert("min-h-screen", "min-height: 100vh;");
        self.utilities.insert("min-h-svh", "min-height: 100svh;");
        self.utilities.insert("min-h-lvh", "min-height: 100lvh;");
        self.utilities.insert("min-h-dvh", "min-height: 100dvh;");
        self.utilities.insert("min-h-min", "min-height: min-content;");
        self.utilities.insert("min-h-max", "min-height: max-content;");
        self.utilities.insert("min-h-fit", "min-height: fit-content;");

        self.utilities.insert("max-h-none", "max-height: none;");
        self.utilities.insert("max-h-full", "max-height: 100%;");
        self.utilities.insert("max-h-screen", "max-height: 100vh;");
        self.utilities.insert("max-h-svh", "max-height: 100svh;");
        self.utilities.insert("max-h-lvh", "max-height: 100lvh;");
        self.utilities.insert("max-h-dvh", "max-height: 100dvh;");
        self.utilities.insert("max-h-min", "max-height: min-content;");
        self.utilities.insert("max-h-max", "max-height: max-content;");
        self.utilities.insert("max-h-fit", "max-height: fit-content;");

        self.utilities.insert("size-auto", "width: auto; height: auto;");
        self.utilities.insert("size-full", "width: 100%; height: 100%;");
        self.utilities.insert("size-min", "width: min-content; height: min-content;");
        self.utilities.insert("size-max", "width: max-content; height: max-content;");
        self.utilities.insert("size-fit", "width: fit-content; height: fit-content;");

        // === TYPOGRAPHY ===
        // Font Family
        self.utilities.insert("font-sans", "font-family: ui-sans-serif, system-ui, sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\", \"Segoe UI Symbol\", \"Noto Color Emoji\";");
        self.utilities.insert("font-serif", "font-family: ui-serif, Georgia, Cambria, \"Times New Roman\", Times, serif;");
        self.utilities.insert("font-mono", "font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", \"Courier New\", monospace;");

        // Font Size
        self.utilities.insert("text-xs", "font-size: 0.75rem; line-height: 1rem;");
        self.utilities.insert("text-sm", "font-size: 0.875rem; line-height: 1.25rem;");
        self.utilities.insert("text-base", "font-size: 1rem; line-height: 1.5rem;");
        self.utilities.insert("text-lg", "font-size: 1.125rem; line-height: 1.75rem;");
        self.utilities.insert("text-xl", "font-size: 1.25rem; line-height: 1.75rem;");
        self.utilities.insert("text-2xl", "font-size: 1.5rem; line-height: 2rem;");
        self.utilities.insert("text-3xl", "font-size: 1.875rem; line-height: 2.25rem;");
        self.utilities.insert("text-4xl", "font-size: 2.25rem; line-height: 2.5rem;");
        self.utilities.insert("text-5xl", "font-size: 3rem; line-height: 1;");
        self.utilities.insert("text-6xl", "font-size: 3.75rem; line-height: 1;");
        self.utilities.insert("text-7xl", "font-size: 4.5rem; line-height: 1;");
        self.utilities.insert("text-8xl", "font-size: 6rem; line-height: 1;");
        self.utilities.insert("text-9xl", "font-size: 8rem; line-height: 1;");

        // Font Smoothing
        self.utilities.insert("antialiased", "-webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;");
        self.utilities.insert("subpixel-antialiased", "-webkit-font-smoothing: auto; -moz-osx-font-smoothing: auto;");

        // Font Style
        self.utilities.insert("italic", "font-style: italic;");
        self.utilities.insert("not-italic", "font-style: normal;");

        // Font Weight
        self.utilities.insert("font-thin", "font-weight: 100;");
        self.utilities.insert("font-extralight", "font-weight: 200;");
        self.utilities.insert("font-light", "font-weight: 300;");
        self.utilities.insert("font-normal", "font-weight: 400;");
        self.utilities.insert("font-medium", "font-weight: 500;");
        self.utilities.insert("font-semibold", "font-weight: 600;");
        self.utilities.insert("font-bold", "font-weight: 700;");
        self.utilities.insert("font-extrabold", "font-weight: 800;");
        self.utilities.insert("font-black", "font-weight: 900;");

        // Font Variant Numeric
        self.utilities.insert("normal-nums", "font-variant-numeric: normal;");
        self.utilities.insert("ordinal", "font-variant-numeric: ordinal;");
        self.utilities.insert("slashed-zero", "font-variant-numeric: slashed-zero;");
        self.utilities.insert("lining-nums", "font-variant-numeric: lining-nums;");
        self.utilities.insert("oldstyle-nums", "font-variant-numeric: oldstyle-nums;");
        self.utilities.insert("proportional-nums", "font-variant-numeric: proportional-nums;");
        self.utilities.insert("tabular-nums", "font-variant-numeric: tabular-nums;");
        self.utilities.insert("diagonal-fractions", "font-variant-numeric: diagonal-fractions;");
        self.utilities.insert("stacked-fractions", "font-variant-numeric: stacked-fractions;");

        // Letter Spacing
        self.utilities.insert("tracking-tighter", "letter-spacing: -0.05em;");
        self.utilities.insert("tracking-tight", "letter-spacing: -0.025em;");
        self.utilities.insert("tracking-normal", "letter-spacing: 0em;");
        self.utilities.insert("tracking-wide", "letter-spacing: 0.025em;");
        self.utilities.insert("tracking-wider", "letter-spacing: 0.05em;");
        self.utilities.insert("tracking-widest", "letter-spacing: 0.1em;");

        // Line Clamp
        self.utilities.insert("line-clamp-none", "overflow: visible; display: block; -webkit-box-orient: horizontal; -webkit-line-clamp: none;");
        for i in 1..=6 {
            let class = Box::leak(format!("line-clamp-{}", i).into_boxed_str());
            let css = Box::leak(format!("overflow: hidden; display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: {};", i).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // Line Height
        self.utilities.insert("leading-none", "line-height: 1;");
        self.utilities.insert("leading-tight", "line-height: 1.25;");
        self.utilities.insert("leading-snug", "line-height: 1.375;");
        self.utilities.insert("leading-normal", "line-height: 1.5;");
        self.utilities.insert("leading-relaxed", "line-height: 1.625;");
        self.utilities.insert("leading-loose", "line-height: 2;");
        for i in 3..=10 {
            let class = Box::leak(format!("leading-{}", i).into_boxed_str());
            let css = Box::leak(format!("line-height: {}rem;", i as f32 * 0.25).into_boxed_str());
            self.utilities.insert(class, css);
        }

        // List Style Image
        self.utilities.insert("list-image-none", "list-style-image: none;");

        // List Style Position
        self.utilities.insert("list-inside", "list-style-position: inside;");
        self.utilities.insert("list-outside", "list-style-position: outside;");

        // List Style Type
        self.utilities.insert("list-none", "list-style-type: none;");
        self.utilities.insert("list-disc", "list-style-type: disc;");
        self.utilities.insert("list-decimal", "list-style-type: decimal;");

        // Text Align
        self.utilities.insert("text-left", "text-align: left;");
        self.utilities.insert("text-center", "text-align: center;");
        self.utilities.insert("text-right", "text-align: right;");
        self.utilities.insert("text-justify", "text-align: justify;");
        self.utilities.insert("text-start", "text-align: start;");
        self.utilities.insert("text-end", "text-align: end;");

        // Text Decoration
        self.utilities.insert("underline", "text-decoration-line: underline;");
        self.utilities.insert("overline", "text-decoration-line: overline;");
        self.utilities.insert("line-through", "text-decoration-line: line-through;");
        self.utilities.insert("no-underline", "text-decoration-line: none;");

        // Text Decoration Style
        self.utilities.insert("decoration-solid", "text-decoration-style: solid;");
        self.utilities.insert("decoration-double", "text-decoration-style: double;");
        self.utilities.insert("decoration-dotted", "text-decoration-style: dotted;");
        self.utilities.insert("decoration-dashed", "text-decoration-style: dashed;");
        self.utilities.insert("decoration-wavy", "text-decoration-style: wavy;");

        // Text Decoration Thickness
        self.utilities.insert("decoration-auto", "text-decoration-thickness: auto;");
        self.utilities.insert("decoration-from-font", "text-decoration-thickness: from-font;");
        self.utilities.insert("decoration-0", "text-decoration-thickness: 0px;");
        self.utilities.insert("decoration-1", "text-decoration-thickness: 1px;");
        self.utilities.insert("decoration-2", "text-decoration-thickness: 2px;");
        self.utilities.insert("decoration-4", "text-decoration-thickness: 4px;");
        self.utilities.insert("decoration-8", "text-decoration-thickness: 8px;");

        // Text Underline Offset
        self.utilities.insert("underline-offset-auto", "text-underline-offset: auto;");
        self.utilities.insert("underline-offset-0", "text-underline-offset: 0px;");
        self.utilities.insert("underline-offset-1", "text-underline-offset: 1px;");
        self.utilities.insert("underline-offset-2", "text-underline-offset: 2px;");
        self.utilities.insert("underline-offset-4", "text-underline-offset: 4px;");
        self.utilities.insert("underline-offset-8", "text-underline-offset: 8px;");

        // Text Transform
        self.utilities.insert("uppercase", "text-transform: uppercase;");
        self.utilities.insert("lowercase", "text-transform: lowercase;");
        self.utilities.insert("capitalize", "text-transform: capitalize;");
        self.utilities.insert("normal-case", "text-transform: none;");

        // Text Overflow
        self.utilities.insert("truncate", "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;");
        self.utilities.insert("text-ellipsis", "text-overflow: ellipsis;");
        self.utilities.insert("text-clip", "text-overflow: clip;");

        // Text Wrap
        self.utilities.insert("text-wrap", "text-wrap: wrap;");
        self.utilities.insert("text-nowrap", "text-wrap: nowrap;");
        self.utilities.insert("text-balance", "text-wrap: balance;");
        self.utilities.insert("text-pretty", "text-wrap: pretty;");

        // Text Indent
        self.utilities.insert("indent-0", "text-indent: 0px;");

        // Vertical Align
        self.utilities.insert("align-baseline", "vertical-align: baseline;");
        self.utilities.insert("align-top", "vertical-align: top;");
        self.utilities.insert("align-middle", "vertical-align: middle;");
        self.utilities.insert("align-bottom", "vertical-align: bottom;");
        self.utilities.insert("align-text-top", "vertical-align: text-top;");
        self.utilities.insert("align-text-bottom", "vertical-align: text-bottom;");
        self.utilities.insert("align-sub", "vertical-align: sub;");
        self.utilities.insert("align-super", "vertical-align: super;");

        // Whitespace
        self.utilities.insert("whitespace-normal", "white-space: normal;");
        self.utilities.insert("whitespace-nowrap", "white-space: nowrap;");
        self.utilities.insert("whitespace-pre", "white-space: pre;");
        self.utilities.insert("whitespace-pre-line", "white-space: pre-line;");
        self.utilities.insert("whitespace-pre-wrap", "white-space: pre-wrap;");
        self.utilities.insert("whitespace-break-spaces", "white-space: break-spaces;");

        // Word Break
        self.utilities.insert("break-normal", "overflow-wrap: normal; word-break: normal;");
        self.utilities.insert("break-words", "overflow-wrap: break-word;");
        self.utilities.insert("break-all", "word-break: break-all;");
        self.utilities.insert("break-keep", "word-break: keep-all;");

        // Hyphens
        self.utilities.insert("hyphens-none", "hyphens: none;");
        self.utilities.insert("hyphens-manual", "hyphens: manual;");
        self.utilities.insert("hyphens-auto", "hyphens: auto;");

        // Content
        self.utilities.insert("content-none", "content: none;");

        // === BACKGROUNDS ===
        // Background Attachment
        self.utilities.insert("bg-fixed", "background-attachment: fixed;");
        self.utilities.insert("bg-local", "background-attachment: local;");
        self.utilities.insert("bg-scroll", "background-attachment: scroll;");

        // Background Clip
        self.utilities.insert("bg-clip-border", "background-clip: border-box;");
        self.utilities.insert("bg-clip-padding", "background-clip: padding-box;");
        self.utilities.insert("bg-clip-content", "background-clip: content-box;");
        self.utilities.insert("bg-clip-text", "background-clip: text;");

        // Background Origin
        self.utilities.insert("bg-origin-border", "background-origin: border-box;");
        self.utilities.insert("bg-origin-padding", "background-origin: padding-box;");
        self.utilities.insert("bg-origin-content", "background-origin: content-box;");

        // Background Position
        self.utilities.insert("bg-bottom", "background-position: bottom;");
        self.utilities.insert("bg-center", "background-position: center;");
        self.utilities.insert("bg-left", "background-position: left;");
        self.utilities.insert("bg-left-bottom", "background-position: left bottom;");
        self.utilities.insert("bg-left-top", "background-position: left top;");
        self.utilities.insert("bg-right", "background-position: right;");
        self.utilities.insert("bg-right-bottom", "background-position: right bottom;");
        self.utilities.insert("bg-right-top", "background-position: right top;");
        self.utilities.insert("bg-top", "background-position: top;");

        // Background Repeat
        self.utilities.insert("bg-repeat", "background-repeat: repeat;");
        self.utilities.insert("bg-no-repeat", "background-repeat: no-repeat;");
        self.utilities.insert("bg-repeat-x", "background-repeat: repeat-x;");
        self.utilities.insert("bg-repeat-y", "background-repeat: repeat-y;");
        self.utilities.insert("bg-repeat-round", "background-repeat: round;");
        self.utilities.insert("bg-repeat-space", "background-repeat: space;");

        // Background Size
        self.utilities.insert("bg-auto", "background-size: auto;");
        self.utilities.insert("bg-cover", "background-size: cover;");
        self.utilities.insert("bg-contain", "background-size: contain;");

        // Background Image
        self.utilities.insert("bg-none", "background-image: none;");
        self.utilities.insert("bg-gradient-to-t", "background-image: linear-gradient(to top, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-tr", "background-image: linear-gradient(to top right, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-r", "background-image: linear-gradient(to right, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-br", "background-image: linear-gradient(to bottom right, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-b", "background-image: linear-gradient(to bottom, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-bl", "background-image: linear-gradient(to bottom left, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-l", "background-image: linear-gradient(to left, var(--tw-gradient-stops));");
        self.utilities.insert("bg-gradient-to-tl", "background-image: linear-gradient(to top left, var(--tw-gradient-stops));");

        // === BORDERS ===
        // Border Radius
        self.utilities.insert("rounded-none", "border-radius: 0px;");
        self.utilities.insert("rounded-sm", "border-radius: 0.125rem;");
        self.utilities.insert("rounded", "border-radius: 0.25rem;");
        self.utilities.insert("rounded-md", "border-radius: 0.375rem;");
        self.utilities.insert("rounded-lg", "border-radius: 0.5rem;");
        self.utilities.insert("rounded-xl", "border-radius: 0.75rem;");
        self.utilities.insert("rounded-2xl", "border-radius: 1rem;");
        self.utilities.insert("rounded-3xl", "border-radius: 1.5rem;");
        self.utilities.insert("rounded-full", "border-radius: 9999px;");

        // Border Radius (specific corners)
        for (suffix, props) in [
            ("t", "border-top-left-radius: {0}; border-top-right-radius: {0};"),
            ("r", "border-top-right-radius: {0}; border-bottom-right-radius: {0};"),
            ("b", "border-bottom-right-radius: {0}; border-bottom-left-radius: {0};"),
            ("l", "border-top-left-radius: {0}; border-bottom-left-radius: {0};"),
            ("tl", "border-top-left-radius: {0};"),
            ("tr", "border-top-right-radius: {0};"),
            ("br", "border-bottom-right-radius: {0};"),
            ("bl", "border-bottom-left-radius: {0};"),
        ] {
            for (size, val) in [("none", "0px"), ("sm", "0.125rem"), ("", "0.25rem"), ("md", "0.375rem"), ("lg", "0.5rem"), ("xl", "0.75rem"), ("2xl", "1rem"), ("3xl", "1.5rem"), ("full", "9999px")] {
                let class_name = if size.is_empty() {
                    format!("rounded-{}", suffix)
                } else {
                    format!("rounded-{}-{}", suffix, size)
                };
                let css_val = props.replace("{0}", val);
                let class_static: &'static str = Box::leak(class_name.into_boxed_str());
                let css_static: &'static str = Box::leak(css_val.into_boxed_str());
                self.utilities.insert(class_static, css_static);
            }
        }

        // Border Width
        self.utilities.insert("border", "border-width: 1px;");
        self.utilities.insert("border-0", "border-width: 0px;");
        self.utilities.insert("border-2", "border-width: 2px;");
        self.utilities.insert("border-4", "border-width: 4px;");
        self.utilities.insert("border-8", "border-width: 8px;");
        self.utilities.insert("border-x", "border-left-width: 1px; border-right-width: 1px;");
        self.utilities.insert("border-x-0", "border-left-width: 0px; border-right-width: 0px;");
        self.utilities.insert("border-x-2", "border-left-width: 2px; border-right-width: 2px;");
        self.utilities.insert("border-x-4", "border-left-width: 4px; border-right-width: 4px;");
        self.utilities.insert("border-x-8", "border-left-width: 8px; border-right-width: 8px;");
        self.utilities.insert("border-y", "border-top-width: 1px; border-bottom-width: 1px;");
        self.utilities.insert("border-y-0", "border-top-width: 0px; border-bottom-width: 0px;");
        self.utilities.insert("border-y-2", "border-top-width: 2px; border-bottom-width: 2px;");
        self.utilities.insert("border-y-4", "border-top-width: 4px; border-bottom-width: 4px;");
        self.utilities.insert("border-y-8", "border-top-width: 8px; border-bottom-width: 8px;");
        self.utilities.insert("border-t", "border-top-width: 1px;");
        self.utilities.insert("border-t-0", "border-top-width: 0px;");
        self.utilities.insert("border-t-2", "border-top-width: 2px;");
        self.utilities.insert("border-t-4", "border-top-width: 4px;");
        self.utilities.insert("border-t-8", "border-top-width: 8px;");
        self.utilities.insert("border-r", "border-right-width: 1px;");
        self.utilities.insert("border-r-0", "border-right-width: 0px;");
        self.utilities.insert("border-r-2", "border-right-width: 2px;");
        self.utilities.insert("border-r-4", "border-right-width: 4px;");
        self.utilities.insert("border-r-8", "border-right-width: 8px;");
        self.utilities.insert("border-b", "border-bottom-width: 1px;");
        self.utilities.insert("border-b-0", "border-bottom-width: 0px;");
        self.utilities.insert("border-b-2", "border-bottom-width: 2px;");
        self.utilities.insert("border-b-4", "border-bottom-width: 4px;");
        self.utilities.insert("border-b-8", "border-bottom-width: 8px;");
        self.utilities.insert("border-l", "border-left-width: 1px;");
        self.utilities.insert("border-l-0", "border-left-width: 0px;");
        self.utilities.insert("border-l-2", "border-left-width: 2px;");
        self.utilities.insert("border-l-4", "border-left-width: 4px;");
        self.utilities.insert("border-l-8", "border-left-width: 8px;");

        // Border Style
        self.utilities.insert("border-solid", "border-style: solid;");
        self.utilities.insert("border-dashed", "border-style: dashed;");
        self.utilities.insert("border-dotted", "border-style: dotted;");
        self.utilities.insert("border-double", "border-style: double;");
        self.utilities.insert("border-hidden", "border-style: hidden;");
        self.utilities.insert("border-none", "border-style: none;");

        // Divide Width
        self.utilities.insert("divide-x", "> :not([hidden]) ~ :not([hidden]) { border-left-width: 1px; }");
        self.utilities.insert("divide-x-0", "> :not([hidden]) ~ :not([hidden]) { border-left-width: 0px; }");
        self.utilities.insert("divide-x-2", "> :not([hidden]) ~ :not([hidden]) { border-left-width: 2px; }");
        self.utilities.insert("divide-x-4", "> :not([hidden]) ~ :not([hidden]) { border-left-width: 4px; }");
        self.utilities.insert("divide-x-8", "> :not([hidden]) ~ :not([hidden]) { border-left-width: 8px; }");
        self.utilities.insert("divide-y", "> :not([hidden]) ~ :not([hidden]) { border-top-width: 1px; }");
        self.utilities.insert("divide-y-0", "> :not([hidden]) ~ :not([hidden]) { border-top-width: 0px; }");
        self.utilities.insert("divide-y-2", "> :not([hidden]) ~ :not([hidden]) { border-top-width: 2px; }");
        self.utilities.insert("divide-y-4", "> :not([hidden]) ~ :not([hidden]) { border-top-width: 4px; }");
        self.utilities.insert("divide-y-8", "> :not([hidden]) ~ :not([hidden]) { border-top-width: 8px; }");
        self.utilities.insert("divide-y-reverse", "--tw-divide-y-reverse: 1;");
        self.utilities.insert("divide-x-reverse", "--tw-divide-x-reverse: 1;");

        // Divide Style
        self.utilities.insert("divide-solid", "> :not([hidden]) ~ :not([hidden]) { border-style: solid; }");
        self.utilities.insert("divide-dashed", "> :not([hidden]) ~ :not([hidden]) { border-style: dashed; }");
        self.utilities.insert("divide-dotted", "> :not([hidden]) ~ :not([hidden]) { border-style: dotted; }");
        self.utilities.insert("divide-double", "> :not([hidden]) ~ :not([hidden]) { border-style: double; }");
        self.utilities.insert("divide-none", "> :not([hidden]) ~ :not([hidden]) { border-style: none; }");

        // Outline Width
        self.utilities.insert("outline-0", "outline-width: 0px;");
        self.utilities.insert("outline-1", "outline-width: 1px;");
        self.utilities.insert("outline-2", "outline-width: 2px;");
        self.utilities.insert("outline-4", "outline-width: 4px;");
        self.utilities.insert("outline-8", "outline-width: 8px;");

        // Outline Style
        self.utilities.insert("outline-none", "outline: 2px solid transparent; outline-offset: 2px;");
        self.utilities.insert("outline", "outline-style: solid;");
        self.utilities.insert("outline-dashed", "outline-style: dashed;");
        self.utilities.insert("outline-dotted", "outline-style: dotted;");
        self.utilities.insert("outline-double", "outline-style: double;");

        // Outline Offset
        self.utilities.insert("outline-offset-0", "outline-offset: 0px;");
        self.utilities.insert("outline-offset-1", "outline-offset: 1px;");
        self.utilities.insert("outline-offset-2", "outline-offset: 2px;");
        self.utilities.insert("outline-offset-4", "outline-offset: 4px;");
        self.utilities.insert("outline-offset-8", "outline-offset: 8px;");

        // Ring Width
        self.utilities.insert("ring-0", "box-shadow: var(--tw-ring-inset) 0 0 0 0px var(--tw-ring-color);");
        self.utilities.insert("ring-1", "box-shadow: var(--tw-ring-inset) 0 0 0 1px var(--tw-ring-color);");
        self.utilities.insert("ring-2", "box-shadow: var(--tw-ring-inset) 0 0 0 2px var(--tw-ring-color);");
        self.utilities.insert("ring", "box-shadow: var(--tw-ring-inset) 0 0 0 3px var(--tw-ring-color);");
        self.utilities.insert("ring-4", "box-shadow: var(--tw-ring-inset) 0 0 0 4px var(--tw-ring-color);");
        self.utilities.insert("ring-8", "box-shadow: var(--tw-ring-inset) 0 0 0 8px var(--tw-ring-color);");
        self.utilities.insert("ring-inset", "--tw-ring-inset: inset;");

        // Ring Offset Width
        self.utilities.insert("ring-offset-0", "--tw-ring-offset-width: 0px;");
        self.utilities.insert("ring-offset-1", "--tw-ring-offset-width: 1px;");
        self.utilities.insert("ring-offset-2", "--tw-ring-offset-width: 2px;");
        self.utilities.insert("ring-offset-4", "--tw-ring-offset-width: 4px;");
        self.utilities.insert("ring-offset-8", "--tw-ring-offset-width: 8px;");

        // === EFFECTS ===
        // Box Shadow
        self.utilities.insert("shadow-sm", "box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);");
        self.utilities.insert("shadow", "box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);");
        self.utilities.insert("shadow-md", "box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);");
        self.utilities.insert("shadow-lg", "box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);");
        self.utilities.insert("shadow-xl", "box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);");
        self.utilities.insert("shadow-2xl", "box-shadow: 0 25px 50px -12px rgb(0 0 0 / 0.25);");
        self.utilities.insert("shadow-inner", "box-shadow: inset 0 2px 4px 0 rgb(0 0 0 / 0.05);");
        self.utilities.insert("shadow-none", "box-shadow: 0 0 #0000;");

        // Opacity
        self.utilities.insert("opacity-0", "opacity: 0;");
        self.utilities.insert("opacity-5", "opacity: 0.05;");
        self.utilities.insert("opacity-10", "opacity: 0.1;");
        self.utilities.insert("opacity-15", "opacity: 0.15;");
        self.utilities.insert("opacity-20", "opacity: 0.2;");
        self.utilities.insert("opacity-25", "opacity: 0.25;");
        self.utilities.insert("opacity-30", "opacity: 0.3;");
        self.utilities.insert("opacity-35", "opacity: 0.35;");
        self.utilities.insert("opacity-40", "opacity: 0.4;");
        self.utilities.insert("opacity-45", "opacity: 0.45;");
        self.utilities.insert("opacity-50", "opacity: 0.5;");
        self.utilities.insert("opacity-55", "opacity: 0.55;");
        self.utilities.insert("opacity-60", "opacity: 0.6;");
        self.utilities.insert("opacity-65", "opacity: 0.65;");
        self.utilities.insert("opacity-70", "opacity: 0.7;");
        self.utilities.insert("opacity-75", "opacity: 0.75;");
        self.utilities.insert("opacity-80", "opacity: 0.8;");
        self.utilities.insert("opacity-85", "opacity: 0.85;");
        self.utilities.insert("opacity-90", "opacity: 0.9;");
        self.utilities.insert("opacity-95", "opacity: 0.95;");
        self.utilities.insert("opacity-100", "opacity: 1;");

        // Mix Blend Mode
        self.utilities.insert("mix-blend-normal", "mix-blend-mode: normal;");
        self.utilities.insert("mix-blend-multiply", "mix-blend-mode: multiply;");
        self.utilities.insert("mix-blend-screen", "mix-blend-mode: screen;");
        self.utilities.insert("mix-blend-overlay", "mix-blend-mode: overlay;");
        self.utilities.insert("mix-blend-darken", "mix-blend-mode: darken;");
        self.utilities.insert("mix-blend-lighten", "mix-blend-mode: lighten;");
        self.utilities.insert("mix-blend-color-dodge", "mix-blend-mode: color-dodge;");
        self.utilities.insert("mix-blend-color-burn", "mix-blend-mode: color-burn;");
        self.utilities.insert("mix-blend-hard-light", "mix-blend-mode: hard-light;");
        self.utilities.insert("mix-blend-soft-light", "mix-blend-mode: soft-light;");
        self.utilities.insert("mix-blend-difference", "mix-blend-mode: difference;");
        self.utilities.insert("mix-blend-exclusion", "mix-blend-mode: exclusion;");
        self.utilities.insert("mix-blend-hue", "mix-blend-mode: hue;");
        self.utilities.insert("mix-blend-saturation", "mix-blend-mode: saturation;");
        self.utilities.insert("mix-blend-color", "mix-blend-mode: color;");
        self.utilities.insert("mix-blend-luminosity", "mix-blend-mode: luminosity;");
        self.utilities.insert("mix-blend-plus-darker", "mix-blend-mode: plus-darker;");
        self.utilities.insert("mix-blend-plus-lighter", "mix-blend-mode: plus-lighter;");

        // Background Blend Mode
        self.utilities.insert("bg-blend-normal", "background-blend-mode: normal;");
        self.utilities.insert("bg-blend-multiply", "background-blend-mode: multiply;");
        self.utilities.insert("bg-blend-screen", "background-blend-mode: screen;");
        self.utilities.insert("bg-blend-overlay", "background-blend-mode: overlay;");
        self.utilities.insert("bg-blend-darken", "background-blend-mode: darken;");
        self.utilities.insert("bg-blend-lighten", "background-blend-mode: lighten;");
        self.utilities.insert("bg-blend-color-dodge", "background-blend-mode: color-dodge;");
        self.utilities.insert("bg-blend-color-burn", "background-blend-mode: color-burn;");
        self.utilities.insert("bg-blend-hard-light", "background-blend-mode: hard-light;");
        self.utilities.insert("bg-blend-soft-light", "background-blend-mode: soft-light;");
        self.utilities.insert("bg-blend-difference", "background-blend-mode: difference;");
        self.utilities.insert("bg-blend-exclusion", "background-blend-mode: exclusion;");
        self.utilities.insert("bg-blend-hue", "background-blend-mode: hue;");
        self.utilities.insert("bg-blend-saturation", "background-blend-mode: saturation;");
        self.utilities.insert("bg-blend-color", "background-blend-mode: color;");
        self.utilities.insert("bg-blend-luminosity", "background-blend-mode: luminosity;");

        // === FILTERS ===
        // Blur
        self.utilities.insert("blur-none", "filter: blur(0);");
        self.utilities.insert("blur-sm", "filter: blur(4px);");
        self.utilities.insert("blur", "filter: blur(8px);");
        self.utilities.insert("blur-md", "filter: blur(12px);");
        self.utilities.insert("blur-lg", "filter: blur(16px);");
        self.utilities.insert("blur-xl", "filter: blur(24px);");
        self.utilities.insert("blur-2xl", "filter: blur(40px);");
        self.utilities.insert("blur-3xl", "filter: blur(64px);");

        // Brightness
        self.utilities.insert("brightness-0", "filter: brightness(0);");
        self.utilities.insert("brightness-50", "filter: brightness(.5);");
        self.utilities.insert("brightness-75", "filter: brightness(.75);");
        self.utilities.insert("brightness-90", "filter: brightness(.9);");
        self.utilities.insert("brightness-95", "filter: brightness(.95);");
        self.utilities.insert("brightness-100", "filter: brightness(1);");
        self.utilities.insert("brightness-105", "filter: brightness(1.05);");
        self.utilities.insert("brightness-110", "filter: brightness(1.1);");
        self.utilities.insert("brightness-125", "filter: brightness(1.25);");
        self.utilities.insert("brightness-150", "filter: brightness(1.5);");
        self.utilities.insert("brightness-200", "filter: brightness(2);");

        // Contrast
        self.utilities.insert("contrast-0", "filter: contrast(0);");
        self.utilities.insert("contrast-50", "filter: contrast(.5);");
        self.utilities.insert("contrast-75", "filter: contrast(.75);");
        self.utilities.insert("contrast-100", "filter: contrast(1);");
        self.utilities.insert("contrast-125", "filter: contrast(1.25);");
        self.utilities.insert("contrast-150", "filter: contrast(1.5);");
        self.utilities.insert("contrast-200", "filter: contrast(2);");

        // Grayscale
        self.utilities.insert("grayscale-0", "filter: grayscale(0);");
        self.utilities.insert("grayscale", "filter: grayscale(100%);");

        // Hue Rotate
        self.utilities.insert("hue-rotate-0", "filter: hue-rotate(0deg);");
        self.utilities.insert("hue-rotate-15", "filter: hue-rotate(15deg);");
        self.utilities.insert("hue-rotate-30", "filter: hue-rotate(30deg);");
        self.utilities.insert("hue-rotate-60", "filter: hue-rotate(60deg);");
        self.utilities.insert("hue-rotate-90", "filter: hue-rotate(90deg);");
        self.utilities.insert("hue-rotate-180", "filter: hue-rotate(180deg);");

        // Invert
        self.utilities.insert("invert-0", "filter: invert(0);");
        self.utilities.insert("invert", "filter: invert(100%);");

        // Saturate
        self.utilities.insert("saturate-0", "filter: saturate(0);");
        self.utilities.insert("saturate-50", "filter: saturate(.5);");
        self.utilities.insert("saturate-100", "filter: saturate(1);");
        self.utilities.insert("saturate-150", "filter: saturate(1.5);");
        self.utilities.insert("saturate-200", "filter: saturate(2);");

        // Sepia
        self.utilities.insert("sepia-0", "filter: sepia(0);");
        self.utilities.insert("sepia", "filter: sepia(100%);");

        // Backdrop Blur
        self.utilities.insert("backdrop-blur-none", "backdrop-filter: blur(0);");
        self.utilities.insert("backdrop-blur-sm", "backdrop-filter: blur(4px);");
        self.utilities.insert("backdrop-blur", "backdrop-filter: blur(8px);");
        self.utilities.insert("backdrop-blur-md", "backdrop-filter: blur(12px);");
        self.utilities.insert("backdrop-blur-lg", "backdrop-filter: blur(16px);");
        self.utilities.insert("backdrop-blur-xl", "backdrop-filter: blur(24px);");
        self.utilities.insert("backdrop-blur-2xl", "backdrop-filter: blur(40px);");
        self.utilities.insert("backdrop-blur-3xl", "backdrop-filter: blur(64px);");

        // === TABLES ===
        self.utilities.insert("border-collapse", "border-collapse: collapse;");
        self.utilities.insert("border-separate", "border-collapse: separate;");
        self.utilities.insert("table-auto", "table-layout: auto;");
        self.utilities.insert("table-fixed", "table-layout: fixed;");
        self.utilities.insert("caption-top", "caption-side: top;");
        self.utilities.insert("caption-bottom", "caption-side: bottom;");

        // === TRANSITIONS & ANIMATION ===
        self.utilities.insert("transition-none", "transition-property: none;");
        self.utilities.insert("transition-all", "transition-property: all; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
        self.utilities.insert("transition", "transition-property: color, background-color, border-color, text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter, backdrop-filter; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
        self.utilities.insert("transition-colors", "transition-property: color, background-color, border-color, text-decoration-color, fill, stroke; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
        self.utilities.insert("transition-opacity", "transition-property: opacity; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
        self.utilities.insert("transition-shadow", "transition-property: box-shadow; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
        self.utilities.insert("transition-transform", "transition-property: transform; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");

        // Duration
        self.utilities.insert("duration-0", "transition-duration: 0s;");
        self.utilities.insert("duration-75", "transition-duration: 75ms;");
        self.utilities.insert("duration-100", "transition-duration: 100ms;");
        self.utilities.insert("duration-150", "transition-duration: 150ms;");
        self.utilities.insert("duration-200", "transition-duration: 200ms;");
        self.utilities.insert("duration-300", "transition-duration: 300ms;");
        self.utilities.insert("duration-500", "transition-duration: 500ms;");
        self.utilities.insert("duration-700", "transition-duration: 700ms;");
        self.utilities.insert("duration-1000", "transition-duration: 1000ms;");

        // Easing
        self.utilities.insert("ease-linear", "transition-timing-function: linear;");
        self.utilities.insert("ease-in", "transition-timing-function: cubic-bezier(0.4, 0, 1, 1);");
        self.utilities.insert("ease-out", "transition-timing-function: cubic-bezier(0, 0, 0.2, 1);");
        self.utilities.insert("ease-in-out", "transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);");

        // Delay
        self.utilities.insert("delay-0", "transition-delay: 0s;");
        self.utilities.insert("delay-75", "transition-delay: 75ms;");
        self.utilities.insert("delay-100", "transition-delay: 100ms;");
        self.utilities.insert("delay-150", "transition-delay: 150ms;");
        self.utilities.insert("delay-200", "transition-delay: 200ms;");
        self.utilities.insert("delay-300", "transition-delay: 300ms;");
        self.utilities.insert("delay-500", "transition-delay: 500ms;");
        self.utilities.insert("delay-700", "transition-delay: 700ms;");
        self.utilities.insert("delay-1000", "transition-delay: 1000ms;");

        // Animation
        self.utilities.insert("animate-none", "animation: none;");
        self.utilities.insert("animate-spin", "animation: spin 1s linear infinite;");
        self.utilities.insert("animate-ping", "animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;");
        self.utilities.insert("animate-pulse", "animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;");
        self.utilities.insert("animate-bounce", "animation: bounce 1s infinite;");

        // === TRANSFORMS ===
        self.utilities.insert("scale-0", "transform: scale(0);");
        self.utilities.insert("scale-50", "transform: scale(.5);");
        self.utilities.insert("scale-75", "transform: scale(.75);");
        self.utilities.insert("scale-90", "transform: scale(.9);");
        self.utilities.insert("scale-95", "transform: scale(.95);");
        self.utilities.insert("scale-100", "transform: scale(1);");
        self.utilities.insert("scale-105", "transform: scale(1.05);");
        self.utilities.insert("scale-110", "transform: scale(1.1);");
        self.utilities.insert("scale-125", "transform: scale(1.25);");
        self.utilities.insert("scale-150", "transform: scale(1.5);");

        self.utilities.insert("rotate-0", "transform: rotate(0deg);");
        self.utilities.insert("rotate-1", "transform: rotate(1deg);");
        self.utilities.insert("rotate-2", "transform: rotate(2deg);");
        self.utilities.insert("rotate-3", "transform: rotate(3deg);");
        self.utilities.insert("rotate-6", "transform: rotate(6deg);");
        self.utilities.insert("rotate-12", "transform: rotate(12deg);");
        self.utilities.insert("rotate-45", "transform: rotate(45deg);");
        self.utilities.insert("rotate-90", "transform: rotate(90deg);");
        self.utilities.insert("rotate-180", "transform: rotate(180deg);");

        self.utilities.insert("translate-x-0", "transform: translateX(0px);");
        self.utilities.insert("translate-y-0", "transform: translateY(0px);");
        self.utilities.insert("translate-x-full", "transform: translateX(100%);");
        self.utilities.insert("translate-y-full", "transform: translateY(100%);");
        self.utilities.insert("-translate-x-full", "transform: translateX(-100%);");
        self.utilities.insert("-translate-y-full", "transform: translateY(-100%);");
        self.utilities.insert("translate-x-1/2", "transform: translateX(50%);");
        self.utilities.insert("translate-y-1/2", "transform: translateY(50%);");
        self.utilities.insert("-translate-x-1/2", "transform: translateX(-50%);");
        self.utilities.insert("-translate-y-1/2", "transform: translateY(-50%);");

        self.utilities.insert("skew-x-0", "transform: skewX(0deg);");
        self.utilities.insert("skew-x-1", "transform: skewX(1deg);");
        self.utilities.insert("skew-x-2", "transform: skewX(2deg);");
        self.utilities.insert("skew-x-3", "transform: skewX(3deg);");
        self.utilities.insert("skew-x-6", "transform: skewX(6deg);");
        self.utilities.insert("skew-x-12", "transform: skewX(12deg);");
        self.utilities.insert("skew-y-0", "transform: skewY(0deg);");
        self.utilities.insert("skew-y-1", "transform: skewY(1deg);");
        self.utilities.insert("skew-y-2", "transform: skewY(2deg);");
        self.utilities.insert("skew-y-3", "transform: skewY(3deg);");
        self.utilities.insert("skew-y-6", "transform: skewY(6deg);");
        self.utilities.insert("skew-y-12", "transform: skewY(12deg);");

        self.utilities.insert("origin-center", "transform-origin: center;");
        self.utilities.insert("origin-top", "transform-origin: top;");
        self.utilities.insert("origin-top-right", "transform-origin: top right;");
        self.utilities.insert("origin-right", "transform-origin: right;");
        self.utilities.insert("origin-bottom-right", "transform-origin: bottom right;");
        self.utilities.insert("origin-bottom", "transform-origin: bottom;");
        self.utilities.insert("origin-bottom-left", "transform-origin: bottom left;");
        self.utilities.insert("origin-left", "transform-origin: left;");
        self.utilities.insert("origin-top-left", "transform-origin: top left;");

        // === INTERACTIVITY ===
        self.utilities.insert("accent-auto", "accent-color: auto;");
        self.utilities.insert("appearance-none", "appearance: none;");
        self.utilities.insert("appearance-auto", "appearance: auto;");

        // Cursor
        self.utilities.insert("cursor-auto", "cursor: auto;");
        self.utilities.insert("cursor-default", "cursor: default;");
        self.utilities.insert("cursor-pointer", "cursor: pointer;");
        self.utilities.insert("cursor-wait", "cursor: wait;");
        self.utilities.insert("cursor-text", "cursor: text;");
        self.utilities.insert("cursor-move", "cursor: move;");
        self.utilities.insert("cursor-help", "cursor: help;");
        self.utilities.insert("cursor-not-allowed", "cursor: not-allowed;");
        self.utilities.insert("cursor-none", "cursor: none;");
        self.utilities.insert("cursor-context-menu", "cursor: context-menu;");
        self.utilities.insert("cursor-progress", "cursor: progress;");
        self.utilities.insert("cursor-cell", "cursor: cell;");
        self.utilities.insert("cursor-crosshair", "cursor: crosshair;");
        self.utilities.insert("cursor-vertical-text", "cursor: vertical-text;");
        self.utilities.insert("cursor-alias", "cursor: alias;");
        self.utilities.insert("cursor-copy", "cursor: copy;");
        self.utilities.insert("cursor-no-drop", "cursor: no-drop;");
        self.utilities.insert("cursor-grab", "cursor: grab;");
        self.utilities.insert("cursor-grabbing", "cursor: grabbing;");
        self.utilities.insert("cursor-all-scroll", "cursor: all-scroll;");
        self.utilities.insert("cursor-col-resize", "cursor: col-resize;");
        self.utilities.insert("cursor-row-resize", "cursor: row-resize;");
        self.utilities.insert("cursor-n-resize", "cursor: n-resize;");
        self.utilities.insert("cursor-e-resize", "cursor: e-resize;");
        self.utilities.insert("cursor-s-resize", "cursor: s-resize;");
        self.utilities.insert("cursor-w-resize", "cursor: w-resize;");
        self.utilities.insert("cursor-ne-resize", "cursor: ne-resize;");
        self.utilities.insert("cursor-nw-resize", "cursor: nw-resize;");
        self.utilities.insert("cursor-se-resize", "cursor: se-resize;");
        self.utilities.insert("cursor-sw-resize", "cursor: sw-resize;");
        self.utilities.insert("cursor-ew-resize", "cursor: ew-resize;");
        self.utilities.insert("cursor-ns-resize", "cursor: ns-resize;");
        self.utilities.insert("cursor-nesw-resize", "cursor: nesw-resize;");
        self.utilities.insert("cursor-nwse-resize", "cursor: nwse-resize;");
        self.utilities.insert("cursor-zoom-in", "cursor: zoom-in;");
        self.utilities.insert("cursor-zoom-out", "cursor: zoom-out;");

        // Caret Color
        self.utilities.insert("caret-transparent", "caret-color: transparent;");

        // Pointer Events
        self.utilities.insert("pointer-events-none", "pointer-events: none;");
        self.utilities.insert("pointer-events-auto", "pointer-events: auto;");

        // Resize
        self.utilities.insert("resize-none", "resize: none;");
        self.utilities.insert("resize-y", "resize: vertical;");
        self.utilities.insert("resize-x", "resize: horizontal;");
        self.utilities.insert("resize", "resize: both;");

        // Scroll Behavior
        self.utilities.insert("scroll-auto", "scroll-behavior: auto;");
        self.utilities.insert("scroll-smooth", "scroll-behavior: smooth;");

        // Scroll Snap Align
        self.utilities.insert("snap-start", "scroll-snap-align: start;");
        self.utilities.insert("snap-end", "scroll-snap-align: end;");
        self.utilities.insert("snap-center", "scroll-snap-align: center;");
        self.utilities.insert("snap-align-none", "scroll-snap-align: none;");

        // Scroll Snap Stop
        self.utilities.insert("snap-normal", "scroll-snap-stop: normal;");
        self.utilities.insert("snap-always", "scroll-snap-stop: always;");

        // Scroll Snap Type
        self.utilities.insert("snap-none", "scroll-snap-type: none;");
        self.utilities.insert("snap-x", "scroll-snap-type: x var(--tw-scroll-snap-strictness);");
        self.utilities.insert("snap-y", "scroll-snap-type: y var(--tw-scroll-snap-strictness);");
        self.utilities.insert("snap-both", "scroll-snap-type: both var(--tw-scroll-snap-strictness);");
        self.utilities.insert("snap-mandatory", "--tw-scroll-snap-strictness: mandatory;");
        self.utilities.insert("snap-proximity", "--tw-scroll-snap-strictness: proximity;");

        // Touch Action
        self.utilities.insert("touch-auto", "touch-action: auto;");
        self.utilities.insert("touch-none", "touch-action: none;");
        self.utilities.insert("touch-pan-x", "touch-action: pan-x;");
        self.utilities.insert("touch-pan-left", "touch-action: pan-left;");
        self.utilities.insert("touch-pan-right", "touch-action: pan-right;");
        self.utilities.insert("touch-pan-y", "touch-action: pan-y;");
        self.utilities.insert("touch-pan-up", "touch-action: pan-up;");
        self.utilities.insert("touch-pan-down", "touch-action: pan-down;");
        self.utilities.insert("touch-pinch-zoom", "touch-action: pinch-zoom;");
        self.utilities.insert("touch-manipulation", "touch-action: manipulation;");

        // User Select
        self.utilities.insert("select-none", "user-select: none;");
        self.utilities.insert("select-text", "user-select: text;");
        self.utilities.insert("select-all", "user-select: all;");
        self.utilities.insert("select-auto", "user-select: auto;");

        // Will Change
        self.utilities.insert("will-change-auto", "will-change: auto;");
        self.utilities.insert("will-change-scroll", "will-change: scroll-position;");
        self.utilities.insert("will-change-contents", "will-change: contents;");
        self.utilities.insert("will-change-transform", "will-change: transform;");

        // === SVG ===
        self.utilities.insert("fill-none", "fill: none;");
        self.utilities.insert("fill-current", "fill: currentColor;");
        self.utilities.insert("stroke-none", "stroke: none;");
        self.utilities.insert("stroke-current", "stroke: currentColor;");
        self.utilities.insert("stroke-0", "stroke-width: 0;");
        self.utilities.insert("stroke-1", "stroke-width: 1;");
        self.utilities.insert("stroke-2", "stroke-width: 2;");

        // === ACCESSIBILITY ===
        self.utilities.insert("sr-only", "position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;");
        self.utilities.insert("not-sr-only", "position: static; width: auto; height: auto; padding: 0; margin: 0; overflow: visible; clip: auto; white-space: normal;");
        self.utilities.insert("forced-color-adjust-auto", "forced-color-adjust: auto;");
        self.utilities.insert("forced-color-adjust-none", "forced-color-adjust: none;");
    }

    /// Generate CSS for classes
    pub fn generate(&self, classes: &[String]) -> TailwindGenResult {
        let mut result = TailwindGenResult::default();
        let mut css_rules: HashMap<String, String> = HashMap::new();
        let mut media_rules: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut seen: HashSet<String> = HashSet::new();

        for class in classes {
            if seen.contains(class) || class.is_empty() {
                continue;
            }
            seen.insert(class.clone());
            result.classes_processed += 1;

            let (breakpoint, base_class) = self.extract_breakpoint(class);
            let (state, final_class) = self.extract_state(base_class);

            if let Some(css) = self.get_css_for_class(final_class) {
                let selector = self.build_selector(class, state);

                if let Some(bp) = breakpoint {
                    media_rules.entry(bp.to_string()).or_default().push((selector, css.to_string()));
                } else {
                    css_rules.insert(selector, css.to_string());
                }
                result.classes_generated += 1;
            } else if self.is_tailwind_class(class) {
                result.unknown_classes.push(class.clone());
            }
        }

        result.css = self.build_css_output(css_rules, media_rules);
        result
    }

    fn get_css_for_class(&self, class: &str) -> Option<&'static str> {
        // Direct lookup
        if let Some(css) = self.utilities.get(class) {
            return Some(*css);
        }

        // Try spacing patterns
        if let Some(css) = self.try_spacing_class(class) {
            return Some(css);
        }

        // Try color patterns
        if let Some(css) = self.try_color_class(class) {
            return Some(css);
        }

        // Try arbitrary value patterns [value]
        if class.contains('[') && class.contains(']') {
            return self.try_arbitrary_value(class);
        }

        None
    }

    fn try_spacing_class(&self, class: &str) -> Option<&'static str> {
        let patterns = [
            ("p-", "padding: {v};"),
            ("px-", "padding-left: {v}; padding-right: {v};"),
            ("py-", "padding-top: {v}; padding-bottom: {v};"),
            ("pt-", "padding-top: {v};"),
            ("pr-", "padding-right: {v};"),
            ("pb-", "padding-bottom: {v};"),
            ("pl-", "padding-left: {v};"),
            ("ps-", "padding-inline-start: {v};"),
            ("pe-", "padding-inline-end: {v};"),
            ("m-", "margin: {v};"),
            ("mx-", "margin-left: {v}; margin-right: {v};"),
            ("my-", "margin-top: {v}; margin-bottom: {v};"),
            ("mt-", "margin-top: {v};"),
            ("mr-", "margin-right: {v};"),
            ("mb-", "margin-bottom: {v};"),
            ("ml-", "margin-left: {v};"),
            ("ms-", "margin-inline-start: {v};"),
            ("me-", "margin-inline-end: {v};"),
            ("gap-", "gap: {v};"),
            ("gap-x-", "column-gap: {v};"),
            ("gap-y-", "row-gap: {v};"),
            ("w-", "width: {v};"),
            ("h-", "height: {v};"),
            ("min-w-", "min-width: {v};"),
            ("min-h-", "min-height: {v};"),
            ("max-w-", "max-width: {v};"),
            ("max-h-", "max-height: {v};"),
            ("size-", "width: {v}; height: {v};"),
            ("top-", "top: {v};"),
            ("right-", "right: {v};"),
            ("bottom-", "bottom: {v};"),
            ("left-", "left: {v};"),
            ("inset-", "inset: {v};"),
            ("inset-x-", "left: {v}; right: {v};"),
            ("inset-y-", "top: {v}; bottom: {v};"),
            ("basis-", "flex-basis: {v};"),
            ("space-x-", "> :not([hidden]) ~ :not([hidden]) { margin-left: {v}; }"),
            ("space-y-", "> :not([hidden]) ~ :not([hidden]) { margin-top: {v}; }"),
            ("indent-", "text-indent: {v};"),
            ("-m-", "margin: -{v};"),
            ("-mx-", "margin-left: -{v}; margin-right: -{v};"),
            ("-my-", "margin-top: -{v}; margin-bottom: -{v};"),
            ("-mt-", "margin-top: -{v};"),
            ("-mr-", "margin-right: -{v};"),
            ("-mb-", "margin-bottom: -{v};"),
            ("-ml-", "margin-left: -{v};"),
            ("-top-", "top: -{v};"),
            ("-right-", "right: -{v};"),
            ("-bottom-", "bottom: -{v};"),
            ("-left-", "left: -{v};"),
            ("-inset-", "inset: -{v};"),
            ("-translate-x-", "transform: translateX(-{v});"),
            ("-translate-y-", "transform: translateY(-{v});"),
            ("translate-x-", "transform: translateX({v});"),
            ("translate-y-", "transform: translateY({v});"),
        ];

        for (prefix, template) in patterns {
            if let Some(value_key) = class.strip_prefix(prefix) {
                // Handle negative values
                let is_neg = prefix.starts_with('-');
                let lookup_key = if is_neg { value_key } else { value_key };

                if let Some(spacing_value) = self.spacing.get(lookup_key) {
                    let css = template.replace("{v}", spacing_value);
                    return Some(Box::leak(css.into_boxed_str()));
                }

                // Handle fractional values like 1/2, 1/3, 2/3, etc.
                if value_key.contains('/') {
                    let parts: Vec<&str> = value_key.split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(num), Ok(denom)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                            if denom > 0.0 {
                                let percent = (num / denom) * 100.0;
                                let css = template.replace("{v}", &format!("{}%", percent));
                                return Some(Box::leak(css.into_boxed_str()));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn try_color_class(&self, class: &str) -> Option<&'static str> {
        let patterns = [
            ("text-", "color: {c};"),
            ("bg-", "background-color: {c};"),
            ("border-", "border-color: {c};"),
            ("outline-", "outline-color: {c};"),
            ("ring-", "--tw-ring-color: {c};"),
            ("ring-offset-", "--tw-ring-offset-color: {c};"),
            ("divide-", "> :not([hidden]) ~ :not([hidden]) { border-color: {c}; }"),
            ("accent-", "accent-color: {c};"),
            ("caret-", "caret-color: {c};"),
            ("fill-", "fill: {c};"),
            ("stroke-", "stroke: {c};"),
            ("decoration-", "text-decoration-color: {c};"),
            ("shadow-", "--tw-shadow-color: {c};"),
            ("from-", "--tw-gradient-from: {c};"),
            ("via-", "--tw-gradient-via: {c};"),
            ("to-", "--tw-gradient-to: {c};"),
        ];

        for (prefix, template) in patterns {
            if let Some(color_key) = class.strip_prefix(prefix) {
                if let Some(color_value) = self.colors.get(color_key) {
                    let css = template.replace("{c}", color_value);
                    return Some(Box::leak(css.into_boxed_str()));
                }
            }
        }

        None
    }

    fn try_arbitrary_value(&self, class: &str) -> Option<&'static str> {
        // Handle arbitrary values like w-[100px], text-[#ff0000], etc.
        if let Some(start) = class.find('[') {
            if let Some(end) = class.find(']') {
                let prefix = &class[..start];
                let value = &class[start + 1..end];

                let patterns = [
                    ("w-", "width: {v};"),
                    ("h-", "height: {v};"),
                    ("p-", "padding: {v};"),
                    ("m-", "margin: {v};"),
                    ("text-", "color: {v};"),
                    ("bg-", "background-color: {v};"),
                    ("top-", "top: {v};"),
                    ("left-", "left: {v};"),
                    ("right-", "right: {v};"),
                    ("bottom-", "bottom: {v};"),
                    ("gap-", "gap: {v};"),
                    ("rounded-", "border-radius: {v};"),
                    ("border-", "border-width: {v};"),
                    ("max-w-", "max-width: {v};"),
                    ("min-w-", "min-width: {v};"),
                    ("max-h-", "max-height: {v};"),
                    ("min-h-", "min-height: {v};"),
                    ("z-", "z-index: {v};"),
                    ("opacity-", "opacity: {v};"),
                    ("leading-", "line-height: {v};"),
                    ("tracking-", "letter-spacing: {v};"),
                ];

                for (p, template) in patterns {
                    if prefix == p {
                        // Handle underscore -> space replacement
                        let clean_value = value.replace('_', " ");
                        let css = template.replace("{v}", &clean_value);
                        return Some(Box::leak(css.into_boxed_str()));
                    }
                }
            }
        }

        None
    }

    fn extract_breakpoint<'a>(&self, class: &'a str) -> (Option<&'a str>, &'a str) {
        for bp in ["2xl:", "xl:", "lg:", "md:", "sm:"] {
            if let Some(rest) = class.strip_prefix(bp) {
                return (Some(&bp[..bp.len() - 1]), rest);
            }
        }
        (None, class)
    }

    fn extract_state<'a>(&self, class: &'a str) -> (Option<&'a str>, &'a str) {
        let states = [
            "hover:", "focus:", "focus-within:", "focus-visible:", "active:", "visited:",
            "target:", "first:", "last:", "only:", "odd:", "even:", "first-of-type:",
            "last-of-type:", "only-of-type:", "empty:", "disabled:", "enabled:", "checked:",
            "indeterminate:", "default:", "required:", "valid:", "invalid:", "in-range:",
            "out-of-range:", "placeholder-shown:", "autofill:", "read-only:", "before:",
            "after:", "first-letter:", "first-line:", "marker:", "selection:", "file:",
            "backdrop:", "placeholder:", "dark:", "motion-safe:", "motion-reduce:",
            "contrast-more:", "contrast-less:", "portrait:", "landscape:", "print:",
            "rtl:", "ltr:", "open:", "group-hover:", "group-focus:", "peer-hover:",
            "peer-focus:", "peer-checked:", "peer-disabled:",
        ];

        for state in states {
            if let Some(rest) = class.strip_prefix(state) {
                return (Some(&state[..state.len() - 1]), rest);
            }
        }
        (None, class)
    }

    fn build_selector(&self, class: &str, state: Option<&str>) -> String {
        let escaped = self.escape_class(class);
        if let Some(s) = state {
            format!(".{}:{}", escaped, s)
        } else {
            format!(".{}", escaped)
        }
    }

    fn escape_class(&self, class: &str) -> String {
        class
            .replace(':', "\\:")
            .replace('/', "\\/")
            .replace('.', "\\.")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('#', "\\#")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace(',', "\\,")
            .replace('%', "\\%")
    }

    fn build_css_output(
        &self,
        rules: HashMap<String, String>,
        media_rules: HashMap<String, Vec<(String, String)>>,
    ) -> String {
        let mut css = String::new();

        // Animation keyframes
        css.push_str("@keyframes spin { to { transform: rotate(360deg); } }\n");
        css.push_str("@keyframes ping { 75%, 100% { transform: scale(2); opacity: 0; } }\n");
        css.push_str("@keyframes pulse { 50% { opacity: .5; } }\n");
        css.push_str("@keyframes bounce { 0%, 100% { transform: translateY(-25%); animation-timing-function: cubic-bezier(0.8,0,1,1); } 50% { transform: none; animation-timing-function: cubic-bezier(0,0,0.2,1); } }\n\n");

        // Regular rules
        let mut sorted: Vec<_> = rules.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        for (selector, properties) in sorted {
            css.push_str(&format!("{} {{ {} }}\n", selector, properties));
        }

        // Media query rules
        for (bp, rules) in &media_rules {
            if let Some(min_width) = self.breakpoints.get(bp.as_str()) {
                css.push_str(&format!("\n@media (min-width: {}) {{\n", min_width));
                for (selector, properties) in rules {
                    css.push_str(&format!("  {} {{ {} }}\n", selector, properties));
                }
                css.push_str("}\n");
            }
        }

        css
    }

    fn is_tailwind_class(&self, class: &str) -> bool {
        let prefixes = [
            "p-", "px-", "py-", "pt-", "pr-", "pb-", "pl-", "ps-", "pe-",
            "m-", "mx-", "my-", "mt-", "mr-", "mb-", "ml-", "ms-", "me-",
            "w-", "h-", "min-w-", "min-h-", "max-w-", "max-h-", "size-",
            "gap-", "space-", "inset-", "top-", "right-", "bottom-", "left-",
            "text-", "bg-", "border-", "rounded-", "shadow-", "ring-",
            "font-", "leading-", "tracking-", "z-", "opacity-", "flex-",
            "grid-", "col-", "row-", "justify-", "items-", "self-", "place-",
            "order-", "basis-", "grow", "shrink", "fill-", "stroke-",
            "duration-", "delay-", "ease-", "animate-", "transition-",
            "scale-", "rotate-", "translate-", "skew-", "origin-",
            "blur-", "brightness-", "contrast-", "grayscale", "hue-rotate-",
            "invert", "saturate-", "sepia", "backdrop-", "divide-", "outline-",
            "decoration-", "underline-offset-", "line-clamp-", "aspect-",
            "columns-", "break-", "box-", "float-", "clear-", "object-",
            "overflow-", "overscroll-", "scroll-", "snap-", "touch-",
            "select-", "cursor-", "caret-", "pointer-events-", "resize-",
            "will-change-", "from-", "via-", "to-", "accent-", "content-",
            "indent-", "-m-", "-mx-", "-my-", "-mt-", "-mr-", "-mb-", "-ml-",
            "-top-", "-right-", "-bottom-", "-left-", "-inset-", "-translate-",
            "sm:", "md:", "lg:", "xl:", "2xl:", "hover:", "focus:", "active:",
            "disabled:", "first:", "last:", "odd:", "even:", "group-", "peer-",
            "dark:", "motion-", "print:", "rtl:", "ltr:",
        ];

        let exact = [
            "flex", "grid", "block", "inline", "inline-block", "inline-flex",
            "inline-grid", "hidden", "contents", "flow-root", "list-item",
            "static", "fixed", "absolute", "relative", "sticky",
            "visible", "invisible", "collapse", "grow", "shrink",
            "truncate", "uppercase", "lowercase", "capitalize", "normal-case",
            "italic", "not-italic", "underline", "overline", "line-through",
            "no-underline", "antialiased", "subpixel-antialiased",
            "sr-only", "not-sr-only", "isolate", "isolation-auto",
            "transition", "transition-all", "transition-none",
            "grayscale", "invert", "sepia", "container",
        ];

        for prefix in prefixes {
            if class.starts_with(prefix) {
                return true;
            }
        }

        for e in exact {
            if class == e {
                return true;
            }
        }

        class.contains('[') && class.contains(']')
    }
}

impl Default for TailwindGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_utilities() {
        let gen = TailwindGenerator::new();
        let classes = vec!["flex".to_string(), "hidden".to_string(), "block".to_string()];
        let result = gen.generate(&classes);
        assert_eq!(result.classes_generated, 3);
    }

    #[test]
    fn test_spacing() {
        let gen = TailwindGenerator::new();
        let classes = vec!["p-4".to_string(), "m-2".to_string(), "gap-6".to_string()];
        let result = gen.generate(&classes);
        assert_eq!(result.classes_generated, 3);
        assert!(result.css.contains("padding: 1rem"));
    }

    #[test]
    fn test_colors() {
        let gen = TailwindGenerator::new();
        let classes = vec!["text-red-500".to_string(), "bg-blue-500".to_string()];
        let result = gen.generate(&classes);
        assert_eq!(result.classes_generated, 2);
        assert!(result.css.contains("#ef4444"));
        assert!(result.css.contains("#3b82f6"));
    }

    #[test]
    fn test_responsive() {
        let gen = TailwindGenerator::new();
        let classes = vec!["md:flex".to_string(), "lg:hidden".to_string()];
        let result = gen.generate(&classes);
        assert!(result.css.contains("@media (min-width: 768px)"));
        assert!(result.css.contains("@media (min-width: 1024px)"));
    }

    #[test]
    fn test_arbitrary_values() {
        let gen = TailwindGenerator::new();
        let classes = vec!["w-[200px]".to_string(), "text-[#ff0000]".to_string()];
        let result = gen.generate(&classes);
        assert_eq!(result.classes_generated, 2);
        assert!(result.css.contains("200px"));
        assert!(result.css.contains("#ff0000"));
    }

    #[test]
    fn test_states() {
        let gen = TailwindGenerator::new();
        let classes = vec!["hover:bg-blue-500".to_string()];
        let result = gen.generate(&classes);
        assert!(result.css.contains(":hover"));
    }
}
