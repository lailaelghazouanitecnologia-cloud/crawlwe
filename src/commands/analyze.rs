//! Analyze command - Analyze captured content

use std::fs;
use std::path::Path;

use crawlwe_core::pipeline::{
    CssMicroparser, JsMicroparser, LibMicroparser, HtmlMicroparser,
};
use crawlwe_core;

pub fn run(path: &Path, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe Analyzer");
    println!("================");
    println!("Path: {:?}", path);
    println!();

    // Determine what to analyze
    let (html, css, js) = if path.is_dir() {
        // Analyze captured directory
        let html = fs::read_to_string(path.join("index.html"))
            .or_else(|_| fs::read_to_string(path.join("data/raw.html")))
            .unwrap_or_default();
        let css = fs::read_to_string(path.join("styles.css")).unwrap_or_default();
        let js = fs::read_to_string(path.join("scripts/main.js")).unwrap_or_default();
        (html, css, js)
    } else {
        // Analyze single file
        let content = fs::read_to_string(path)?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match ext {
            "html" | "htm" => (content, String::new(), String::new()),
            "css" => (String::new(), content, String::new()),
            "js" => (String::new(), String::new(), content),
            _ => (content, String::new(), String::new()),
        }
    };

    // Run analysis
    let html_analysis = if !html.is_empty() {
        Some(HtmlMicroparser::parse(&html))
    } else {
        None
    };

    let css_analysis = if !css.is_empty() {
        Some(CssMicroparser::parse(&css))
    } else {
        None
    };

    let js_analysis = if !js.is_empty() {
        Some(JsMicroparser::parse(&js))
    } else {
        None
    };

    // Detect libraries
    let script_urls: Vec<String> = html_analysis
        .as_ref()
        .map(|h| h.scripts.iter().filter_map(|s| s.src.clone()).collect())
        .unwrap_or_default();
    let lib_analysis = LibMicroparser::parse(&script_urls, &js);

    // Output results
    match format {
        "json" => {
            let result = serde_json::json!({
                "html": html_analysis,
                "css": css_analysis,
                "js": js_analysis,
                "libraries": lib_analysis,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "toml" => {
            // Simplified TOML output
            println!("[analysis]");
            if let Some(ref html) = html_analysis {
                println!("html_title = {:?}", html.title);
                println!("html_scripts = {}", html.scripts.len());
                println!("html_stylesheets = {}", html.stylesheets.len());
            }
            if let Some(ref css) = css_analysis {
                println!("css_variables = {}", css.variables.len());
                println!("css_keyframes = {}", css.keyframes.len());
                println!("css_colors = {}", css.colors.len());
            }
            if let Some(ref js) = js_analysis {
                println!("js_imports = {}", js.imports.len());
                println!("js_exports = {}", js.exports.len());
            }
            println!("\n[libraries]");
            for lib in &lib_analysis.libraries {
                println!("{} = {{ category = \"{}\", version = {:?} }}",
                    lib.name, lib.category, lib.version);
            }
        }
        _ => {
            // Text format (default)
            println!("ANALYSIS RESULTS");
            println!("================\n");

            if let Some(ref html) = html_analysis {
                println!("HTML:");
                println!("  Title: {:?}", html.title);
                println!("  Scripts: {} ({} external)",
                    html.scripts.len(),
                    html.scripts.iter().filter(|s| s.src.is_some()).count());
                println!("  Stylesheets: {}", html.stylesheets.len());
                println!("  Images: {}", html.images.len());
                println!("  Total elements: ~{}", html.stats.total_elements);
                println!();
            }

            if let Some(ref css) = css_analysis {
                println!("CSS:");
                println!("  Variables: {}", css.variables.len());
                for var in css.variables.iter().take(5) {
                    println!("    {}: {} (used {} times)", var.name, var.value, var.usages);
                }
                if css.variables.len() > 5 {
                    println!("    ... and {} more", css.variables.len() - 5);
                }
                println!("  Keyframes: {}", css.keyframes.len());
                for kf in &css.keyframes {
                    println!("    @keyframes {}", kf.name);
                }
                println!("  Colors: {} unique", css.colors.len());
                println!("  Fonts: {:?}", css.fonts);
                println!("  Media queries: {}", css.media_queries.len());
                println!();
            }

            if let Some(ref js) = js_analysis {
                println!("JavaScript:");
                println!("  Imports: {}", js.imports.len());
                for imp in js.imports.iter().take(5) {
                    println!("    {} {}", if imp.is_dynamic { "dynamic" } else { "static" }, imp.source);
                }
                println!("  Exports: {}", js.exports.len());
                println!("  Functions: {}", js.functions.len());
                println!("  Classes: {}", js.classes.len());
                println!("  Uses ESM: {}", js.stats.uses_esm);
                println!("  Uses async/await: {}", js.stats.uses_async);
                println!();
            }

            println!("Libraries Detected: {}", lib_analysis.libraries.len());
            for lib in &lib_analysis.libraries {
                let version = lib.version.as_deref().unwrap_or("unknown");
                println!("  {} v{} [{}]", lib.name, version, lib.category);
            }

            if !lib_analysis.by_category.is_empty() {
                println!("\nBy Category:");
                for (cat, libs) in &lib_analysis.by_category {
                    println!("  {}: {}", cat, libs.join(", "));
                }
            }
        }
    }

    Ok(())
}
