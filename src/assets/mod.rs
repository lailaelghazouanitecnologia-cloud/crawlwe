//! Asset Management Module
//!
//! Comprehensive asset handling for extracted web resources:
//! - SVG extraction, deduplication, and categorization
//! - Image cataloging and format detection
//! - Font registry with metadata
//! - TOML generation for project imports

pub mod registry;

pub use registry::{
    AssetRegistry, AssetStats,
    SvgAsset, SvgCategory,
    ImageAsset, ImageFormat, ImageContext,
    FontAsset, FontFormat,
    CssStats,
};
