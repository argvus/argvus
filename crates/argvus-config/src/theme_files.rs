//! The theme files of the applications that are not the desktop shell: the
//! Hyprland toolkit and application style, the qt6ct icon theme, yazi, superfile
//! and the Waybar mode stylesheet.
//!
//! Each step writes only when its source exists, as the shell did. The
//! `theme-hypr` stage runs before the systemd environment is set, and the
//! `theme-tail` stage after the system monitor, so the order of the switch is
//! kept.

use crate::session_paths::SessionPaths;
use crate::text_edit::{self, Layout};
use crate::user_copy;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Copies a source file over `destination`, creating its directory. A missing
/// source is not an error.
fn copy_if_present(source: &Path, destination: &Path) -> io::Result<()> {
  if !source.is_file() {
    return Ok(());
  }
  if let Some(parent) = destination.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::copy(source, destination)?;
  Ok(())
}

/// `argvus config setting` for a spaced setting: replaces the key or appends it,
/// and does nothing when the file is missing.
pub fn set_setting(path: &Path, key: &str, value: &str) -> io::Result<()> {
  if !path.is_file() {
    return Ok(());
  }
  let text = fs::read_to_string(path)?;
  let edited = text_edit::set_setting(&text, key, value, Layout::Spaced);
  if edited != text {
    text_edit::replace_file(path, &edited)?;
  }
  Ok(())
}

/// The Hyprland toolkit and application style of the theme, and the qt6ct icon
/// theme. The toolkit file gets the system font.
pub fn apply_hypr_stage(
  paths: &SessionPaths,
  hypr_themes: &Path,
  theme: &str,
  system_family: &str,
  system_size: &str,
  icon_theme: &str,
) -> io::Result<()> {
  let toolkit_source = hypr_themes.join(theme).join("hyprtoolkit.conf");
  // The user copy is made only when the theme has the file, as the shell did.
  let toolkit = if toolkit_source.is_file() {
    user_copy::config_path(paths, "appearance/config/hypr/hyprtoolkit.conf")
  } else {
    None
  };
  if let Some(toolkit) = toolkit {
    copy_if_present(&toolkit_source, &toolkit)?;
    set_setting(&toolkit, "font_family", &format!("\"{system_family}\""))?;
    set_setting(&toolkit, "font_size", system_size)?;
  }

  let style_source = hypr_themes.join(theme).join("application-style.conf");
  let style = if style_source.is_file() {
    user_copy::config_path(paths, "appearance/config/hypr/application-style.conf")
  } else {
    None
  };
  if let Some(style) = style {
    copy_if_present(&style_source, &style)?;
  }

  // The icon line is rewritten only where qt6ct already has one, and only when
  // the theme has its colors. The theme-file calls of the shell run before it.
  let qt6ct = user_copy::config_path(paths, "appearance/config/qt6ct/qt6ct.conf");
  let colors = user_copy::config_path(
    paths,
    &format!("appearance/config/qt6ct/colors/{theme}.conf"),
  );
  let qt6ct = qt6ct
    .zip(colors)
    .map(|(qt6ct, _)| qt6ct)
    .filter(|qt6ct| qt6ct.is_file());
  if let Some(qt6ct) = qt6ct {
    let text = fs::read_to_string(&qt6ct)?;
    let rules = [crate::accent::Rule::literal(
      r"^[[:space:]]*icon_theme[[:space:]]*=.*",
      format!("icon_theme={icon_theme}"),
    )];
    let edited = crate::accent::apply_rules(&text, &rules);
    if edited != text {
      text_edit::replace_file(&qt6ct, &edited)?;
    }
  }
  Ok(())
}

/// The yazi flavor, superfile and the Waybar mode stylesheet. Returns the flavor
/// file that is missing, so the shell can report it in the user's language.
pub fn apply_tail_stage(paths: &SessionPaths, theme: &str) -> io::Result<Option<PathBuf>> {
  let missing = apply_yazi(paths, theme)?;
  apply_superfile(paths, theme)?;
  if let Some(mode_css) = user_copy::config_path(paths, "appearance/config/waybar/mode.css") {
    if let Some(parent) = mode_css.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(&mode_css, "/* mode.css — reset on theme switch */\n")?;
  }
  Ok(missing)
}

/// Copies the packaged flavor into the user's yazi directory and selects it.
fn apply_yazi(paths: &SessionPaths, theme: &str) -> io::Result<Option<PathBuf>> {
  let yazi_theme = match theme {
    "sunset-float" => "sunset",
    "matrix-float" => "matrix",
    other => other,
  };
  let system_flavor = paths
    .system_config("app-profiles/config/yazi/flavors")
    .join(format!("{yazi_theme}.yazi"));
  let Some(yazi_root) = user_copy::config_path(paths, "app-profiles/config/yazi") else {
    return Ok(None);
  };
  if system_flavor.is_dir() {
    let destination = yazi_root.join("flavors").join(format!("{theme}.yazi"));
    fs::create_dir_all(&destination)?;
    copy_tree(&system_flavor, &destination)?;
  }
  let flavor = yazi_root
    .join("flavors")
    .join(format!("{theme}.yazi"))
    .join("flavor.toml");
  if flavor.is_file() {
    fs::write(
      yazi_root.join("theme.toml"),
      format!("[flavor]\ndark = \"{theme}\"\n"),
    )?;
    Ok(None)
  } else {
    Ok(Some(flavor))
  }
}

/// Copies a directory tree over the destination, as `cp -R src/. dst/` did.
fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
  for entry in fs::read_dir(source)? {
    let entry = entry?;
    let target = destination.join(entry.file_name());
    if entry.file_type()?.is_dir() {
      fs::create_dir_all(&target)?;
      copy_tree(&entry.path(), &target)?;
    } else {
      fs::copy(entry.path(), &target)?;
    }
  }
  Ok(())
}

/// superfile: the theme id in its config, and the theme file in the native
/// superfile directory when that directory exists.
fn apply_superfile(paths: &SessionPaths, theme: &str) -> io::Result<()> {
  let Some(config_root) = user_copy::config_path(paths, "app-profiles/config/superfile") else {
    return Ok(());
  };
  let config = config_root.join("config.toml");
  let theme_source = user_copy::config_path(
    paths,
    &format!("app-profiles/config/superfile/theme/{theme}.toml"),
  );
  let Some(theme_source) = theme_source.filter(|path| path.is_file()) else {
    return Ok(());
  };
  if !config.is_file() {
    return Ok(());
  }
  set_setting(&config, "theme", &format!("\"{theme}\""))?;
  let native = paths.config_home.join("superfile");
  if native.is_dir() {
    fs::create_dir_all(native.join("theme"))?;
    fs::copy(
      &theme_source,
      native.join("theme").join(format!("{theme}.toml")),
    )?;
    let native_config = native.join("config.toml");
    if !native_config.exists() {
      fs::copy(&config, &native_config)?;
    }
    set_setting(&native_config, "theme", &format!("\"{theme}\""))?;
  }
  Ok(())
}
