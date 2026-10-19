//! The Waybar and rofi layout of a theme switch.
//!
//! Each theme class has its own outer margins and corner radii, and the taskbar
//! border follows the class too. The managed copies of the Waybar files are
//! refreshed from the packaged ones first, and the font blocks are written last.
//! Rust home of the Waybar section of the theme switch.

use crate::css_edit::{self, Declaration};
use crate::session_paths::SessionPaths;
use crate::text_edit;
use crate::user_copy;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const FONTS_START: &str = "/* argvus-control-center-fonts:start */";
const FONTS_END: &str = "/* argvus-control-center-fonts:end */";
const LEGACY_FONTS_START: &str = "/* argvus-settings-fonts:start */";
const LEGACY_FONTS_END: &str = "/* argvus-settings-fonts:end */";

/// The themes that get the compact layout. Matched by the exact theme id, as
/// the shell's case statement did, so a Float id uses the default layout.
const COMPACT_THEMES: [&str; 21] = [
  "one-dark",
  "dracula",
  "argvus-dark",
  "silver-dark",
  "argvus-light",
  "github-light",
  "solarized-light",
  "one-light",
  "everforest-light",
  "catppuccin-latte",
  "gruvbox-light",
  "slate-dark",
  "universe",
  "gruvbox-high-dark",
  "gruvbox-dark",
  "rose-pine",
  "tokyo-night",
  "solitude",
  "sunset",
  "matrix",
  "monokai-dark",
];

