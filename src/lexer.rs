//! Minimal Markdown lexical helpers.
#[derive(Debug, Clone)]
pub(crate) struct Line<'a> {
    /// Line number in the original source (0-based).
    pub number: usize,
    /// Raw text without trailing newline.
    pub text: &'a str,
}

/// Splits source text into lines, preserving line numbers.
pub(crate) fn split_lines(src: &str) -> Vec<Line<'_>> {
    src.lines()
        .enumerate()
        .map(|(i, text)| Line { number: i, text })
        .collect()
}

/// Returns the number of leading spaces.
pub(crate) fn leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

/// Checks if a line is blank (empty or only whitespace).
pub(crate) fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Extracts ATX heading level and content.
pub(crate) fn parse_atx_heading(s: &str) -> Option<(u8, String)> {
    let trimmed = s.trim_start();
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &trimmed[level..];
    // Must be followed by a space or end of line.
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let mut content = rest.trim().to_string();
    // Strip closing hashes: "## text ##" → "text".
    if content.ends_with('#') {
        content = content.trim_end_matches('#').trim_end().to_string();
    }
    Some((level as u8, content))
}

/// Checks if a line is a thematic break (`---`, `***`, `___` with optional spaces).
pub(crate) fn is_thematic_break(s: &str) -> bool {
    let t = s.trim();
    if t.len() < 3 {
        return false;
    }
    let chars: Vec<char> = t.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.len() < 3 {
        return false;
    }
    let first = chars[0];
    matches!(first, '-' | '*' | '_') && chars.iter().all(|c| *c == first)
}

/// Extracts fence info for ``` or ~~~ fenced code blocks.
pub(crate) fn parse_code_fence(s: &str) -> Option<(char, usize, String)> {
    let t = s.trim_start();
    let ch = t.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let count = t.chars().take_while(|c| *c == ch).count();
    if count < 3 {
        return None;
    }
    let info = t[count..].trim().to_string();
    Some((ch, count, info))
}

/// Checks if a line closes a fence opened with (ch, count).
pub(crate) fn is_fence_close(s: &str, ch: char, count: usize) -> bool {
    let t = s.trim();
    let close_count = t.chars().take_while(|c| *c == ch).count();
    close_count >= count && t[close_count..].trim().is_empty()
}

/// Extracts blockquote content (strips leading "> " or ">").
pub(crate) fn parse_blockquote(s: &str) -> Option<String> {
    let t = s.trim_start();
    if !t.starts_with('>') {
        return None;
    }
    let rest = &t[1..];
    if rest.starts_with(' ') {
        Some(rest[1..].to_string())
    } else {
        Some(rest.to_string())
    }
}

/// Information extracted from a list item marker.
pub(crate) struct ListMarker {
    /// Ordered (1.) or unordered (-, *, +).
    pub ordered: bool,
    /// Start number for ordered lists.
    pub start: u32,
    /// Width of the marker plus following space (used for indentation calculation).
    pub width: usize,
    /// Content after the marker.
    pub content: String,
    /// Task list checkbox state: Some(true) = [x], Some(false) = [ ].
    pub task: Option<bool>,
}

pub(crate) fn parse_list_marker(s: &str) -> Option<ListMarker> {
    let indent = leading_spaces(s);
    if indent >= 4 {
        return None; // 4+ spaces = indented code block territory.
    }
    let t = &s[indent..];

    // Unordered: - * +
    if let Some(first) = t.chars().next() {
        if first == '-' || first == '*' || first == '+' {
            let rest = &t[1..];
            if rest.starts_with(' ') || rest.is_empty() {
                let content = if rest.is_empty() {
                    String::new()
                } else {
                    rest[1..].to_string()
                };
                let task = parse_task_marker(&content);
                let content = if task.is_some() {
                    // Strip "[x] " or "[ ] " prefix.
                    content[4..].to_string()
                } else {
                    content
                };
                return Some(ListMarker {
                    ordered: false,
                    start: 1,
                    width: indent + 2,
                    content,
                    task,
                });
            }
        }
    }

    // Ordered: N. or N)
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        let after_digits = &t[digits.len()..];
        if after_digits.starts_with(". ") || after_digits.starts_with(") ") {
            let start: u32 = digits.parse().unwrap_or(1);
            let content = after_digits[2..].to_string();
            let width = indent + digits.len() + 2;
            let task = parse_task_marker(&content);
            let content = if task.is_some() {
                content[4..].to_string()
            } else {
                content
            };
            return Some(ListMarker {
                ordered: true,
                start,
                width,
                content,
                task,
            });
        }
    }

    None
}

fn parse_task_marker(s: &str) -> Option<bool> {
    if s.len() >= 3 && s.starts_with("[ ]") {
        Some(false)
    } else if s.len() >= 3 && (s.starts_with("[x]") || s.starts_with("[X]")) {
        Some(true)
    } else {
        None
    }
}

pub(crate) fn split_table_row(s: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = s.chars().peekable();
    let mut in_code = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if chars.peek() == Some(&'|') {
                    current.push('|');
                    chars.next();
                } else {
                    current.push('\\');
                }
            }
            '`' => {
                in_code = !in_code;
                current.push('`');
            }
            '|' if !in_code => {
                cells.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() || !cells.is_empty() {
        cells.push(current.trim().to_string());
    }
    // Remove leading empty cell from "| a | b |" form.
    if cells.first().map(|c| c.is_empty()).unwrap_or(false) {
        cells.remove(0);
    }
    cells
}

pub(crate) fn is_table_delimiter(s: &str) -> bool {
    let cells = split_table_row(s);
    !cells.is_empty()
        && cells.iter().all(|c| {
            let t = c.trim();
            !t.is_empty() && t.chars().all(|c| matches!(c, '-' | ':' | ' ')) && t.contains('-')
        })
}

pub(crate) fn parse_alignment(cell: &str) -> Option<String> {
    let t = cell.trim();
    if !t.contains('-') {
        return None;
    }
    let left = t.starts_with(':');
    let right = t.ends_with(':');
    match (left, right) {
        (true, true) => Some("center".to_string()),
        (true, false) => Some("left".to_string()),
        (false, true) => Some("right".to_string()),
        _ => None,
    }
}
