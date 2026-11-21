//! Inline structures for text content.
#[derive(Debug, Clone)]
pub enum FInline {
    /// Plain or styled text run.
    Text(FText),
    /// Link or image.
    Link(FLink),
    /// Inline code span.
    Code(FCode),
    /// Hard line break (two trailing spaces or backslash).
    HardBreak,
}
/// A run of text with uniform formatting.
#[derive(Debug, Clone, Default)]
pub struct FText {
    /// Text content.
    pub text: String,
    /// Bold (**text** or __text__).
    pub bold: bool,
    /// Italic (*text* or _text_).
    pub italic: bool,
    /// Strikethrough (~~text~~).
    pub strike: bool,
    /// Raw source slice (preserved for round-trip fidelity).
    pub raw: Option<String>,
}
/// A link or image.
#[derive(Debug, Clone, Default)]
pub struct FLink {
    /// Display text (or alt text for images).
    pub text: String,
    /// Target URL or path.
    pub url: String,
    /// Optional title (hover tooltip).
    pub title: Option<String>,
    /// True if this is an image (![](...)).
    pub is_image: bool,
    /// Raw source slice.
    pub raw: Option<String>,
}
/// An inline code span.
#[derive(Debug, Clone, Default)]
pub struct FCode {
    /// Code content.
    pub text: String,
    /// Raw source slice.
    pub raw: Option<String>,
}
