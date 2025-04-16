use docx_rs::*;
use log::{error, info};
use std::path::PathBuf;

mod emitter;
mod image_reference_collector;
mod parser;
mod traverser;
use parser::Parser;

const SIMPLE_MARKDOWN_YFM: &str = r#"
---
title: A Simple Proposal
author: Nathan Bleier
---

# Section One

This is my **Sample Proposal**

"#;

use clap::Parser as ClapParser;
use std::fs;

/// A tool to convert Markdown to DOCX files
#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input markdown file
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output DOCX file (defaults to input filename with .docx extension)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Use sample markdown content for testing
    #[arg(short, long)]
    sample: bool,

    /// Verbose output (debug logging)
    #[arg(short, long)]
    verbose: bool,

    /// Trace-level logging (includes all debug info plus markdown content)
    #[arg(long)]
    trace: bool,
}

fn main() -> Result<(), DocxError> {
    // Parse CLI arguments first to get verbosity flags
    let cli = Cli::parse();

    // Initialize logger with the appropriate verbosity level
    unsafe {
        if cli.trace {
            std::env::set_var("RUST_LOG", "trace");
        } else if cli.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    info!("Starting md2docx application");

    // Determine the markdown content to use
    let markdown_content = if let Some(input_path) = &cli.input {
        // Read from specified input file
        match fs::read_to_string(input_path) {
            Ok(content) => content,
            Err(e) => {
                error!("Error reading file {}: {}", input_path.display(), e);
                return Ok(());
            }
        }
    } else if cli.sample {
        // Use the sample content for testing
        info!("Using sample content");
        SIMPLE_MARKDOWN_YFM.to_string()
    } else {
        // No input file or sample flag, print usage
        error!("No input file specified. Use --sample to use sample content");
        return Ok(());
    };

    // Create the parser with the markdown content and base path for image resolution
    let base_path = cli
        .input
        .as_ref()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()));
    let mut parser = Parser::new(&markdown_content, base_path);

    // Determine the output filename
    let output_path = if let Some(output) = cli.output {
        output
    } else if let Some(input) = cli.input {
        // Derive output path from input path by changing extension
        let mut output = input.clone();
        output.set_extension("docx");
        output
    } else {
        // Default output path for sample content
        PathBuf::from("output.docx")
    };

    // Create the DOCX file
    let file = match fs::File::create(&output_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Error creating file {}: {}", output_path.display(), e);
            return Ok(());
        }
    };

    // Parse markdown and generate DOCX
    let docx = parser.parse_to_docx();
    match docx.build().pack(file) {
        Ok(_) => {
            info!(
                "Successfully created DOCX file at: {}",
                output_path.display()
            );
            info!("Conversion completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Error creating DOCX file: {}", e);
            error!("Conversion failed");
            Ok(()) // Return Ok to avoid double error messages
        }
    }
}
