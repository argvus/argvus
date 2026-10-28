//! Stylesheets of the calendar: the packaged and user CSS, the active ARGVUS theme and the accent overrides.

use super::*;

pub(super) fn file_fingerprint(path: &Path) -> (i64, u64) {
  let Ok(metadata) = std::fs::metadata(path) else {
    return (0, 0);
  };
  let mtime = metadata
    .modified()
    .ok()
    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
    .map_or(0, |duration| duration.as_nanos() as i64);
  (mtime, metadata.len())
}

pub(super) fn theme_fingerprint(paths: &Paths) -> ThemeFingerprint {
  let (cache_mtime, cache_len) = file_fingerprint(&paths.cache_theme_file);
  let (active_mtime, active_len) = file_fingerprint(&paths.active_argvus_theme_file);
  let accent_file = paths
    .active_argvus_theme_file
    .parent()
    .map(|parent| parent.join(".accent-color"))
    .unwrap_or_default();
  let (accent_mtime, accent_len) = file_fingerprint(&accent_file);
  ThemeFingerprint {
    cache_mtime,
    cache_len,
    active_mtime,
    active_len,
    accent_mtime,
    accent_len,
  }
}

pub(super) fn load_css(paths: &Paths, providers: &Rc<RefCell<Vec<gtk::CssProvider>>>) {
  if let Some(display) = gtk::gdk::Display::default() {
    for old in providers.borrow_mut().drain(..) {
      gtk::style_context_remove_provider_for_display(&display, &old);
    }

    let mut new_providers = Vec::new();
    for (priority, stylesheet) in stylesheet_paths(paths).into_iter().enumerate() {
      if stylesheet.exists() {
        let provider = gtk::CssProvider::new();
        provider.load_from_path(stylesheet);
        gtk::style_context_add_provider_for_display(
          &display,
          &provider,
          gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + priority as u32,
        );
        new_providers.push(provider);
      }
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_string(&runtime_css(&paths.config_file, paths));
    gtk::style_context_add_provider_for_display(
      &display,
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 10,
    );
    new_providers.push(provider);

    *providers.borrow_mut() = new_providers;
  }
}

pub(super) fn stylesheet_paths(paths: &Paths) -> Vec<std::path::PathBuf> {
  let mut stylesheets = vec![paths.user_style.clone(), paths.theme_file.clone()];
  let active_theme_id = active_theme_id(paths);
  if cache_theme_is_current(paths, active_theme_id.as_deref()) {
    stylesheets.push(paths.cache_theme_file.clone());
  }
  stylesheets
}

pub(super) fn active_theme_id(paths: &Paths) -> Option<String> {
  let theme = std::fs::read_to_string(&paths.active_argvus_theme_file).ok()?;
  let theme = theme.trim();
  (!theme.is_empty()).then(|| theme.to_string())
}

pub(super) fn cache_theme_is_current(paths: &Paths, active_theme: Option<&str>) -> bool {
  let Ok(cache_css) = std::fs::read_to_string(&paths.cache_theme_file) else {
    return false;
  };
  let Some(marker) = cache_css.lines().next().and_then(|line| {
    line
      .trim()
      .strip_prefix("/* argvus-theme: ")
      .and_then(|value| value.strip_suffix(" */"))
  }) else {
    return active_theme.is_none();
  };
  active_theme.is_some_and(|theme| theme == marker)
}

pub(super) fn runtime_css(_config_path: &std::path::Path, paths: &Paths) -> String {
  let settings = Settings::load(paths).unwrap_or_default();
  let family = css_string(&settings.appearance.font_family);
  let size = settings.appearance.font_size.clamp(8, 32);
  format!(
    ".argvus-calendar, .event-editor {{ font-family: \"{family}\", monospace; font-size: {size}px; }}\n\
         window.argvus-calendar-window, .argvus-calendar-window {{ background-color: transparent; background-image: none; box-shadow: none; }}\n{}",
    accent_runtime_css(paths)
  )
}

pub(super) fn accent_runtime_css(paths: &Paths) -> String {
  let accent_file = paths
    .active_argvus_theme_file
    .parent()
    .map(|parent| parent.join(".accent-color"));
  let Some(accent_file) = accent_file else {
    return String::new();
  };
  let Ok(value) = std::fs::read_to_string(accent_file) else {
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
    ".today-button, .add-event, .weekday, .day-current, .editor-label, .editor-save, .events-toggle:checked {{ color: {value}; }}\n\
     .nav-button:hover, .add-event:hover, .day:hover, .event-button:hover, .editor-action:hover, .agenda row:hover, .event-row:hover {{ background-color: rgba({red}, {green}, {blue}, 0.16); border-color: {value}; }}\n\
     .nav-button:focus, .add-event:focus, .day:focus, .event-button:focus, .editor-action:focus, .editor-entry:focus, .event-editor entry:focus, .day-selected {{ border-color: {value}; }}\n\
     .day-current.day-selected {{ background-color: rgba({red}, {green}, {blue}, 0.22); }}\n\
     .event-divider {{ color: rgba({red}, {green}, {blue}, 0.54); }}\n\
     .day-selected, .editor-save {{ color: {text}; }}"
  )
}

pub(super) fn css_string(value: &str) -> String {
  value
    .chars()
    .filter(|ch| !matches!(ch, '"' | '\\' | '\n' | '\r'))
    .collect()
}
