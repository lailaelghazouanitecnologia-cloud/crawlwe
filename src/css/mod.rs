//! CSS Processing Module
//!
//! Comprehensive CSS handling:
//! - Asset extraction (fonts, images, @imports)
//! - URL resolution and replacement
//! - Font downloading and embedding
//! - Image downloading
//! - CSS formatting for readability
//!
//! This module provides deep CSS analysis, not just surface-level
//! pattern matching.

pub mod asset_extractor;
pub mod formatter;

pub use asset_extractor::{
    CssAssetExtractor,
    CssAssetExtractionResult,
    FontInfo,
    ImageInfo,
    ImageContext,
    ExtractionStats,
};

pub use formatter::{
    CssFormatter,
    CssFormatOptions,
    CssFormatResult,
    FormatStats,
};
