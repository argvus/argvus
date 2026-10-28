//! Rewrites each surface that consumes the effects: Waybar, widget telemetry, launcher, dunst, calendar, superfile and hyprlock.

use super::css_text::{
  apply_border_transparency, delete_range, ensure_waybar_namespace, imported_build_theme,
  insert_after, launcher_background, opacity_factor, replace_in_lines, set_define,
};
use super::*;

impl Effects {
  pub(super) fn apply_taskbar_surface(&self, animations: &str) -> io::Result<()> {
    let css = self.live("taskbar/config/argvus-taskbar.css");
    ensure_waybar_namespace(
      self.live("taskbar/config/argvus-taskbar.jsonc").as_deref(),
      "argvus-taskbar",
    )?;
    let Some(css) = css.filter(|path| path.is_file()) else {
      return Ok(());
    };
    let mut text = fs::read_to_string(&css)?;
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color th-background-effective "))
    {
      text = insert_after(
        &text,
        "@define-color transparent transparent;",
        "@define-color th-background-effective @th-background-rgba;",
      );
      text = replace_in_lines(
        &text,
        "background: @th-background-rgba;",
        "background: @th-background-effective;",
      );
    }
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color th-mpris-bg-effective "))
    {
      text = insert_after(
        &text,
        "@define-color th-background-effective ",
        "@define-color th-mpris-bg-effective @th-mpris-bg;",
      );
      text = replace_in_lines(
        &text,
        "background: @th-mpris-bg;",
        "background: @th-mpris-bg-effective;",
      );
    }
    if self.effect_enabled("taskbar.transparency") == "disabled" {
      text = set_define(&text, "th-background-effective", "@th-background;");
      text = set_define(&text, "th-mpris-bg-effective", "@th-right1-bg;");
    } else {
      let opacity = opacity_factor(self.effect_value("taskbar.transparency"));
      text = set_define(
        &text,
        "th-background-effective",
        &format!("alpha(@th-background, {opacity});"),
      );
      text = set_define(
        &text,
        "th-mpris-bg-effective",
        &format!("alpha(@th-mpris-bg, {opacity});"),
      );
    }
    apply_border_transparency(
      &css,
      &mut text,
      &["th-border-rights", "th-mpris-border"],
      "th-background",
      "@define-color th-mpris-bg-effective ",
      self.effect_enabled("taskbar.transparency") != "disabled",
    );
    const MARK_START: &str = "/* ARGVUS animations disabled */";
    const MARK_END: &str = "/* ARGVUS animations disabled end */";
    if animations == "disabled" {
      if !text.lines().any(|line| line == MARK_START) {
        text.push_str(&format!(
                    "{MARK_START}\n#workspaces button, #right-2, #custom-expand-icon {{\n  transition: none;\n}}\n#mpris, #custom-spotify-mpris, #custom-recording.recording {{\n  animation: none;\n}}\n{MARK_END}\n"
                ));
      }
    } else {
      text = delete_range(&text, MARK_START, MARK_END);
    }
    fs::write(&css, text)
  }

  pub(super) fn apply_widget_telemetry_surface(&self) -> io::Result<()> {
    let css = self.live("widget-telemetry/config/argvus-widget-telemetry.css");
    ensure_waybar_namespace(
      self
        .live("widget-telemetry/config/argvus-widget-telemetry.jsonc")
        .as_deref(),
      "argvus-widget-telemetry",
    )?;
    let Some(css) = css.filter(|path| path.is_file()) else {
      return Ok(());
    };
    let mut text = fs::read_to_string(&css)?;
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color th-window-bg-effective "))
    {
      text = insert_after(
        &text,
        "/*  Waybar — Painel vertical",
        "@define-color th-window-bg-effective @th-window-bg-rgba;",
      );
      text = replace_in_lines(
        &text,
        "background: @th-window-bg;",
        "background: @th-window-bg-effective;",
      );
    }
    if self.effect_enabled("widget-telemetry.transparency") == "disabled" {
      text = set_define(&text, "th-window-bg-effective", "@th-window-bg;");
    } else {
      let opacity = opacity_factor(self.effect_value("widget-telemetry.transparency"));
      text = set_define(
        &text,
        "th-window-bg-effective",
        &format!("alpha(@th-window-bg, {opacity});"),
      );
    }
    apply_border_transparency(
      &css,
      &mut text,
      &["th-border", "th-border-header"],
      "th-window-bg",
      "@define-color th-window-bg-effective ",
      self.effect_enabled("widget-telemetry.transparency") != "disabled",
    );
    fs::write(&css, text)
  }

