use anyhow::Result;
use docx_rs::*;
use log::debug;
use log::warn;
use markdown::mdast;
use markdown::mdast::Table;
use markdown::mdast::{Heading, Node};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;

use crate::{
    image_reference_collector::ImageModifiers,
    parser::{EMUS_PER_INCH, Metadata, PPI},
    traverser::MarkdownNodeTraverser,
};

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
pub struct Emitter {
    strong_state: StackCounter,
    base_path: Option<PathBuf>,
    em_state: StackCounter,
    list_type: Vec<ListType>,
    image_references: HashMap<String, usize>,
    table: Vec<docx_rs::TableRow>,
    table_cells: Vec<docx_rs::TableCell>,
    paragraph: docx_rs::Paragraph,
    paragraph_alignment: Option<AlignmentType>,
}

impl Emitter {
    pub fn new(base_path: Option<PathBuf>) -> Self {
        Self {
            base_path: base_path,
            ..Default::default()
        }
    }

    pub fn set_image_refernces(&mut self, image_references: HashMap<String, usize>) {
        self.image_references = image_references;
    }
    // Handle document metadata (title, author)
    pub fn add_document_metadata(&self, metadata: &Option<Metadata>, mut docx: Docx) -> Docx {
        // Add title and author from metadata if available
        if let Some(metadata) = metadata {
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
    pub fn initialize_numbering(&self, docx: Docx) -> Docx {
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
}

impl MarkdownNodeTraverser for Emitter {
    type Output = Docx;

    fn visit_heading(&mut self, heading: &Heading, docx: Docx) -> Docx {
        let mut text = String::new();
        for child in &heading.children {
            if let Node::Text(text_node) = child {
                text.push_str(&text_node.value);
            } else {
                warn!("Found non-text node in Heading: {:?}", child);
            }
        }
        self.add_heading(docx, &text, heading.depth)
    }

    fn visit_image(&mut self, image: &mdast::Image, docx: Docx) -> Docx {
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

    fn visit_text(&mut self, text: &mdast::Text, docx: Docx) -> Docx {
        // Process the text value to ensure proper spacing
        // First, ensure there's a space between words that were separated by newlines
        let with_spaces = text.value.replace("\n", " ");
        
        // Then normalize any multiple spaces that might have been created
        let normalized_text = with_spaces.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        // Finally check for references
        let textval = self.check_references(&normalized_text);

        // Create a run with appropriate formatting based on current state
        let mut run = Run::new().add_text(&textval);

        // Apply bold if in bold state
        if self.strong_state.set() {
            run = run.bold();
        }

        // Apply italic if in italic state
        if self.em_state.set() {
            run = run.italic();
        }

        // Add the formatted run to the current paragraph
        let paragraph = std::mem::take(&mut self.paragraph);
        self.paragraph = paragraph.add_run(run);

        docx
    }

    fn visit_strong(&mut self, strong: &mdast::Strong, mut docx: Docx) -> Docx {
        self.strong_state.push();
        for node in strong.children.iter() {
            docx = self.process_node(node, docx);
        }
        self.strong_state.pop();
        docx
    }

    fn visit_emphasis(&mut self, em: &mdast::Emphasis, mut docx: Docx) -> Docx {
        self.em_state.push();
        for node in em.children.iter() {
            docx = self.process_node(node, docx);
        }
        self.em_state.pop();
        docx
    }

    fn visit_list(&mut self, list: &mdast::List, mut docx: Docx) -> Docx {
        // Determine list type (ordered/numbered or unordered/bullet)
        if list.ordered {
            self.list_type.push(ListType::Ordered);
        } else {
            self.list_type.push(ListType::Unordered);
        }

        // Process all list items
        for child in &list.children {
            docx = self.process_node(child, docx);
        }
        // Remove list type from stack
        self.list_type.pop();
        docx
    }

    fn visit_list_item(&mut self, list_item: &mdast::ListItem, mut docx: Docx) -> Docx {
        if list_item.checked.is_some() {
            debug!("Check Boxes not yet supported");
        }
        if list_item.spread {
            debug!("Spread list items not yet supported");
        }

        if self.list_type.is_empty() {
            debug!("List item found outside of a list context");
            return docx;
        }

        let numbering_id = match self.list_type.last().unwrap() {
            ListType::Ordered => 2,   // Numbered list
            ListType::Unordered => 1, // Bullet list
        };
        let indent_level = self.list_type.len() - 1;

        // Create a paragraph with numbering properties
        self.paragraph = docx_rs::Paragraph::new().numbering(
            NumberingId::new(numbering_id),
            IndentLevel::new(indent_level),
        );

        // Process the content of the list item and add to the paragraph
        for child in &list_item.children {
            match child {
                Node::Paragraph(para) => {
                    // For paragraph nodes in list items, process their children directly
                    // This avoids creating a new paragraph
                    for para_child in &para.children {
                        // Process each child node which will add runs to self.paragraph
                        docx = self.process_node(para_child, docx);
                    }
                }
                _ => {
                    // For other types of content, process recursively
                    // This handles nested lists, code blocks, etc.
                    docx = self.process_node(child, docx);
                }
            }
        }

        // Add the list item paragraph to the document
        docx.add_paragraph(mem::take(&mut self.paragraph))
    }

    fn visit_paragraph(&mut self, para: &mdast::Paragraph, mut docx: Self::Output) -> Self::Output {
        // Initialize a new paragraph with proper first line indentation
        let paragraph = docx_rs::Paragraph::new().indent(Some(720), None, Some(720), None);
        self.paragraph = paragraph;

        // Reset paragraph alignment
        self.paragraph_alignment = None;

        // Process all children which will add runs to self.paragraph
        for child in para.children.iter() {
            docx = self.process_child(child, docx);
        }

        // Apply paragraph alignment if set
        if let Some(alignment) = self.paragraph_alignment {
            let paragraph = std::mem::take(&mut self.paragraph);
            self.paragraph = paragraph.align(alignment);
        }

        // Add the complete paragraph to the document
        docx.add_paragraph(mem::take(&mut self.paragraph))
    }

    fn visit_table(&mut self, table: &Table, mut docx: Self::Output) -> Self::Output {
        self.table.clear();
        let _table_alignment: Vec<AlignmentType> = table
            .align
            .iter()
            .map(|alig| match alig {
                mdast::AlignKind::None | mdast::AlignKind::Left => AlignmentType::Left,
                mdast::AlignKind::Right => AlignmentType::Right,
                mdast::AlignKind::Center => AlignmentType::Center,
            })
            .collect();
        for child in table.children.iter() {
            docx = self.process_child(child, docx);
        }
        docx
    }

    fn visit_table_row(
        &mut self,
        row: &markdown::mdast::TableRow,
        mut docx: Self::Output,
    ) -> Self::Output {
        self.table_cells.clear();
        for child in row.children.iter() {
            docx = self.process_child(child, docx);
        }
        let cells = mem::take(&mut self.table_cells);
        self.table.push(docx_rs::TableRow::new(cells));
        docx
    }

    fn visit_table_cell(&mut self, _cell: &mdast::TableCell, docx: Self::Output) -> Self::Output {
        // let paragraph = docx_rs::Paragraph::new();
        // let cell_paragraph = self.build_paragraph
        docx
    }
}

/// Returns the image dimensions in (EMU, EMU)
fn get_image_dimensions(file_path: &PathBuf) -> Result<(u32, u32)> {
    let reader = image::io::Reader::open(file_path)?;
    let (dim1, dim2) = reader.into_dimensions()?;
    Ok((EMUS_PER_INCH * dim1 / PPI, EMUS_PER_INCH * dim2 / PPI))
}

static REF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\{ref:\s*([^}]*)\s*}"#).unwrap());

fn extract_ref(text: &str) -> Option<&str> {
    REF_REGEX
        .captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
}