//! Capture Module
//!
//! Utilities for web page extraction.
//!
//! - `resources` - Extract scripts, stylesheets, and fonts from HTML
//! - `cleaner` - Clean HTML by removing scripts, data attributes, etc.

pub mod resources;
pub mod cleaner;

pub use resources::*;
pub use cleaner::*;
