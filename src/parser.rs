use anyhow::Result;
use docx_rs::*;
use log::{debug, error, trace, warn};
use markdown::mdast::{Heading, Node};
use markdown::to_mdast;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::image_reference_collector::{ImageModifiers, ImageReferenceCollector};
use crate::traverser::MarkdownNodeTraverser;

pub const PPI: u32 = 220;
pub const EMUS_PER_INCH: u32 = 914_400;

#[derive(Default, Debug, Clone)]
pub struct Parser {
    metadata: Option<Metadata>,
    content: String,
    base_path: Option<std::path::PathBuf>, // For resolving relative paths
    strong_state: StackCounter,
    em_state: StackCounter,
    list_type: Vec<ListType>,
    image_references: HashMap<String, usize>,
}

impl Parser {
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
            reference_collector.process_node(&ast, ());

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
                if let Some(figure_number) = self.image_references.get(reference_key) {
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
                    *self.image_references.get(reference).unwrap_or(&0)
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
            Node::Table(table) => {
                for child in table.children.iter() {
                    docx = self.process_node(docx, child);
                }
                docx
            }
            Node::ThematicBreak(_) => todo!(),
            Node::TableRow(table_row) => {
                for child in table_row.children.iter() {
                    docx = self.process_node(docx, child);
                }
                docx
            }
            Node::TableCell(table_cell) => {
                todo!();
                docx
            }
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
