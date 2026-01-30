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

pub mod parser;
pub mod analyzer;
pub mod vm;
pub mod libraries;
pub mod converter;

pub use parser::{JsParser, JsParseResult};
pub use analyzer::{JsAnalyzer, JsAnalysisResult};
pub use vm::{MicroVm, VmState, JsExecutionResult};
pub use libraries::{LibraryRegistry, LibraryParser, LibraryAnalysis, LibraryCategory, DetectedLibrary};
pub use converter::{JsToCssConverter, JsToCssResult, KeyframeAnimation, Keyframe, ConversionStats};
