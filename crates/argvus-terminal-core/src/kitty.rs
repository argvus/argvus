//! The kitty configurations the launcher generates. The base configuration gets
//! the theme, the font, the opacity and the accent of the session; the TUI
//! configuration includes the base one and takes the same font.

use crate::is_blank;

/// The settings that rewrite the base configuration.
pub struct BaseSettings<'a> {
  /// The theme file the configuration includes.
  pub theme_file: &'a str,
  pub family: &'a str,
  /// The size, with its `.0` suffix as kitty reads it.
  pub size: &'a str,
  /// `None` keeps the theme's own `background_opacity` lines.
  pub opacity: Option<&'a str>,
  pub accent: &'a str,
}

/// The base kitty configuration for a theme, a font and an accent.
pub fn base_config(source: &str, settings: &BaseSettings) -> String {
  let mut output = String::with_capacity(source.len() + 128);
  for line in source.lines() {
    if is_theme_include(line) {
      push(&mut output, &format!("include {}", settings.theme_file));
    } else if has_key(line, "font_family") {
      push(
        &mut output,
        &format!("font_family      {}", settings.family),
      );
    } else if has_key(line, "font_size") {
      push(&mut output, &format!("font_size        {}", settings.size));
    } else if has_key(line, "background_opacity") {
      match settings.opacity {
        Some(opacity) => push(&mut output, &format!("background_opacity {opacity}")),
        None => push(&mut output, line),
      }
    } else {
      push(&mut output, line);
    }
  }
  // The theme's include is read before the base lines, so the opacity and the
  // accent are written last to win over the theme.
  if let Some(opacity) = settings.opacity {
    push(&mut output, &format!("background_opacity {opacity}"));
  }
  push(&mut output, "");
  push(&mut output, "# Runtime ARGVUS accent override");
  push(
    &mut output,
    &format!("active_tab_foreground   {}", settings.accent),
  );
  output
}

/// The TUI kitty configuration: it includes the base configuration and takes the
/// font of the profile.
pub fn tui_config(source: &str, base: &str, family: &str, size: &str) -> String {
  let mut output = String::with_capacity(source.len() + base.len());
  for line in source.lines() {
    if is_include(line) {
      push(&mut output, &format!("include {base}"));
    } else if has_key(line, "font_family") {
      push(&mut output, &format!("font_family      {family}"));
    } else if has_key(line, "font_size") {
      push(&mut output, &format!("font_size        {size}"));
    } else {
      push(&mut output, line);
    }
  }
  output
}

fn push(output: &mut String, line: &str) {
  output.push_str(line);
  output.push('\n');
}

/// `^[[:space:]]*KEY[[:space:]]+`: the key, followed by blanks.
fn has_key(line: &str, key: &str) -> bool {
  let trimmed = line.trim_start_matches(is_blank);
  trimmed
    .strip_prefix(key)
    .is_some_and(|rest| rest.starts_with(is_blank))
}

/// `^[[:space:]]*include[[:space:]]+`.
fn is_include(line: &str) -> bool {
  has_key(line, "include")
}

/// An include of a theme file: `include ...ID/theme.conf` where `ID` sits under a
/// `themes` directory and contains no blanks.
fn is_theme_include(line: &str) -> bool {
  let trimmed = line.trim_start_matches(is_blank);
  let Some(rest) = trimmed.strip_prefix("include") else {
    return false;
  };
  if !rest.starts_with(is_blank) {
    return false;
  }
  let body = rest.trim_end_matches(is_blank);
  let Some(path) = body.strip_suffix("/theme.conf") else {
    return false;
  };
  match path.rfind("/themes/") {
    Some(index) => {
      let identifier = &path[index + "/themes/".len()..];
      !identifier.is_empty() && !identifier.chars().any(is_blank)
    }
    None => false,
  }
}

#[cfg(test)]
mod tests {
  use super::{BaseSettings, base_config, tui_config};

  fn settings(opacity: Option<&str>) -> BaseSettings<'_> {
    BaseSettings {
      theme_file: "/usr/share/argvus/terminal/config/themes/argvus-dark/theme.conf",
      family: "IBM Plex Mono",
      size: "13.0",
      opacity,
      accent: "#3590bd",
    }
  }

  #[test]
  fn base_config_rewrites_theme_font_and_accent() {
    let source = "include /x/themes/old/theme.conf\nfont_family      Old\nfont_size 9\nbackground_opacity 0.9\nshell zsh\n";
    assert_eq!(
      base_config(source, &settings(None)),
      "include /usr/share/argvus/terminal/config/themes/argvus-dark/theme.conf\n\
             font_family      IBM Plex Mono\n\
             font_size        13.0\n\
             background_opacity 0.9\n\
             shell zsh\n\
             \n\
             # Runtime ARGVUS accent override\n\
             active_tab_foreground   #3590bd\n"
    );
  }

  #[test]
  fn opacity_overrides_the_theme_at_the_end() {
    let output = base_config("font_size 9\n", &settings(Some("0.75")));
    assert!(output.ends_with("background_opacity 0.75\n\n# Runtime ARGVUS accent override\nactive_tab_foreground   #3590bd\n"));
  }

  #[test]
  fn tui_config_includes_the_base() {
    let source =
      "include /usr/share/argvus/terminal/config/kitty.conf\nfont_family x\nfont_size 1\n";
    assert_eq!(
      tui_config(source, "/cache/kitty.conf", "JetBrains Mono", "14.0"),
      "include /cache/kitty.conf\nfont_family      JetBrains Mono\nfont_size        14.0\n"
    );
  }
}
