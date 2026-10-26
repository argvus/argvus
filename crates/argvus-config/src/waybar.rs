//! The Waybar and rofi layout of a theme switch.
//!
//! Each theme class has its own outer margins and corner radii, and the taskbar
//! border follows the class too. The managed copies of the Waybar files are
//! refreshed from the packaged ones first, and the font blocks are written last.
//! Rust home of the Waybar section of the theme switch.

use crate::ConfigDocument;
use crate::css_edit::{self, Declaration};
use crate::session_paths::SessionPaths;
use crate::text_edit;
use crate::user_copy;
use serde_json::Value;
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

/// The taskbar heights the Control Center offers, in pixels.
pub const TASKBAR_HEIGHTS: [u32; 4] = [24, 28, 32, 36];

/// The taskbar border widths, in pixels. `0` turns the border off; the others
/// are the thickness when it is on.
pub const TASKBAR_BORDER_WIDTHS: [u32; 4] = [0, 1, 2, 3];

/// The blocks whose borders follow the taskbar border width: the taskbar window,
/// the right group with the MPRIS player, and the MPRIS player's hover edge.
const TASKBAR_BORDER_BLOCKS: [&str; 4] = [
  "window#waybar {",
  "#right-0, #right-1, #right-2, #mpris, #custom-spotify-mpris {",
  "#mpris,",
  "#mpris:hover,",
];

/// Sets the border width of every taskbar block that draws a border.
pub fn set_taskbar_border_width(text: &str, width: u32) -> String {
  TASKBAR_BORDER_BLOCKS
    .iter()
    .fold(text.to_string(), |text, block| {
      css_edit::set_border_width_in_blocks(&text, block, width)
    })
}

/// The taskbar height and border width of the live taskbar files. Each one is
/// `None` when its file is missing or has no such setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskbarShape {
  pub height: Option<u32>,
  pub border_width: Option<u32>,
}

impl TaskbarShape {
  /// The shape chosen in the canonical configuration under `/taskbar/shape`.
  /// An unset value is `None` and follows the packaged taskbar and the theme.
  pub fn chosen(document: &ConfigDocument) -> Self {
    let pixels = |pointer: &str| {
      document
        .get(pointer)
        .and_then(Value::as_u64)
        .and_then(|pixels| u32::try_from(pixels).ok())
    };
    Self {
      height: pixels("/taskbar/shape/height"),
      border_width: pixels("/taskbar/shape/border_width"),
    }
  }
}

/// Writes a chosen shape into the live taskbar files. `None` values leave the
/// files as they are.
pub fn apply_taskbar_shape(paths: &SessionPaths, shape: TaskbarShape) -> io::Result<()> {
  if let Some(height) = shape.height {
    set_taskbar_height(paths, height)?;
  }
  if let Some(width) = shape.border_width {
    set_taskbar_border(paths, width)?;
  }
  Ok(())
}

/// Sets the taskbar height in the live JSONC file. A missing file is skipped.
pub fn set_taskbar_height(paths: &SessionPaths, height: u32) -> io::Result<()> {
  match live(paths, "taskbar/config/argvus-taskbar.jsonc") {
    Some(path) => edit_text(&path, |text| text_edit::set_jsonc_height(text, height)),
    None => Ok(()),
  }
}

/// Sets the taskbar border width in the live stylesheet. A missing file is skipped.
pub fn set_taskbar_border(paths: &SessionPaths, width: u32) -> io::Result<()> {
  match live(paths, "taskbar/config/argvus-taskbar.css") {
    Some(path) => edit_text(&path, |text| set_taskbar_border_width(text, width)),
    None => Ok(()),
  }
}

/// Reads the taskbar height and border width from the live files.
pub fn taskbar_shape(paths: &SessionPaths) -> TaskbarShape {
  let read = |relative: &str| live(paths, relative).and_then(|path| fs::read_to_string(path).ok());
  TaskbarShape {
    height: read("taskbar/config/argvus-taskbar.jsonc")
      .and_then(|text| text_edit::jsonc_height(&text)),
    border_width: read("taskbar/config/argvus-taskbar.css")
      .and_then(|text| css_edit::border_width_in_block(&text, "window#waybar {")),
  }
}

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
  fn chosen_shape_reads_set_values_and_leaves_null_to_the_theme() {
    let mut document = ConfigDocument::default();
    document.sections.insert(
      "taskbar".into(),
      serde_json::json!({ "shape": { "height": 32, "border_width": null } }),
    );
    assert_eq!(
      TaskbarShape::chosen(&document),
      TaskbarShape {
        height: Some(32),
        border_width: None,
      }
    );
    assert_eq!(
      TaskbarShape::chosen(&ConfigDocument::default()),
      TaskbarShape {
        height: None,
        border_width: None,
      }
    );
  }

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
  fn taskbar_border_width_reaches_every_bordered_block() {
    let css = "window#waybar {\n  border: 1px solid @th-decorate;\n}\n\n#right-0, #right-1, #right-2, #mpris, #custom-spotify-mpris {\n  border: 1px solid @th-border-rights;\n}\n\n#mpris,\n#custom-spotify-mpris {\n  border: 1px solid @th-mpris-border;\n}\n\n#mpris:hover,\n#custom-spotify-mpris:hover {\n  border-bottom: 1px solid @th-decorate;\n}\n";
    let off = set_taskbar_border_width(css, 0);
    assert_eq!(off.matches("0px solid").count(), 4);
    assert_eq!(off.matches("border-bottom: 0px").count(), 1);
    let thick = set_taskbar_border_width(css, 3);
    assert_eq!(thick.matches("3px solid").count(), 4);
    assert_eq!(
      css_edit::border_width_in_block(&thick, "window#waybar {"),
      Some(3)
    );
  }

  #[test]
  fn default_taskbar_radii_keep_the_shell_values() {
    // `#workspaces button.active,` is 4px while the button itself is 5px.
    assert_eq!(DEFAULT.taskbar_radii, ["5px", "4px", "8px", "5px"]);
    assert_eq!(DEFAULT.taskbar_window_radius, "4px");
  }
}
