//! Block-level structures for Markdown documents.
use super::inline::FInline;
/// A block-level element in the document.
#[derive(Debug, Clone)]
pub enum FBlock {
    /// Heading (# ~ ######).
    Heading(FHeading),
    /// Paragraph of text.
    Paragraph(FParagraph),
    /// Fenced code block (``` or ~~~).
    CodeBlock(FCodeBlock),
    /// Blockquote (>).
    Quote(FQuote),
    /// List (ordered or unordered).
    List(FList),
    /// Table (GFM extension).
    Table(FTable),
    /// Thematic break (---, ***, ___).
    ThematicBreak,
    /// Raw HTML block.
    Html(FHtmlBlock),
}
/// Heading with level 1-6.
#[derive(Debug, Clone, Default)]
pub struct FHeading {
    /// Heading level (1-6).
    pub level: u8,
    /// Inline content.
    pub inlines: Vec<FInline>,
    /// Raw source line.
    pub raw: Option<String>,
}
/// Paragraph.
#[derive(Debug, Clone, Default)]
pub struct FParagraph {
    /// Inline content.
    pub inlines: Vec<FInline>,
    /// Raw source lines.
    pub raw: Option<String>,
}
/// Fenced code block.
#[derive(Debug, Clone, Default)]
pub struct FCodeBlock {
    /// Language identifier (e.g. "rust").
    pub language: Option<String>,
    /// Code content (lines between fences, each ending with \n).
    pub code: String,
    /// Raw source lines.
    pub raw: Option<String>,
}
/// Blockquote containing nested blocks.
#[derive(Debug, Clone, Default)]
pub struct FQuote {
    /// Nested blocks inside the quote.
    pub blocks: Vec<FBlock>,
    /// Raw source lines.
    pub raw: Option<String>,
}
/// A single list item.
#[derive(Debug, Clone, Default)]
pub struct FListItem {
    /// Inline content of the item.
    pub inlines: Vec<FInline>,
    /// Nested sub-items.
    pub children: Vec<FListItem>,
    /// Task list checkbox state: Some(true) = checked, Some(false) = unchecked.
    pub task: Option<bool>,
    /// Raw source line.
    pub raw: Option<String>,
}
/// List (ordered or unordered).
#[derive(Debug, Clone, Default)]
pub struct FList {
    /// Ordered (1.) or unordered (-, *, +).
    pub ordered: bool,
    /// Start number for ordered lists.
    pub start: u32,
    /// Top-level items.
    pub items: Vec<FListItem>,
    /// Raw source lines.
    pub raw: Option<String>,
}
/// Table cell.
#[derive(Debug, Clone, Default)]
pub struct FTableCell {
    /// Inline content.
    pub inlines: Vec<FInline>,
    /// Alignment hint (left, center, right).
    pub align: Option<String>,
}
/// Table row.
#[derive(Debug, Clone, Default)]
pub struct FTableRow {
    /// Cells in the row.
    pub cells: Vec<FTableCell>,
}
/// GFM table.
#[derive(Debug, Clone, Default)]
pub struct FTable {
    /// Header row.
    pub header: FTableRow,
    /// Body rows.
    pub rows: Vec<FTableRow>,
    /// Column alignments parsed from the delimiter row.
    pub aligns: Vec<Option<String>>,
    /// Raw source lines.
    pub raw: Option<String>,
}
/// Raw HTML block.
#[derive(Debug, Clone, Default)]
pub struct FHtmlBlock {
    /// HTML content.
    pub html: String,
    /// Raw source lines.
    pub raw: Option<String>,
}
