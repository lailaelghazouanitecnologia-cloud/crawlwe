//! CrawlWe Core - High-performance web page extractor
//!
//! This library provides the Rust core for the CrawlWe web scraper.
//! It handles: browser automation, HTML/CSS parsing, content analysis, and optimization.
//!
//! Also includes ZAD - a template metaprogramming language.
//!
//! Output structure:
//! - index.html (clean semantic HTML)
//! - styles.css (unified CSS)
//! - scripts/main.js (relevant JS only)
//! - project.toml (metadata and dependencies)

pub mod analyzer;
pub mod models;
pub mod zad;
pub mod export;
pub mod capture;
pub mod vm;
pub mod pipeline;
pub mod js;
pub mod css;

pub mod parser {
    pub mod css;
    pub mod html;
}

// Re-export main types
pub use analyzer::Analyzer;
pub use models::*;
