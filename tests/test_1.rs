#[cfg(test)]
mod tests {
    use f_md::*;
    use std::io::Read;
    const windows_output_url: &str = "C:\\Users\\admin\\Downloads";
    const macos_output_url: &str = "/Users/max/Downloads/tests";
    /// Output path for demo Markdown file.
    fn demo_path() -> std::path::PathBuf {
        #[cfg(target_os = "windows")]
        {
            std::path::PathBuf::from(windows_output_url).join("demo.md")
        }
        #[cfg(target_os = "macos")]
        {
            std::path::PathBuf::from(macos_output_url).join("demo.md")
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            std::env::temp_dir().join("demo.md")
        }
    }
    /// Builds a text run with the given text.
    fn make_text(text: &str) -> FText {
        FText {
            text: text.to_string(),
            ..Default::default()
        }
    }
    /// Builds a styled text run.
    fn make_styled_text(text: &str, bold: bool, italic: bool, strike: bool) -> FText {
        FText {
            text: text.to_string(),
            bold,
            italic,
            strike,
            ..Default::default()
        }
    }
    /// Builds a link.
    fn make_link(text: &str, url: &str) -> FLink {
        FLink {
            text: text.to_string(),
            url: url.to_string(),
            title: None,
            is_image: false,
            ..Default::default()
        }
    }
    /// Builds an image.
    fn make_image(alt: &str, url: &str) -> FLink {
        FLink {
            text: alt.to_string(),
            url: url.to_string(),
            title: None,
            is_image: true,
            ..Default::default()
        }
    }
    /// Builds an inline code.
    fn make_code(text: &str) -> FCode {
        FCode {
            text: text.to_string(),
            ..Default::default()
        }
    }
    /// Builds a heading block.
    fn make_heading(level: u8, text: &str) -> FBlock {
        FBlock::Heading(FHeading {
            level,
            inlines: vec![FInline::Text(make_text(text))],
            ..Default::default()
        })
    }
    /// Builds a paragraph block from inline elements.
    fn make_paragraph(inlines: Vec<FInline>) -> FBlock {
        FBlock::Paragraph(FParagraph {
            inlines,
            ..Default::default()
        })
    }
    /// Builds a simple paragraph with plain text.
    fn make_simple_paragraph(text: &str) -> FBlock {
        make_paragraph(vec![FInline::Text(make_text(text))])
    }
    /// Builds a code block.
    fn make_code_block(language: &str, code: &str) -> FBlock {
        FBlock::CodeBlock(FCodeBlock {
            language: Some(language.to_string()),
            code: code.to_string(),
            ..Default::default()
        })
    }
    /// Builds a blockquote.
    fn make_quote(blocks: Vec<FBlock>) -> FBlock {
        FBlock::Quote(FQuote {
            blocks,
            ..Default::default()
        })
    }
    /// Builds a list item.
    fn make_list_item(text: &str, children: Vec<FListItem>) -> FListItem {
        FListItem {
            inlines: vec![FInline::Text(make_text(text))],
            children,
            ..Default::default()
        }
    }
    /// Builds a task list item.
    fn make_task_item(text: &str, done: bool) -> FListItem {
        FListItem {
            inlines: vec![FInline::Text(make_text(text))],
            task: Some(done),
            ..Default::default()
        }
    }
    /// Builds an unordered list.
    fn make_unordered_list(items: Vec<FListItem>) -> FBlock {
        FBlock::List(FList {
            ordered: false,
            start: 1,
            items,
            ..Default::default()
        })
    }
    /// Builds an ordered list.
    fn make_ordered_list(start: u32, items: Vec<FListItem>) -> FBlock {
        FBlock::List(FList {
            ordered: true,
            start,
            items,
            ..Default::default()
        })
    }
    /// Builds a table cell.
    fn make_cell(text: &str) -> FTableCell {
        FTableCell {
            inlines: vec![FInline::Text(make_text(text))],
            ..Default::default()
        }
    }
    /// Builds a table.
    fn make_table(header: Vec<&str>, rows: Vec<Vec<&str>>, aligns: Vec<Option<String>>) -> FBlock {
        FBlock::Table(FTable {
            header: FTableRow {
                cells: header.iter().map(|h| make_cell(h)).collect(),
            },
            rows: rows
                .iter()
                .map(|r| FTableRow {
                    cells: r.iter().map(|c| make_cell(c)).collect(),
                })
                .collect(),
            aligns,
            ..Default::default()
        })
    }
    /// Creates the front matter metadata for the document.
    fn make_metadata() -> FMetadata {
        let mut meta = FMetadata::default();
        meta.set("title", "f_md Demo Document");
        meta.set("author", "happyboy");
        meta.set("date", "2026-11-21");
        meta.set("tags", "markdown, rust, demo");
        meta
    }
    /// Writes a comprehensive Markdown demo covering all features.
    #[test]
    fn write_demo_md() {
        let mut doc = FDocument::new_empty();
        doc.metadata = make_metadata();
        // Section 1: Headings.
        doc.blocks.push(make_heading(1, "f_md Demo Document"));
        doc.blocks.push(make_simple_paragraph(
            "This document demonstrates all features of the f_md parsing engine.",
        ));
        doc.blocks.push(FBlock::ThematicBreak);
        // Section 2: Text formatting.
        doc.blocks.push(make_heading(2, "Text Formatting"));
        doc.blocks.push(make_paragraph(vec![
            FInline::Text(make_styled_text("Bold", true, false, false)),
            FInline::Text(make_text(", ")),
            FInline::Text(make_styled_text("Italic", false, true, false)),
            FInline::Text(make_text(", ")),
            FInline::Text(make_styled_text("Strikethrough", false, false, true)),
            FInline::Text(make_text(", ")),
            FInline::Code(make_code("inline_code")),
            FInline::Text(make_text(", and ")),
            FInline::Link(make_link("a link", "https://github.com/0xhappyboy/f_md")),
            FInline::Text(make_text(".")),
        ]));
        doc.blocks.push(make_paragraph(vec![
            FInline::Text(make_text("Here is an image: ")),
            FInline::Link(make_image(
                "Rust Logo",
                "https://www.rust-lang.org/static/images/rust-logo-blk.svg",
            )),
        ]));
        // Section 3: Code block.
        doc.blocks.push(make_heading(2, "Code Block"));
        doc.blocks.push(make_code_block(
            "rust",
            "fn main() {\n    println!(\"Hello, f_md!\");\n    let doc = FDocument::with_str(\"# Hi\").unwrap();\n    println!(\"{}\", doc.plain_text());\n}\n",
        ));
        // Section 4: Lists.
        doc.blocks.push(make_heading(2, "Lists"));
        doc.blocks.push(make_unordered_list(vec![
            make_list_item("First item", vec![]),
            make_list_item(
                "Second item",
                vec![
                    make_list_item("Nested 2.1", vec![]),
                    make_list_item("Nested 2.2", vec![]),
                ],
            ),
            make_list_item("Third item", vec![]),
        ]));
        doc.blocks.push(make_ordered_list(
            1,
            vec![
                make_list_item("Step one", vec![]),
                make_list_item("Step two", vec![]),
                make_list_item("Step three", vec![]),
            ],
        ));
        doc.blocks.push(make_unordered_list(vec![
            make_task_item("Completed task", true),
            make_task_item("Pending task", false),
        ]));
        // Section 5: Blockquote.
        doc.blocks.push(make_heading(2, "Blockquote"));
        doc.blocks.push(make_quote(vec![
            make_simple_paragraph("The quick brown fox jumps over the lazy dog."),
            make_simple_paragraph("— A famous pangram"),
        ]));
        // Section 6: Table.
        doc.blocks.push(make_heading(2, "Table"));
        doc.blocks
            .push(make_simple_paragraph("A GFM table with alignment:"));
        doc.blocks.push(make_table(
            vec!["Feature", "Status", "Priority"],
            vec![
                vec!["Heading", "Done", "High"],
                vec!["Paragraph", "Done", "High"],
                vec!["Code Block", "Done", "Medium"],
                vec!["Table", "Done", "Medium"],
                vec!["Image", "Done", "Low"],
            ],
            vec![
                Some("left".to_string()),
                Some("center".to_string()),
                Some("right".to_string()),
            ],
        ));
        // Section 7: HTML.
        doc.blocks.push(make_heading(2, "Raw HTML"));
        doc.blocks.push(FBlock::Html(FHtmlBlock {
            html: "<div class=\"note\">\n  <p>This is a raw HTML block.</p>\n</div>\n".to_string(),
            ..Default::default()
        }));
        // Serialize and write.
        let output = doc.to_string().expect("failed to serialize document");
        println!("[writer] serialized document: {} bytes", output.len());
        std::fs::write(demo_path(), &output).expect("failed to write demo.md");
        println!("demo.md written to {}", demo_path().display());
    }
    /// Reads the demo Markdown and prints its contents.
    #[test]
    fn read_demo_md() {
        let doc = FDocument::with_path(demo_path()).expect("failed to read demo.md");
        println!("=== Document Info ===");
        println!("Block Count: {}", doc.block_count());
        println!();
        println!("=== Metadata ===");
        for (key, value) in &doc.metadata.values {
            println!("  {}: {}", key, value);
        }
        println!();
        println!("=== Blocks ===");
        for (idx, block) in doc.blocks.iter().enumerate() {
            match block {
                FBlock::Heading(h) => {
                    let text: String = h
                        .inlines
                        .iter()
                        .map(|i| match i {
                            FInline::Text(t) => t.text.clone(),
                            _ => String::new(),
                        })
                        .collect();
                    println!("[{}] Heading H{}: '{}'", idx, h.level, text);
                }
                FBlock::Paragraph(p) => {
                    let text: String = p
                        .inlines
                        .iter()
                        .map(|i| match i {
                            FInline::Text(t) => t.text.clone(),
                            FInline::Link(l) => {
                                if l.is_image {
                                    format!("![{}]({})", l.text, l.url)
                                } else {
                                    format!("[{}]({})", l.text, l.url)
                                }
                            }
                            FInline::Code(c) => format!("`{}`", c.text),
                            FInline::HardBreak => "\\n".to_string(),
                        })
                        .collect();
                    println!(
                        "[{}] Paragraph: '{}'",
                        idx,
                        text.chars().take(60).collect::<String>()
                    );
                }
                FBlock::CodeBlock(c) => {
                    println!(
                        "[{}] CodeBlock ({}): {} lines",
                        idx,
                        c.language.as_deref().unwrap_or("none"),
                        c.code.lines().count()
                    );
                }
                FBlock::Quote(q) => {
                    println!("[{}] Quote: {} nested blocks", idx, q.blocks.len());
                    for b in &q.blocks {
                        if let FBlock::Paragraph(p) = b {
                            let text: String = p
                                .inlines
                                .iter()
                                .filter_map(|i| match i {
                                    FInline::Text(t) => Some(t.text.clone()),
                                    _ => None,
                                })
                                .collect();
                            println!("    > {}", text);
                        }
                    }
                }
                FBlock::List(l) => {
                    let list_type = if l.ordered { "Ordered" } else { "Unordered" };
                    println!("[{}] List ({}): {} items", idx, list_type, l.items.len());
                    for (i, item) in l.items.iter().enumerate() {
                        let text: String = item
                            .inlines
                            .iter()
                            .filter_map(|inl| match inl {
                                FInline::Text(t) => Some(t.text.clone()),
                                _ => None,
                            })
                            .collect();
                        let task = match item.task {
                            Some(true) => "[x] ",
                            Some(false) => "[ ] ",
                            None => "",
                        };
                        println!("    {}. {}{}", i + 1, task, text);
                        for child in &item.children {
                            let child_text: String = child
                                .inlines
                                .iter()
                                .filter_map(|inl| match inl {
                                    FInline::Text(t) => Some(t.text.clone()),
                                    _ => None,
                                })
                                .collect();
                            println!("      - {}", child_text);
                        }
                    }
                }
                FBlock::Table(t) => {
                    println!(
                        "[{}] Table: {} rows x {} cols",
                        idx,
                        t.rows.len() + 1,
                        t.header.cells.len()
                    );
                    let header_text: Vec<String> = t
                        .header
                        .cells
                        .iter()
                        .map(|c| {
                            c.inlines
                                .iter()
                                .filter_map(|i| match i {
                                    FInline::Text(t) => Some(t.text.clone()),
                                    _ => None,
                                })
                                .collect()
                        })
                        .collect();
                    println!("    Header: {:?}", header_text);
                    for (ri, row) in t.rows.iter().enumerate() {
                        let cells: Vec<String> = row
                            .cells
                            .iter()
                            .map(|c| {
                                c.inlines
                                    .iter()
                                    .filter_map(|i| match i {
                                        FInline::Text(t) => Some(t.text.clone()),
                                        _ => None,
                                    })
                                    .collect()
                            })
                            .collect();
                        println!("    Row {}: {:?}", ri + 1, cells);
                    }
                }
                FBlock::ThematicBreak => {
                    println!("[{}] ThematicBreak", idx);
                }
                FBlock::Html(h) => {
                    println!("[{}] HtmlBlock: {} bytes", idx, h.html.len());
                }
            }
        }
        println!();
        println!("=== Plain Text ===");
        println!("{}", doc.plain_text());
    }
}