  pub(super) fn apply_launcher_surface(&self) -> io::Result<()> {
    let Some(theme) = self
      .live("launcher/config/theme.rasi")
      .filter(|path| path.is_file())
    else {
      return Ok(());
    };
    // Without transparency the launcher is opaque. With it, the alpha is the
    // inverse of the transparency, rounded as the shell rounded it: the opacity
    // is printed with two decimals first.
    let opacity: f64 = if self.effect_enabled("launchers.transparency") == "disabled" {
      1.0
    } else {
      opacity_factor(self.effect_value("launchers.transparency"))
        .parse()
        .unwrap_or(1.0)
    };
    // The colour always comes from the build the theme file imports, so every
    // theme, official or the user's own, gets its own background in both modes.
    let text = fs::read_to_string(&theme)?;
    let build_theme = imported_build_theme(&theme, &text)
      .and_then(|path| fs::read_to_string(path).ok())
      .unwrap_or_default();
    let background = launcher_background(&build_theme, opacity);
    let rules = [Rule::literal(
      r"^[[:space:]]*bg:[[:space:]].*;",
      format!("    bg:               {background};"),
    )];
    edit_file(Some(&theme), |text| apply_rules(text, &rules))
  }

  pub(super) fn apply_dunst_surface(&self, status: &str) -> io::Result<()> {
    let Some(dunstrc) = self
      .live("notifications/config/dunstrc")
      .filter(|path| path.is_file())
    else {
      return Ok(());
    };
    let backup = self.state_home.join("effects-dunst-transparency");
    let text = fs::read_to_string(&dunstrc)?;
    if status == "disabled" {
      if !backup.is_file() {
        let current = text
          .lines()
          .find_map(|line| {
            let rest = line.trim_start().strip_prefix("transparency")?.trim_start();
            let digits: String = rest
              .strip_prefix('=')?
              .trim_start()
              .chars()
              .take_while(char::is_ascii_digit)
              .collect();
            (!digits.is_empty()).then_some(digits)
          })
          .filter(|digits| digits.chars().all(|digit| digit.is_ascii_digit()))
          .unwrap_or_else(|| "10".to_string());
        if let Some(parent) = backup.parent() {
          fs::create_dir_all(parent)?;
        }
        fs::write(&backup, format!("{current}\n"))?;
      }
      let rules = [Rule::literal(
        r"^[[:space:]]*transparency[[:space:]]*=.*",
        "    transparency = 0".to_string(),
      )];
      return fs::write(&dunstrc, apply_rules(&text, &rules));
    }
    let current = fs::read_to_string(&backup)
      .ok()
      .and_then(|value| value.lines().next().map(str::to_string))
      .filter(|value| !value.is_empty() && value.chars().all(|digit| digit.is_ascii_digit()))
      .unwrap_or_else(|| "10".to_string());
    let rules = [Rule::literal(
      r"^[[:space:]]*transparency[[:space:]]*=.*",
      format!("    transparency = {current}"),
    )];
    fs::write(&dunstrc, apply_rules(&text, &rules))?;
    let _ = fs::remove_file(&backup);
    Ok(())
  }

