//! The highlight color of the desktop.
//!
//! One color is applied to the theme files of every component, the qt6ct and
//! GTK settings, the greeter and the running applications. The callers reach
//! these rules through `argvus config accent` (see `accent_switch`),
//! which adds the picker, the icon theme and the messages. This module decides
//! what is written and where.

use crate::session_paths::SessionPaths;
use crate::text_edit;
use crate::user_copy;
use regex::Regex;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const DEFAULT_ACCENT: &str = "#798186";
pub const DEFAULT_THEME: &str = "argvus-dark";

/// A validated color, with the channels and the text color that reads on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accent {
  /// `#RRGGBB`, uppercase.
  pub color: String,
  pub red: u8,
  pub green: u8,
  pub blue: u8,
  /// Black on light colors, white on dark ones.
  pub text: &'static str,
}

impl Accent {
  /// The six hex digits, without `#`.
  pub fn hex(&self) -> &str {
    &self.color[1..]
  }

  fn rgb(&self) -> String {
    format!("rgb({}, {}, {})", self.red, self.green, self.blue)
  }

  fn rgba(&self, alpha: &str) -> String {
    format!("rgba({}, {}, {}, {alpha})", self.red, self.green, self.blue)
  }
}

/// Validates a color: an optional `#` and six hex digits, in either case.
pub fn normalize(requested: &str) -> Option<Accent> {
  let upper = requested.to_ascii_uppercase();
  let digits = upper.strip_prefix('#').unwrap_or(&upper);
  if digits.len() != 6
    || !digits
      .bytes()
      .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
  {
    return None;
  }
  let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
  let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
  let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
  let luminance = (299 * u32::from(red) + 587 * u32::from(green) + 114 * u32::from(blue)) / 1000;
  Some(Accent {
    color: format!("#{digits}"),
    red,
    green,
    blue,
    text: if luminance >= 128 {
      "#000000"
    } else {
      "#FFFFFF"
    },
  })
}

/// The theme ids the switch keeps as the Catppuccin Latte family.
pub fn canonical_theme_id(theme: &str) -> &str {
  match theme {
    "argvus-catppuccin-latte" | "argvus-light-catppuccin-latte" => "catppuccin-latte",
    "argvus-catppuccin-latte-float" | "argvus-light-catppuccin-latte-float" => {
      "catppuccin-latte-float"
    }
    other => other,
  }
}

/// The accent a theme owns when the user has not chosen one: its manifest, or
/// the built-in table for themes that predate manifests.
pub fn theme_default_accent(themes_dir: &Path, theme: &str) -> Option<String> {
  let family = theme.strip_suffix("-float").unwrap_or(theme);
  if let Some(accent) = command_output("argvus-appearance", &["themes", "get", family, "accent"]) {
    return Some(accent);
  }
  if let Some(accent) = crate::theme_resolve::top_value(themes_dir, family, "accent") {
    return Some(accent);
  }
  let accent = match family {
    "one-dark" => "#61AFEF",
    "dracula" => "#BD93F9",
    "argvus-dark" => "#3590bd",
    "silver-dark" => "#595959",
    "argvus-light" => "#181818",
    "github-light" => "#0969DA",
    "one-light" => "#4078F2",
    "everforest-light" => "#3A94C5",
    "solarized-light" => "#268BD2",
    "rose-pine" => "#C4A7E7",
    "frost" => "#0969DA",
    "catppuccin-latte" => "#1E66F5",
    "gruvbox-light" => "#458588",
    "slate-dark" => "#7391a5",
    "universe" => "#eeeeee",
    "gruvbox-high-dark" => "#D79921",
    "gruvbox-dark" => "#D4BE98",
    "tokyo-night" => "#7AA2F7",
    "solitude" => "#798186",
    "sunset" => "#E2BE8A",
    "matrix" => "#82FB9C",
    "monokai-dark" => "#78DCE8",
    _ => return None,
  };
  Some(accent.to_string())
}

/// The first line of a state file, trimmed, or `fallback` when the file is
/// missing or empty.
pub fn read_state(path: &Path, fallback: &str) -> String {
  match fs::read_to_string(path) {
    Ok(text) if !text.is_empty() => text
      .split('\n')
      .next()
      .unwrap_or("")
      .trim_end_matches('\r')
      .trim()
      .to_string(),
    _ => fallback.to_string(),
  }
}