/// The margins and radii of one theme class.
struct Layout {
  /// Taskbar margins: top, left, right; the bottom is always zero.
  taskbar_margin: u32,
  /// Radii of `#workspaces button`, its active state, `tooltip` and the right group.
  taskbar_radii: [&'static str; 4],
  taskbar_window_radius: &'static str,
  telemetry_margin: u32,
  telemetry_radius: &'static str,
  rofi_radii: [&'static str; 2],
}

const COMPACT: Layout = Layout {
  taskbar_margin: 0,
  taskbar_radii: ["0px", "0px", "0px", "0px"],
  taskbar_window_radius: "0px",
  telemetry_margin: 0,
  telemetry_radius: "0px",
  rofi_radii: ["0px", "0px"],
};

const DEFAULT: Layout = Layout {
  taskbar_margin: 16,
  taskbar_radii: ["5px", "4px", "8px", "5px"],
  taskbar_window_radius: "4px",
  telemetry_margin: 16,
  telemetry_radius: "8px",
  rofi_radii: ["6px", "5px"],
};

fn layout_for(theme: &str) -> &'static Layout {
  if COMPACT_THEMES.contains(&theme) {
    &COMPACT
  } else {
    &DEFAULT
  }
}

/// Copies the packaged Waybar file over its managed copy. A packaged file that
/// does not exist leaves the managed copy as it is.
pub fn refresh_managed(paths: &SessionPaths, relative: &str) -> io::Result<()> {
  let system = paths.system_config(relative);
  if !system.is_file() {
    return Ok(());
  }
  let managed = paths.user_config(relative);
  if let Some(parent) = managed.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::copy(system, managed)?;
  Ok(())
}

/// Sets the margins, the corner radii and the taskbar border of a theme class.
/// Files that are missing are skipped, as the shell's `argvus config` calls
/// were.
pub fn apply_layout(paths: &SessionPaths, theme: &str) -> io::Result<()> {
  let layout = layout_for(theme);
  let taskbar_json = live(paths, "taskbar/config/argvus-taskbar.jsonc");
  let telemetry_json = live(
    paths,
    "widget-telemetry/config/argvus-widget-telemetry.jsonc",
  );
  let taskbar_css = live(paths, "taskbar/config/argvus-taskbar.css");
  let telemetry_css = live(paths, "widget-telemetry/config/argvus-widget-telemetry.css");
  let rofi_css = live(paths, "launcher/config/theme.rasi");

  if let Some(path) = &taskbar_json {
    edit_text(path, |text| {
      let text = text_edit::set_jsonc_margin(text, "top", layout.taskbar_margin, false);
      let text = text_edit::set_jsonc_margin(&text, "left", layout.taskbar_margin, false);
      let text = text_edit::set_jsonc_margin(&text, "right", layout.taskbar_margin, false);
      text_edit::set_jsonc_margin(&text, "bottom", 0, true)
    })?;
  }
  if let Some(path) = &telemetry_json {
    edit_text(path, |text| {
      let text = text_edit::set_jsonc_margin(text, "top", layout.telemetry_margin, true);
      let text = text_edit::set_jsonc_margin(&text, "left", layout.telemetry_margin, true);
      text_edit::set_jsonc_margin(&text, "bottom", 0, true)
    })?;
  }
  if let Some(path) = &taskbar_css {
    let radii = [
      ("window#waybar {", layout.taskbar_window_radius),
      ("#workspaces button", layout.taskbar_radii[0]),
      ("#workspaces button.active,", layout.taskbar_radii[1]),
      ("tooltip {", layout.taskbar_radii[2]),
      (
        "#right-0, #right-1, #right-2, #right-search, #mpris",
        layout.taskbar_radii[3],
      ),
    ];
    edit_text(path, |text| {
      let mut text = text.to_string();
      for (block, value) in radii {
        text = css_edit::set_declaration_in_blocks(&text, block, Declaration::Radius, value);
      }
      // Only slate-dark drops the taskbar border; every other theme keeps the accent border.
      let border = if theme == "slate-dark" {
        "none"
      } else {
        "1px solid @th-decorate"
      };
      css_edit::set_declaration_in_blocks(&text, "window#waybar {", Declaration::Border, border)
    })?;
  }
  if let Some(path) = &telemetry_css {
    edit_text(path, |text| {
      css_edit::set_declaration_in_blocks(
        text,
        "window#waybar {",
        Declaration::Radius,
        layout.telemetry_radius,
      )
    })?;
  }
  if let Some(path) = &rofi_css {
    edit_text(path, |text| {
      let text = css_edit::set_declaration_in_blocks(
        text,
        "window {",
        Declaration::Radius,
        layout.rofi_radii[0],
      );
      css_edit::set_declaration_in_blocks(
        &text,
        "element selected.normal {",
        Declaration::Radius,
        layout.rofi_radii[1],
      )
    })?;
  }
  Ok(())
}

/// Writes the font block of a Waybar stylesheet between its markers. The
/// legacy markers are renamed, and a file without markers gets the block at its
/// end. A missing file is skipped.
pub fn write_font_block(path: &Path, block: &str) -> io::Result<()> {
  if !path.is_file() {
    return Ok(());
  }
  let text = fs::read_to_string(path)?;
  let edited = if text.contains(FONTS_START) {
    replace_block(&text, FONTS_START, FONTS_END, FONTS_START, FONTS_END, block)
  } else if text.contains(LEGACY_FONTS_START) {
    replace_block(
      &text,
      LEGACY_FONTS_START,
      LEGACY_FONTS_END,
      FONTS_START,
      FONTS_END,
      block,
    )
  } else {
    format!("{text}\n{FONTS_START}\n{block}\n{FONTS_END}\n")
  };
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

/// The awk program of the shell: a line equal to `start` is kept with the new
/// start marker, the block is printed, and everything up to a line equal to
/// `end` is dropped.
fn replace_block(
  text: &str,
  start: &str,
  end: &str,
  new_start: &str,
  new_end: &str,
  block: &str,
) -> String {
  let mut output = String::new();
  let mut skipping = false;
  for line in text.split_inclusive('\n') {
    let content = line.strip_suffix('\n').unwrap_or(line);
    if !skipping && content == start {
      output.push_str(&format!("{new_start}\n{block}\n"));
      skipping = true;
    } else if skipping && content == end {
      output.push_str(&format!("{new_end}\n"));
      skipping = false;
    } else if !skipping {
      output.push_str(line);
      if !line.ends_with('\n') {
        output.push('\n');
      }
    }
  }
  output
}

/// The mutable copy of a file, as the shell's `paths_config` gave it.
fn live(paths: &SessionPaths, relative: &str) -> Option<PathBuf> {
  user_copy::config_path(paths, relative).filter(|path| path.is_file())
}

fn edit_text(path: &Path, edit: impl Fn(&str) -> String) -> io::Result<()> {
  let text = fs::read_to_string(path)?;
  let edited = edit(&text);
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn compact_list_matches_exact_ids_only() {
    assert_eq!(layout_for("rose-pine").taskbar_margin, 0);
  }

  #[test]
  fn font_block_replaces_only_its_markers() {
    let text = format!("a\n{FONTS_START}\nold\n{FONTS_END}\nb\n");
    let edited = replace_block(&text, FONTS_START, FONTS_END, FONTS_START, FONTS_END, "new");
    assert_eq!(edited, format!("a\n{FONTS_START}\nnew\n{FONTS_END}\nb\n"));
  }

  #[test]
  fn legacy_font_markers_are_renamed() {
    let text = format!("a\n{LEGACY_FONTS_START}\nold\n{LEGACY_FONTS_END}\n");
    let edited = replace_block(
      &text,
      LEGACY_FONTS_START,
      LEGACY_FONTS_END,
      FONTS_START,
      FONTS_END,
      "new",
    );
    assert_eq!(edited, format!("a\n{FONTS_START}\nnew\n{FONTS_END}\n"));
  }

  #[test]
  fn default_taskbar_radii_keep_the_shell_values() {
    // `#workspaces button.active,` is 4px while the button itself is 5px.
    assert_eq!(DEFAULT.taskbar_radii, ["5px", "4px", "8px", "5px"]);
    assert_eq!(DEFAULT.taskbar_window_radius, "4px");
  }
}
