//! Safe fallback rendering.
use super::parser::Rgba;

pub const BG: Rgba = Rgba::opaque(0x11, 0x13, 0x16);
pub const SURFACE_ALT: Rgba = Rgba::opaque(0x26, 0x29, 0x33);
pub const FG: Rgba = Rgba::opaque(0xdf, 0xe5, 0xea);
pub const MUTED: Rgba = Rgba::opaque(0xb0, 0xbf, 0xcb);
pub const ACCENT: Rgba = Rgba::opaque(0x35, 0x90, 0xbd);
pub const BORDER: Rgba = Rgba {
  r: 53,
  g: 144,
  b: 189,
  a: 71,
};
pub const FOCUS: Rgba = Rgba {
  r: 53,
  g: 144,
  b: 189,
  a: 191,
};

pub fn definitions() -> [(&'static str, &'static str); 9] {
  [
    ("argvus_bg", "#111316"),
    ("argvus_surface", "#201f27"),
    ("argvus_surface_alt", "#262933"),
    ("argvus_fg", "#dfe5ea"),
    ("argvus_muted", "#b0bfcb"),
    ("argvus_accent", "#3590bd"),
    ("argvus_border", "rgba(53, 144, 189, 0.28)"),
    ("argvus_focus", "rgba(53, 144, 189, 0.75)"),
    ("argvus_danger", "#d1174f"),
  ]
}
