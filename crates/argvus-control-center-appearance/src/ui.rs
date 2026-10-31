//! Terminal UI rendering and interaction of the Appearance page.
mod icon_page;
mod rows;

use crate::{
  animations::{AnimationCurve, AnimationField, AnimationSettings, cycle},
  backend,
  blur::{BlurField, BlurSettings},
  model::{
    AppearancePage, AppearanceState, ControlPanelCard, ControlPanelCards, CustomTheme,
    EffectSurface, HexColor, PromptGoal, SurfaceSection, TaskbarDateFormat, TaskbarTimeFormat,
    TaskbarUtilityGroupMode, TaskbarUtilityWidgets, WidgetTelemetryBlock, normalize_hex_color,
    theme_family_label,
  },
  shadow::{ShadowColor, ShadowField, ShadowSettings},
};
use argvus_control_center_core::navigation::HomeCardId;
use argvus_theme::discovery::ThemeCategory;
use rows::Item;

const THEME_CATEGORIES: [ThemeCategory; 2] = [ThemeCategory::Dark, ThemeCategory::Light];

/// Gap between footer segments, matching `argvus_tui::hints`.
const FOOTER_GAP: &str = "   ";

use argvus_control_center_core::{
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  action_buttons::ActionFocus,
  components::{StatusKind, StatusMessage},
  confirm::{ConfirmDialog, ConfirmOutcome, ConfirmState, draw_confirm},
  hints::{HintContext, confirm_hints, hints},
  menu::{MenuEvent, MenuState, MenuStyle, Row, RowKind, draw_menu},
  page::{shell, status},
};
use crossterm::event::KeyCode;
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Rect},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::Paragraph,
};

mod activation;
mod drafts;
mod input;
mod jobs;
mod navigation;
mod prompts;
mod render;

#[derive(Debug, Clone)]
enum JobData {
  Loaded(AppearancePage, Box<AppearanceState>),
  /// Hardware availability of the Control Panel cards, probed apart from the
  /// page load so that the page never waits for it.
  Availability(Vec<(ControlPanelCard, bool)>),
  Archives(Vec<std::path::PathBuf>),
  Action(String),
  ImportReady {
    path: std::path::PathBuf,
    name: String,
    duplicate: bool,
  },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SurfaceDraft {
  surface: EffectSurface,
  utility_group: TaskbarUtilityGroupMode,
  audio_player_enabled: bool,
  focus_enabled: bool,
  launcher_enabled: bool,
  utility_widgets: TaskbarUtilityWidgets,
  date_format: TaskbarDateFormat,
  time_seconds_enabled: bool,
  time_format: TaskbarTimeFormat,
  height: u32,
  border_enabled: bool,
  border_thickness: u32,
  widget_enabled: bool,
  widget_blocks: crate::model::WidgetTelemetryBlocks,
  widget_order: Vec<WidgetTelemetryBlock>,
  control_panel_enabled: bool,
  control_panel_cards: ControlPanelCards,
  transparency_enabled: bool,
  transparency: i32,
  blur_enabled: bool,
  blur: i32,
}

/// What the open confirmation dialog decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirmation {
  /// Esc on a page whose unapplied draft would be dropped by going back.
  DiscardDraft,
  /// `d` on a custom theme (`delete_theme`).
  DeleteTheme,
  /// Importing an archive whose theme already exists (`pending_import`).
  ReplaceImport,
}

fn theme_category_label_key(category: ThemeCategory) -> &'static str {
  match category {
    ThemeCategory::Dark => "control_center.theme_category_dark",
    ThemeCategory::Light => "control_center.theme_category_light",
  }
}

/// Name of the active theme family as declared by its manifest, localized for
/// the current language. Falls back to the built-in label table.
fn current_family_label(state: &AppearanceState, lang: &Lang) -> String {
  let family = state.theme.strip_suffix("-float").unwrap_or(&state.theme);
  state
    .official_themes
    .iter()
    .find(|entry| entry.id == family)
    .map(|entry| {
      let locale = lang.locale();
      entry
        .name_i18n
        .get(&locale)
        .cloned()
        .unwrap_or_else(|| entry.name.clone())
    })
    .unwrap_or_else(|| theme_family_label(&state.theme))
}

/// The official themes of a category; the user's own are in Custom Themes.
fn family_indices(
  category: ThemeCategory,
  official_themes: &[argvus_theme::discovery::ThemeEntry],
) -> Vec<usize> {
  official_themes
    .iter()
    .enumerate()
    .filter_map(|(index, entry)| (entry.category == category && !entry.user).then_some(index))
    .collect()
}

