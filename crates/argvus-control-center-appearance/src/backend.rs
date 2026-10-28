//! Isolated integration with system tools and APIs of the Appearance page.
use crate::animations::AnimationValues;
use crate::blur::BlurValues;
use crate::model::{
  AppearanceState, ControlPanelCard, ControlPanelCards, EffectSurface, TASKBAR_BORDER_THICKNESSES,
  TASKBAR_HEIGHTS, TaskbarDateFormat, TaskbarPosition, TaskbarTimeFormat, TaskbarUtilityGroupMode,
  TaskbarUtilityWidget, TaskbarUtilityWidgets, WallpaperCollection, WallpaperEntry, WallpaperMode,
  WidgetTelemetryBlock, WidgetTelemetryBlocks, canonical_theme_id, normalize_hex_color,
};
use argvus_control_center_core::{
  config::reload_argvus_config_service,
  paths::{argvus_data_home, cache_home, system_config_root},
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::Duration;

pub use crate::user_themes::{
  active_custom_theme, apply_custom_theme, convert_legacy_profiles, custom_themes,
  delete_custom_theme, export_named as export_theme, import_archive as import_theme,
  inspect_archive as inspect_theme_archive, list_import_archives,
  preview_export_path as preview_theme_export_path,
};

mod commands;
mod control_panel;
mod effects;
mod icon_themes;
mod layout;
mod taskbar;
mod telemetry;
mod themes;
mod wallpapers;

use commands::read_first;
use control_panel::{ControlPanelReading, read_control_panel};
pub use control_panel::{
  control_panel_availability, move_control_panel_card, reload_control_panel,
  set_control_panel_cards, set_control_panel_enabled,
};
use effects::{
  GlobalEffects, SURFACES_WITH_EFFECTS, SurfaceEffects, animation_values, effect_state,
  read_global_effects, read_surface_effects, store_surface_effects,
};
pub use effects::{
  apply_animations, apply_blur, apply_shadow, apply_surface_effects, set_effect_value,
};
pub(crate) use icon_themes::load_icon_themes;
pub use icon_themes::{choose_launcher_icon, set_icon_theme};
use layout::{
  LayoutStatus, apply_canonical_layout, apply_canonical_taskbar, parse_borders_status,
  parse_spacing_status, parse_taskbar_shape, read_layout_status,
};
pub use layout::{set_border, set_layout_mode, set_rounded, set_spacing, set_waybar_position};
pub use taskbar::{
  ApplyOverlay, apply_taskbar_icons_and_format, reload_taskbar, set_taskbar_shape,
  set_taskbar_utility_group,
};
pub use telemetry::apply_widget_telemetry;
use telemetry::{TelemetryReading, read_telemetry, telemetry_blocks, telemetry_order};
use themes::{accent_file, active_theme_file, discovered_default_accent, refresh_official_themes};
pub use themes::{set_accent, set_theme};
pub use wallpapers::{active_wallpaper, choose_wallpaper, list_wallpapers, set_wallpaper};

const WALLPAPERS_DIR: &str = "/usr/share/backgrounds/argvus";
const DEFAULT_THEME: &str = "argvus-dark";
const DEFAULT_ACCENT: &str = "#3590bd";

fn parse_pairs(output: &str) -> impl Iterator<Item = (&str, &str)> {
  output.lines().filter_map(|line| {
    line
      .split_once('=')
      .map(|(key, value)| (key.trim(), value.trim()))
  })
}

/// What the concurrent readers of [`load_page`] collected. `None` means the
/// page does not show that data, so the previous values are kept.
struct PageReadings {
  global_effects: Option<GlobalEffects>,
  surface_effects: Vec<(EffectSurface, SurfaceEffects)>,
  telemetry: Option<TelemetryReading>,
  control_panel: Option<ControlPanelReading>,
  layout: Option<LayoutStatus>,
}

/// The result of a reader that ran, or `None` when it did not run or panicked
/// (the page then keeps its previous values, as when a reader fails).
fn joined<T>(handle: Option<std::thread::ScopedJoinHandle<'_, T>>) -> Option<T> {
  handle.and_then(|handle| handle.join().ok())
}

/// Collect only data used by the requested page. Existing values survive unrelated refreshes.
pub fn load_page(
  page: crate::model::AppearancePage,
  mut state: AppearanceState,
) -> AppearanceState {
  use crate::model::AppearancePage;
  let theme = canonical_theme_id(&read_first(&active_theme_file(), DEFAULT_THEME));
  state.theme = theme.clone();
  let default_accent =
    discovered_default_accent(&theme).unwrap_or_else(|| DEFAULT_ACCENT.to_string());
  state.accent = normalize_hex_color(&read_first(&accent_file(), &default_accent))
    .unwrap_or_else(|| default_accent.clone());
  // The Desktop page shows the active wallpaper name, so it loads the active
  // value too. The file list is only needed by the Wallpapers pages.
  if matches!(page, AppearancePage::Desktop | AppearancePage::Wallpapers) {
    state.wallpaper_active = active_wallpaper();
  }
  if page == AppearancePage::Wallpapers {
    state.wallpapers = list_wallpapers();
  }
  if page == AppearancePage::Icons {
    load_icon_themes(&mut state);
  }
  let wants_global_effects = matches!(
    page,
    AppearancePage::Blur
      | AppearancePage::BordersShadow
      | AppearancePage::Animations
      | AppearancePage::Taskbar
      | AppearancePage::WidgetTelemetry
      | AppearancePage::ControlPanel
      | AppearancePage::SurfaceSection { .. }
  );
  let wants_surface_effects = matches!(
    page,
    AppearancePage::Taskbar
      | AppearancePage::WidgetTelemetry
      | AppearancePage::ControlPanel
      | AppearancePage::Terminal
      | AppearancePage::Launchers
      | AppearancePage::TerminalTransparency
      | AppearancePage::SurfaceSection { .. }
  );
  let wants_telemetry = matches!(
    page,
    AppearancePage::WidgetTelemetry
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::WidgetTelemetry,
        ..
      }
  );
  let wants_control_panel = matches!(
    page,
    AppearancePage::ControlPanel
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::ControlPanel,
        ..
      }
  );
  let wants_layout = matches!(
    page,
    AppearancePage::Borders
      | AppearancePage::Taskbar
      | AppearancePage::TaskbarPosition
      | AppearancePage::TaskbarSpaces
      | AppearancePage::TaskbarHeight
      | AppearancePage::TaskbarBorder
      | AppearancePage::TaskbarBorderThickness
      | AppearancePage::WindowSpaces
      | AppearancePage::WindowSpacesInner
      | AppearancePage::WindowSpacesOuter
      | AppearancePage::GeneralBorders
      | AppearancePage::EdgeThickness
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        ..
      }
  );
  // Every reader below is an independent external process (about twenty for
  // the surface pages), so they run concurrently instead of one after the
  // other. Nothing is written to `state` until all of them are joined.
  let previous_cards = state.control_panel_cards.clone();
  let readings = std::thread::scope(|scope| {
    let global_effects = wants_global_effects.then(|| scope.spawn(read_global_effects));
    let surface_effects: Vec<_> = if wants_surface_effects {
      SURFACES_WITH_EFFECTS
        .into_iter()
        .map(|surface| (surface, scope.spawn(move || read_surface_effects(surface))))
        .collect()
    } else {
      Vec::new()
    };
    let telemetry = wants_telemetry.then(|| scope.spawn(read_telemetry));
    let control_panel =
      wants_control_panel.then(|| scope.spawn(|| read_control_panel(&previous_cards)));
    let layout = wants_layout.then(|| scope.spawn(read_layout_status));
    PageReadings {
      global_effects: joined(global_effects),
      surface_effects: surface_effects
        .into_iter()
        .filter_map(|(surface, handle)| handle.join().ok().map(|effects| (surface, effects)))
        .collect(),
      telemetry: joined(telemetry),
      control_panel: joined(control_panel),
      layout: joined(layout),
    }
  });
  if let Some(global_effects) = readings.global_effects {
    state.animations = global_effects.animations;
    state.transparency = global_effects.transparency;
    state.blur = global_effects.blur;
    state.blur_values = global_effects.blur_values;
    state.shadow = global_effects.shadow;
  }
  if page.is_animations() {
    state.animations = effect_state("animations");
    state.animation_values = animation_values();
  }
  for (surface, effects) in readings.surface_effects {
    store_surface_effects(&mut state, surface, effects);
  }
  if let Some(telemetry) = readings.telemetry {
    state.widget_telemetry = telemetry.enabled;
    state.widget_telemetry_blocks = telemetry_blocks(&telemetry.blocks_status);
    state.widget_telemetry_order = telemetry_order(&telemetry.blocks_status);
  }
  if let Some(control_panel) = readings.control_panel {
    state.control_panel_cards = control_panel.cards;
    state.control_panel_order = control_panel.order;
    state.control_panel_enabled = control_panel.enabled;
  }
  if matches!(
    page,
    AppearancePage::Themes
      | AppearancePage::ThemeFamilies { .. }
      | AppearancePage::CustomThemes
      | AppearancePage::OfficialThemes
  ) {
    state.theme_notices = convert_legacy_profiles();
    state.custom_themes = custom_themes();
    state.active_custom_theme = active_custom_theme();
    // Discover and load official themes (built-in + drop-in packages)
    refresh_official_themes(&mut state);
  }
  if let Some(layout) = readings.layout {
    if let Some(output) = layout.utility_group {
      state.taskbar_utility_group = TaskbarUtilityGroupMode::from_value(output.trim());
    }
    if let Some(output) = layout.spacing {
      parse_spacing_status(&output, &mut state);
    }
    if let Some(output) = layout.borders {
      parse_borders_status(&output, &mut state);
    }
    if let Some(output) = layout.taskbar_shape {
      parse_taskbar_shape(&output, &mut state);
    }
    apply_canonical_layout(&mut state);
  }
  if matches!(
    page,
    AppearancePage::Taskbar
      | AppearancePage::TaskbarIcons
      | AppearancePage::TaskbarDate
      | AppearancePage::TaskbarDateFormat
      | AppearancePage::TaskbarTime
      | AppearancePage::TaskbarTimeFormat
      | AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        ..
      }
  ) {
    apply_canonical_taskbar(&mut state);
  }
  state
}

