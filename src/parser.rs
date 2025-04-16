use anyhow::Result;
use bimap::BiMap;
use docx_rs::*;
use log::{debug, error, info, trace, warn};
use markdown::mdast::{Heading, Node};
use markdown::to_mdast;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;
use yaml_front_matter::YamlFrontMatter;

use crate::traverser::MarkdownNodeTraverser;

const PPI: u32 = 220;
const EMUS_PER_INCH: u32 = 914_400;

static REF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\{ref:\s*([^}]*)\s*}"#).unwrap());
fn extract_ref(text: &str) -> Option<&str> {
    REF_REGEX
        .captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
}

/// Returns the image dimensions in (EMU, EMU)
fn get_image_dimensions(file_path: &PathBuf) -> Result<(u32, u32)> {
    let reader = image::io::Reader::open(file_path)?;
    let (dim1, dim2) = reader.into_dimensions()?;
    Ok((EMUS_PER_INCH * dim1 / PPI, EMUS_PER_INCH * dim2 / PPI))
}

#[derive(Deserialize, Debug, Clone)]
struct Metadata {
    title: Option<String>,
    author: Option<String>,
    affiliation: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ImageModifiers {
    scale: f64,
    r#ref: Option<String>,
}

impl Default for ImageModifiers {
    fn default() -> Self {
        Self {
            scale: 1.,
            r#ref: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct StackCounter {
    value_: u32,
}

impl StackCounter {
    pub fn new() -> Self {
        StackCounter { value_: 0 }
    }
    pub fn push(&mut self) {
        self.value_ += 1;
    }
    pub fn pop(&mut self) {
        if self.value_ > 0 {
            self.value_ -= 1;
        }
    }
    pub fn set(&self) -> bool {
        self.value_ > 0
    }
}

#[derive(Debug, Clone, Copy)]
enum ListType {
    Ordered,
    Unordered,
}

impl From<StackCounter> for bool {
    fn from(value: StackCounter) -> Self {
        value.set()
    }
}

impl Default for StackCounter {
    fn default() -> Self {
        StackCounter::new()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ImageReferenceCollector {
    image_figure_count: usize,
    image_references: BiMap<String, usize>,
}

impl ImageReferenceCollector {
    pub fn new() -> Self {
        Self {
            image_figure_count: 0,
            image_references: BiMap::new(),
        }
    }

    pub fn get_references(&self) -> &BiMap<String, usize> {
        &self.image_references
    }
}

impl MarkdownNodeTraverser for ImageReferenceCollector {
    type Output = ();

    // We don't need to do anything special with the result in process_child
    fn process_child(&mut self, node: &Node, _result: &mut Self::Output) {
        self.process_node(node);
    }

    // Override only the image visit method to collect references
    fn visit_image(&mut self, image: &markdown::mdast::Image) -> Self::Output {
        debug!(
            "First pass - collecting image reference: url={}, alt={}",
            image.url, image.alt
        );

        // Check if the image has a reference ID in its alt text
        let res: ImageModifiers =
            serde_json::from_str(&image.alt).unwrap_or(ImageModifiers::default());

        if let Some(reference) = res.r#ref {
            self.image_figure_count += 1;
            let figure_number = self.image_figure_count;

            if self.image_references.contains_left(&reference) {
                error!("Multiple defined reference: {}", reference);
            } else {
                info!("Adding image reference: {} -> {}", reference, figure_number);
                self.image_references.insert(reference, figure_number);
            }
        }

        ()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Parser {
    metadata: Option<Metadata>,
    content: String,
    base_path: Option<std::path::PathBuf>, // For resolving relative paths
    strong_state: StackCounter,
    em_state: StackCounter,
    list_type: Vec<ListType>,
    image_references: BiMap<String, usize>,
}

impl Parser {
    pub fn new(filedata: &str, base_path: Option<PathBuf>) -> Self {
        match YamlFrontMatter::parse::<Metadata>(filedata) {
            Ok(document) => Parser {
                metadata: Some(document.metadata),
                content: document.content,
                base_path,
                ..Default::default()
            },
            Err(_) => Parser {
                metadata: None,
                content: String::from(filedata),
                base_path,
                ..Default::default()
            },
        }
    }

    // Handle document metadata (title, author)
    fn add_document_metadata(&self, mut docx: Docx) -> Docx {
        // Add title and author from metadata if available
        if let Some(metadata) = &self.metadata {
            if let Some(title) = &metadata.title {
                let mut run = Run::new().add_text(title).size(40);
                if !title.is_empty() {
                    run = run.bold();
                }
                // Create paragraph with center justification
                let title_paragraph = docx_rs::Paragraph::new()
                    .add_run(run)
                    .align(AlignmentType::Center);
                docx = docx.add_paragraph(title_paragraph);
            }

            if let Some(author) = &metadata.author {
                let author_paragraph = docx_rs::Paragraph::new()
                    .add_run(Run::new().add_text(author).size(24).italic())
                    .align(AlignmentType::Center);
                docx = docx.add_paragraph(author_paragraph);
                if let Some(affiliation) = &metadata.affiliation {
                    let affiliation_paragraph = docx_rs::Paragraph::new()
                        .add_run(Run::new().add_text(affiliation).size(24).italic())
                        .align(AlignmentType::Center);
                    docx = docx.add_paragraph(affiliation_paragraph);
                }
            }

            // Add a blank line after metadata
            docx = docx.add_paragraph(docx_rs::Paragraph::new());
        }

        docx
    }

    // Handle insertion of images and return the updated docx
    fn handle_image(
        &mut self,
        docx: Docx,
        url: &str,
        alt: &str,
        title: Option<&str>,
        figure_number: usize,
    ) -> Docx {
        let img_url = url.to_string();
        let mut docx = docx;

        // Try to resolve the image path
        if let Some(base_dir) = &self.base_path {
            let img_path = base_dir.join(&img_url);
            debug!("Resolving image path: {}", img_path.display());

            // Check if image exists
            if img_path.exists() {
                debug!("Image file found at: {}", img_path.display());
                // Try to read the image file
                match std::fs::read(&img_path) {
                    Ok(buffer) => {
                        let res: ImageModifiers =
                            serde_json::from_str(alt).unwrap_or(ImageModifiers::default());
                        debug!("Successfully read image file ({} bytes)", buffer.len());
                        let (dim1, dim2) = get_image_dimensions(&img_path).unwrap();
                        let (dim1, dim2) = (
                            (dim1 as f64 * res.scale) as u32,
                            (dim2 as f64 * res.scale) as u32,
                        );

                        // Reference handling is now done in the first pass
                        if let Some(reference) = res.r#ref {
                            debug!("Using reference: {} -> {}", reference, figure_number);
                        } else {
                            debug!("Image has no reference");
                        }

                        // Create a Pic object from the image data
                        // Use a standard image size (5 inches width max)
                        let pic = Pic::new(&buffer).size(dim1, dim2);

                        // Create a new paragraph with centered alignment
                        let img_paragraph = docx_rs::Paragraph::new()
                            .add_run(Run::new().add_image(pic))
                            .align(AlignmentType::Center);

                        // Add the image paragraph to the document
                        docx = docx.add_paragraph(img_paragraph);

                        // Create a caption text with figure number
                        let display_title = title.unwrap_or(alt);
                        let caption_text = if !display_title.is_empty() {
                            format!("Figure {}: {}", figure_number, display_title)
                        } else {
                            format!("Figure {}", figure_number)
                        };

                        // Add a centered caption below the image
                        let caption_paragraph = docx_rs::Paragraph::new()
                            .add_run(Run::new().add_text(caption_text).italic())
                            .align(AlignmentType::Center);

                        docx = docx.add_paragraph(caption_paragraph);
                    }
                    Err(e) => {
                        // If image couldn't be read, add placeholder text
                        warn!("Failed to read image file: {}", e);
                        let placeholder =
                            format!("[Image: {} (could not read file)]", img_path.display());
                        let placeholder_paragraph = docx_rs::Paragraph::new()
                            .add_run(Run::new().add_text(placeholder).italic())
                            .align(AlignmentType::Center);
                        docx = docx.add_paragraph(placeholder_paragraph);
                    }
                }
            } else {
                // If image doesn't exist, use placeholder text
                warn!("Image file not found: {}", img_path.display());
                let placeholder = format!("[Image: {} (not found)]", img_url);
                let placeholder_paragraph = docx_rs::Paragraph::new()
                    .add_run(Run::new().add_text(placeholder).italic())
                    .align(AlignmentType::Center);
                docx = docx.add_paragraph(placeholder_paragraph);
            }
        } else {
            // No base path available, use placeholder text
            warn!("No base path available to resolve image: {}", img_url);
            let placeholder = format!("[Image: {}]", img_url);
            let placeholder_paragraph = docx_rs::Paragraph::new()
                .add_run(Run::new().add_text(placeholder).italic())
                .align(AlignmentType::Center);
            docx = docx.add_paragraph(placeholder_paragraph);
        }

        docx
    }

    // Format text with bold if needed
    fn format_text(&self, text: &str) -> Run {
        let mut run = Run::new().add_text(text);
        if self.strong_state.into() {
            run = run.bold();
        }
        if self.em_state.into() {
            run = run.italic();
        }
        run
    }

    // Add a formatted heading and return the updated docx
    fn add_heading(&self, docx: Docx, text: &str, level: u8) -> Docx {
        let size = match level {
            1 => 36,
            2 => 28,
            3 => 24,
            _ => 20,
        };

        let heading_paragraph =
            docx_rs::Paragraph::new().add_run(Run::new().add_text(text).size(size).bold());

        docx.add_paragraph(heading_paragraph)
    }

    // Initialize numbering for lists based on the docx-rs API
    fn initialize_numbering(&self, docx: Docx) -> Docx {
        // Create bullet list (ID: 1)
        let docx = docx.add_abstract_numbering(
            AbstractNumbering::new(1)
                .add_level(
                    Level::new(
                        0,
                        Start::new(1),
                        NumberFormat::new("bullet"),
                        LevelText::new("•"),
                        LevelJc::new("left"),
                    )
                    .indent(
                        Some(720),
                        Some(SpecialIndentType::Hanging(360)),
                        None,
                        None,
                    ),
                )
                .add_level(
                    Level::new(
                        1,
                        Start::new(1),
                        NumberFormat::new("bullet"),
                        LevelText::new("○"),
                        LevelJc::new("left"),
                    )
                    .indent(
                        Some(1440),
                        Some(SpecialIndentType::Hanging(360)),
                        None,
                        None,
                    ),
                ),
        );

        // Create numbered list (ID: 2)
        let docx = docx.add_abstract_numbering(
            AbstractNumbering::new(2)
                .add_level(
                    Level::new(
                        0,
                        Start::new(1),
                        NumberFormat::new("decimal"),
                        LevelText::new("%1."),
                        LevelJc::new("left"),
                    )
                    .indent(
                        Some(720),
                        Some(SpecialIndentType::Hanging(360)),
                        None,
                        None,
                    ),
                )
                .add_level(
                    Level::new(
                        1,
                        Start::new(1),
                        NumberFormat::new("lowerLetter"),
                        LevelText::new("%2)"),
                        LevelJc::new("left"),
                    )
                    .indent(
                        Some(1440),
                        Some(SpecialIndentType::Hanging(360)),
                        None,
                        None,
                    ),
                ),
        );

        // Associate abstract numberings with concrete numberings
        let docx = docx.add_numbering(Numbering::new(1, 1)); // Bullet list
        let docx = docx.add_numbering(Numbering::new(2, 2)); // Numbered list

        docx
    }

    // Main function to parse markdown and create a DOCX document
    pub fn parse_to_docx(&mut self) -> Docx {
        let mut docx = Docx::new();

        // Initialize numbering for lists
        docx = self.initialize_numbering(docx);

        // Add title and author information
        docx = self.add_document_metadata(docx);

        debug!("Parsing markdown content");
        trace!("Content: {}", self.content);

        // Parse markdown to AST
        if let Ok(ast) = to_mdast(&self.content, &markdown::ParseOptions::default()) {
            debug!("Successfully parsed markdown AST");

            // Multi-pass parsing
            // Pass 1: Collect image references
            let mut reference_collector = ImageReferenceCollector::new();
            reference_collector.process_node(&ast);

            // Transfer collected references to our parser
            self.image_references = reference_collector.get_references().clone();
            debug!("Collected image references: {:?}", self.image_references);

            // Pass 2: Process the AST and generate DOCX with reference resolution
            docx = self.process_node(docx, &ast);
        } else {
            error!("Failed to parse markdown content");
        }

        docx
    }

    fn check_references(&self, text: &str) -> String {
        let mut result = String::from(text);

        // Find all references in the text
        let mut matched_any = false;
        let mut start_idx = 0;

        while let Some(reference_match) = REF_REGEX.find_at(&result, start_idx) {
            matched_any = true;
            let match_range = reference_match.start()..reference_match.end();
            let reference_text = reference_match.as_str();

            if let Some(reference_key) = extract_ref(reference_text) {
                if let Some(figure_number) = self.image_references.get_by_left(reference_key) {
                    // Replace the {ref:key} with "Figure X"
                    debug!(
                        "Replacing reference '{}' with 'Figure {}'",
                        reference_key, figure_number
                    );
                    let replacement = format!("Figure {}", figure_number);
                    result.replace_range(match_range.clone(), &replacement);

                    // Adjust the start index for the next search
                    start_idx = match_range.start + replacement.len();
                } else {
                    warn!(
                        "Reference '{}' not found in collected references",
                        reference_key
                    );
                    start_idx = match_range.end;
                }
            } else {
                start_idx = match_range.end;
            }
        }

        if !matched_any {
            // No references found, return the original text
            return String::from(text);
        }

        result
    }

    // Recursively process AST nodes
    fn process_node(&mut self, mut docx: Docx, node: &Node) -> Docx {
        match node {
            Node::Root(root) => {
                let mut result_docx = docx;

                for child in &root.children {
                    result_docx = self.process_node(result_docx, child);
                }

                result_docx
            }
            Node::Paragraph(para) => {
                for node in para.children.iter() {
                    docx = self.process_node(docx, node);
                }
                docx
            }
            Node::Heading(heading) => self.process_heading(docx, heading),
            Node::Image(image) => {
                debug!(
                    "Processing image: url={}, alt={}, title={:?}",
                    image.url, image.alt, image.title
                );

                // Get figure number from the image reference or generate a new one
                let res: ImageModifiers =
                    serde_json::from_str(&image.alt).unwrap_or(ImageModifiers::default());

                let figure_number = if let Some(reference) = &res.r#ref {
                    // Use the figure number from the first pass
                    *self.image_references.get_by_left(reference).unwrap_or(&0)
                } else {
                    // For images without references, use the position in the document
                    let pos = self.image_references.len() + 1;
                    debug!("Image without reference, assigning position: {}", pos);
                    pos
                };

                self.handle_image(
                    docx,
                    &image.url,
                    &image.alt,
                    image.title.as_deref(),
                    figure_number,
                )
            }
            Node::Text(text) => {
                let textval = self.check_references(&text.value);
                let text_paragraph = docx_rs::Paragraph::new().add_run(self.format_text(&textval));
                docx.add_paragraph(text_paragraph)
            }
            Node::Strong(strong) => {
                self.strong_state.push();
                for node in &strong.children {
                    docx = self.process_node(docx, node);
                }
                self.strong_state.pop();
                docx
            }
            Node::Blockquote(_) => todo!(),
            Node::FootnoteDefinition(_) => todo!(),
            Node::MdxJsxFlowElement(_) => todo!(),
            Node::List(list) => {
                // Determine list type (ordered/numbered or unordered/bullet)
                if list.ordered {
                    self.list_type.push(ListType::Ordered);
                } else {
                    self.list_type.push(ListType::Unordered);
                }

                // Process all list items
                let mut result_docx = docx;
                for child in &list.children {
                    result_docx = self.process_node(result_docx, child);
                }

                // Remove list type from stack
                self.list_type.pop();
                result_docx
            }
            Node::MdxjsEsm(_) => todo!(),
            Node::Toml(_) => todo!(),
            Node::Yaml(_) => todo!(),
            Node::Break(_) => todo!(),
            Node::InlineCode(_) => todo!(),
            Node::InlineMath(_) => todo!(),
            Node::Delete(_) => todo!(),
            Node::Emphasis(emphasis) => {
                self.em_state.push();
                for node in &emphasis.children {
                    docx = self.process_node(docx, node);
                }
                self.em_state.pop();
                docx
            }
            Node::MdxTextExpression(_) => todo!(),
            Node::FootnoteReference(_) => todo!(),
            Node::Html(_) => todo!(),
            Node::ImageReference(_) => todo!(),
            Node::MdxJsxTextElement(_) => todo!(),
            Node::Link(_) => todo!(),
            Node::LinkReference(_) => todo!(),
            Node::Code(_) => todo!(),
            Node::Math(_) => todo!(),
            Node::MdxFlowExpression(_) => todo!(),
            Node::Table(_) => todo!(),
            Node::ThematicBreak(_) => todo!(),
            Node::TableRow(_) => todo!(),
            Node::TableCell(_) => todo!(),
            Node::ListItem(list_item) => {
                // Log unsupported features
                if list_item.checked.is_some() {
                    debug!("Check Boxes not yet supported");
                }
                if list_item.spread {
                    debug!("Spread list items not yet supported");
                }

                // Get the current list type
                if self.list_type.is_empty() {
                    debug!("List item found outside of a list context");
                    return docx;
                }

                // Determine the appropriate numbering ID based on list type
                let numbering_id = match self.list_type.last().unwrap() {
                    ListType::Ordered => 2,   // Numbered list
                    ListType::Unordered => 1, // Bullet list
                };

                // Calculate indent level based on nesting depth
                let indent_level = self.list_type.len() - 1;

                // Create a paragraph with numbering properties
                let mut paragraph = docx_rs::Paragraph::new().numbering(
                    NumberingId::new(numbering_id),
                    IndentLevel::new(indent_level),
                );

                // Process the content of the list item and add to the paragraph
                for child in &list_item.children {
                    match child {
                        Node::Paragraph(para) => {
                            // For paragraph nodes in list items, process their children inline
                            for para_child in &para.children {
                                paragraph = self.add_inline_content(paragraph, para_child);
                            }
                        }
                        _ => {
                            // For other types of content, process recursively
                            // This handles nested lists, code blocks, etc.
                            docx = self.process_node(docx, child);
                        }
                    }
                }

                // Add the list item paragraph to the document
                docx.add_paragraph(paragraph)
            }
            Node::Definition(_) => todo!(),
        }
    }

    // Process heading nodes
    fn process_heading(&self, docx: Docx, heading: &Heading) -> Docx {
        let mut text = String::new();

        for child in &heading.children {
            if let Node::Text(text_node) = child {
                text.push_str(&text_node.value);
            }
        }

        self.add_heading(docx, &text, heading.depth)
    }

    // Add inline content to a paragraph
    fn add_inline_content(
        &self,
        mut paragraph: docx_rs::Paragraph,
        node: &Node,
    ) -> docx_rs::Paragraph {
        match node {
            Node::Text(text) => {
                let textval = self.check_references(&text.value);
                paragraph = paragraph.add_run(Run::new().add_text(&textval));
            }
            Node::Strong(strong) => {
                for child in &strong.children {
                    if let Node::Text(text) = child {
                        let textval = self.check_references(&text.value);
                        paragraph = paragraph.add_run(Run::new().add_text(&textval).bold());
                    }
                }
            }
            Node::Emphasis(emphasis) => {
                for child in &emphasis.children {
                    if let Node::Text(text) = child {
                        let textval = self.check_references(&text.value);
                        paragraph = paragraph.add_run(Run::new().add_text(&textval).italic());
                    }
                }
            }
            // Handle other inline node types if needed
            _ => {}
        }
        paragraph
    }
}
