use serde::Deserialize;

use crate::image_reference_collector::ImageReferenceCollector;

#[derive(Debug, Clone, Deserialize)]
pub struct TableMetadata {
    pub caption: String,
    pub r#ref: String,
}

impl TableMetadata {
    pub fn to_string(&self, imc: &ImageReferenceCollector) -> String {
        match imc.get(&self.r#ref) {
            Some(n) => format!("{}: {}", n, self.caption),
            None => format!("Table ??: {}", self.caption),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StackCounter {
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
pub enum ListType {
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
