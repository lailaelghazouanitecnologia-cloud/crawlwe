//! Parse command - Run VM parser programs on content

use std::fs;
use std::path::Path;

use crawlwe_core::vm::{VM, ParserBuilder, Value};
use crawlwe_core;

pub fn run(
    input: &Path,
    program: &str,
    output: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("CrawlWe Parser (VM)");
    println!("===================");
    println!("Input: {:?}", input);
    println!("Program: {}", program);
    println!();

    // Read input file
    let content = fs::read_to_string(input)?;
    let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("");

    // Select program
    let vm_program = match program {
        "css_analyzer" | "css" => ParserBuilder::css_analyzer(),
        "html_extractor" | "html" => ParserBuilder::html_extractor(),
        "library_detector" | "libs" => ParserBuilder::library_detector(),
        "auto" => {
            match ext {
                "css" => ParserBuilder::css_analyzer(),
                "html" | "htm" => ParserBuilder::html_extractor(),
                "js" => ParserBuilder::library_detector(),
                _ => ParserBuilder::html_extractor(),
            }
        }
        _ => {
            eprintln!("Unknown program: {}", program);
            eprintln!("Available: css_analyzer, html_extractor, library_detector");
            return Ok(());
        }
    };

    // Execute
    let mut vm = VM::new().with_debug(false);
    let state = vm.execute(&vm_program, &content)?;

    // Format output
    let result = format_results(&state.results);

    // Output
    if let Some(output_path) = output {
        fs::write(output_path, &result)?;
        println!("Output saved to: {:?}", output_path);
    } else {
        println!("Results:\n{}", result);
    }

    Ok(())
}

fn format_results(results: &std::collections::HashMap<String, Vec<Value>>) -> String {
    let mut output = String::new();

    for (key, values) in results {
        output.push_str(&format!("## {}\n", key));

        for value in values {
            match value {
                Value::String(s) => output.push_str(&format!("  - {}\n", s)),
                Value::Map(m) => {
                    output.push_str("  - {\n");
                    for (k, v) in m {
                        output.push_str(&format!("      {}: {:?}\n", k, v));
                    }
                    output.push_str("    }\n");
                }
                Value::List(l) => {
                    output.push_str(&format!("  - {:?}\n", l));
                }
                _ => output.push_str(&format!("  - {:?}\n", value)),
            }
        }

        output.push('\n');
    }

    output
}
