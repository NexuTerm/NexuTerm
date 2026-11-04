//! Source-preserving media descriptors from CommonMark/GFM, independent of GTK.
use pulldown_cmark::{Alignment, Event, Options, Parser, Tag, TagEnd};
use std::ops::Range;
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub range: Range<usize>,
    pub raw: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub rows: Vec<Vec<Cell>>,
    pub align: Vec<Alignment>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Table(Table),
    Image { url: String, alt: String },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub range: Range<usize>,
    pub raw: String,
    pub kind: Kind,
}
/// Oversized blocks remain ordinary, editable source, without silent truncation.
pub fn blocks(source: &str) -> Vec<Block> {
    let mut result = Vec::new();
    let mut table: Option<(Range<usize>, Table)> = None;
    let mut image: Option<(Range<usize>, String, String)> = None;
    let mut cells = 0;
    let mut row_bounds = 0..0;
    let mut valid = true;
    for (event, range) in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .into_offset_iter()
    {
        match event {
            Event::Start(Tag::Table(align)) => {
                valid = true;
                table = Some((
                    range,
                    Table {
                        rows: vec![],
                        align,
                    },
                ))
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => {
                if let Some((_, t)) = &mut table {
                    row_bounds = range;
                    t.rows.push(vec![]);
                }
            }
            Event::Start(Tag::TableCell) => {
                if let Some((_, t)) = &mut table {
                    if let Some(row) = t.rows.last_mut() {
                        if range.start < row_bounds.start || range.end > row_bounds.end {
                            valid = false;
                        }
                        row.push(Cell {
                            raw: source[range.clone()].to_owned(),
                            range,
                        });
                    }
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some((range, t)) = table.take() {
                    let n = t.rows.iter().map(Vec::len).sum::<usize>();
                    if valid
                        && t.rows
                            .iter()
                            .flatten()
                            .all(|c| c.raw.chars().count() <= 1024)
                        && !t.rows.is_empty()
                        && n > 0
                        && t.rows.len() <= 32
                        && t.align.len() <= 12
                        && cells + n <= 256
                    {
                        cells += n;
                        result.push(Block {
                            raw: source[range.clone()].to_owned(),
                            range,
                            kind: Kind::Table(t),
                        });
                    }
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) if table.is_none() => {
                image = Some((range, dest_url.into_string(), String::new()))
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, _, alt)) = &mut image {
                    alt.push_str(&text);
                }
            }
            Event::End(TagEnd::Image) => {
                if let Some((mut range, url, alt)) = image.take() {
                    let line = source[..range.start]
                        .rfind('\n')
                        .map(|n| n + 1)
                        .unwrap_or(0);
                    let end = source[range.end..]
                        .find('\n')
                        .map(|n| range.end + n)
                        .unwrap_or(source.len());
                    // Inline images stay literal: a block widget must not conceal adjacent prose.
                    if source[line..range.start].trim().is_empty()
                        && source[range.end..end].trim().is_empty()
                    {
                        range = line..if end < source.len() { end + 1 } else { end };
                        result.push(Block {
                            raw: source[range.clone()].into(),
                            range,
                            kind: Kind::Image { url, alt },
                        });
                    }
                }
            }
            _ => {}
        }
        if result.len() >= 16 {
            break;
        }
    }
    result
}
/// Preserve the table's source cells and GFM alignment during explicit structural edits.
pub fn table_text(table: &Table, rows: usize, columns: usize) -> Result<String, String> {
    if !(1..=32).contains(&rows) || !(1..=12).contains(&columns) || rows * columns > 256 {
        return Err("Máximo 32 filas, 12 columnas y 256 celdas".into());
    }
    let mut result = String::new();
    for row in 0..rows {
        result.push('|');
        for col in 0..columns {
            result.push(' ');
            result.push_str(
                table
                    .rows
                    .get(row)
                    .and_then(|r| r.get(col))
                    .map(|c| c.raw.trim())
                    .unwrap_or(""),
            );
            result.push_str(" |");
        }
        result.push('\n');
        if row == 0 {
            result.push('|');
            for col in 0..columns {
                result.push_str(match table.align.get(col) {
                    Some(Alignment::Left) => " :--- |",
                    Some(Alignment::Center) => " :---: |",
                    Some(Alignment::Right) => " ---: |",
                    _ => " --- |",
                });
            }
            result.push('\n');
        }
    }
    Ok(result)
}
pub fn cell_text(value: &str) -> Result<String, String> {
    if value.len() > 4096 || value.chars().any(|c| c == '\n' || c == '\r' || c == '\0') {
        return Err("Celda de hasta 4096 bytes, sin saltos de línea".into());
    }
    // Input is Markdown source, so an existing escape must not be doubled.
    let mut result = String::new();
    let mut backslashes = 0;
    for c in value.chars() {
        if c == '|' && backslashes % 2 == 0 {
            result.push('\\');
        }
        result.push(c);
        if c == '\\' {
            backslashes += 1;
        } else {
            backslashes = 0;
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_cells_and_alignment() {
        let s = "Antes 👩‍💻\n\n| Nombre | Nota |\n| :--- | ---: |\n| **uno** | a\\|b |\n\n![Foto](foto.png)\n\nDespués\n";
        let b = blocks(s);
        assert_eq!(b.len(), 2);
        let Kind::Table(t) = &b[0].kind else { panic!() };
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.align, [Alignment::Left, Alignment::Right]);
        for row in &t.rows {
            for c in row {
                assert_eq!(c.raw, s[c.range.clone()]);
            }
        }
        assert!(t.rows[1][1].raw.contains("a\\|b"));
        let text = table_text(t, 3, 3).unwrap();
        assert!(text.contains("**uno**"));
        assert!(text.contains(":---"));
        assert_eq!(blocks(&text).len(), 1);
    }
    #[test]
    fn eof_padding_and_large_blocks_preserve_source() {
        let source = "Antes\n\n| Uno | Dos |\n| --- | --- |\n| a | b |";
        let found = blocks(source);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].range.end, source.len());
        assert_eq!(found[0].raw, source[found[0].range.clone()]);
        // Padded parser cells must never point into unrelated text.
        let padded = "Antes\n\n| Uno | Dos |\n| --- | --- |\n| solo |\n";
        for block in blocks(padded) {
            if let Kind::Table(t) = block.kind {
                for cell in t.rows.iter().flatten() {
                    assert!(cell.range.start >= block.range.start);
                }
            }
        }
        let large = format!(
            "| Uno | Dos |\n| --- | --- |\n| {} | b |\n",
            "x".repeat(1025)
        );
        assert!(blocks(&large).is_empty());
        assert_eq!(blocks("![Foto](foto.png)")[0].range.end, 17);
    }
    #[test]
    fn literal_code_inline_and_limits() {
        let source = "```md\n![no](x)\n|a|b|\n|-|-|\n```\ntexto ![inline](x) final\n";
        assert!(blocks(source).is_empty());
        assert!(
            table_text(
                &Table {
                    rows: vec![],
                    align: vec![]
                },
                100,
                4
            )
            .is_err()
        );
        assert_eq!(cell_text("a|b").unwrap(), "a\\|b");
        assert_eq!(cell_text("a\\|b").unwrap(), "a\\|b");
        assert!(cell_text("a\nb").is_err());
    }
}
