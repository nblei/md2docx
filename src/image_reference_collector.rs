use log::{debug, error, info};
use serde::Deserialize;

use crate::traverser::MarkdownNodeTraverser;
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
    image_figure_count: usize,
    image_references: HashMap<String, usize>,
}

impl Into<HashMap<String, usize>> for ImageReferenceCollector {
    fn into(self) -> HashMap<String, usize> {
        self.image_references
    }
}

impl ImageReferenceCollector {
    pub fn get_references(&self) -> &HashMap<String, usize> {
        &self.image_references
    }
}

impl MarkdownNodeTraverser for ImageReferenceCollector {
    type Output = ();

    // Override only the image visit method to collect references
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
            self.image_figure_count += 1;
            let figure_number = self.image_figure_count;

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
}
