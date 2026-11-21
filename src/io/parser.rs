//! Document parsing logic.

use crate::core::block::*;
use crate::core::document::FDocument;
use crate::core::inline::{FCode, FInline, FLink, FText};
use crate::core::metadata::FMetadata;
use crate::lexer::*;

/// Parses a complete Markdown document from source text.
pub(crate) fn parse_document(src: &str) -> crate::FMdResult<FDocument> {
    let mut doc = FDocument::default();
    doc.source = Some(src.to_string());

    let lines = split_lines(src);
    let mut cursor = 0usize;

    // Parse YAML front matter if present.
    if let Some(end) = find_front_matter(&lines) {
        doc.metadata = parse_front_matter(&lines[..end]);
        cursor = end;
    }

    // Parse blocks.
    while cursor < lines.len() {
        let line = &lines[cursor];

        if is_blank(line.text) {
            cursor += 1;
            continue;
        }

        // ATX heading.
        if let Some((level, content)) = parse_atx_heading(line.text) {
            doc.blocks.push(FBlock::Heading(FHeading {
                level,
                inlines: parse_inlines(&content),
                raw: Some(line.text.to_string()),
            }));
            cursor += 1;
            continue;
        }

        // Thematic break.
        if is_thematic_break(line.text) {
            doc.blocks.push(FBlock::ThematicBreak);
            cursor += 1;
            continue;
        }

        // Fenced code block.
        if let Some((ch, count, info)) = parse_code_fence(line.text) {
            let (block, next) = parse_code_block(&lines, cursor, ch, count, info);
            doc.blocks.push(block);
            cursor = next;
            continue;
        }

        // Blockquote.
        if parse_blockquote(line.text).is_some() {
            let (block, next) = parse_quote(&lines, cursor);
            doc.blocks.push(block);
            cursor = next;
            continue;
        }

        // List.
        if parse_list_marker(line.text).is_some() {
            let (block, next) = parse_list(&lines, cursor);
            doc.blocks.push(block);
            cursor = next;
            continue;
        }

        // Table (current line is header, next is delimiter).
        if cursor + 1 < lines.len()
            && line.text.contains('|')
            && is_table_delimiter(lines[cursor + 1].text)
        {
            let (block, next) = parse_table(&lines, cursor);
            doc.blocks.push(block);
            cursor = next;
            continue;
        }

        // HTML block.
        if line.text.trim_start().starts_with('<') {
            let (block, next) = parse_html_block(&lines, cursor);
            doc.blocks.push(block);
            cursor = next;
            continue;
        }

        // Paragraph (default).
        let (block, next) = parse_paragraph(&lines, cursor);
        doc.blocks.push(block);
        cursor = next;
    }

    doc.dirty = false;
    Ok(doc)
}

/// Finds the end of YAML front matter (the second `---` line).
fn find_front_matter(lines: &[Line]) -> Option<usize> {
    if lines.is_empty() || lines[0].text.trim() != "---" {
        return None;
    }
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.text.trim() == "---" {
            return Some(i + 1);
        }
    }
    None
}

