//! Labels and icons of categories, settings and fonts, and the search match rule.

use super::*;

pub fn search_matches(query: &str, values: &[&str]) -> bool {
  query.is_empty()
    || values
      .iter()
      .any(|value| value.to_lowercase().contains(&query.to_lowercase()))
}

pub fn category_icon(category: Category) -> &'static str {
  match category {
    Category::Terminal => argvus_tui::icons::TERMINAL,
    Category::FileManager => argvus_tui::icons::FOLDER,
    Category::TextEditor => argvus_tui::icons::TEXT_EDITOR,
    Category::TerminalEditor => argvus_tui::icons::KEYBOARD,
    Category::Browser => argvus_tui::icons::EARTH,
    Category::ImageViewer => argvus_tui::icons::IMAGE,
    Category::PdfViewer => argvus_tui::icons::PDF,
    Category::VideoPlayer => argvus_tui::icons::VIDEO,
    Category::AudioPlayer => argvus_tui::icons::MUSIC,
    Category::Archive => argvus_tui::icons::PACKAGES,
    Category::Launcher => argvus_tui::icons::LAUNCHER,
  }
}

pub fn category_label(lang: Lang, category: Category) -> &'static str {
  let key = match category {
    Category::Terminal => "control_center.terminal",
    Category::FileManager => "control_center.file_manager",
    Category::TextEditor => "control_center.text_editor",
    Category::TerminalEditor => "control_center.terminal_editor",
    Category::Browser => "control_center.browser",
    Category::ImageViewer => "control_center.image_viewer",
    Category::PdfViewer => "control_center.pdf_viewer",
    Category::VideoPlayer => "control_center.video_player",
    Category::AudioPlayer => "control_center.audio_player",
    Category::Archive => "control_center.archive",
    Category::Launcher => "control_center.launcher",
  };
  crate::i18n::tr(lang, key)
}

pub fn font_target_icon(target: FontTarget) -> &'static str {
  match target {
    FontTarget::Taskbar => argvus_tui::icons::TASKBAR,
    FontTarget::Sysinfo => argvus_tui::icons::DIAGNOSTICS,
    FontTarget::ControlPanel => argvus_tui::icons::CONTROL_PANEL,
    FontTarget::System => argvus_tui::icons::SETTINGS,
    FontTarget::Apps => argvus_tui::icons::APPS,
    FontTarget::Terminal => argvus_tui::icons::TERMINAL,
    FontTarget::Browser => argvus_tui::icons::EARTH,
  }
}

pub fn setting_icon(setting: SettingKind) -> &'static str {
  match setting {
    SettingKind::Antialiasing => argvus_tui::icons::EFFECT,
    SettingKind::Hinting => argvus_tui::icons::RULER,
    SettingKind::Subpixel => argvus_tui::icons::PALETTE,
    SettingKind::Dpi => argvus_tui::icons::MONITOR,
  }
}

pub fn font_target_label(lang: Lang, target: FontTarget) -> &'static str {
  match target {
    FontTarget::Taskbar => tr(lang, "control_center.taskbar_font"),
    FontTarget::Sysinfo => tr(lang, "control_center.widget_telemetry_font"),
    FontTarget::ControlPanel => tr(lang, "control_center.control_panel_font"),
    FontTarget::System => tr(lang, "control_center.system_font"),
    FontTarget::Apps => tr(lang, "control_center.applications_font"),
    FontTarget::Terminal => tr(lang, "control_center.terminal_font"),
    FontTarget::Browser => tr(lang, "control_center.browser_font"),
  }
}

pub fn setting_label(lang: Lang, setting: SettingKind) -> &'static str {
  match setting {
    SettingKind::Antialiasing => tr(lang, "control_center.antialiasing"),
    SettingKind::Hinting => "Hinting",
    SettingKind::Subpixel => tr(lang, "control_center.subpixel_order"),
    SettingKind::Dpi => "DPI",
  }
}

pub(super) fn setting_value_label(lang: Lang, value: &str) -> String {
  match value {
    "enabled" => tr(lang, "control_center.enabled_16b283").to_string(),
    "disabled" => tr(lang, "control_center.disabled_8ccfd8").to_string(),
    "none" => tr(lang, "control_center.none").to_string(),
    "slight" => tr(lang, "control_center.slight").to_string(),
    "medium" => tr(lang, "control_center.medium").to_string(),
    "full" => tr(lang, "control_center.full").to_string(),
    "automatic" => tr(lang, "control_center.automatic_b0d36e").to_string(),
    _ => value.to_string(),
  }
}

pub(super) fn enabled_label(lang: Lang, enabled: bool) -> &'static str {
  if enabled {
    tr(lang, "control_center.enabled")
  } else {
    tr(lang, "control_center.disabled")
  }
}

pub(super) fn non_empty(value: &str) -> String {
  if value.is_empty() {
    "-".to_string()
  } else {
    value.to_string()
  }
}

pub(super) fn canonical_language() -> Option<Lang> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/session/language", "--effective", "--raw"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  match String::from_utf8_lossy(&output.stdout).trim() {
    "pt-BR" | "pt_BR" => Some(Lang::for_locale("pt-BR")),
    "en-US" | "en_US" => Some(Lang::for_locale("en-US")),
    _ => None,
  }
}
