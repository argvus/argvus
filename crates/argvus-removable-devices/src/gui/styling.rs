//! Stylesheet of the menu, with the accent of the active ARGVUS theme.

use super::*;

// Loads theme.css from disk instead of embedding CSS in the binary.
// Candidates are tried in the same precedence order as config.json (system
// defaults, then user override, then a theme.css next to an explicit
// --config file); every one that exists is applied, in that order, so a
// later file's rules win over an earlier file's for the same selector.
// Missing files are silently skipped — if none exist, the menu falls back
// to unstyled GTK widgets rather than failing to open. GTK4 reports invalid
// CSS through its own log, so a bad file no longer aborts this step.
pub(super) fn install_css(config_path: &str) {
  let Some(display) = gdk::Display::default() else {
    return;
  };
  for path in Config::theme_css_paths(config_path) {
    if !std::path::Path::new(&path).is_file() {
      continue;
    }
    let provider = gtk::CssProvider::new();
    provider.load_from_path(&path);
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
  }
  let accent = accent_runtime_css();
  if !accent.is_empty() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&accent);
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
    );
  }
}

pub(super) fn accent_runtime_css() -> String {
  let Some(config_home) = std::env::var_os("ARGVUS_CONFIG_HOME")
    .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
    .or_else(|| {
      std::env::var_os("HOME").map(|home| {
        let mut path = std::path::PathBuf::from(home);
        path.push(".config");
        path.into_os_string()
      })
    })
  else {
    return String::new();
  };
  let path = std::path::PathBuf::from(config_home)
    .join("argvus")
    .join(".accent-color");
  let Ok(value) = std::fs::read_to_string(path) else {
    return String::new();
  };
  let value = value.trim();
  let hex = value.strip_prefix('#').unwrap_or(value);
  if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
    return String::new();
  }
  let Ok(red) = u8::from_str_radix(&hex[0..2], 16) else {
    return String::new();
  };
  let Ok(green) = u8::from_str_radix(&hex[2..4], 16) else {
    return String::new();
  };
  let Ok(blue) = u8::from_str_radix(&hex[4..6], 16) else {
    return String::new();
  };
  let luminance = (299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue)) / 1000;
  let text = if luminance >= 128 {
    "#000000"
  } else {
    "#FFFFFF"
  };
  format!(
    ".storage-popup-menu {{ border-color: rgba({red}, {green}, {blue}, 0.54); }}\n\
     .storage-menu-row:hover, .storage-menu-row.submenu-open, .storage-menu-row.pointer-over {{ background-color: {value}; color: {text}; }}"
  )
}
