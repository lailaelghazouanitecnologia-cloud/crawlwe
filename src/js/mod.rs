//! JavaScript analysis module
//!
//! Parses JavaScript to AST and extracts:
//! - CSS class manipulations (className, classList)
//! - Style assignments
//! - DOM operations
//! - CSS-in-JS patterns
//!
//! Uses OXC parser (fast Rust-native JavaScript parser).
//!
//! ## Library-Specific Parsers
//!
//! The `libraries` module contains specialized parsers for common JS libraries:
//! - GSAP animations
//! - Framer Motion
//! - styled-components / Emotion
//! - React/JSX patterns
//! - Three.js 3D scenes
//! - Tailwind CSS utilities
//! - anime.js animations
//!
//! ## JS to CSS Converter
//!
//! The `converter` module converts JavaScript styling to pure CSS:
//! - CSS-in-JS extraction (styled-components, emotion)
//! - Animation library conversion (GSAP, Framer Motion)
//! - Inline styles to CSS classes
//!
//! ## CSS Modules Parser
//!
//! The `css_modules` module parses CSS Module class patterns:
//! - ComponentName_className__hash patterns
//! - Semantic class name generation
//! - Class mapping for renaming
//!
//! ## Tailwind Generator
//!
//! The `tailwind_gen` module generates CSS for Tailwind utilities:
//! - Spacing, typography, colors
//! - Responsive prefixes
//! - State variants

pub mod parser;
pub mod analyzer;
pub mod vm;
pub mod libraries;
pub mod converter;
pub mod css_modules;
pub mod tailwind_gen;
pub mod smart_css;

pub use parser::{JsParser, JsParseResult};
pub use analyzer::{JsAnalyzer, JsAnalysisResult};
pub use vm::{MicroVm, VmState, JsExecutionResult};
pub use libraries::{LibraryRegistry, LibraryParser, LibraryAnalysis, LibraryCategory, DetectedLibrary};
pub use converter::{JsToCssConverter, JsToCssResult, KeyframeAnimation, Keyframe, ConversionStats};
pub use css_modules::{CssModulesParser, CssModulesResult, CssModuleComponent, CssModuleClass};
pub use tailwind_gen::{TailwindGenerator, TailwindGenResult};
pub use smart_css::{SmartCssExtractor, SmartCssResult, CssRule, CssProperty, CssSource};
