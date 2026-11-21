//! Document rendering logic.

use crate::core::block::*;
use crate::core::document::FDocument;
use crate::core::inline::FInline;

/// Serializes a complete document to Markdown text.
pub(crate) fn render_document(doc: &FDocument) -> crate::FMdResult<String> {
    let mut out = String::new();

    // Front matter.
    if let Some(raw) = &doc.metadata.raw {
        out.push_str(raw);
        out.push_str("\n\n");
    } else if !doc.metadata.values.is_empty() {
        out.push_str("---\n");
        for (k, v) in &doc.metadata.values {
            out.push_str(&format!("{}: {}\n", k, v));
        }
        out.push_str("---\n\n");
    }

    for block in &doc.blocks {
        render_block(block, &mut out);
        out.push('\n');
    }

    Ok(out)
}

/// Renders a single block.
fn render_block(block: &FBlock, out: &mut String) {
    match block {
        FBlock::Heading(h) => {
            for _ in 0..h.level {
                out.push('#');
            }
            out.push(' ');
            render_inlines(&h.inlines, out);
            out.push('\n');
        }
        FBlock::Paragraph(p) => {
            render_inlines(&p.inlines, out);
            out.push('\n');
        }
        FBlock::CodeBlock(c) => {
            out.push_str("```");
            if let Some(lang) = &c.language {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(&c.code);
            out.push_str("```\n");
        }
        FBlock::Quote(q) => {
            for b in &q.blocks {
                let mut inner = String::new();
                render_block(b, &mut inner);
                for line in inner.lines() {
                    out.push_str("> ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        FBlock::List(l) => {
            for (idx, item) in l.items.iter().enumerate() {
                let marker = if l.ordered {
                    format!("{}.", l.start + idx as u32)
                } else {
                    "-".to_string()
                };
                out.push_str(&marker);
                out.push(' ');
                if let Some(done) = item.task {
                    out.push_str(if done { "[x] " } else { "[ ] " });
                }
                render_inlines(&item.inlines, out);
                out.push('\n');
                for child in &item.children {
                    render_nested_item(child, l.ordered, 1, out);
                }
            }
        }
        FBlock::Table(t) => {
            render_table_row(&t.header, out);
            out.push('|');
            for align in &t.aligns {
                match align.as_deref() {
                    Some("left") => out.push_str(" :--- |"),
                    Some("center") => out.push_str(" :---: |"),
                    Some("right") => out.push_str(" ---: |"),
                    _ => out.push_str(" --- |"),
                }
            }
            out.push('\n');
            for row in &t.rows {
                render_table_row(row, out);
            }
        }
        FBlock::ThematicBreak => {
            out.push_str("---\n");
        }
        FBlock::Html(h) => {
            out.push_str(&h.html);
        }
    }
}

/// Renders a nested list item with indentation.
fn render_nested_item(item: &FListItem, ordered: bool, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    let marker = if ordered { "1." } else { "-" };
    out.push_str(&indent);
    out.push_str(marker);
    out.push(' ');
    if let Some(done) = item.task {
        out.push_str(if done { "[x] " } else { "[ ] " });
    }
    render_inlines(&item.inlines, out);
    out.push('\n');
    for child in &item.children {
        render_nested_item(child, ordered, depth + 1, out);
    }
}

/// Renders a table row.
fn render_table_row(row: &FTableRow, out: &mut String) {
    out.push('|');
    for cell in &row.cells {
        out.push(' ');
        render_inlines(&cell.inlines, out);
        out.push_str(" |");
    }
    out.push('\n');
}

/// Renders inline elements.
fn render_inlines(inlines: &[FInline], out: &mut String) {
    for inline in inlines {
        match inline {
            FInline::Text(t) => {
                if t.bold {
                    out.push_str("**");
                }
                if t.italic {
                    out.push('*');
                }
                if t.strike {
                    out.push_str("~~");
                }
                out.push_str(&t.text);
                if t.strike {
                    out.push_str("~~");
                }
                if t.italic {
                    out.push('*');
                }
                if t.bold {
                    out.push_str("**");
                }
            }
            FInline::Link(l) => {
                if l.is_image {
                    out.push('!');
                }
                out.push_str(&format!("[{}]({}", l.text, l.url));
                if let Some(title) = &l.title {
                    out.push_str(&format!(" \"{}\"", title));
                }
                out.push(')');
            }
            FInline::Code(c) => {
                out.push_str(&format!("`{}`", c.text));
            }
            FInline::HardBreak => {
                out.push_str("  \n");
            }
        }
    }
}
