//! Implements safe fallback rendering in crate `argvus theme`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use super::parser::Rgba;

/// Defines the constant `BG`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const BG: Rgba = Rgba::opaque(0x11, 0x13, 0x16);
/// Defines the constant `SURFACE_ALT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const SURFACE_ALT: Rgba = Rgba::opaque(0x26, 0x29, 0x33);
/// Defines the constant `FG`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const FG: Rgba = Rgba::opaque(0xdf, 0xe5, 0xea);
/// Defines the constant `MUTED`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const MUTED: Rgba = Rgba::opaque(0xb0, 0xbf, 0xcb);
/// Defines the constant `ACCENT`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const ACCENT: Rgba = Rgba::opaque(0x35, 0x90, 0xbd);
/// Defines the constant `BORDER`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const BORDER: Rgba = Rgba {
  r: 53,
  g: 144,
  b: 189,
  a: 71,
};
/// Defines the constant `FOCUS`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const FOCUS: Rgba = Rgba {
  r: 53,
  g: 144,
  b: 189,
  a: 191,
};

/// Executes the `definitions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