#[cfg(test)]
mod tests {
  use super::control_panel::{control_panel_cards_from_status, parse_control_panel_cards};
  use super::effects::settings_arguments;
  use super::themes::discovered_default_accent_in;
  use super::wallpapers::classify_wallpaper;
  use super::*;

  #[test]
  fn effect_settings_carry_only_what_changed_in_one_call() {
    let values = Some(vec!["size=6".to_string()]);
    assert_eq!(
      settings_arguments(Some(false), values.clone()),
      ["enabled=disabled", "size=6"]
    );
    assert_eq!(settings_arguments(None, values), ["size=6"]);
    assert_eq!(settings_arguments(Some(true), None), ["enabled=enabled"]);
    assert!(settings_arguments(None, None).is_empty());
  }

  #[test]
  fn wallpaper_paths_are_grouped_by_collection_and_mode() {
    assert_eq!(
      classify_wallpaper(Path::new("argvus-dark.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Dark))
    );
    assert_eq!(
      classify_wallpaper(Path::new("argvus-light.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Light))
    );
    assert_eq!(
      classify_wallpaper(Path::new("abstract/dark/gruvbox-abstract-dark.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Dark))
    );
    assert_eq!(
      classify_wallpaper(Path::new("abstract/light/everforest-abstract-light.jxl")),
      Some((WallpaperCollection::Abstract, WallpaperMode::Light))
    );
    assert_eq!(
      classify_wallpaper(Path::new("landscape/light/gruvbox-landscape-light.jxl")),
      Some((WallpaperCollection::Landscape, WallpaperMode::Light))
    );
    assert_eq!(classify_wallpaper(Path::new("dark/wallpaper.jxl")), None);
    assert_eq!(
      classify_wallpaper(Path::new("abstract/dark/readme.png")),
      None
    );
  }

  #[test]
  fn drop_in_theme_accent_comes_from_its_manifest() {
    let system_config = tempfile::tempdir().unwrap();
    let family_dir = system_config.path().join("themes/nord-light");
    std::fs::create_dir_all(&family_dir).unwrap();
    std::fs::write(
      family_dir.join("theme.toml"),
      "schema = 2\nid = \"nord-light\"\nname = \"Nord Light\"\ncategory = \"light\"\n\
       [palette]\nbg = \"#ECEFF4\"\nfg = \"#2E3440\"\naccent = \"#5E81AC\"\n",
    )
    .unwrap();

    assert_eq!(
      discovered_default_accent_in(system_config.path(), "nord-light").as_deref(),
      Some("#5E81AC")
    );
    assert_eq!(
      discovered_default_accent_in(system_config.path(), "nord-light-float").as_deref(),
      Some("#5E81AC")
    );
    assert_eq!(
      discovered_default_accent_in(system_config.path(), "not-installed"),
      None
    );
  }

  #[test]
  fn status_parsers_ignore_unknown_keys() {
    let mut state = AppearanceState::default();
    parse_spacing_status(
      "waybar_pos=top\nwaybar_top=16\nwaybar_left=14\nwaybar_right=14\nwaybar_bottom=0\ngaps_in=8\ngaps_out_top=8\ngaps_out_left=10\ngaps_out_right=10\ngaps_out_bottom=8\nfuture_property=123\n",
      &mut state,
    );
    parse_borders_status(
      "rounded=1\nrounding=4\nthickness=1\nfuture_property=123\n",
      &mut state,
    );
    assert_eq!(state.waybar_top, 16);
    assert_eq!(state.gaps_out_left, 10);
    assert!(state.rounded);
    assert_eq!(state.rounding, 4);
  }

  #[test]
  fn control_panel_status_keeps_defaults_and_applies_known_cards() {
    let cards = parse_control_panel_cards(
      r#"{"cards":[
        {"id":"weather","enabled":false},
        {"id":"future-card","enabled":false},
        {"id":"power","enabled":true}
      ]}"#,
    );
    assert!(!cards.enabled(ControlPanelCard::Weather));
    assert!(cards.enabled(ControlPanelCard::Power));
    assert!(cards.enabled(ControlPanelCard::User));
    assert!(parse_control_panel_cards("invalid").enabled(ControlPanelCard::User));
  }

  #[test]
  fn control_panel_status_keeps_the_known_availability() {
    let mut previous = ControlPanelCards::default();
    previous.set_available(ControlPanelCard::Bluetooth, false);
    let (cards, order) = control_panel_cards_from_status(
      Some(r#"{"cards":[{"id":"bluetooth","enabled":false}]}"#),
      &previous,
    );
    assert!(!cards.available(ControlPanelCard::Bluetooth));
    assert!(!cards.enabled(ControlPanelCard::Bluetooth));
    assert!(cards.available(ControlPanelCard::Brightness));
    assert_eq!(order.first(), Some(&ControlPanelCard::Bluetooth));

    let (cards, order) = control_panel_cards_from_status(None, &previous);
    assert!(!cards.available(ControlPanelCard::Bluetooth));
    assert!(cards.enabled(ControlPanelCard::Bluetooth));
    assert_eq!(order, ControlPanelCard::ALL.to_vec());
  }

  #[test]
  fn taskbar_shape_status_keeps_offered_values_only() {
    let mut state = AppearanceState::default();
    parse_taskbar_shape("height=36\nborder=2\n", &mut state);
    assert_eq!(state.taskbar_height, 36);
    assert!(state.taskbar_border_enabled);
    assert_eq!(state.taskbar_border_thickness, 2);

    // A border of 0 is off and keeps the last thickness for the next enable.
    parse_taskbar_shape("border=0\n", &mut state);
    assert!(!state.taskbar_border_enabled);
    assert_eq!(state.taskbar_border_thickness, 2);

    parse_taskbar_shape("height=30\nborder=9\n", &mut state);
    assert_eq!(state.taskbar_height, 36, "30 is not offered");
    assert!(!state.taskbar_border_enabled, "9 is not offered");
  }
}
