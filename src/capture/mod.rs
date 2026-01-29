//! Capture Module
//!
//! Unified capture functionality in Rust for web page extraction.
//!
//! ## Modules
//!
//! - `browser` - Browser automation and page navigation
//! - `resources` - Extract scripts, stylesheets, and fonts from HTML
//! - `cleaner` - Clean HTML by removing scripts, data attributes, etc.

pub mod browser;
pub mod resources;
pub mod cleaner;

pub use browser::*;
pub use resources::*;
pub use cleaner::*;