pub struct AppearanceApp {
  pub page: AppearancePage,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
  state: AppearanceState,
  loaded: bool,
  status_loading: bool,
  /// Cursor of the current page's menu; reset on every page change.
  menu: MenuState,
  /// Height of the menu during the last draw, used as the PgUp/PgDn step.
  list_height: u16,
  prompt_buffer: String,
  prompt_error: Option<String>,
  prompt_back: Option<AppearancePage>,
  job: Option<JobHandle<JobData>>,
  /// The page `job` is loading, which may no longer be the current one.
  job_page: Option<AppearancePage>,
  /// The Control Panel cards' hardware probe; it does not make the page busy.
  availability: Option<JobHandle<JobData>>,
  /// The last probed card availability. Reapplied after every page load,
  /// whose state was copied before the probe finished.
  known_availability: Option<Vec<(ControlPanelCard, bool)>>,
  refreshed: Vec<(AppearancePage, std::time::Instant)>,
  action: Option<JobHandle<JobData>>,
  reload_requested: bool,
  control_panel_draft: Option<ControlPanelCards>,
  effect_draft: Option<i32>,
  /// Unapplied Blur page edits: the enable switch and the parameters.
  blur_draft: Option<BlurSettings>,
  /// Unapplied Shadow page edits: the switches, the color and the parameters.
  shadow_draft: Option<ShadowSettings>,
  /// Unapplied Animations edits, shared by every Animations page.
  animations_draft: Option<AnimationSettings>,
  surface_draft: Option<SurfaceDraft>,
  theme_dirty: bool,
  /// Open confirmation dialog, drawn over the page.
  confirm: Option<(Confirmation, ConfirmState)>,
  delete_theme: Option<CustomTheme>,
  import_archives: Vec<std::path::PathBuf>,
  pending_import: Option<std::path::PathBuf>,
  pending_import_name: Option<String>,
  manager: JobManager,
  /// Set by Desktop > Fonts; the host app takes it and opens the Fonts page.
  open_fonts: bool,
  /// Whether the host compiles the Fonts page in (the `fonts` feature).
  pub fonts_available: bool,
  /// Icons page: the search text, whether it is being typed, the unapplied
  /// theme choice (`None` means the configured theme) and the Actions focus.
  icon_search: String,
  icon_searching: bool,
  icon_draft: Option<String>,
  icon_actions: ActionFocus,
}
impl AppearanceApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Consumes the pending signal that an applied change altered the palette.
  pub fn take_theme_dirty(&mut self) -> bool {
    std::mem::take(&mut self.theme_dirty)
  }

  pub fn new(lang: Lang, theme: Theme) -> Self {
    Self {
      page: AppearancePage::Home,
      lang,
      theme,
      status: None,
      state: AppearanceState::default(),
      loaded: false,
      status_loading: false,
      menu: MenuState::default(),
      list_height: 0,
      prompt_buffer: String::new(),
      prompt_error: None,
      prompt_back: None,
      job: None,
      job_page: None,
      availability: None,
      known_availability: None,
      refreshed: Vec::new(),
      action: None,
      reload_requested: false,
      control_panel_draft: None,
      effect_draft: None,
      blur_draft: None,
      shadow_draft: None,
      animations_draft: None,
      surface_draft: None,
      theme_dirty: false,
      confirm: None,
      delete_theme: None,
      import_archives: Vec::new(),
      pending_import: None,
      pending_import_name: None,
      manager: JobManager::default(),
      open_fonts: false,
      fonts_available: true,
      icon_search: String::new(),
      icon_searching: false,
      icon_draft: None,
      icon_actions: ActionFocus::default(),
    }
  }

  /// Returns and clears the request to open the Fonts page, which lives in
  /// the settings crate. The host app routes to it after the key is handled.
  pub fn take_fonts_request(&mut self) -> bool {
    std::mem::take(&mut self.open_fonts)
  }
}

impl AppearanceApp {
  pub fn onboarding_ready(&self) -> bool {
    self.action.is_none()
      && self.confirm.is_none()
      && self.prompt_back.is_none()
      && !self.surface_draft_has_changes()
      && !self.effect_draft_has_changes()
      && !self.blur_draft_has_changes()
      && !self.shadow_draft_has_changes()
      && !self.animations_draft_has_changes()
  }
}

#[cfg(test)]
mod tests;
