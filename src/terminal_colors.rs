//! Terminal colors follow libadwaita's effective theme, including system changes.
use gtk::{gdk::RGBA, prelude::*};
use vte4::prelude::*;

struct Scheme {
    background: u32,
    foreground: u32,
    selection: u32,
    palette: [u32; 16],
}
const DARK: Scheme = Scheme {
    background: 0x24272b,
    foreground: 0xd6d8dc,
    selection: 0x414d60,
    palette: [
        0x30343b, 0xe39595, 0x8bbf82, 0xcbb579, 0x8fadd9, 0xb6a1cf, 0x81bdb5, 0xd6d8dc, 0xa3aab5,
        0xf0aaa6, 0xa4cf99, 0xe0c68c, 0xa6c1e8, 0xcbb5e1, 0x9ad1c7, 0xe6e8ec,
    ],
};
const LIGHT: Scheme = Scheme {
    background: 0xf4f1ea,
    foreground: 0x30343b,
    selection: 0xd4deeb,
    palette: [
        0x30343b, 0x9f343b, 0x386a36, 0x806018, 0x365c96, 0x774e92, 0x286b70, 0x63686f, 0x565d66,
        0xa03c43, 0x3e703a, 0x806019, 0x3b639a, 0x805496, 0x2e7073, 0x484e57,
    ],
};
fn color(hex: u32) -> RGBA {
    RGBA::new(
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
        1.0,
    )
}
fn apply(terminal: &vte4::Terminal, dark: bool) {
    let scheme = if dark { &DARK } else { &LIGHT };
    let foreground = color(scheme.foreground);
    let background = color(scheme.background);
    let palette = scheme.palette.map(color);
    let references: Vec<&RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&foreground), Some(&background), &references);
    terminal.set_color_bold(Some(&foreground));
    terminal.set_color_cursor(Some(&foreground));
    terminal.set_color_cursor_foreground(Some(&background));
    terminal.set_color_highlight(Some(&color(scheme.selection)));
    terminal.set_color_highlight_foreground(Some(&foreground));
}
pub fn bind(terminal: &vte4::Terminal) {
    let manager = adw::StyleManager::default();
    apply(terminal, manager.is_dark());
    let weak = terminal.downgrade();
    let handler = manager.connect_dark_notify(move |manager| {
        if let Some(terminal) = weak.upgrade() {
            apply(&terminal, manager.is_dark());
        }
    });
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_COLORS").is_some()
    {
        let original = manager.color_scheme();
        manager.set_color_scheme(adw::ColorScheme::ForceDark);
        assert_eq!(terminal.color_background_for_draw(), color(DARK.background));
        manager.set_color_scheme(adw::ColorScheme::ForceLight);
        assert_eq!(
            terminal.color_background_for_draw(),
            color(LIGHT.background)
        );
        manager.set_color_scheme(original);
        eprintln!("NEXUTERM_SMOKE_COLORS_OK dark=1 light=1 live_switch=1");
    }
    let handler = std::cell::RefCell::new(Some(handler));
    terminal.connect_destroy(move |_| {
        if let Some(handler) = handler.borrow_mut().take() {
            manager.disconnect(handler);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(hex: u32) -> f64 {
        let channel = |shift: u32| {
            let c = f64::from((hex >> shift) & 255u32) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }
    fn contrast(a: u32, b: u32) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    #[test]
    fn text_and_ansi_colors_remain_readable_in_both_themes() {
        for scheme in [&DARK, &LIGHT] {
            assert!(contrast(scheme.foreground, scheme.background) >= 7.0);
            assert!(contrast(scheme.foreground, scheme.selection) >= 4.5);
            for (index, entry) in scheme.palette.iter().copied().enumerate() {
                // Preserve dark ANSI black for prompts that use colored backgrounds.
                if scheme.background == DARK.background && index == 0 {
                    continue;
                }
                assert!(
                    contrast(entry, scheme.background) >= 4.5,
                    "Color {entry:06x}"
                );
            }
            if scheme.background == DARK.background {
                assert!(contrast(scheme.palette[0], scheme.palette[3]) >= 4.5);
            }
            if scheme.background == DARK.background {
                assert!(contrast(scheme.palette[7], scheme.palette[0]) >= 4.5);
            }
        }
    }
}
