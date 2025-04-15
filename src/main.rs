use docx_rs::*;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd};
use serde::Deserialize;
use yaml_front_matter::YamlFrontMatter;

#[derive(Deserialize)]
struct Metadata {
    title: Option<String>,
    author: Option<String>,
}

const SIMPLE_MARKDOWN_YFM: &str = r#"
---
title: A Simple Proposal
author: Nathan Bleier
---

# Section One

This is my **Sample Proposal**

"#;

struct Parser {
    metadata: Option<Metadata>,
    content: String,
}

impl Parser {
    fn new(filedata: &str) -> Self {
        match YamlFrontMatter::parse::<Metadata>(filedata) {
            Ok(document) => Parser {
                metadata: Some(document.metadata),
                content: document.content,
            },
            Err(_) => Parser {
                metadata: None,
                content: String::from(filedata),
            },
        }
    }

    fn parse_to_docx(&self) -> Docx {
        let mut docx = Docx::new();

        // Add title and author from metadata if available
        if let Some(metadata) = &self.metadata {
            if let Some(title) = &metadata.title {
                let mut run = Run::new().add_text(title).size(40);
                if !title.is_empty() {
                    run = run.bold();
                }
                let title_paragraph = Paragraph::new().add_run(run);
                docx = docx.add_paragraph(title_paragraph);
            }

            if let Some(author) = &metadata.author {
                let author_paragraph =
                    Paragraph::new().add_run(Run::new().add_text(author).size(24).italic());
                docx = docx.add_paragraph(author_paragraph);
            }

            // Add a blank line after metadata
            docx = docx.add_paragraph(Paragraph::new());
        }

        // Set up the markdown parser with basic options
        let options = Options::empty();
        let parser = MarkdownParser::new_ext(&self.content, options);

        // State tracking
        let mut current_heading_level = HeadingLevel::H1;
        let mut is_bold = false;
        let mut paragraph = Paragraph::new();
        let mut current_text = String::new();

        // Process each event in the markdown
        for event in parser {
            match event {
                // Headers
                Event::Start(Tag::Heading { level, .. }) => {
                    // Start a new heading paragraph
                    if !current_text.is_empty() {
                        let text = current_text.clone();
                        paragraph = paragraph.add_run(Run::new().add_text(text));
                        docx = docx.add_paragraph(paragraph);
                        current_text.clear();
                    }

                    current_heading_level = level;
                    paragraph = Paragraph::new();
                }

                // Paragraphs
                Event::Start(Tag::Paragraph) => {
                    // Start a new paragraph
                    if !current_text.is_empty() {
                        let text = current_text.clone();
                        paragraph = paragraph.add_run(Run::new().add_text(text));
                        docx = docx.add_paragraph(paragraph);
                        current_text.clear();
                    }

                    paragraph = Paragraph::new();
                }

                // Bold text
                Event::Start(Tag::Strong) => {
                    // If we have accumulated non-bold text, add it first
                    if !current_text.is_empty() {
                        let text = current_text.clone();
                        paragraph = paragraph.add_run(Run::new().add_text(text));
                        current_text.clear();
                    }
                    is_bold = true;
                }

                // End tags
                Event::End(tag) => {
                    match tag {
                        TagEnd::Heading(_) => {
                            // Add the heading text with proper formatting
                            if !current_text.is_empty() {
                                let size = match current_heading_level {
                                    HeadingLevel::H1 => 36,
                                    HeadingLevel::H2 => 28,
                                    HeadingLevel::H3 => 24,
                                    _ => 20,
                                };

                                let text = current_text.clone();
                                paragraph =
                                    paragraph.add_run(Run::new().add_text(text).size(size).bold());
                                docx = docx.add_paragraph(paragraph);

                                current_text.clear();
                                paragraph = Paragraph::new();
                            }
                        }

                        TagEnd::Paragraph => {
                            // Add any remaining text in the paragraph
                            if !current_text.is_empty() {
                                let text = current_text.clone();
                                let mut run = Run::new().add_text(text);

                                if is_bold {
                                    run = run.bold();
                                }

                                paragraph = paragraph.add_run(run);
                                docx = docx.add_paragraph(paragraph);

                                current_text.clear();
                            } else if !paragraph.children.is_empty() {
                                docx = docx.add_paragraph(paragraph);
                            }

                            is_bold = false;
                            paragraph = Paragraph::new();
                        }

                        TagEnd::Strong => {
                            // End of bold text - add it with bold formatting
                            if !current_text.is_empty() {
                                let text = current_text.clone();
                                paragraph = paragraph.add_run(Run::new().add_text(text).bold());
                                current_text.clear();
                            }
                            is_bold = false;
                        }

                        _ => {}
                    }
                }

                // Text content
                Event::Text(text) => {
                    current_text.push_str(&text);
                }

                // Soft breaks (newlines)
                Event::SoftBreak => {
                    current_text.push(' ');
                }

                // Hard breaks (new paragraphs)
                Event::HardBreak => {
                    if !current_text.is_empty() {
                        let text = current_text.clone();
                        let mut run = Run::new().add_text(text);

                        if is_bold {
                            run = run.bold();
                        }

                        paragraph = paragraph.add_run(run);
                        docx = docx.add_paragraph(paragraph);

                        current_text.clear();
                    }

                    paragraph = Paragraph::new();
                }

                // Ignore other events
                _ => {}
            }
        }

        // Add any final content
        if !current_text.is_empty() {
            let text = current_text.clone();
            let mut run = Run::new().add_text(text);

            if is_bold {
                run = run.bold();
            }

            paragraph = paragraph.add_run(run);
            docx = docx.add_paragraph(paragraph);
        }

        docx
    }
}

use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;

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
}

fn main() -> Result<(), DocxError> {
    let cli = Cli::parse();

    // Determine the markdown content to use
    let markdown_content = if let Some(input_path) = &cli.input {
        // Read from specified input file
        match fs::read_to_string(input_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file {}: {}", input_path.display(), e);
                return Ok(());
            }
        }
    } else if cli.sample {
        // Use the sample content for testing
        println!("Using sample content.");
        SIMPLE_MARKDOWN_YFM.to_string()
    } else {
        // No input file or sample flag, print usage
        eprintln!("Error: No input file specified. Use --sample to use sample content.");
        return Ok(());
    };

    // Create the parser with the markdown content
    let parser = Parser::new(&markdown_content);

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
            eprintln!("Error creating file {}: {}", output_path.display(), e);
            return Ok(());
        }
    };

    // Parse markdown and generate DOCX
    let docx = parser.parse_to_docx();
    match docx.build().pack(file) {
        Ok(_) => {
            println!(
                "Successfully created DOCX file at: {}",
                output_path.display()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("Error creating DOCX file: {}", e);
            Ok(()) // Return Ok to avoid double error messages
        }
    }
}
