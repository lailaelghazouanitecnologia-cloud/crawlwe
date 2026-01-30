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

pub mod parser;
pub mod analyzer;
pub mod vm;
pub mod libraries;

pub use parser::{JsParser, JsParseResult};
pub use analyzer::{JsAnalyzer, JsAnalysisResult};
pub use vm::{MicroVm, VmState, JsExecutionResult};
pub use libraries::{LibraryRegistry, LibraryParser, LibraryAnalysis, LibraryCategory, DetectedLibrary};
