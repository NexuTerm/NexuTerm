//! Source ranges only: rendering never serializes or rewrites the document.
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::ops::Range;
#[derive(Clone, Debug, PartialEq)]
pub enum Style {
    Heading(u8),
    Bold,
    Italic,
    Strike,
    Code,
    Quote,
    Link,
    Hidden,
}
#[derive(Clone, Debug)]
pub struct Span {
    pub range: Range<usize>,
    pub style: Style,
}
pub fn supported(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|v| v.to_str())
        .is_some_and(|v| v.eq_ignore_ascii_case("md") || v.eq_ignore_ascii_case("markdown"))
}
pub fn spans(source: &str) -> Vec<Span> {
    let mut result = Vec::new();
    fn add(result: &mut Vec<Span>, range: Range<usize>, style: Style) {
        if !range.is_empty() {
            result.push(Span { range, style });
        }
    }
    for (event, range) in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .into_offset_iter()
    {
        let raw = &source[range.clone()];
        match event {
            Event::Start(tag) => {
                let (style, marker) = match tag {
                    Tag::Heading { level, .. } => (Style::Heading(level as u8), 0),
                    Tag::Strong => (Style::Bold, 2),
                    Tag::Emphasis => (Style::Italic, 1),
                    Tag::Strikethrough => (Style::Strike, 2),
                    Tag::CodeBlock(_) => (Style::Code, 0),
                    Tag::BlockQuote(_) => (Style::Quote, 0),
                    Tag::Link { .. } => (Style::Link, 0),
                    _ => continue,
                };
                if matches!(style, Style::Heading(_)) {
                    let count = raw.bytes().take_while(|b| *b == b'#').count();
                    if count > 0
                        && raw
                            .as_bytes()
                            .get(count)
                            .is_some_and(u8::is_ascii_whitespace)
                    {
                        let mut end = count;
                        while raw
                            .as_bytes()
                            .get(end)
                            .is_some_and(|b| *b == b' ' || *b == b'\t')
                        {
                            end += 1;
                        }
                        add(&mut result, range.start..range.start + end, Style::Hidden);
                    }
                }
                if matches!(style, Style::Code)
                    && (raw.starts_with("```") || raw.starts_with("~~~"))
                {
                    if let Some(first) = raw.find('\n') {
                        add(&mut result, range.start..range.start + first, Style::Hidden);
                        let trimmed = raw.trim_end_matches(['\n', '\r']);
                        if let Some(last) = trimmed.rfind('\n') {
                            let closing = &trimmed[last + 1..];
                            if closing.trim().len() >= 3
                                && closing.trim().bytes().all(|b| b == raw.as_bytes()[0])
                            {
                                add(
                                    &mut result,
                                    range.start + last + 1..range.start + trimmed.len(),
                                    Style::Hidden,
                                );
                            }
                        }
                    }
                }
                if matches!(style, Style::Quote) {
                    let mut offset = range.start;
                    for line in raw.split_inclusive('\n') {
                        if line.starts_with("> ") {
                            add(&mut result, offset..offset + 2, Style::Hidden);
                        }
                        offset += line.len();
                    }
                }
                if marker > 0 && raw.len() >= marker * 2 {
                    add(
                        &mut result,
                        range.start..range.start + marker,
                        Style::Hidden,
                    );
                    add(&mut result, range.end - marker..range.end, Style::Hidden);
                }
                add(&mut result, range, style);
            }
            Event::Code(_) => {
                let count = raw.bytes().take_while(|b| *b == b'`').count();
                add(&mut result, range.clone(), Style::Code);
                if count > 0 && raw.len() >= count * 2 {
                    add(&mut result, range.start..range.start + count, Style::Hidden);
                    add(&mut result, range.end - count..range.end, Style::Hidden);
                }
            }
            _ => {}
        }
        if result.len() > 65536 {
            return Vec::new();
        }
    }
    result
}
/// Linear offset map, including the final boundary. GTK counts Unicode scalars.
pub fn offsets(source: &str) -> Vec<i32> {
    let mut map = vec![0; source.len() + 1];
    for (n, (start, c)) in source.char_indices().enumerate() {
        for v in &mut map[start..start + c.len_utf8()] {
            *v = n as i32;
        }
        map[start + c.len_utf8()] = n as i32 + 1;
    }
    map
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_unicode_and_literal_code() {
        let source = "# Título 👩\u{200d}💻\n\n**hola *mundo*** y `**literal**`\n\n```rust\n**no formato**\n```\n";
        let parsed = spans(source);
        assert!(parsed.iter().any(|v| v.style == Style::Heading(1)));
        let bold: Vec<_> = parsed.iter().filter(|v| v.style == Style::Bold).collect();
        assert_eq!(bold.len(), 1);
        assert!(source[bold[0].range.clone()].contains("mundo"));
        let map = offsets(source);
        for (i, _) in source.char_indices() {
            assert_eq!(map[i], source[..i].chars().count() as i32);
        }
        assert_eq!(*map.last().unwrap(), source.chars().count() as i32);
        assert!(supported(std::path::Path::new("README.MD")));
    }
    #[test]
    fn escaped_incomplete_and_html_remain_source() {
        let source = "\\*literal\\* **abierto\n<script>alert(1)</script>\n";
        assert!(!spans(source).iter().any(|v| v.style == Style::Hidden));
    }
}
