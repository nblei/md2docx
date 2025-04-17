use std::path::PathBuf;

use docx_rs::*;
use log::{debug, error, trace};
use markdown::to_mdast;
use serde::Deserialize;
use yaml_front_matter::YamlFrontMatter;

use crate::emitter::Emitter;
use crate::image_reference_collector::ImageReferenceCollector;
use crate::traverser::MarkdownNodeTraverser;

pub const PPI: u32 = 220;
pub const EMUS_PER_INCH: u32 = 914_400;

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub affiliation: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct Parser {
    metadata: Option<Metadata>,
    content: String,
    base_path: Option<std::path::PathBuf>, // For resolving relative paths
    image_reference_collector: ImageReferenceCollector,
    emitter: Emitter,
}

impl Parser {
    pub fn new(filedata: &str, base_path: Option<PathBuf>) -> Self {
        match YamlFrontMatter::parse::<Metadata>(filedata) {
            Ok(document) => Self {
                metadata: Some(document.metadata),
                content: document.content,
                base_path,
                ..Default::default()
            },
            Err(_) => Self {
                metadata: None,
                content: String::from(filedata),
                emitter: Emitter::new(base_path.clone()),
                base_path,
                ..Default::default()
            },
        }
    }

    // Main function to parse markdown and create a DOCX document
    pub fn parse_to_docx(&mut self) -> Docx {
        let mut docx = Docx::new();

        // Initialize numbering for lists
        self.emitter
            .set_image_refernces(self.image_reference_collector.clone().into());
        docx = self.emitter.initialize_numbering(docx);

        // Add title and author information
        docx = self.emitter.add_document_metadata(&self.metadata, docx);

        debug!("Parsing markdown content");
        trace!("Content: {}", self.content);

        // Parse markdown to AST
        if let Ok(ast) = to_mdast(&self.content, &markdown::ParseOptions::default()) {
            debug!("Successfully parsed markdown AST");
            // Multi-pass parsing
            // Pass 1: Collect image references
            self.image_reference_collector.process_node(&ast, ());

            // Pass 2: Process the AST and generate DOCX with reference resolution
            docx = self.emitter.process_node(&ast, docx);
        } else {
            error!("Failed to parse markdown content");
        }

        docx
    }
}