/// The canonical highlight color, when the user owns it. The canonical document
/// is read through the same store as `argvus config get`.
pub fn canonical_accent(config_store: Option<&crate::ConfigStore>) -> Option<String> {
  let store = config_store?;
  let document = store.load().ok()?;
  let owned = match document.get("/appearance/accent_custom") {
    Some(serde_json::Value::Bool(value)) => *value,
    Some(serde_json::Value::String(value)) => value == "true",
    _ => false,
  };
  if !owned {
    return None;
  }
  Some(match document.get("/appearance/accent") {
    Some(serde_json::Value::String(value)) => value.trim().to_string(),
    Some(other) => other.to_string(),
    None => "null".to_string(),
  })
}

/// The accent the state files say should be applied: the saved one, unless the
/// canonical document owns a color, or the saved one is not a color.
pub fn state_accent(themes_dir: &Path, state_dir: &Path, canonical: Option<String>) -> String {
  let state_theme = read_state(&state_dir.join(".active-theme"), DEFAULT_THEME);
  let state_default =
    theme_default_accent(themes_dir, &state_theme).unwrap_or_else(|| DEFAULT_ACCENT.to_string());
  let mut accent = read_state(&state_dir.join(".accent-color"), &state_default);
  if let Some(canonical) = canonical.filter(|value| !value.is_empty()) {
    accent = canonical;
  }
  let accent = match accent.as_str() {
    "blue" => "#3590BD".to_string(),
    "slate-blue" => "#7391A5".to_string(),
    "brown" => "#996548".to_string(),
    "green" => "#17D174".to_string(),
    "magenta" => "#CB17D1".to_string(),
    "red" => "#D1174F".to_string(),
    "yellow" => "#D1CE17".to_string(),
    "purple" => "#9617D1".to_string(),
    "silver" => "#595959".to_string(),
    _ => accent,
  };
  let is_color = (accent.len() == 7 && accent.starts_with('#')) || accent.chars().count() == 6;
  if is_color { accent } else { state_default }
}