  pub(super) fn apply_calendar_surface(&self, status: &str) -> io::Result<()> {
    let cache = env::var_os("XDG_CACHE_HOME")
      .filter(|value| !value.is_empty())
      .map(PathBuf::from)
      .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".cache"));
    let css = cache.join("argvus-taskbar-calendar/theme.css");
    if !css.is_file() {
      return Ok(());
    }
    let mut text = fs::read_to_string(&css)?;
    if !text
      .lines()
      .any(|line| line.starts_with("@define-color argvus_bg_effective "))
    {
      text = insert_after(
        &text,
        "@define-color argvus_bg_alpha ",
        "@define-color argvus_bg_effective @argvus_bg_alpha;",
      );
      text = replace_in_lines(
        &text,
        "background: @argvus_bg_alpha;",
        "background: @argvus_bg_effective;",
      );
    }
    let value = if status == "disabled" {
      "@argvus_bg;"
    } else {
      "@argvus_bg_alpha;"
    };
    text = set_define(&text, "argvus_bg_effective", value);
    fs::write(&css, text)
  }

  pub(super) fn apply_superfile_surface(&self, status: &str) -> io::Result<()> {
    let Some(config) = self
      .live("app-profiles/config/superfile/config.toml")
      .filter(|path| path.is_file())
    else {
      return Ok(());
    };
    let transparent = if status == "disabled" {
      "false"
    } else {
      "true"
    };
    let text = fs::read_to_string(&config)?;
    if text.lines().any(|line| {
      line.trim_start().starts_with("transparent_background")
        && line.trim_start()["transparent_background".len()..]
          .trim_start()
          .starts_with('=')
    }) {
      let rules = [Rule::literal(
        r"^[[:space:]]*transparent_background[[:space:]]*=.*",
        format!("transparent_background = {transparent}"),
      )];
      return fs::write(&config, apply_rules(&text, &rules));
    }
    let mut output = text;
    output.push_str(&format!("\ntransparent_background = {transparent}\n"));
    fs::write(&config, output)
  }

  /// Runs the hyprlock theme helper, when the package has one.
  pub(super) fn apply_hyprlock_effects(&self) {
    if let Some(script) = self
      .live("lock/sh/hyprlock-theme.sh")
      .filter(|path| path.is_file())
    {
      let _ = Command::new("sh")
        .arg(script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
  }

  pub(super) fn apply_surfaces(&self, transparency: &str, animations: &str) -> io::Result<()> {
    self.apply_taskbar_surface(animations)?;
    self.apply_widget_telemetry_surface()?;
    self.apply_launcher_surface()?;
    self.apply_dunst_surface(transparency)?;
    self.apply_calendar_surface(transparency)?;
    self.apply_superfile_surface(transparency)?;
    self.apply_hyprlock_effects();
    Ok(())
  }

  /// The services and palettes that follow an effects change. It runs in the
  /// background after the command answered, as the shell did.
  pub(super) fn apply_runtime(&self) {
    if on_path("argvus") {
      let _ = Command::new("argvus")
        .args(["config", "hyprland", "reload"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    // The bars, the Control Panel and Dunst reload their files in place:
    // restarting them would move every window.
    if on_path("argvus") {
      let _ = Command::new("argvus")
        .args(["config", "theme-runtime", "consumers"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    let calendar = self.commands_dir.join("taskbar-calendar");
    if calendar.is_file() {
      let _ = Command::new(&calendar)
        .arg("reload")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    if on_path("argvus") {
      let _ = Command::new("argvus")
        .args(["terminal", "--apply", &self.active_theme()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
      signal_kitty();
    }
  }

  pub(super) fn rematerialize_control_center(&self) {
    // The launcher is the `argvus` command itself; without it nothing is rewritten.
    {
      let _ = Command::new("argvus")
        .args([
          "tui-terminal",
          "--materialize-kitty",
          "--profile",
          "control-center",
          "--class",
          "argvus-control-center",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    }
    signal_kitty();
  }
}

/// Writes the launcher background of the current build into the user's Rofi
/// theme. Rofi reads its background from that file, not from the build, so a
/// theme switch calls this as soon as the build is active: a launcher opened
/// while the rest of the switch runs then shows the new theme. Failures are
/// reported on stderr; the full `apply` at the end of the switch writes the
/// file again.
pub(crate) fn apply_launcher() {
  let Some(effects) = Effects::from_environment() else {
    eprintln!("argvus-config: cannot resolve the configuration directory");
    return;
  };
  if let Err(error) = effects.apply_launcher_surface() {
    eprintln!("argvus-config: launcher effects: {error}");
  }
}

/// Asks running kitty instances to reload their generated config.
pub(super) fn signal_kitty() {
  // Only this user's kitty instances: other users' processes cannot be signalled
  // anyway, and must not be enumerated or targeted.
  let uid = crate::theme_surfaces::current_user_id().to_string();
  let Ok(output) = Command::new("pgrep")
    .args(["-u", uid.as_str(), "-x", "kitty"])
    .stderr(Stdio::null())
    .output()
  else {
    return;
  };
  for pid in String::from_utf8_lossy(&output.stdout).lines() {
    if let Ok(pid) = pid.trim().parse::<i32>() {
      // SAFETY: SIGUSR1 to a pid that pgrep just reported.
      unsafe {
        libc::kill(pid, libc::SIGUSR1);
      }
    }
  }
}

/// Runs the runtime refresh in a detached process, so the command answers
/// before the components restart.
pub(super) fn spawn_runtime() {
  if let Ok(executable) = env::current_exe() {
    let _ = Command::new(executable)
      .args(["config", "effects", "runtime"])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn();
  }
}
