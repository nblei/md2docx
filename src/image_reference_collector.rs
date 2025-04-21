use log::{debug, error, info};
use serde::Deserialize;

use crate::{metadata::TableMetadata, traverser::MarkdownNodeTraverser};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct ImageModifiers {
    pub scale: f64,
    pub r#ref: Option<String>,
}

impl Default for ImageModifiers {
    fn default() -> Self {
        Self {
            scale: 1.,
            r#ref: None,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ImageReferenceCollector {
    image_count: usize,
    image_references: HashMap<String, usize>,
    table_count: usize,
    table_references: HashMap<String, usize>,
}

impl Into<HashMap<String, usize>> for ImageReferenceCollector {
    fn into(self) -> HashMap<String, usize> {
        self.image_references
    }
}

impl ImageReferenceCollector {
    pub fn get(&self, r#ref: &str) -> Option<String> {
        if let Some(n) = self.image_references.get(r#ref) {
            Some(format!("Figure {}", *n))
        } else if let Some(n) = self.table_references.get(r#ref) {
            Some(format!("Table {}", *n))
        } else {
            None
        }
    }
}

impl MarkdownNodeTraverser for ImageReferenceCollector {
    type Output = ();

    fn visit_image(
        &mut self,
        image: &markdown::mdast::Image,
        mut _result: Self::Output,
    ) -> Self::Output {
        debug!(
            "First pass - collecting image reference: url={}, alt={}",
            image.url, image.alt
        );

        // Check if the image has a reference ID in its alt text
        let res: ImageModifiers =
            serde_json::from_str(&image.alt).unwrap_or(ImageModifiers::default());

        if let Some(reference) = res.r#ref {
            self.image_count += 1;
            let figure_number = self.image_count;

            match self.image_references.get(&reference) {
                Some(_) => {
                    error!("Multiple defined reference: {}", reference);
                }
                None => {
                    info!("Adding image reference: {} -> {}", reference, figure_number);
                    self.image_references.insert(reference, figure_number);
                }
            }
        }
        ()
    }

    fn visit_table(
        &mut self,
        table: &markdown::mdast::Table,
        result: Self::Output,
    ) -> Self::Output {
        self.table_count += 1;
        for row in table.children.iter() {
            if let markdown::mdast::Node::TableRow(row) = row {
                for cell in row.children.iter() {
                    if let markdown::mdast::Node::TableCell(cell) = cell {
                        if cell.children.is_empty() {
                            continue;
                        }
                        if let markdown::mdast::Node::Text(text) = cell.children.get(0).unwrap() {
                            let metadata: Result<TableMetadata, serde_json::Error> =
                                serde_json::from_str(&text.value);
                            if let Ok(metadata) = metadata {
                                self.table_references
                                    .insert(metadata.r#ref, self.table_count);
                            }
                        }
                    } else {
                        error!("Unexpected Node Type in TableRow");
                    }
                }
            } else {
                error!("Unexpected Node Type in Table");
            }
        }
        result
    }
}
