//! CrawlWe - Fast web page extractor for ML training
//!
//! A Rust-native tool for capturing web pages with full fidelity.
//!
//! Usage:
//!   crawlwe fetch <URL>           Capture a single page
//!   crawlwe batch <file>          Capture multiple pages from file
//!   crawlwe analyze <path>        Analyze captured content
//!   crawlwe export <path>         Generate project.toml

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(name = "crawlwe")]
#[command(author = "CrawlWe Team")]
#[command(version = "0.1.0")]
#[command(about = "Web page extractor for ML training - captures HTML + CSS with full fidelity")]
#[command(long_about = r#"
CrawlWe extracts web pages preserving:
  - Fully rendered HTML (after JavaScript execution)
  - All CSS (inline, external, computed)
  - External resources (scripts, fonts, images)
  - Library detection (GSAP, Three.js, React, etc.)

Output structure:
  output/
  ├── index.html       Clean HTML (scripts removed)
  ├── styles.css       Combined CSS
  ├── project.toml     Dependencies and metadata
  ├── screenshot.png   Full page screenshot
  └── data/
      ├── raw.html     Original HTML
      └── metadata.json
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture a single web page
    #[command(alias = "f")]
    Fetch {
        /// URL to capture
        url: String,

        /// Output directory
        #[arg(short, long, default_value = "output")]
        output: PathBuf,

        /// Wait time in seconds for JS to load
        #[arg(short, long, default_value = "5")]
        wait: u64,

        /// Viewport width
        #[arg(long, default_value = "1440")]
        width: u32,

        /// Viewport height
        #[arg(long, default_value = "900")]
        height: u32,

        /// Take full page screenshot
        #[arg(long, default_value = "true")]
        screenshot: bool,

        /// Detect libraries (GSAP, React, etc.)
        #[arg(long, default_value = "true")]
        detect_libs: bool,

        /// Generate project.toml
        #[arg(long, default_value = "true")]
        project_toml: bool,

        /// Use static HTTP fetch (no browser, faster but no JS rendering)
        #[arg(long)]
        r#static: bool,
    },

    /// Capture multiple pages from a file (one URL per line)
    #[command(alias = "b")]
    Batch {
        /// File containing URLs (one per line)
        file: PathBuf,

        /// Base output directory
        #[arg(short, long, default_value = "captures")]
        output: PathBuf,

        /// Wait time per page
        #[arg(short, long, default_value = "5")]
        wait: u64,

        /// Number of parallel captures
        #[arg(short, long, default_value = "1")]
        parallel: usize,
    },

    /// Analyze captured content
    #[command(alias = "a")]
    Analyze {
        /// Path to captured content (directory or HTML file)
        path: PathBuf,

        /// Output format (json, toml, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate or update project.toml for captured content
    Export {
        /// Path to captured content directory
        path: PathBuf,

        /// Force overwrite existing project.toml
        #[arg(short, long)]
        force: bool,
    },

    /// Parse content with the VM (advanced)
    Parse {
        /// Input file (HTML, CSS, or JS)
        input: PathBuf,

        /// Parser program to use (css_analyzer, html_extractor, library_detector)
        #[arg(short, long, default_value = "auto")]
        program: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch {
            url,
            output,
            wait,
            width,
            height,
            screenshot,
            detect_libs,
            project_toml,
            r#static,
        } => {
            if r#static {
                commands::fetch_static::run(&url, &output, project_toml).await?;
            } else {
                commands::fetch::run(
                    &url, &output, wait, width, height, screenshot, detect_libs, project_toml,
                )
                .await?;
            }
        }

        Commands::Batch {
            file,
            output,
            wait,
            parallel,
        } => {
            commands::batch::run(&file, &output, wait, parallel).await?;
        }

        Commands::Analyze { path, format } => {
            commands::analyze::run(&path, &format)?;
        }

        Commands::Export { path, force } => {
            commands::export::run(&path, force)?;
        }

        Commands::Parse {
            input,
            program,
            output,
        } => {
            commands::parse::run(&input, &program, output.as_deref())?;
        }
    }

    Ok(())
}
