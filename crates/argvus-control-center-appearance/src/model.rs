//! Domain state and models consumed by the UI of the Appearance page.

use crate::animations::{
  AnimationCurve, AnimationField, AnimationGroup, AnimationLeaf, AnimationValues,
};
use crate::blur::{BlurField, BlurValues};
use crate::shadow::{ShadowField, ShadowSettings};
use argvus_theme::discovery::ThemeCategory;
use std::str::FromStr;

mod control_panel_model;
mod media_model;
mod taskbar_model;
mod telemetry_model;

pub use control_panel_model::{ControlPanelCard, ControlPanelCards};
pub use media_model::{
  IconThemeEntry, IconThemeSource, IconThemeVariant, WallpaperCollection, WallpaperEntry,
  WallpaperMode,
};
pub use taskbar_model::{
  TaskbarDateFormat, TaskbarPosition, TaskbarTimeFormat, TaskbarUtilityGroupMode,
  TaskbarUtilityWidget, TaskbarUtilityWidgets,
};
pub use telemetry_model::{WidgetTelemetryBlock, WidgetTelemetryBlocks};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl HexColor {
  pub fn normalized(self) -> String {
    format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
  }

  pub fn foreground(self) -> ratatui::style::Color {
    let luminance =
      (299 * u32::from(self.r) + 587 * u32::from(self.g) + 114 * u32::from(self.b)) / 1000;
    if luminance >= 128 {
      ratatui::style::Color::Black
    } else {
      ratatui::style::Color::White
    }
  }
}

impl FromStr for HexColor {
  type Err = String;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
      return Err("expected six hexadecimal digits".into());
    }
    Ok(Self {
      r: u8::from_str_radix(&value[0..2], 16).map_err(|_| "invalid red channel")?,
      g: u8::from_str_radix(&value[2..4], 16).map_err(|_| "invalid green channel")?,
      b: u8::from_str_radix(&value[4..6], 16).map_err(|_| "invalid blue channel")?,
    })
  }
}

pub fn normalize_hex_color(value: &str) -> Option<String> {
  value.parse::<HexColor>().ok().map(HexColor::normalized)
}
// ThemeCategory is now from argvus_theme::discovery
// Use it via: use argvus_theme::discovery::ThemeCategory;

