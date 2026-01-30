//! JavaScript analysis module
//!
//! Parses JavaScript to AST and extracts:
//! - CSS class manipulations (className, classList)
//! - Style assignments
//! - DOM operations
//! - CSS-in-JS patterns
//!
//! Uses OXC parser (fast Rust-native JavaScript parser).

pub mod parser;
pub mod analyzer;
pub mod vm;

pub use parser::{JsParser, JsParseResult};
pub use analyzer::{JsAnalyzer, JsAnalysisResult};
pub use vm::{MicroVm, VmState, JsExecutionResult};
