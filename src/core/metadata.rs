//! Front matter metadata.
use std::collections::HashMap;
/// Document metadata parsed from front matter.
#[derive(Debug, Clone, Default)]
pub struct FMetadata {
    /// Key-value pairs from YAML front matter.
    pub values: HashMap<String, String>,
    /// Raw front matter text (including delimiters).
    pub raw: Option<String>,
}
impl FMetadata {
    /// Gets a metadata value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
    /// Sets a metadata value.
    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }
    /// Returns the number of metadata entries.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Returns true if there are no metadata entries.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