/// The theme id the switch applies: the active theme, mapped to a registered
/// id when it has no manifest.
pub fn applied_theme(themes_dir: &Path, active_theme: &str) -> String {
  let theme = canonical_theme_id(active_theme).to_string();
  let family = theme.strip_suffix("-float").unwrap_or(&theme);
  if themes_dir.join(family).join("theme.toml").is_file() {
    return theme;
  }
  const REGISTERED: [&str; 22] = [
    "one-dark",
    "dracula",
    "argvus-dark",
    "silver-dark",
    "argvus-light",
    "github-light",
    "solarized-light",
    "one-light",
    "everforest-light",
    "frost",
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
  if REGISTERED.contains(&family) {
    return theme;
  }
  if theme.ends_with("-dark-float") {
    "argvus-dark-float".to_string()
  } else if theme.ends_with("-light-float") {
    "argvus-light-float".to_string()
  } else if theme.ends_with("-light") {
    "argvus-light".to_string()
  } else {
    DEFAULT_THEME.to_string()
  }
}

/// Runs a command and returns its first line, when it succeeds with output.
fn command_output(program: &str, arguments: &[&str]) -> Option<String> {
  if !on_path(program) {
    return None;
  }
  let output = Command::new(program)
    .args(arguments)
    .stderr(Stdio::null())
    .output()
    .ok()
    .filter(|output| output.status.success())?;
  let text = String::from_utf8_lossy(&output.stdout);
  let line = text.trim_end_matches('\n');
  (!line.is_empty()).then(|| line.to_string())
}

fn on_path(program: &str) -> bool {
  env::var_os("PATH")
    .map(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
    .unwrap_or(false)
}

/// One line edit, the Rust form of a `sed -e 's|…|…|'` expression: the first
/// match on a line is replaced. `$1`-style references are kept only where the
/// template asks for them.
pub struct Rule {
  pattern: Regex,
  template: String,
  literal: bool,
}

impl Rule {
  pub fn literal(pattern: &str, replacement: String) -> Self {
    Self {
      pattern: Regex::new(pattern).expect("accent rule pattern is valid"),
      template: replacement,
      literal: true,
    }
  }

  /// A rule whose template refers to capture groups, as `\1` did in `sed`.
  pub fn groups(pattern: &str, template: String) -> Self {
    Self {
      pattern: Regex::new(pattern).expect("accent rule pattern is valid"),
      template,
      literal: false,
    }
  }

  fn apply(&self, line: &str) -> String {
    if self.literal {
      self
        .pattern
        .replace(line, regex::NoExpand(&self.template))
        .into_owned()
    } else {
      self
        .pattern
        .replace(line, self.template.as_str())
        .into_owned()
    }
  }
}

/// Applies the rules to each line in order, keeping line endings.
pub fn apply_rules(text: &str, rules: &[Rule]) -> String {
  text
    .split_inclusive('\n')
    .map(|line| {
      let (content, ending) = match line.strip_suffix('\n') {
        Some(content) => (content, "\n"),
        None => (line, ""),
      };
      let edited = rules
        .iter()
        .fold(content.to_string(), |current, rule| rule.apply(&current));
      format!("{edited}{ending}")
    })
    .collect()
}

/// Edits a file in place when it exists. A missing file is not an error: the
/// shell skipped it the same way.
pub fn edit_file(path: Option<&Path>, edit: impl Fn(&str) -> String) -> io::Result<()> {
  let Some(path) = path.filter(|path| path.is_file()) else {
    return Ok(());
  };
  let text = fs::read_to_string(path)?;
  let edited = edit(&text);
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

/// `replace_setting`: `NAME = VALUE`, replacing the first line that sets NAME.
fn replace_setting(path: Option<&Path>, name: &str, value: &str) -> io::Result<()> {
  let pattern = format!(r"^{}[[:space:]]*=.*", regex::escape(name));
  let rules = [Rule::literal(&pattern, format!("{name} = {value}"))];
  edit_file(path, |text| apply_rules(text, &rules))
}

/// The per-file rules, kept next to the names they touch.
fn waybar_rules(accent: &Accent) -> Vec<Rule> {
  vec![
    Rule::literal(
      r"^@define-color th-decorate .*",
      format!("@define-color th-decorate        {};", accent.color),
    ),
    Rule::literal(
      r"^@define-color th-decorate-rgba .*",
      format!("@define-color th-decorate-rgba   {};", accent.rgba("0.45")),
    ),
    Rule::literal(
      r"^@define-color th-border-rights .*",
      format!("@define-color th-border-rights   {};", accent.rgba("0.25")),
    ),
    Rule::literal(
      r"^@define-color th-power .*",
      format!("@define-color th-power           {};", accent.color),
    ),
    Rule::literal(
      r"^@define-color th-mpris-border .*",
      format!("@define-color th-mpris-border    {};", accent.rgba("0.25")),
    ),
  ]
}

fn telemetry_rules(accent: &Accent) -> Vec<Rule> {
  vec![
    Rule::literal(
      r"^@define-color th-header .*",
      format!("@define-color th-header        {};", accent.color),
    ),
    Rule::literal(
      r"^@define-color th-border .*",
      format!("@define-color th-border        {};", accent.rgba("0.15")),
    ),
    Rule::literal(
      r"^@define-color th-border-header .*",
      format!("@define-color th-border-header {};", accent.rgba("0.20")),
    ),
  ]
}

/// The rofi colors of a theme file (the theme's own stylesheet).
fn rofi_theme_rules(accent: &Accent) -> Vec<Rule> {
  vec![
    Rule::literal(
      r"^[[:space:]]*th-fg:.*",
      format!("    th-fg:            {};", accent.rgb()),
    ),
    Rule::literal(
      r"^[[:space:]]*th-row-alt:.*",
      format!("    th-row-alt:       {};", accent.rgba("7%")),
    ),
    Rule::literal(
      r"^[[:space:]]*th-border-color:.*",
      format!("    th-border-color:  {};", accent.rgba("0.36")),
    ),
  ]
}

/// The Light mode overlays: the accent variables keep their alpha, the text
/// variables are left as Light mode chose them.
fn mode_css_rules(accent: &Accent) -> Vec<Rule> {
  let rgba_keep_alpha = |name: &str| {
    Rule::groups(
      &format!(r"^(@define-color {name} *)rgba\([^,]*,[^,]*,[^,]*, *([^),]*)\)"),
      format!("${{1}}{}", accent.rgba("${2}")),
    )
  };
  vec![
    Rule::literal(
      r"^@define-color th-decorate .*",
      format!("@define-color th-decorate        {};", accent.color),
    ),
    Rule::literal(
      r"^@define-color th-power .*",
      format!("@define-color th-power           {};", accent.color),
    ),
    rgba_keep_alpha("th-decorate-rgba"),
    rgba_keep_alpha("th-border-rights"),
    rgba_keep_alpha("th-mpris-border"),
  ]
}

fn mode_rasi_rules(accent: &Accent) -> Vec<Rule> {
  let rgba_keep_alpha = |name: &str| {
    Rule::groups(
      &format!(r"^([[:space:]]*{name}:[[:space:]]*)rgba\([^,]*,[^,]*,[^,]*, *([^),]*)\)"),
      format!("${{1}}{}", accent.rgba("${2}")),
    )
  };
  vec![
    Rule::literal(
      r"^[[:space:]]*th-fg:.*",
      format!("    th-fg:            {};", accent.rgb()),
    ),
    rgba_keep_alpha("th-row-alt"),
    rgba_keep_alpha("th-border-color"),
    Rule::groups(
      r"^([[:space:]]*th-border-color:[[:space:]]*)rgb\([^,)]*,[^,)]*,[^,)]*\)",
      format!("${{1}}{}", accent.rgb()),
    ),
  ]
}

fn quickshell_rules(accent: &Accent) -> Vec<Rule> {
  let hex = accent.hex();
  // The values start in column 17 after the indentation, as the sed lines did.
  let property = |name: &str, value: String| {
    Rule::literal(
      &format!(r"^[[:space:]]*readonly property color {name}:.*"),
      format!(
        "    readonly property color {:<17}\"{value}\"",
        format!("{name}:")
      ),
    )
  };
  vec![
    property("accent", accent.color.clone()),
    property("accentDim", format!("#22{hex}")),
    property("accentMid", format!("#55{hex}")),
    property("accentFaint", format!("#0f{hex}")),
    property("accentLight", accent.color.clone()),
    property("fgTitle", accent.color.clone()),
    property("fgOnAccent", accent.text.to_string()),
    property("bgActive", format!("#22{hex}")),
    property("border", format!("#22{hex}")),
    property("borderStrong", format!("#55{hex}")),
    property("borderItem", format!("#0f{hex}")),
  ]
}

/// The qt6ct palette: the highlight entries of the three color groups take the
/// accent, as the awk program did.
pub fn qt_palette(text: &str, accent: &Accent) -> String {
  text
    .split_inclusive('\n')
    .map(|line| {
      let (content, _) = line
        .strip_suffix('\n')
        .map_or((line, ""), |content| (content, "\n"));
      let group = ["active_colors=", "disabled_colors=", "inactive_colors="]
        .into_iter()
        .find(|prefix| content.starts_with(prefix));
      let Some(_) = group else {
        return format!("{content}\n");
      };
      let equals = content.find('=').unwrap_or(0);
      let (prefix, values) = content.split_at(equals + 1);
      let mut colors: Vec<String> = if values.is_empty() {
        Vec::new()
      } else {
        values.split(", ").map(str::to_string).collect()
      };
      for position in [3usize, 8, 13, 16] {
        if position <= colors.len() {
          colors[position - 1] = format!("#ff{}", accent.hex());
        }
      }
      format!("{prefix}{}\n", colors.join(", "))
    })
    .collect()
}

/// The dunst sections whose frame and highlight take the accent.
const DUNST_SECTIONS: [&str; 10] = [
  "global",
  "urgency_low",
  "urgency_normal",
  "urgency_critical",
  "hyprshot",
  "volume",
  "gpu-screen-recorder",
  "network",
  "spotify",
  "discord",
];

/// Sets `frame_color` and `highlight` in the dunst sections, as the awk program
/// did. A key line is rewritten only inside one of the sections.
pub fn dunst_accent(text: &str, accent: &Accent) -> String {
  let mut section = String::new();
  text
    .split_inclusive('\n')
    .map(|line| {
      let (content, ending) = line
        .strip_suffix('\n')
        .map_or((line, ""), |content| (content, "\n"));
      if content.starts_with('[') {
        section = content.to_string();
        return format!("{content}{ending}");
      }
      let in_target = DUNST_SECTIONS
        .iter()
        .any(|name| section == format!("[{name}]"));
      if in_target {
        for key in ["frame_color", "highlight"] {
          let is_key = content
            .trim_start_matches([' ', '\t'])
            .strip_prefix(key)
            .map(|rest| rest.trim_start_matches([' ', '\t']).starts_with('='))
            .unwrap_or(false);
          if is_key {
            return format!("    {key} = \"{}\"{ending}", accent.color);
          }
        }
      }
      format!("{content}{ending}")
    })
    .collect()
}

/// Replaces the old theme accent in a yazi flavor, in any letter case, as the
/// per-digit pattern did.
pub fn replace_old_accent(text: &str, old_accent: &str, accent: &Accent) -> String {
  let hex = old_accent.trim_start_matches('#');
  if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    return text.to_string();
  }
  let pattern = Regex::new(&format!("(?i)#{hex}")).expect("validated hex is a valid pattern");
  text
    .split_inclusive('\n')
    .map(|line| {
      pattern
        .replace_all(line, regex::NoExpand(&accent.color))
        .into_owned()
    })
    .collect()
}

/// The `[flavor]` table of the yazi theme file that selects the theme.
pub fn yazi_theme_file(theme: &str) -> String {
  format!("[flavor]\ndark = \"{theme}\"\n")
}

/// Everything the apply step needs from its caller.
pub struct Context<'a> {
  pub paths: &'a SessionPaths,
  pub themes_dir: &'a Path,
  pub accent: &'a Accent,
  pub theme: &'a str,
  /// Effective GTK and Qt icon theme, from the icon helpers.
  pub icon_theme: &'a str,
  pub icon_default_dark: &'a str,
  pub icon_default_light: &'a str,
  pub runtime: bool,
  pub theme_switch: bool,
  pub greeter_state_dir: &'a Path,
  pub cache_home: &'a Path,
  pub xdg_config_home: &'a Path,
}

/// The user copy of a packaged file, when it can be made. A failed copy leaves
/// the file out, as the shell's empty path did.
fn live(context: &Context, relative: &str) -> Option<PathBuf> {
  user_copy::config_path(context.paths, relative)
}

/// Applies the accent to every surface. Each surface is independent: a failure
/// is reported on stderr and the next surface still runs, as the shell did.
pub fn apply(context: &Context) {
  report(apply_theme_references(context));
  report(apply_waybar(context));
  report(apply_control_center(context));
  report(apply_rofi(context));
  report(apply_mode_overlays(context));
  report(apply_qt_palette(context));
  report(apply_quickshell(context));
  report(apply_application_colors(context));
  if context.runtime {
    refresh_runtime(context);
  }
}

fn report(result: io::Result<()>) {
  if let Err(error) = result {
    eprintln!("argvus-config: accent: {error}");
  }
}

fn apply_theme_references(context: &Context) -> io::Result<()> {
  let theme = context.theme;
  let rofi_config = live(context, "launcher/config/config.rasi");
  let rofi_theme_file = live(context, "launcher/config/theme.rasi");
  let rofi_mode = live(context, "launcher/config/mode.rasi");
  let rofi_theme = live(
    context,
    &format!("launcher/config/themes/{theme}/theme.rasi"),
  );
  let rofi_theme_path = rofi_theme_file.clone().unwrap_or_default();

  edit_file(
    live(context, "taskbar/config/argvus-taskbar.css").as_deref(),
    |text| {
      apply_rules(
        text,
        &[Rule::groups(
          r#"@import url\("\./themes/.*/theme\.css"\);"#,
          format!(r#"@import url("./themes/{theme}/theme.css");"#),
        )],
      )
    },
  )?;
  edit_file(
    live(
      context,
      "widget-telemetry/config/argvus-widget-telemetry.css",
    )
    .as_deref(),
    |text| {
      apply_rules(
        text,
        &[Rule::groups(
          r#"@import url\("\./themes/.*/widget-telemetry-theme\.css"\);"#,
          format!(r#"@import url("./themes/{theme}/widget-telemetry-theme.css");"#),
        )],
      )
    },
  )?;
  edit_file(rofi_config.as_deref(), |text| {
    apply_rules(
      text,
      &[Rule::literal(
        r#"^@theme ".*theme.rasi"$"#,
        format!("@theme \"{}\"", rofi_theme_path.display()),
      )],
    )
  })?;
  edit_file(rofi_theme_file.as_deref(), |text| {
    let mut edited = apply_rules(
      text,
      &[Rule::literal(
        r#"^@import ".*themes/.*/theme.rasi"$"#,
        format!(
          "@import \"{}\"",
          rofi_theme.as_deref().unwrap_or(Path::new("")).display()
        ),
      )],
    );
    edited = apply_rules(
      &edited,
      &[Rule::literal(
        r#"^@import ".*mode.rasi"$"#,
        format!(
          "@import \"{}\"",
          rofi_mode.as_deref().unwrap_or(Path::new("")).display()
        ),
      )],
    );
    edited
  })?;
  // The terminal and monitor palettes follow the theme. The shell ran these
  // only when the command existed, and ignored the result.
  command_output("argvus", &["terminal", "--apply", theme]);
  command_output("argvus", &["system-monitor", "--apply", theme]);
  let superfile = live(context, "app-profiles/config/superfile");
  replace_setting(
    superfile
      .as_ref()
      .map(|dir| dir.join("config.toml"))
      .as_deref(),
    "theme",
    &format!("\"{theme}\""),
  )?;
  let qt6ct = live(context, "appearance/config/qt6ct/qt6ct.conf");
  let colors_file = live(
    context,
    &format!("appearance/config/qt6ct/colors/{theme}.conf"),
  );
  replace_setting(
    qt6ct.as_deref(),
    "color_scheme_path",
    &colors_file.unwrap_or_default().display().to_string(),
  )?;
  sync_native_qt6ct(context, qt6ct.as_deref())?;
  let yazi = live(context, "app-profiles/config/yazi").filter(|yazi| {
    yazi
      .join("flavors")
      .join(format!("{theme}.yazi"))
      .join("flavor.toml")
      .is_file()
  });
  if let Some(yazi) = yazi {
    fs::write(yazi.join("theme.toml"), yazi_theme_file(theme))?;
  }
  Ok(())
}

/// Keeps the user's own qt6ct file in step. The file is taken over only when it
/// is absent or was generated by ARGVUS; any other icon theme is a user choice.
fn sync_native_qt6ct(context: &Context, managed: Option<&Path>) -> io::Result<()> {
  let Some(managed) = managed else {
    return Ok(());
  };
  let native = context.xdg_config_home.join("qt6ct/qt6ct.conf");
  if native == managed {
    return Ok(());
  }
  if !native.exists() {
    if let Some(parent) = native.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::copy(managed, &native)?;
    return Ok(());
  }
  let text = fs::read_to_string(&native)?;
  let lines: Vec<&str> = text.lines().collect();
  let owns_icon = lines.contains(&format!("icon_theme={}", context.icon_default_dark).as_str())
    || lines.contains(&format!("icon_theme={}", context.icon_default_light).as_str())
    || lines.contains(&format!("icon_theme={}", context.icon_theme).as_str());
  let owns_scheme = lines.iter().any(|line| {
    line
      .strip_prefix("color_scheme_path")
      .map(|rest| {
        rest.trim_start().starts_with('=')
          && (rest.contains("argvus/colors/argvus-")
            || rest.contains("appearance/config/qt6ct/colors/argvus-"))
      })
      .unwrap_or(false)
  });
  if !(owns_icon || owns_scheme) {
    return Ok(());
  }
  let colors = live(
    context,
    &format!("appearance/config/qt6ct/colors/{}.conf", context.theme),
  )
  .unwrap_or_default();
  replace_setting(
    Some(&native),
    "color_scheme_path",
    &colors.display().to_string(),
  )?;
  replace_setting(Some(&native), "custom_palette", "true")?;
  let icon = context.icon_theme.to_string();
  let rules = [Rule::literal(
    r"^[[:space:]]*icon_theme[[:space:]]*=.*",
    format!("icon_theme={icon}"),
  )];
  edit_file(Some(&native), |text| apply_rules(text, &rules))
}

fn apply_waybar(context: &Context) -> io::Result<()> {
  let theme_css = live(
    context,
    &format!("taskbar/config/themes/{}/theme.css", context.theme),
  );
  edit_file(theme_css.as_deref(), |text| {
    apply_rules(text, &waybar_rules(context.accent))
  })?;
  let telemetry = live(
    context,
    &format!(
      "widget-telemetry/config/themes/{}/widget-telemetry-theme.css",
      context.theme
    ),
  );
  edit_file(telemetry.as_deref(), |text| {
    apply_rules(text, &telemetry_rules(context.accent))
  })
}

fn apply_control_center(context: &Context) -> io::Result<()> {
  let Some(source) = live(
    context,
    &format!("appearance/config/tui/themes/{}.css", context.theme),
  ) else {
    return Ok(());
  };
  if !source.is_file() {
    return Ok(());
  }
  let cache = context.cache_home.join("argvus-control-center/theme.css");
  if let Some(parent) = cache.parent() {
    fs::create_dir_all(parent)?;
  }
  let accent = context.accent;
  let text = format!(
    "@define-color argvus_accent {};\n@define-color argvus_accent_alpha rgba({}, {}, {}, 0.45);\n",
    accent.color, accent.red, accent.green, accent.blue
  );
  let temporary = cache.with_extension(format!("css.{}", std::process::id()));
  fs::write(&temporary, text)?;
  fs::rename(&temporary, &cache)
}

fn apply_rofi(context: &Context) -> io::Result<()> {
  edit_file(
    live(
      context,
      &format!("launcher/config/themes/{}/theme.rasi", context.theme),
    )
    .as_deref(),
    |text| apply_rules(text, &rofi_theme_rules(context.accent)),
  )
}

fn apply_mode_overlays(context: &Context) -> io::Result<()> {
  edit_file(
    live(context, "appearance/config/waybar/mode.css").as_deref(),
    |text| apply_rules(text, &mode_css_rules(context.accent)),
  )?;
  edit_file(
    live(context, "launcher/config/mode.rasi").as_deref(),
    |text| apply_rules(text, &mode_rasi_rules(context.accent)),
  )
}

fn apply_qt_palette(context: &Context) -> io::Result<()> {
  let file = live(
    context,
    &format!("appearance/config/qt6ct/colors/{}.conf", context.theme),
  );
  edit_file(file.as_deref(), |text| qt_palette(text, context.accent))
}

fn apply_quickshell(context: &Context) -> io::Result<()> {
  let file = live(
    context,
    &format!(
      "control-panel/config/quickshell/argvus-control-panel/themes/{}/Theme.qml",
      context.theme
    ),
  );
  edit_file(file.as_deref(), |text| {
    apply_rules(text, &quickshell_rules(context.accent))
  })
}

fn apply_application_colors(context: &Context) -> io::Result<()> {
  let accent = context.accent;
  let hyprlock = live(context, "lock/config/hyprlock.conf");
  edit_file(hyprlock.as_deref(), |text| {
    apply_rules(
      text,
      &[Rule::literal(
        r"^[[:space:]]*outer_color = .*",
        format!("  outer_color = rgb({})", accent.hex()),
      )],
    )
  })?;
  let toolkit = live(context, "appearance/config/hypr/hyprtoolkit.conf");
  edit_file(toolkit.as_deref(), |text| {
    apply_rules(
      text,
      &[
        Rule::literal(
          r"^bright_text = .*",
          format!("bright_text = 0xFF{}", accent.hex()),
        ),
        Rule::literal(r"^accent = .*", format!("accent = 0xFF{}", accent.hex())),
        Rule::literal(
          r"^link_text = .*",
          format!("link_text = 0xFF{}", accent.hex()),
        ),
      ],
    )
  })?;
  let dunst = live(context, "notifications/config/dunstrc");
  edit_file(dunst.as_deref(), |text| dunst_accent(text, accent))?;
  let yazi_flavor = live(
    context,
    &format!(
      "app-profiles/config/yazi/flavors/{}.yazi/flavor.toml",
      context.theme
    ),
  );
  if let (Some(flavor), Some(old)) = (
    yazi_flavor.as_deref(),
    theme_default_accent(context.themes_dir, context.theme),
  ) {
    edit_file(Some(flavor), |text| replace_old_accent(text, &old, accent))?;
  }
  command_output("argvus", &["system-monitor", "--apply", context.theme]);
  let superfile = live(
    context,
    &format!("app-profiles/config/superfile/theme/{}.toml", context.theme),
  );
  let mut rules: Vec<Rule> = [
    "file_panel_border_active",
    "file_panel_top_directory_icon",
    "footer_border_active",
    "sidebar_title",
    "sidebar_border_active",
    "modal_border_active",
    "modal_confirm_bg",
    "help_menu_hotkey",
    "correct",
    "hint",
  ]
  .into_iter()
  .map(|key| {
    Rule::literal(
      &format!(r"^{key}[[:space:]]*=.*"),
      format!("{key} = \"{}\"", accent.color),
    )
  })
  .collect();
  rules.push(Rule::literal(
    r##"^gradient_color = \["#[0-9A-Fa-f]*","##,
    format!("gradient_color = [\"{}\",", accent.color),
  ));
  rules.push(Rule::literal(
    r"^modal_confirm_fg = .*",
    format!("modal_confirm_fg = \"{}\"", accent.text),
  ));
  edit_file(superfile.as_deref(), |text| apply_rules(text, &rules))?;
  if let Some(superfile) = superfile.filter(|path| path.is_file()) {
    let native_superfile = context.paths.config_home.join("superfile");
    if native_superfile.join("config.toml").is_file() && native_superfile.join("theme").is_dir() {
      fs::copy(
        &superfile,
        native_superfile
          .join("theme")
          .join(format!("{}.toml", context.theme)),
      )?;
    }
  }
  Ok(())
}

/// Writes the accent for the greeter, in the file named by the user id.
pub fn publish_greeter_accent(greeter_state_dir: &Path, accent: &Accent) -> io::Result<()> {
  if !greeter_state_dir.is_dir() {
    return Ok(());
  }
  let user_id = crate::theme_surfaces::current_user_id();
  let target = greeter_state_dir.join(format!("{user_id}.accent"));
  let temporary = greeter_state_dir.join(format!("{user_id}.accent.tmp.{}", std::process::id()));
  let written = fs::write(&temporary, format!("{}\n", accent.color)).and_then(|()| {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o644));
    fs::rename(&temporary, &target)
  });
  if written.is_err() {
    let _ = fs::remove_file(&temporary);
  }
  Ok(())
}

/// Tells the running components about the new accent. The shell ran each of
/// these only when the command existed; so does this.
fn refresh_runtime(context: &Context) {
  command_output("argvus", &["config", "hyprland", "reload"]);
  command_output("argvus", &["terminal", "--apply", context.theme]);
  if !context.theme_switch {
    command_output(
      "argvus",
      &["sessionctl", "restart", "waybar", "dunst", "shell"],
    );
  }
  if let Some(pids) = command_output("pgrep", &["-x", "kitty"]) {
    for pid in pids.lines() {
      if let Ok(pid) = pid.trim().parse::<i32>() {
        // SAFETY: SIGUSR1 to a pid that pgrep just reported.
        unsafe {
          libc::kill(pid, libc::SIGUSR1);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn colors_are_normalized_and_the_text_color_contrasts() {
    let accent = normalize("#3590bd").expect("valid");
    assert_eq!(accent.color, "#3590BD");
    assert_eq!(accent.text, "#FFFFFF");
    assert_eq!(normalize("ffffff").expect("valid").text, "#000000");
    assert!(normalize("#3590B").is_none(), "five digits");
    assert!(normalize("#3590BG").is_none(), "not hex");
    assert!(normalize("##3590BD").is_none(), "two hashes");
  }

  #[test]
  fn legacy_theme_ids_map_to_registered_ones() {
    let themes = Path::new("/nonexistent");
    assert_eq!(applied_theme(themes, "frost-float"), "frost-float");
    assert_eq!(
      applied_theme(themes, "custom-dark-float"),
      "argvus-dark-float"
    );
    assert_eq!(applied_theme(themes, "custom-light"), "argvus-light");
    assert_eq!(applied_theme(themes, "unknown"), DEFAULT_THEME);
  }

  #[test]
  fn highlight_rules_keep_alpha_of_the_mode_overlays() {
    let accent = normalize("#0969DA").expect("valid");
    let text = "@define-color th-decorate-rgba rgba(1, 2, 3, 0.25);\n";
    let edited = apply_rules(text, &mode_css_rules(&accent));
    assert_eq!(
      edited,
      "@define-color th-decorate-rgba rgba(9, 105, 218, 0.25);\n"
    );
  }

  #[test]
  fn qt_palette_changes_only_the_highlight_entries() {
    let accent = normalize("#0969DA").expect("valid");
    let text = "active_colors=a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p\nother=x\n";
    let edited = qt_palette(text, &accent);
    assert_eq!(
      edited,
      "active_colors=a, b, #ff0969DA, d, e, f, g, #ff0969DA, i, j, k, l, #ff0969DA, n, o, #ff0969DA\nother=x\n"
    );
  }

  #[test]
  fn dunst_changes_only_its_sections() {
    let accent = normalize("#0969DA").expect("valid");
    let text = "[global]\n    frame_color = \"#111111\"\n[other]\n    frame_color = \"#222222\"\n";
    let edited = dunst_accent(text, &accent);
    assert_eq!(
      edited,
      "[global]\n    frame_color = \"#0969DA\"\n[other]\n    frame_color = \"#222222\"\n"
    );
  }

  #[test]
  fn old_yazi_accent_is_replaced_in_any_case() {
    let accent = normalize("#0969DA").expect("valid");
    let text = "a = \"#3590bd\"\nb = \"#3590BD\"\nc = \"#111111\"\n";
    assert_eq!(
      replace_old_accent(text, "#3590bd", &accent),
      "a = \"#0969DA\"\nb = \"#0969DA\"\nc = \"#111111\"\n"
    );
  }
}
