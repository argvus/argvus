//! Line-based edits of the CSS files the surfaces read: color parsing, `@define-color` values and marked blocks.

use super::*;

pub(super) fn opacity_factor(transparency: u32) -> String {
  format!("{:.2}", 1.0 - f64::from(transparency) / 100.0)
}

/// Every `@define-color` line of the theme files imported by a Taskbar stylesheet,
/// so the opaque border colors can be derived from the theme palette.
pub(super) fn imported_theme_text(css: &Path, text: &str) -> String {
  let parent = css.parent().unwrap_or(Path::new("."));
  text
    .lines()
    .filter_map(|line| {
      let rest = line.trim().strip_prefix("@import url(\"")?;
      let relative = rest.split('"').next()?;
      fs::read_to_string(parent.join(relative)).ok()
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// The value text of `@define-color NAME VALUE;` in `theme`.
pub(super) fn defined_color(theme: &str, name: &str) -> Option<String> {
  let prefix = format!("@define-color {name} ");
  theme.lines().find_map(|line| {
    let value = line.trim_start().strip_prefix(&prefix)?;
    Some(value.trim().trim_end_matches(';').trim().to_string())
  })
}

/// Parses `#rrggbb` or `rgba(r, g, b, a)` into 0..=255 channels and an alpha.
pub(super) fn parse_css_color(value: &str) -> Option<([f64; 3], f64)> {
  let value = value.trim();
  if let Some(hex) = value.strip_prefix('#') {
    if hex.len() != 6 {
      return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    return Some((
      [
        f64::from(channel(0..2)?),
        f64::from(channel(2..4)?),
        f64::from(channel(4..6)?),
      ],
      1.0,
    ));
  }
  let inner = value
    .strip_prefix("rgba(")
    .or_else(|| value.strip_prefix("rgb("))?
    .strip_suffix(')')?;
  let parts: Vec<f64> = inner
    .split(',')
    .map(|part| part.trim().parse::<f64>().ok())
    .collect::<Option<_>>()?;
  match parts.as_slice() {
    [red, green, blue] => Some(([*red, *green, *blue], 1.0)),
    [red, green, blue, alpha] => Some(([*red, *green, *blue], *alpha)),
    _ => None,
  }
}

/// The opaque color a translucent `foreground` shows over `background`, as
/// `#rrggbb`. `None` when either color cannot be parsed.
pub(super) fn solid_over(foreground: &str, background: &str) -> Option<String> {
  let (front, alpha) = parse_css_color(foreground)?;
  let (back, _) = parse_css_color(background)?;
  let mixed = |index: usize| {
    (front[index] * alpha + back[index] * (1.0 - alpha))
      .round()
      .clamp(0.0, 255.0) as u8
  };
  Some(format!("#{:02X}{:02X}{:02X}", mixed(0), mixed(1), mixed(2)))
}

/// Routes the translucent border colors of a stylesheet through
/// `NAME-effective` variables. With transparency on they keep the theme value;
/// with it off they become the opaque color the theme value shows over
/// `background_name`, so no part of the surface stays see-through.
pub(super) fn apply_border_transparency(
  css: &Path,
  text: &mut String,
  border_names: &[&str],
  background_name: &str,
  anchor: &str,
  transparent: bool,
) {
  let theme = imported_theme_text(css, text);
  let background = defined_color(&theme, background_name);
  for name in border_names {
    let effective = format!("{name}-effective");
    if !text
      .lines()
      .any(|line| line.starts_with(&format!("@define-color {effective} ")))
    {
      // Route every use through the new variable first, then declare it, so the
      // declaration itself keeps pointing at the theme color.
      *text = replace_in_lines(text, &format!("@{name};"), &format!("@{effective};"));
      *text = insert_after(text, anchor, &format!("@define-color {effective} @{name};"));
    }
    let solid = match (transparent, defined_color(&theme, name), &background) {
      (false, Some(border), Some(background)) => solid_over(&border, background),
      _ => None,
    };
    let value = solid.unwrap_or_else(|| format!("@{name}"));
    *text = set_define(text, &effective, &format!("{value};"));
  }
}

/// `sed "s|^\(key\)…"` on every `@define-color NAME …` line.
pub(super) fn set_define(text: &str, name: &str, value: &str) -> String {
  let rules = [Rule::literal(
    &format!(r"^@define-color {name} .*"),
    format!("@define-color {name} {value}"),
  )];
  apply_rules(text, &rules)
}

pub(super) fn replace_in_lines(text: &str, from: &str, to: &str) -> String {
  let rules = [Rule::literal(&regex::escape(from), to.to_string())];
  apply_rules(text, &rules)
}

/// `sed '/exact/a text'`: inserts `insert` after each line that starts with
/// `after`, the first match on the line counting as in `sed`.
pub(super) fn insert_after(text: &str, after: &str, insert: &str) -> String {
  let mut output = String::new();
  for line in text.split_inclusive('\n') {
    output.push_str(line);
    let content = line.strip_suffix('\n').unwrap_or(line);
    if content.starts_with(after) {
      if !line.ends_with('\n') {
        output.push('\n');
      }
      output.push_str(insert);
      output.push('\n');
    }
  }
  output
}

/// `sed '/start/,/end/d'`: removes each block from a `start` line through the
/// next `end` line.
pub(super) fn delete_range(text: &str, start: &str, end: &str) -> String {
  let mut inside = false;
  let mut output = String::new();
  for line in text.split_inclusive('\n') {
    let content = line.strip_suffix('\n').unwrap_or(line);
    if !inside && content == start {
      inside = true;
    }
    if inside {
      if content == end {
        inside = false;
      }
      continue;
    }
    output.push_str(line);
  }
  output
}

/// Sets the `"name"` field of a Taskbar JSONC file, or adds it after the first line.
pub(super) fn ensure_taskbar_namespace(path: Option<&Path>, namespace: &str) -> io::Result<()> {
  let Some(path) = path.filter(|path| path.is_file()) else {
    return Ok(());
  };
  let text = fs::read_to_string(path)?;
  let has_name = text.lines().any(|line| {
    line.trim_start().starts_with("\"name\"")
      && line.trim_start()["\"name\"".len()..]
        .trim_start()
        .starts_with(':')
  });
  let edited = if has_name {
    let rules = [Rule::literal(
      r#"^[[:space:]]*"name"[[:space:]]*:.*"#,
      format!("  \"name\": \"{namespace}\","),
    )];
    apply_rules(&text, &rules)
  } else {
    insert_after_first_line(&text, &format!("  \"name\": \"{namespace}\","))
  };
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

pub(super) fn insert_after_first_line(text: &str, insert: &str) -> String {
  let mut lines = text.split_inclusive('\n');
  let mut output = String::new();
  if let Some(first) = lines.next() {
    output.push_str(first);
    if !first.ends_with('\n') {
      output.push('\n');
    }
  }
  output.push_str(insert);
  output.push('\n');
  for line in lines {
    output.push_str(line);
  }
  output
}

pub(super) fn read_first_line(path: &Path) -> String {
  fs::read_to_string(path)
    .ok()
    .and_then(|text| text.lines().next().map(|line| line.trim().to_string()))
    .unwrap_or_default()
}
