//! Markdown document parsing engine.
mod core;
mod error;
mod io;
mod lexer;

pub use core::block::{
    FBlock, FCodeBlock, FHeading, FHtmlBlock, FList, FListItem, FParagraph, FQuote, FTable,
    FTableCell, FTableRow,
};
pub use core::document::FDocument;
pub use core::inline::{FCode, FInline, FLink, FText};
pub use core::metadata::FMetadata;
pub use error::{FMdError, FMdResult};
pub use io::*;