/// Parses simple YAML front matter into key-value pairs.
fn parse_front_matter(lines: &[Line]) -> FMetadata {
    let mut meta = FMetadata::default();
    let raw: Vec<String> = lines.iter().map(|l| l.text.to_string()).collect();
    meta.raw = Some(raw.join("\n"));

    for line in lines {
        let t = line.text.trim();
        if t.is_empty() || t.starts_with('#') || t == "---" {
            continue;
        }
        if let Some(colon) = t.find(':') {
            let key = t[..colon].trim().to_string();
            let value = t[colon + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if !key.is_empty() {
                meta.values.insert(key, value);
            }
        }
    }
    meta
}

/// Parses a fenced code block starting at `start`.
fn parse_code_block(
    lines: &[Line],
    start: usize,
    ch: char,
    count: usize,
    info: String,
) -> (FBlock, usize) {
    let mut code = String::new();
    let mut cursor = start + 1;

    while cursor < lines.len() {
        if is_fence_close(lines[cursor].text, ch, count) {
            cursor += 1;
            break;
        }
        code.push_str(lines[cursor].text);
        code.push('\n');
        cursor += 1;
    }

    let language = if info.is_empty() {
        None
    } else {
        Some(info.split_whitespace().next().unwrap_or("").to_string())
    };

    (
        FBlock::CodeBlock(FCodeBlock {
            language,
            code,
            raw: None,
        }),
        cursor,
    )
}

/// Parses a blockquote, consuming consecutive `>` lines.
fn parse_quote(lines: &[Line], start: usize) -> (FBlock, usize) {
    let mut inner_lines: Vec<String> = Vec::new();
    let mut cursor = start;

    while cursor < lines.len() {
        if let Some(content) = parse_blockquote(lines[cursor].text) {
            inner_lines.push(content);
            cursor += 1;
        } else if is_blank(lines[cursor].text) && cursor + 1 < lines.len() {
            // Blank line inside quote (lazy continuation check).
            if parse_blockquote(lines[cursor + 1].text).is_some() {
                inner_lines.push(String::new());
                cursor += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Recursively parse inner content.
    let inner_src = inner_lines.join("\n");
    let inner_doc = parse_document(&inner_src).unwrap_or_default();

    (
        FBlock::Quote(FQuote {
            blocks: inner_doc.blocks,
            raw: None,
        }),
        cursor,
    )
}

/// Parses a list, handling nesting via indentation.
fn parse_list(lines: &[Line], start: usize) -> (FBlock, usize) {
    let first_marker = parse_list_marker(lines[start].text).unwrap();
    let ordered = first_marker.ordered;
    let start_num = first_marker.start;
    let base_indent = leading_spaces(lines[start].text);
    let mut items: Vec<FListItem> = Vec::new();
    let mut cursor = start;

    while cursor < lines.len() {
        let line = &lines[cursor];

        if is_blank(line.text) {
            // Check if the next line continues the list.
            if cursor + 1 < lines.len() {
                let next = &lines[cursor + 1];
                if let Some(_m) = parse_list_marker(next.text) {
                    if leading_spaces(next.text) == base_indent {
                        cursor += 1;
                        continue;
                    }
                }
            }
            break;
        }

        let indent = leading_spaces(line.text);
        if indent != base_indent {
            break;
        }

        let Some(marker) = parse_list_marker(line.text) else {
            break;
        };
        if marker.ordered != ordered {
            break;
        }

        // Collect continuation lines for this item.
        let mut content = marker.content.clone();
        let mut item_cursor = cursor + 1;
        let item_indent = base_indent + marker.width;

        while item_cursor < lines.len() {
            let l = &lines[item_cursor];
            if is_blank(l.text) {
                break;
            }
            let l_indent = leading_spaces(l.text);
            if l_indent < item_indent {
                break;
            }
            if l_indent == base_indent && parse_list_marker(l.text).is_some() {
                break; // Next sibling item.
            }
            // Continuation line — append to current item.
            content.push(' ');
            content.push_str(l.text.trim());
            item_cursor += 1;
        }

        // Check for nested list.
        let mut children: Vec<FListItem> = Vec::new();
        if item_cursor < lines.len() {
            let l = &lines[item_cursor];
            if !is_blank(l.text)
                && parse_list_marker(l.text).is_some()
                && leading_spaces(l.text) >= item_indent
            {
                let (nested, next) = parse_list(lines, item_cursor);
                if let FBlock::List(nl) = nested {
                    children = nl.items;
                }
                item_cursor = next;
            }
        }

        items.push(FListItem {
            inlines: parse_inlines(&content),
            children,
            task: marker.task,
            raw: None,
        });

        cursor = item_cursor;
    }

    (
        FBlock::List(FList {
            ordered,
            start: start_num,
            items,
            raw: None,
        }),
        cursor,
    )
}

/// Parses a GFM table starting at `start`.
fn parse_table(lines: &[Line], start: usize) -> (FBlock, usize) {
    let header_cells = split_table_row(lines[start].text);
    let delim_cells = split_table_row(lines[start + 1].text);
    let aligns: Vec<Option<String>> = delim_cells.iter().map(|c| parse_alignment(c)).collect();

    let make_row = |cells: Vec<String>| FTableRow {
        cells: cells
            .into_iter()
            .map(|c| FTableCell {
                inlines: parse_inlines(&c),
                align: None,
            })
            .collect(),
    };

    let header = make_row(header_cells);
    let mut rows = Vec::new();
    let mut cursor = start + 2;

    while cursor < lines.len() {
        let line = &lines[cursor];
        if is_blank(line.text) || !line.text.contains('|') {
            break;
        }
        rows.push(make_row(split_table_row(line.text)));
        cursor += 1;
    }

    (
        FBlock::Table(FTable {
            header,
            rows,
            aligns,
            raw: None,
        }),
        cursor,
    )
}

/// Parses an HTML block starting at `start`.
fn parse_html_block(lines: &[Line], start: usize) -> (FBlock, usize) {
    let mut html = String::new();
    let mut cursor = start;

    while cursor < lines.len() {
        let line = &lines[cursor];
        if is_blank(line.text) && cursor > start {
            break;
        }
        html.push_str(line.text);
        html.push('\n');
        cursor += 1;
    }

    (FBlock::Html(FHtmlBlock { html, raw: None }), cursor)
}

/// Parses a paragraph (consecutive non-blank, non-special lines).
fn parse_paragraph(lines: &[Line], start: usize) -> (FBlock, usize) {
    let mut text = String::new();
    let mut cursor = start;

    while cursor < lines.len() {
        let line = &lines[cursor];

        if is_blank(line.text) {
            break;
        }

        // Stop at any block-level construct (but not on the first line).
        if cursor > start {
            if parse_atx_heading(line.text).is_some()
                || is_thematic_break(line.text)
                || parse_code_fence(line.text).is_some()
                || parse_blockquote(line.text).is_some()
                || parse_list_marker(line.text).is_some()
            {
                break;
            }
        }

        if cursor > start {
            text.push(' ');
        }
        text.push_str(line.text.trim());
        cursor += 1;
    }

    (
        FBlock::Paragraph(FParagraph {
            inlines: parse_inlines(&text),
            raw: None,
        }),
        cursor,
    )
}

/// Parses inline elements within a text segment.
pub(crate) fn parse_inlines(text: &str) -> Vec<FInline> {
    let mut inlines = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    let mut buf = String::new();
    let mut bold = false;
    let mut italic = false;
    let mut strike = false;

    /// Flushes the current text buffer into the output as an FText node.
    macro_rules! flush {
        () => {
            if !buf.is_empty() {
                inlines.push(FInline::Text(FText {
                    text: std::mem::take(&mut buf),
                    bold,
                    italic,
                    strike,
                    raw: None,
                }));
            }
        };
    }

    while i < chars.len() {
        let c = chars[i];
        match c {
            // Bold (**) or italic (*).
            '*' | '_' => {
                let count = chars[i..].iter().take_while(|c| **c == chars[i]).count();
                flush!();
                if count >= 2 {
                    bold = !bold;
                    i += 2;
                } else {
                    italic = !italic;
                    i += 1;
                }
            }
            // Strikethrough (~~).
            '~' if i + 1 < chars.len() && chars[i + 1] == '~' => {
                flush!();
                strike = !strike;
                i += 2;
            }
            // Inline code (`).
            '`' => {
                flush!();
                let end = chars[i + 1..]
                    .iter()
                    .position(|c| *c == '`')
                    .map(|p| p + i + 1);
                if let Some(end) = end {
                    inlines.push(FInline::Code(FCode {
                        text: chars[i + 1..end].iter().collect(),
                        raw: None,
                    }));
                    i = end + 1;
                } else {
                    buf.push('`');
                    i += 1;
                }
            }
            // Image (![alt](url)).
            '!' if i + 1 < chars.len() && chars[i + 1] == '[' => {
                if let Some((link, next)) = parse_link(&chars, i + 1, true) {
                    flush!();
                    inlines.push(FInline::Link(link));
                    i = next;
                } else {
                    buf.push('!');
                    i += 1;
                }
            }
            // Link ([text](url)).
            '[' => {
                if let Some((link, next)) = parse_link(&chars, i, false) {
                    flush!();
                    inlines.push(FInline::Link(link));
                    i = next;
                } else {
                    buf.push('[');
                    i += 1;
                }
            }
            // Escape character (\).
            '\\' if i + 1 < chars.len() => {
                buf.push(chars[i + 1]);
                i += 2;
            }
            // Regular character.
            _ => {
                buf.push(c);
                i += 1;
            }
        }
    }

    flush!();
    inlines
}

/// Parses a link `[text](url "title")` starting at `[`.
fn parse_link(chars: &[char], start: usize, is_image: bool) -> Option<(FLink, usize)> {
    // chars[start] must be '['.
    let mut depth = 0usize;
    let mut text_end = None;

    for j in start..chars.len() {
        match chars[j] {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    text_end = Some(j);
                    break;
                }
            }
            _ => {}
        }
    }

    let text_end = text_end?;
    let mut k = text_end + 1;

    // Must be followed by '('.
    if k >= chars.len() || chars[k] != '(' {
        return None;
    }
    k += 1;

    // Parse URL.
    let url_start = k;
    while k < chars.len() && chars[k] != ')' && chars[k] != ' ' {
        k += 1;
    }
    let url: String = chars[url_start..k].iter().collect();

    // Parse optional title.
    let mut title = None;
    if k < chars.len() && chars[k] == ' ' {
        k += 1;
        let title_start = k;
        while k < chars.len() && chars[k] != ')' {
            k += 1;
        }
        let raw_title: String = chars[title_start..k].iter().collect();
        title = Some(raw_title.trim_matches('"').to_string());
    }

    // Must end with ')'.
    if k >= chars.len() || chars[k] != ')' {
        return None;
    }

    let text: String = chars[start + 1..text_end].iter().collect();

    Some((
        FLink {
            text,
            url,
            title,
            is_image,
            raw: None,
        },
        k + 1,
    ))
}