/// Canonicalizes identifiers from profiles and persisted state after the
/// official theme identifiers were simplified.
pub fn canonical_theme_id(theme_id: &str) -> String {
  match theme_id {
    "argvus-dark-aether" => "argvus-dark".into(),
    "argvus-dark-aether-float" => "argvus-dark-float".into(),
    "argvus-light-veil" => "argvus-light".into(),
    "argvus-light-veil-float" => "argvus-light-float".into(),
    "argvus-onedark" => "one-dark".into(),
    "argvus-onedark-float" => "one-dark-float".into(),
    "argvus-dark-dracula" => "dracula".into(),
    "argvus-dark-dracula-float" => "dracula-float".into(),
    "argvus-dark-silver" => "silver-dark".into(),
    "argvus-dark-silver-float" => "silver-dark-float".into(),
    "argvus-dark-slate" => "slate-dark".into(),
    "argvus-dark-slate-float" => "slate-dark-float".into(),
    "argvus-dark-universe" => "universe".into(),
    "argvus-dark-universe-float" => "universe-float".into(),
    "argvus-dark-gruvbox-high" => "gruvbox-high-dark".into(),
    "argvus-dark-gruvbox-high-float" => "gruvbox-high-dark-float".into(),
    "argvus-dark-gruvbox" => "gruvbox-dark".into(),
    "argvus-dark-gruvbox-float" => "gruvbox-dark-float".into(),
    "argvus-light-gruvbox" => "gruvbox-light".into(),
    "argvus-light-gruvbox-float" => "gruvbox-light-float".into(),
    "argvus-dark-rose-pine" | "argvus-dark-rosepine" => "rose-pine".into(),
    "argvus-dark-rose-pine-float" | "argvus-dark-rosepine-float" => "rose-pine-float".into(),
    "argvus-dark-tokio-night" | "argvus-dark-tokyo-night" => "tokyo-night".into(),
    "argvus-dark-tokio-night-float" | "argvus-dark-tokyo-night-float" => "tokyo-night-float".into(),
    "argvus-dark-solitude" => "solitude".into(),
    "argvus-dark-solitude-float" => "solitude-float".into(),
    "argvus-dark-sunset" => "sunset".into(),
    "argvus-dark-sunset-float" => "sunset-float".into(),
    "argvus-dark-matrix" => "matrix".into(),
    "argvus-dark-matrix-float" => "matrix-float".into(),
    "argvus-dark-monokai" => "monokai-dark".into(),
    "argvus-dark-monokai-float" => "monokai-dark-float".into(),
    "argvus-github-light" => "github-light".into(),
    "argvus-github-light-float" => "github-light-float".into(),
    "argvus-light-solarized" => "solarized-light".into(),
    "argvus-light-solarized-float" => "solarized-light-float".into(),
    "argvus-dark-catppuccin-latte"
    | "argvus-light-catppuccin-latte"
    | "argvus-catppuccin-latte" => "catppuccin-latte".into(),
    "argvus-dark-catppuccin-latte-float"
    | "argvus-light-catppuccin-latte-float"
    | "argvus-catppuccin-latte-float" => "catppuccin-latte-float".into(),
    _ => theme_id.to_owned(),
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppearancePage {
  Home,
  /// Appearance > Desktop: Theme, Appearance Mode, Accent Color and Wallpaper.
  Desktop,
  /// Appearance > Application Style: Terminal and Launcher transparency.
  ApplicationStyle,
  Themes,
  OfficialThemes,
  ThemeFamilies {
    category: ThemeCategory,
  },
  CustomThemes,
  ThemeImport,
  /// Sticky/Float, independent of theme selection (Appearance > Mode).
  Mode,
  /// Appearance > Desktop > Icons: the installed icon themes. Selecting one
  /// applies it at once, like a wallpaper.
  Icons,
  Wallpapers,
  WallpaperModes {
    collection: WallpaperCollection,
  },
  WallpaperItems {
    collection: WallpaperCollection,
    mode: WallpaperMode,
  },
  Accents,
  AccentEdit,
  Terminal,
  TerminalTransparency,
  Launchers,
  /// Entry point reached from the Hyprland category (crate principal):
  /// a single "Enable" toggle, applied immediately like it was on the
  /// removed Effects page.
  Animations,
  /// "Custom curves" under Animations: one editor per curve type.
  AnimationsCustomCurves,
  /// One curve of "Custom curves" (Bézier, Spring, Linear or Ease-out).
  AnimationsCurve {
    curve: AnimationCurve,
  },
  /// One animation category under Animations (Windows, Workspaces, ...).
  AnimationsGroup {
    group: AnimationGroup,
  },
  /// One animation leaf: its Enabled, Style, Speed, Curve and Pop-in rows.
  AnimationsLeaf {
    leaf: AnimationLeaf,
  },
  Transparency,
  TransparencySurface {
    surface: EffectSurface,
  },
  Blur,
  TaskbarPosition,
  TaskbarSpaces,
  Taskbar,
  TaskbarIcons,
  TaskbarDate,
  TaskbarDateFormat,
  TaskbarTime,
  TaskbarTimeFormat,
  TaskbarHeight,
  TaskbarBorder,
  TaskbarBorderThickness,
  WidgetTelemetry,
  ControlPanel,
  SurfaceSection {
    surface: EffectSurface,
    section: SurfaceSection,
  },
  /// Entry point reached from the Hyprland category: groups the "Inner"
  /// and "Outer" gap submenus.
  WindowSpaces,
  WindowSpacesInner,
  WindowSpacesOuter,
  Borders,
  GeneralBorders,
  EdgeThickness,
  /// Shadow of the window borders (Borders > Shadow). Edits a draft until `Apply`.
  BordersShadow,
  Prompt {
    goal: PromptGoal,
  },
}

impl AppearancePage {
  /// Whether the page belongs to the Animations tree, whose pages share one
  /// draft and are applied together with its `Apply` row.
  pub fn is_animations(self) -> bool {
    matches!(
      self,
      Self::Animations
        | Self::AnimationsCustomCurves
        | Self::AnimationsCurve { .. }
        | Self::AnimationsGroup { .. }
        | Self::AnimationsLeaf { .. }
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectSurface {
  Taskbar,
  ControlPanel,
  WidgetTelemetry,
  Terminal,
  Launchers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceSection {
  Launcher,
  UtilityIcons,
  Sessions,
  Transparency,
}

impl EffectSurface {
  pub const ALL: [Self; 3] = [Self::Taskbar, Self::ControlPanel, Self::WidgetTelemetry];

  pub const fn key(self) -> &'static str {
    match self {
      Self::Taskbar => "taskbar",
      Self::ControlPanel => "control-panel",
      Self::WidgetTelemetry => "widget-telemetry",
      Self::Terminal => "terminal",
      Self::Launchers => "launchers",
    }
  }

  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Taskbar => "control_center.taskbar",
      Self::ControlPanel => "control_center.control_panel",
      Self::WidgetTelemetry => "control_center.widget_telemetry",
      Self::Terminal => "control_center.terminal",
      Self::Launchers => "control_center.launcher",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptGoal {
  ExportProfile,
  WaybarTop,
  WaybarLeft,
  WaybarRight,
  WaybarBottom,
  GapsIn,
  GapsOutTop,
  GapsOutLeft,
  GapsOutRight,
  GapsOutBottom,
  Rounding,
  Thickness,
  /// A percentage typed into the open draft (effect editors and the
  /// transparency sections). It only edits the draft; `Apply` still
  /// writes it.
  DraftValue,
  /// A Blur parameter typed into the Blur page draft. Like `DraftValue`, it
  /// only edits the draft.
  BlurValue(BlurField),
  /// A speed, pop-in percentage or curve parameter typed into the Animations
  /// draft. Like `BlurValue`, it only edits the draft.
  AnimationValue(AnimationField),
  /// A numeric shadow parameter typed into the Shadow page draft.
  ShadowValue(ShadowField),
  /// The shadow color typed as `RRGGBBAA`, or empty to follow the accent. It
  /// only edits the Shadow draft.
  ShadowColor,
}
impl PromptGoal {
  pub fn key(self) -> &'static str {
    match self {
      Self::ExportProfile => "export_profile",
      Self::WaybarTop => "waybar_top",
      Self::WaybarLeft => "waybar_left",
      Self::WaybarRight => "waybar_right",
      Self::WaybarBottom => "waybar_bottom",
      Self::GapsIn => "gaps_in",
      Self::GapsOutTop => "gaps_out_top",
      Self::GapsOutLeft => "gaps_out_left",
      Self::GapsOutRight => "gaps_out_right",
      Self::GapsOutBottom => "gaps_out_bottom",
      Self::Rounding => "rounding",
      Self::Thickness => "thickness",
      Self::DraftValue => "draft_value",
      Self::BlurValue(field) => field.name(),
      Self::AnimationValue(_) => "animation_value",
      Self::ShadowValue(field) => field.name(),
      Self::ShadowColor => "shadow_color",
    }
  }
  pub const fn range(self) -> (i32, i32) {
    match self {
      Self::ExportProfile => (0, 0),
      Self::Rounding => (2, 10),
      Self::Thickness => (0, 10),
      _ => (0, 100),
    }
  }
}

pub const THEMES: &[(&str, &str)] = &[
  ("argvus-dark", "ARGVUS Dark"),
  ("argvus-dark-float", "ARGVUS Dark Float"),
  ("dracula", "Dracula"),
  ("dracula-float", "Dracula Float"),
  ("gruvbox-dark", "Gruvbox Dark"),
  ("gruvbox-dark-float", "Gruvbox Dark Float"),
  ("gruvbox-high-dark", "Gruvbox High Dark"),
  ("gruvbox-high-dark-float", "Gruvbox High Dark Float"),
  ("monokai-dark", "Monokai Dark"),
  ("monokai-dark-float", "Monokai Dark Float"),
  ("one-dark", "One Dark"),
  ("one-dark-float", "One Dark Float"),
  ("rose-pine", "Rosé Pine"),
  ("rose-pine-float", "Rosé Pine Float"),
  ("silver-dark", "Silver Dark"),
  ("silver-dark-float", "Silver Dark Float"),
  ("slate-dark", "Slate Dark"),
  ("slate-dark-float", "Slate Dark Float"),
  ("sunset", "Sunset"),
  ("sunset-float", "Sunset Float"),
  ("tokyo-night", "Tokyo-Night"),
  ("tokyo-night-float", "Tokyo-Night Float"),
  ("matrix", "Matrix"),
  ("matrix-float", "Matrix Float"),
  ("solitude", "Solitude"),
  ("solitude-float", "Solitude Float"),
  ("universe", "Universe"),
  ("universe-float", "Universe Float"),
  ("argvus-light", "ARGVUS Light"),
  ("argvus-light-float", "ARGVUS Light Float"),
  ("catppuccin-latte", "Catppuccin Latte"),
  ("catppuccin-latte-float", "Catppuccin Latte Float"),
  ("github-light", "GitHub Light"),
  ("github-light-float", "GitHub Light Float"),
  ("gruvbox-light", "Gruvbox Light"),
  ("gruvbox-light-float", "Gruvbox Light Float"),
  ("solarized-light", "Solarized Light"),
  ("solarized-light-float", "Solarized Light Float"),
  ("one-light", "One Light"),
  ("one-light-float", "One Light Float"),
  ("everforest-light", "Everforest Light"),
  ("everforest-light-float", "Everforest Light Float"),
  ("nord-light", "Nord Light"),
  ("nord-light-float", "Nord Light Float"),
];
pub const THEME_FAMILIES: &[(&str, &str)] = &[
  ("argvus-dark", "ARGVUS Dark"),
  ("dracula", "Dracula"),
  ("gruvbox-dark", "Gruvbox Dark"),
  ("gruvbox-high-dark", "Gruvbox High Dark"),
  ("monokai-dark", "Monokai Dark"),
  ("one-dark", "One Dark"),
  ("rose-pine", "Rosé Pine"),
  ("silver-dark", "Silver Dark"),
  ("slate-dark", "Slate Dark"),
  ("sunset", "Sunset"),
  ("tokyo-night", "Tokyo-Night"),
  ("matrix", "Matrix"),
  ("solitude", "Solitude"),
  ("universe", "Universe"),
  ("argvus-light", "ARGVUS Light"),
  ("catppuccin-latte", "Catppuccin Latte"),
  ("github-light", "GitHub Light"),
  ("gruvbox-light", "Gruvbox Light"),
  ("solarized-light", "Solarized Light"),
  ("one-light", "One Light"),
  ("everforest-light", "Everforest Light"),
  ("nord-light", "Nord Light"),
];
pub fn theme_label(name: &str) -> String {
  THEMES
    .iter()
    .find(|(code, _)| *code == name)
    .map(|(_, label)| (*label).to_string())
    .unwrap_or_else(|| name.to_string())
}
pub fn theme_family_label(name: &str) -> String {
  let family = name.strip_suffix("-float").unwrap_or(name);
  THEME_FAMILIES
    .iter()
    .find(|(code, _)| *code == family)
    .map(|(_, label)| (*label).to_string())
    .unwrap_or_else(|| theme_label(family))
}
pub const ACCENTS: &[(&str, &str)] = &[
  ("Blue", "#3590bd"),
  ("Slate Blue", "#7391a5"),
  ("Brown", "#996548"),
  ("Green", "#17d174"),
  ("Magenta", "#cb17d1"),
  ("Red", "#d1174f"),
  ("Yellow", "#d1ce17"),
  ("Purple", "#9617d1"),
  ("Silver", "#595959"),
];
pub fn accent_label(color: &str) -> String {
  ACCENTS
    .iter()
    .find(|(_, value)| *value == color)
    .map(|(label, _)| (*label).to_string())
    .unwrap_or_else(|| color.to_string())
}

/// Taskbar heights in pixels that the Control Center offers. They mirror
/// `TASKBAR_HEIGHTS` in `argvus-config`, which validates the value written.
pub const TASKBAR_HEIGHTS: [u32; 4] = [24, 28, 32, 36];

/// Taskbar border thicknesses in pixels that the Control Center offers. They
/// mirror `argvus-config`, which writes `0` for a border that is off.
pub const TASKBAR_BORDER_THICKNESSES: [u32; 3] = [1, 2, 3];

#[derive(Debug, Clone, PartialEq)]
pub struct AppearanceState {
  pub theme: String,
  pub accent: String,
  pub wallpapers: Vec<WallpaperEntry>,
  pub wallpaper_active: Option<String>,
  /// Configured icon theme name. An empty configuration resolves to the
  /// ARGVUS default, so this always holds a name.
  pub icon_theme: String,
  /// Installed icon themes, loaded by the Icons page.
  pub icon_themes: Vec<IconThemeEntry>,
  pub animations: bool,
  pub transparency: bool,
  pub blur: bool,
  pub blur_values: BlurValues,
  pub shadow: ShadowSettings,
  /// Animation values other than the global switch, which is `animations`.
  pub animation_values: AnimationValues,
  pub taskbar_transparency_enabled: bool,
  pub control_panel_transparency_enabled: bool,
  pub widget_telemetry_transparency_enabled: bool,
  pub taskbar_blur_enabled: bool,
  pub control_panel_blur_enabled: bool,
  pub widget_telemetry_blur_enabled: bool,
  pub taskbar_transparency: i32,
  pub control_panel_transparency: i32,
  pub widget_telemetry_transparency: i32,
  pub taskbar_blur: i32,
  pub control_panel_blur: i32,
  pub widget_telemetry_blur: i32,
  pub terminal_transparency_enabled: bool,
  pub terminal_transparency: i32,
  pub launcher_transparency_enabled: bool,
  pub launcher_transparency: i32,
  pub widget_telemetry: bool,
  pub control_panel_enabled: bool,
  pub widget_telemetry_blocks: WidgetTelemetryBlocks,
  pub widget_telemetry_order: Vec<WidgetTelemetryBlock>,
  pub control_panel_cards: ControlPanelCards,
  pub control_panel_order: Vec<ControlPanelCard>,
  pub waybar_pos: TaskbarPosition,
  /// Taskbar height in pixels, one of `waybar::TASKBAR_HEIGHTS`.
  pub taskbar_height: u32,
  pub taskbar_border_enabled: bool,
  /// Taskbar border thickness in pixels, from 1 to 3.
  pub taskbar_border_thickness: u32,
  pub taskbar_utility_group: TaskbarUtilityGroupMode,
  pub taskbar_audio_player_enabled: bool,
  pub taskbar_launcher_enabled: bool,
  pub taskbar_launcher_custom_icon_path: Option<String>,
  pub taskbar_utility_widgets: TaskbarUtilityWidgets,
  pub taskbar_date_format: TaskbarDateFormat,
  pub taskbar_time_seconds_enabled: bool,
  pub taskbar_time_format: TaskbarTimeFormat,
  pub waybar_top: i32,
  pub waybar_left: i32,
  pub waybar_right: i32,
  pub waybar_bottom: i32,
  pub gaps_in: i32,
  pub gaps_out_top: i32,
  pub gaps_out_left: i32,
  pub gaps_out_right: i32,
  pub gaps_out_bottom: i32,
  pub rounded: bool,
  pub rounding: i32,
  pub thickness: i32,
  pub custom_themes: Vec<CustomTheme>,
  /// What the one-time conversion of legacy appearance profiles did, shown once.
  pub theme_notices: Vec<String>,
  pub active_custom_theme: Option<String>,
  /// Discovered official themes (built-in + drop-in packages).
  /// This replaces the hard-coded `THEMES`/`THEME_FAMILIES` constants.
  /// Loaded on page entry via backend::refresh_official_themes().
  pub official_themes: Vec<argvus_theme::discovery::ThemeEntry>,
  /// Warnings from theme discovery (e.g., malformed manifests).
  pub discovery_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomTheme {
  /// The theme id: the name of its directory under `$XDG_DATA_HOME/argvus/themes`.
  pub id: String,
  pub name: String,
}
impl Default for AppearanceState {
  fn default() -> Self {
    Self {
      theme: "argvus-dark".into(),
      accent: "#3590bd".into(),
      wallpapers: Vec::new(),
      wallpaper_active: None,
      icon_theme: crate::icons::DEFAULT_ICON_THEME.into(),
      icon_themes: Vec::new(),
      animations: true,
      transparency: true,
      blur: true,
      blur_values: BlurValues::default(),
      shadow: ShadowSettings::default(),
      animation_values: AnimationValues::default(),
      taskbar_transparency_enabled: true,
      control_panel_transparency_enabled: true,
      widget_telemetry_transparency_enabled: true,
      taskbar_blur_enabled: true,
      control_panel_blur_enabled: true,
      widget_telemetry_blur_enabled: true,
      taskbar_transparency: 50,
      control_panel_transparency: 50,
      widget_telemetry_transparency: 50,
      taskbar_blur: 50,
      control_panel_blur: 50,
      widget_telemetry_blur: 50,
      terminal_transparency_enabled: true,
      terminal_transparency: 50,
      launcher_transparency_enabled: true,
      launcher_transparency: 15,
      widget_telemetry: true,
      control_panel_enabled: true,
      widget_telemetry_blocks: WidgetTelemetryBlocks::default(),
      widget_telemetry_order: WidgetTelemetryBlock::ALL.to_vec(),
      control_panel_cards: ControlPanelCards::default(),
      control_panel_order: ControlPanelCard::ALL.to_vec(),
      waybar_pos: TaskbarPosition::Top,
      taskbar_height: 28,
      taskbar_border_enabled: true,
      taskbar_border_thickness: 1,
      taskbar_utility_group: TaskbarUtilityGroupMode::AlwaysExpanded,
      taskbar_audio_player_enabled: true,
      taskbar_launcher_enabled: true,
      taskbar_launcher_custom_icon_path: None,
      taskbar_utility_widgets: TaskbarUtilityWidgets::default(),
      taskbar_date_format: TaskbarDateFormat::WeekdayDayMonth,
      taskbar_time_seconds_enabled: false,
      taskbar_time_format: TaskbarTimeFormat::TwentyFourHour,
      waybar_top: 0,
      waybar_left: 0,
      waybar_right: 0,
      waybar_bottom: 0,
      gaps_in: 3,
      gaps_out_top: 1,
      gaps_out_left: 1,
      gaps_out_right: 1,
      gaps_out_bottom: 1,
      rounded: false,
      rounding: 0,
      thickness: 1,
      custom_themes: Vec::new(),
      theme_notices: Vec::new(),
      active_custom_theme: None,
      official_themes: vec![
        argvus_theme::discovery::ThemeEntry {
          id: "argvus-dark".to_string(),
          name: "ARGVUS Dark".to_string(),
          category: argvus_theme::discovery::ThemeCategory::Dark,
          accent: "#3590bd".to_string(),
          wallpaper: "argvus-dark.jxl".to_string(),
          name_i18n: std::collections::HashMap::new(),
          user: false,
        },
        argvus_theme::discovery::ThemeEntry {
          id: "argvus-light".to_string(),
          name: "ARGVUS Light".to_string(),
          category: argvus_theme::discovery::ThemeCategory::Light,
          accent: "#181818".to_string(),
          wallpaper: "argvus-light.jxl".to_string(),
          name_i18n: std::collections::HashMap::new(),
          user: false,
        },
      ],
      discovery_warnings: Vec::new(),
    }
  }
}
impl AppearanceState {
  pub fn is_float_theme(&self) -> bool {
    self.theme.ends_with("-float")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Every selectable family has its base and `-float` entries in `THEMES`, and
  /// `THEMES` lists nothing that is not a family, so the selector cannot offer a theme
  /// whose variant is missing.
  fn every_family_has_its_base_and_float_entries() {
    for (family, _) in THEME_FAMILIES {
      for id in [family.to_string(), format!("{family}-float")] {
        assert!(
          THEMES.iter().any(|(code, _)| *code == id),
          "{id} is a family without a THEMES entry"
        );
      }
    }
    for (code, _) in THEMES {
      let family = code.strip_suffix("-float").unwrap_or(code);
      assert!(
        THEME_FAMILIES.iter().any(|(id, _)| *id == family),
        "{code} is in THEMES but not in THEME_FAMILIES"
      );
    }
  }

  #[test]
  fn prompt_metadata_is_explicit() {
    assert_eq!(
      (PromptGoal::WaybarTop.key(), PromptGoal::WaybarTop.range()),
      ("waybar_top", (0, 100))
    );
    assert_eq!(
      (
        PromptGoal::GapsOutBottom.key(),
        PromptGoal::GapsOutBottom.range()
      ),
      ("gaps_out_bottom", (0, 100))
    );
    assert_eq!(
      (PromptGoal::Rounding.key(), PromptGoal::Rounding.range()),
      ("rounding", (2, 10))
    );
    assert_eq!(
      (PromptGoal::Thickness.key(), PromptGoal::Thickness.range()),
      ("thickness", (0, 10))
    );
  }
  #[test]
  fn position_is_type_safe() {
    assert_eq!(
      TaskbarPosition::from_value("bottom"),
      TaskbarPosition::Bottom
    );
    assert_eq!(TaskbarPosition::Top.value(), "top");
    assert_eq!(
      TaskbarUtilityGroupMode::from_value("always-expanded"),
      TaskbarUtilityGroupMode::AlwaysExpanded
    );
    assert_eq!(
      TaskbarUtilityGroupMode::from_value("unknown"),
      TaskbarUtilityGroupMode::Auto
    );
    assert_eq!(WidgetTelemetryBlock::CpuGpu.key(), "cpu_gpu");
    assert!(WidgetTelemetryBlocks::default().enabled(WidgetTelemetryBlock::Network));
    assert!(ControlPanelCards::default().enabled(ControlPanelCard::Power));
  }
  #[test]
  fn float_theme_detection() {
    assert!(
      AppearanceState {
        theme: "argvus-dark-silver-float".into(),
        ..Default::default()
      }
      .is_float_theme()
    );
  }

  #[test]
  fn hex_colors_parse_and_normalize() {
    assert_eq!(
      "#12ABEF".parse::<HexColor>().unwrap().normalized(),
      "#12ABEF"
    );
    assert_eq!(normalize_hex_color("12abef"), Some("#12ABEF".into()));
    assert!(normalize_hex_color("#123").is_none());
    assert!(normalize_hex_color("#12345G").is_none());
    assert!(normalize_hex_color("foo").is_none());
  }

  #[test]
  fn hex_contrast_uses_black_for_light_and_white_for_dark() {
    assert_eq!(
      "#FFFFFF".parse::<HexColor>().unwrap().foreground(),
      ratatui::style::Color::Black
    );
    assert_eq!(
      "#000000".parse::<HexColor>().unwrap().foreground(),
      ratatui::style::Color::White
    );
  }
}
