//! Batch command - Capture multiple pages from a file

use std::fs;
use std::path::Path;
use url::Url;

pub fn run(
    file: &Path,
    output: &Path,
    wait: u64,
    _parallel: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe Batch Capture");
    println!("=====================");

    let content = fs::read_to_string(file)?;
    let urls: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    println!("Found {} URLs to capture", urls.len());
    println!("Output: {:?}", output);
    println!();

    fs::create_dir_all(output)?;

    for (i, url) in urls.iter().enumerate() {
        println!("\n[{}/{}] {}", i + 1, urls.len(), url);

        let domain = Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("site_{}", i));

        let site_output = output.join(&domain);

        match super::fetch::run(
            url,
            &site_output,
            wait,
            1440,
            900,
            true,
            true,
            true,
        ) {
            Ok(_) => println!("  Done: {:?}", site_output),
            Err(e) => {
                eprintln!("  ERROR: {}", e);
                let error_file = site_output.join("error.txt");
                fs::create_dir_all(&site_output).ok();
                fs::write(&error_file, format!("URL: {}\nError: {}", url, e)).ok();
            }
        }
    }

    println!("\n=====================");
    println!("Batch capture complete!");
    println!("Output: {:?}", output);

    Ok(())
}
