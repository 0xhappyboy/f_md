//! Root document structure.
use super::block::FBlock;
use super::metadata::FMetadata;
/// Root of the Markdown document data model.
#[derive(Debug, Clone, Default)]
pub struct FDocument {
    /// Ordered list of top-level blocks.
    pub blocks: Vec<FBlock>,
    /// Front matter metadata.
    pub metadata: FMetadata,
    /// Original source text (preserved for round-trip fidelity).
    pub source: Option<String>,
    /// Whether the document has been modified since it was loaded.
    pub dirty: bool,
}
impl FDocument {
    /// Creates an empty in-memory document.
    pub fn new_empty() -> Self {
        FDocument::default()
    }
    /// Loads a Markdown document from a file on disk.
    pub fn with_path<P: AsRef<std::path::Path>>(path: P) -> crate::FMdResult<Self> {
        let data = std::fs::read_to_string(path)?;
        Self::with_str(&data)
    }
    /// Parses a Markdown document from a string.
    pub fn with_str(data: &str) -> crate::FMdResult<Self> {
        crate::io::parser::parse_document(data)
    }
    /// Serializes the document back to Markdown text.
    pub fn to_string(&self) -> crate::FMdResult<String> {
        crate::io::renderer::render_document(self)
    }
    /// Marks the document as modified.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    /// Clears the modified flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    /// Returns the number of top-level blocks.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
    /// Returns plain text content of all blocks.
    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for block in &self.blocks {
            collect_block_text(block, &mut text);
        }
        text
    }
}
/// Recursively collects plain text from a block.
fn collect_block_text(block: &FBlock, out: &mut String) {
    match block {
        FBlock::Heading(h) => {
            for inline in &h.inlines {
                collect_inline_text(inline, out);
            }
            out.push('\n');
        }
        FBlock::Paragraph(p) => {
            for inline in &p.inlines {
                collect_inline_text(inline, out);
            }
            out.push('\n');
        }
        FBlock::CodeBlock(c) => {
            out.push_str(&c.code);
            out.push('\n');
        }
        FBlock::Quote(q) => {
            for b in &q.blocks {
                collect_block_text(b, out);
            }
        }
        FBlock::List(l) => {
            for item in &l.items {
                collect_item_text(item, out);
            }
        }
        FBlock::Table(t) => {
            for row in std::iter::once(&t.header).chain(t.rows.iter()) {
                for cell in &row.cells {
                    for inline in &cell.inlines {
                        collect_inline_text(inline, out);
                    }
                    out.push('\t');
                }
                out.push('\n');
            }
        }
        _ => {}
    }
}
/// Recursively collects plain text from a list item and its children.
fn collect_item_text(item: &super::block::FListItem, out: &mut String) {
    for inline in &item.inlines {
        collect_inline_text(inline, out);
    }
    out.push('\n');
    for child in &item.children {
        collect_item_text(child, out);
    }
}
/// Collects plain text from an inline element.
fn collect_inline_text(inline: &super::inline::FInline, out: &mut String) {
    match inline {
        super::inline::FInline::Text(t) => out.push_str(&t.text),
        super::inline::FInline::Link(l) => out.push_str(&l.text),
        super::inline::FInline::Code(c) => out.push_str(&c.text),
        super::inline::FInline::HardBreak => out.push('\n'),
    }
}
