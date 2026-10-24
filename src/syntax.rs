//! Syntax colouring runs off the GTK thread, with one bounded job per document.
use gtk::{glib, prelude::*};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::LazyLock,
    time::Duration,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
const HIGHLIGHT_LIMIT: usize = 1024 * 1024;
const SPAN_LIMIT: usize = 65_536;
type Colour = (u8, u8, u8, u8);
#[derive(Debug)]
struct Span {
    start: i32,
    end: i32,
    colour: Colour,
}
fn highlight(path: &std::path::Path, contents: &str, dark: bool) -> Result<Vec<Span>, String> {
    let syntaxes = &*SYNTAXES;
    let syntax = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(|e| syntaxes.find_syntax_by_extension(e))
        .or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| syntaxes.find_syntax_by_extension(n))
        })
        .or_else(|| {
            contents
                .lines()
                .next()
                .and_then(|line| syntaxes.find_syntax_by_first_line(line))
        })
        .unwrap_or_else(|| syntaxes.find_syntax_plain_text());
    if syntax.name == "Plain Text" {
        return Ok(Vec::new());
    }
    let theme = &THEMES.themes[if dark {
        "base16-ocean.dark"
    } else {
        "InspiredGitHub"
    }];
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut spans: Vec<Span> = Vec::new();
    let mut offset = 0;
    let mut bytes = 0;
    for line in LinesWithEndings::from(contents) {
        bytes += line.len();
        if bytes > HIGHLIGHT_LIMIT {
            break;
        }
        for (style, token) in highlighter
            .highlight_line(line, syntaxes)
            .map_err(|e| e.to_string())?
        {
            let end = offset + token.chars().count() as i32;
            let c = style.foreground;
            let colour = (c.r, c.g, c.b, style.font_style.bits());
            if let Some(previous) = spans
                .last_mut()
                .filter(|s| s.colour == colour && s.end == offset)
            {
                previous.end = end;
            } else {
                if spans.len() >= SPAN_LIMIT {
                    return Ok(spans);
                }
                spans.push(Span {
                    start: offset,
                    end,
                    colour,
                });
            }
            offset = end;
        }
    }
    Ok(spans)
}

pub fn bind(buffer: &gtk::TextBuffer, path: PathBuf) {
    let version = Rc::new(Cell::new(1_u64));
    let changed = version.clone();
    buffer.connect_changed(move |_| changed.set(changed.get().wrapping_add(1)));
    let styles = adw::StyleManager::default();
    let changed = version.clone();
    let weak = buffer.downgrade();
    let theme_handler = styles.connect_dark_notify(move |_| {
        if weak.upgrade().is_some() {
            changed.set(changed.get().wrapping_add(1));
        }
    });
    let styles_weak = styles.downgrade();
    let busy = Rc::new(Cell::new(false));
    let last = Cell::new(0);
    let tags: Rc<RefCell<HashMap<Colour, gtk::TextTag>>> = Rc::new(RefCell::new(HashMap::new()));
    let weak = buffer.downgrade();
    let mut theme_handler = Some(theme_handler);
    glib::timeout_add_local(Duration::from_millis(250), move || {
        let Some(buffer) = weak.upgrade() else {
            if let (Some(styles), Some(handler)) = (styles_weak.upgrade(), theme_handler.take()) {
                styles.disconnect(handler);
            }
            return glib::ControlFlow::Break;
        };
        if busy.get() || version.get() == last.get() {
            return glib::ControlFlow::Continue;
        }
        busy.set(true);
        let requested = version.get();
        last.set(requested);
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
        let path = path.clone();
        let dark = adw::StyleManager::default().is_dark();
        let task =
            nexuterm::runtime::runtime().spawn_blocking(move || highlight(&path, &text, dark));
        let version = version.clone();
        let busy = busy.clone();
        let weak = weak.clone();
        let tags = tags.clone();
        glib::spawn_future_local(async move {
            let result = task.await;
            if let Some(buffer) = weak.upgrade()
                && version.get() == requested
                && let Ok(Ok(spans)) = result
            {
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                for tag in tags.borrow().values() {
                    buffer.remove_tag(tag, &start, &end);
                }
                for span in &spans {
                    let mut tags = tags.borrow_mut();
                    let tag = tags.entry(span.colour).or_insert_with(|| {
                        let (r, g, b, style) = span.colour;
                        let tag = gtk::TextTag::builder()
                            .foreground_rgba(&gtk::gdk::RGBA::new(
                                f32::from(r) / 255.0,
                                f32::from(g) / 255.0,
                                f32::from(b) / 255.0,
                                1.0,
                            ))
                            .weight(if style & FontStyle::BOLD.bits() != 0 {
                                700
                            } else {
                                400
                            })
                            .style(if style & FontStyle::ITALIC.bits() != 0 {
                                gtk::pango::Style::Italic
                            } else {
                                gtk::pango::Style::Normal
                            })
                            .build();
                        buffer.tag_table().add(&tag);
                        tag
                    });
                    buffer.apply_tag(
                        tag,
                        &buffer.iter_at_offset(span.start),
                        &buffer.iter_at_offset(span.end),
                    );
                }
                #[cfg(debug_assertions)]
                if std::env::var_os("NEXUTERM_SMOKE_SYNTAX").is_some() && !spans.is_empty() {
                    assert!(
                        tags.borrow().len() >= 3,
                        "Code must contain several distinct syntax styles"
                    );
                    eprintln!(
                        "NEXUTERM_SMOKE_SYNTAX_OK spans={} styles={}",
                        spans.len(),
                        tags.borrow().len()
                    );
                }
            }
            busy.set(false);
        });
        glib::ControlFlow::Continue
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn languages_unicode_multiline_and_theme() {
        for (name, code) in [
            (
                "test.py",
                "# comentario\ndef función(valor):\n    return \"á\" + str(valor)\n",
            ),
            (
                "test.rs",
                "// comentario\nfn main() { let text = \"á\"; }\n",
            ),
            (
                "test.cs",
                "// comentario\nclass Example { string text = \"á\"; }\n",
            ),
            (
                "test.sql",
                "-- comentario\nSELECT nombre FROM tabla WHERE id = 42;\n",
            ),
        ] {
            let dark = highlight(name.as_ref(), code, true).unwrap();
            let light = highlight(name.as_ref(), code, false).unwrap();
            assert!(
                dark.iter()
                    .map(|s| s.colour)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    >= 3,
                "{name}"
            );
            assert_ne!(dark[0].colour, light[0].colour);
            assert_eq!(dark.last().unwrap().end as usize, code.chars().count());
        }
        assert!(
            highlight("unknown.txt".as_ref(), "ordinary text", true)
                .unwrap()
                .is_empty()
        );
        let code = "value = '''a\nreturn inside string\n'''\nreturn 42\n";
        assert!(highlight("a.py".as_ref(), code, true).unwrap().len() > 3);
    }
}
