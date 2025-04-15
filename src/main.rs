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

        // Setup parser options
        let options = Options::empty();
        let parser = MarkdownParser::new_ext(&self.content, options);

        // Add title if available
        if let Some(metadata) = &self.metadata {
            if let Some(title) = &metadata.title {
                let mut run = Run::new().add_text(title).size(40);
                if title.len() > 0 {
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

        // Parse markdown content
        let mut current_text = String::new();
        let mut is_bold = false;
        let mut current_paragraph = Paragraph::new();
        let mut in_heading = false;
        let mut current_heading_level = HeadingLevel::H1;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    // Start a new heading
                    in_heading = true;
                    current_heading_level = level;
                    current_paragraph = Paragraph::new();
                }
                Event::End(TagEnd::Heading(_)) => {
                    // End of heading - add text with heading formatting
                    if !current_text.is_empty() {
                        let size = match current_heading_level {
                            HeadingLevel::H1 => 36,
                            HeadingLevel::H2 => 28,
                            HeadingLevel::H3 => 24,
                            _ => 20,
                        };

                        // Apply heading formatting
                        let mut run = Run::new().add_text(&current_text).size(size);

                        // Make headings bold
                        run = run.bold();

                        current_paragraph = current_paragraph.add_run(run);
                        docx = docx.add_paragraph(current_paragraph);

                        // Reset
                        current_text.clear();
                        current_paragraph = Paragraph::new();
                        in_heading = false;
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    // Start new paragraph
                    current_paragraph = Paragraph::new();
                }
                Event::End(TagEnd::Paragraph) => {
                    // Add paragraph if not empty
                    if !current_text.is_empty() {
                        let mut run = Run::new().add_text(&current_text);
                        if is_bold {
                            run = run.bold();
                        }
                        current_paragraph = current_paragraph.add_run(run);
                        current_text.clear();
                    }

                    // Add paragraph to document
                    docx = docx.add_paragraph(current_paragraph);
                    current_paragraph = Paragraph::new();
                    is_bold = false;
                }
                Event::Start(Tag::Strong) => {
                    // Add any current text as normal text
                    if !current_text.is_empty() {
                        current_paragraph =
                            current_paragraph.add_run(Run::new().add_text(&current_text));
                        current_text.clear();
                    }
                    is_bold = true;
                }
                Event::End(TagEnd::Strong) => {
                    // Add bold text
                    if !current_text.is_empty() {
                        current_paragraph =
                            current_paragraph.add_run(Run::new().add_text(&current_text).bold());
                        current_text.clear();
                    }
                    is_bold = false;
                }
                Event::Text(text) => {
                    current_text.push_str(&text);
                }
                _ => {}
            }
        }

        // Handle any remaining text
        if !current_text.is_empty() {
            let mut run = Run::new().add_text(&current_text);
            if is_bold {
                run = run.bold();
            }
            if in_heading {
                let size = match current_heading_level {
                    HeadingLevel::H1 => 36,
                    HeadingLevel::H2 => 28,
                    HeadingLevel::H3 => 24,
                    _ => 20,
                };
                run = run.size(size);
            }

            current_paragraph = current_paragraph.add_run(run);
            docx = docx.add_paragraph(current_paragraph);
        }

        docx
    }
}

fn main() -> Result<(), DocxError> {
    // Load from constant example for now
    let parser = Parser::new(SIMPLE_MARKDOWN_YFM);

    if let Some(metadata) = &parser.metadata {
        assert_eq!(metadata.author.as_ref().unwrap(), "Nathan Bleier");
        assert_eq!(metadata.title.as_ref().unwrap(), "A Simple Proposal");
    }

    // Create the DOCX file
    let path = std::path::Path::new("./hello.docx");
    let file = std::fs::File::create(path).unwrap();

    // Parse markdown and generate DOCX
    let docx = parser.parse_to_docx();
    docx.build().pack(file)?;

    println!("Successfully created DOCX file at: {}", path.display());
    Ok(())
}
