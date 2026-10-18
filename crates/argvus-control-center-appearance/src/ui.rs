//! Implements terminal UI rendering and interaction in crate `argvus control center appearance`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
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

#[derive(Debug, Clone)]
/// Defines `JobData`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
enum JobData {
  Loaded(AppearancePage, Box<AppearanceState>),
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
  launcher_enabled: bool,
  utility_widgets: TaskbarUtilityWidgets,
  date_format: TaskbarDateFormat,
  time_seconds_enabled: bool,
  time_format: TaskbarTimeFormat,
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

fn family_indices(
  category: ThemeCategory,
  official_themes: &[argvus_theme::discovery::ThemeEntry],
) -> Vec<usize> {
  official_themes
    .iter()
    .enumerate()
    .filter_map(|(index, entry)| (entry.category == category).then_some(index))
    .collect()
}

/// Represents `AppearanceApp`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
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
  refreshed: Vec<(AppearancePage, std::time::Instant)>,
  action: Option<JobHandle<JobData>>,
  reload_requested: bool,
  control_panel_draft: Option<ControlPanelCards>,
  effect_draft: Option<i32>,
  /// Unapplied Blur page edits: the enable switch and the parameters.
  blur_draft: Option<BlurSettings>,
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
  /// Executes the `reload` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn reload(&mut self) {
    self.ensure_page();
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  /// Consumes the pending signal that an applied change altered the palette.
  pub fn take_theme_dirty(&mut self) -> bool {
    std::mem::take(&mut self.theme_dirty)
  }

  fn ensure_page(&mut self) {
    let lifetime = if self.page == AppearancePage::ControlPanel {
      60
    } else {
      30
    };
    if !self
      .refreshed
      .iter()
      .any(|(page, at)| *page == self.page && at.elapsed().as_secs() < lifetime)
    {
      self.refresh();
    }
  }

  pub fn paste(&mut self, text: &str) {
    if self.page != AppearancePage::AccentEdit {
      return;
    }
    let value = text.lines().next().unwrap_or_default().trim();
    if let Some(color) = normalize_hex_color(value) {
      self.prompt_buffer = color;
      self.prompt_error = None;
    } else {
      self.prompt_error = Some(tr(self.lang, "control_center.accent_invalid").into());
    }
  }
  /// Constructs `new` with this module's expected initial state. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
      refreshed: Vec::new(),
      action: None,
      reload_requested: false,
      control_panel_draft: None,
      effect_draft: None,
      blur_draft: None,
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
  /// Executes the `refresh` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn refresh(&mut self) {
    if self.job.is_none() && self.action.is_none() {
      self.status_loading = true;
      let page = self.page;
      let state = self.state.clone();
      self.job = Some(self.manager.spawn(move |_| {
        if page == AppearancePage::ThemeImport {
          Ok(JobData::Archives(backend::list_import_archives()))
        } else {
          Ok(JobData::Loaded(
            page,
            Box::new(backend::load_page(page, state)),
          ))
        }
      }));
    }
  }
  /// Executes the `poll` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if let Some(job) = &self.job
      && let JobState::Finished(result) = job.try_state()
    {
      self.job = None;
      match result {
        Ok(JobData::Loaded(page, state)) => self.on_loaded(page, *state),
        Ok(JobData::Archives(paths)) => {
          self.import_archives = paths;
          self.status_loading = false;
        }
        Err(error) => {
          self.status_loading = false;
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: format!("{} {error}", tr(self.lang, "control_center.error")),
          });
        }
        Ok(JobData::Action(_)) => {}
        Ok(JobData::ImportReady { .. }) => {}
      }
      changed = true;
    }
    if let Some(action) = &self.action
      && let JobState::Finished(result) = action.try_state()
    {
      self.action = None;
      // The action already ran the external scripts that regenerate the theme
      // files, so this is the first safe point to signal a palette reload.
      self.theme_dirty = true;
      if self.reload_requested {
        self.reload_requested = false;
        self.refreshed.clear();
        self.refresh();
      }
      match result {
        Ok(JobData::Action(text)) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Success,
            text: text.clone(),
          })
        }
        Ok(JobData::ImportReady {
          path,
          name,
          duplicate,
        }) => {
          self.pending_import = Some(path);
          self.pending_import_name = Some(name);
          if duplicate {
            self.status = None;
            self.confirm = Some((Confirmation::ReplaceImport, ConfirmState::new()));
          } else {
            self.start_pending_import();
            self.go(AppearancePage::Themes);
          }
        }
        Err(error) => {
          self.status = Some(StatusMessage {
            kind: StatusKind::Error,
            text: format!("{} {error}", tr(self.lang, "control_center.error")),
          })
        }
        Ok(_) => {}
      }
      changed = true;
    }
    changed
  }

  /// Stores a freshly loaded state. The page's draft is rebuilt from it only
  /// while the draft has no unapplied changes, so a reload (`r`, or the
  /// automatic one when a page is entered) never drops the user's edits;
  /// after `Apply` the draft matches the applied values and is rebuilt.
  fn on_loaded(&mut self, page: AppearancePage, state: AppearanceState) {
    self.refreshed.retain(|(old, _)| *old != page);
    self.refreshed.push((page, std::time::Instant::now()));
    let keep_draft = self.surface_draft_has_changes();
    self.state = state;
    self.loaded = true;
    self.status_loading = false;
    if page == AppearancePage::ControlPanel && self.page == page {
      self.control_panel_draft = Some(self.state.control_panel_cards.clone());
    }
    if let Some(surface) = Self::surface_for_page(page)
      && self.page == page
      && !keep_draft
    {
      self.surface_draft = Some(self.make_surface_draft(surface));
    }
    if self.page != page {
      self.refresh();
    }
  }

  /// Whether the surface draft differs from the loaded state.
  fn surface_draft_has_changes(&self) -> bool {
    self
      .surface_draft
      .as_ref()
      .is_some_and(|draft| *draft != self.make_surface_draft(draft.surface))
  }

  /// Whether the effect editor value differs from the stored one.
  fn effect_draft_has_changes(&self) -> bool {
    match (Self::effect_spec(self.page), self.effect_draft) {
      (Some((kind, surface)), Some(value)) => value != self.effect_value(kind, surface),
      _ => false,
    }
  }

  /// The Blur page as stored, before any unapplied edit.
  fn blur_saved(&self) -> BlurSettings {
    BlurSettings {
      enabled: self.state.blur,
      values: self.state.blur_values,
    }
  }

  /// The Blur page as shown: the draft when there is one, otherwise the stored state.
  fn blur_editor(&self) -> BlurSettings {
    self.blur_draft.unwrap_or_else(|| self.blur_saved())
  }

  /// Whether the Blur draft differs from the stored settings.
  fn blur_draft_has_changes(&self) -> bool {
    self
      .blur_draft
      .is_some_and(|draft| draft != self.blur_saved())
  }

  /// Changes the Blur draft; nothing is written until `Apply`.
  fn edit_blur(&mut self, edit: impl FnOnce(&mut BlurSettings)) {
    let mut draft = self.blur_editor();
    edit(&mut draft);
    self.blur_draft = Some(draft);
  }

  /// Whether the current page has an unapplied draft, which enables `Apply`.
  fn has_pending_changes(&self) -> bool {
    if self.page.is_animations() {
      self.animations_draft_has_changes()
    } else if self.page == AppearancePage::Blur {
      self.blur_draft_has_changes()
    } else if Self::effect_spec(self.page).is_some() {
      self.effect_draft_has_changes()
    } else {
      Self::surface_for_page(self.page).is_some() && self.surface_draft_has_changes()
    }
  }

  /// Whether this page edits a draft that is applied with the `Apply` row.
  fn is_draft_page(page: AppearancePage) -> bool {
    page == AppearancePage::Blur
      || page.is_animations()
      || Self::surface_for_page(page).is_some()
      || Self::effect_spec(page).is_some()
  }

  fn start_pending_import(&mut self) {
    let Some(path) = self.pending_import.take() else {
      return;
    };
    self.apply(
      tr(self.lang, "control_center.theme_profile_imported").into(),
      move || backend::import_theme_profile(&path).map(|_| ()),
    );
  }

  /// Applies the `apply` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn apply(&mut self, message: String, task: impl FnOnce() -> Result<(), String> + Send + 'static) {
    if self.action.is_some() {
      return;
    }
    self.reload_requested = true;
    self.action = Some(self.manager.spawn(move |_| {
      task()?;
      Ok(JobData::Action(message))
    }));
  }

  fn apply_result(&mut self, task: impl FnOnce() -> Result<String, String> + Send + 'static) {
    if self.action.is_some() {
      return;
    }
    self.reload_requested = true;
    self.action = Some(self.manager.spawn(move |_| {
      let text = task()?;
      Ok(JobData::Action(text))
    }));
  }
  /// Executes the `go` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn go(&mut self, page: AppearancePage) {
    if self.page != page
      && let Some(job) = self.job.take()
    {
      job.cancel();
      self.status_loading = false;
    }
    if self.page == AppearancePage::ControlPanel && page != AppearancePage::ControlPanel {
      self.control_panel_draft = None;
    }
    self.page = page;
    self.menu = MenuState::default();
    // The Icons draft, search and Actions focus belong to that page only.
    self.icon_search.clear();
    self.icon_searching = false;
    self.icon_draft = None;
    self.icon_actions = ActionFocus::default();
    if page == AppearancePage::ControlPanel {
      self.control_panel_draft = Some(self.state.control_panel_cards.clone());
    }
    if let Some(surface) = Self::surface_for_page(page) {
      if self.surface_draft.is_none()
        || self
          .surface_draft
          .as_ref()
          .is_some_and(|draft| draft.surface != surface)
      {
        self.surface_draft = Some(self.make_surface_draft(surface));
      }
    } else if !matches!(page, AppearancePage::SurfaceSection { .. }) {
      self.surface_draft = None;
    }
    self.ensure_page();
  }

  fn surface_for_page(page: AppearancePage) -> Option<EffectSurface> {
    match page {
      AppearancePage::Taskbar
      | AppearancePage::TaskbarIcons
      | AppearancePage::TaskbarDate
      | AppearancePage::TaskbarDateFormat
      | AppearancePage::TaskbarTime
      | AppearancePage::TaskbarTimeFormat => Some(EffectSurface::Taskbar),
      AppearancePage::WidgetTelemetry => Some(EffectSurface::WidgetTelemetry),
      AppearancePage::ControlPanel => Some(EffectSurface::ControlPanel),
      AppearancePage::SurfaceSection { surface, .. } => Some(surface),
      _ => None,
    }
  }

  fn make_surface_draft(&self, surface: EffectSurface) -> SurfaceDraft {
    let (transparency_enabled, transparency, blur_enabled, blur) = match surface {
      EffectSurface::Taskbar => (
        self.state.taskbar_transparency_enabled,
        self.state.taskbar_transparency,
        self.state.taskbar_blur_enabled,
        self.state.taskbar_blur,
      ),
      EffectSurface::ControlPanel => (
        self.state.control_panel_transparency_enabled,
        self.state.control_panel_transparency,
        self.state.control_panel_blur_enabled,
        self.state.control_panel_blur,
      ),
      EffectSurface::WidgetTelemetry => (
        self.state.widget_telemetry_transparency_enabled,
        self.state.widget_telemetry_transparency,
        self.state.widget_telemetry_blur_enabled,
        self.state.widget_telemetry_blur,
      ),
      EffectSurface::Terminal => (
        self.state.terminal_transparency_enabled,
        self.state.terminal_transparency,
        true,
        0,
      ),
      EffectSurface::Launchers => (
        self.state.launcher_transparency_enabled,
        self.state.launcher_transparency,
        true,
        0,
      ),
    };
    SurfaceDraft {
      surface,
      utility_group: self.state.taskbar_utility_group,
      audio_player_enabled: self.state.taskbar_audio_player_enabled,
      launcher_enabled: self.state.taskbar_launcher_enabled,
      utility_widgets: self.state.taskbar_utility_widgets.clone(),
      date_format: self.state.taskbar_date_format,
      time_seconds_enabled: self.state.taskbar_time_seconds_enabled,
      time_format: self.state.taskbar_time_format,
      widget_enabled: self.state.widget_telemetry,
      widget_blocks: self.state.widget_telemetry_blocks.clone(),
      widget_order: self.state.widget_telemetry_order.clone(),
      control_panel_enabled: self.state.control_panel_enabled,
      control_panel_cards: self.state.control_panel_cards.clone(),
      transparency_enabled,
      transparency,
      blur_enabled,
      blur,
    }
  }

  fn surface_draft(&self) -> Option<&SurfaceDraft> {
    self.surface_draft.as_ref()
  }

  fn surface_draft_mut(&mut self) -> Option<&mut SurfaceDraft> {
    self.surface_draft.as_mut()
  }

  fn apply_surface_changes(&mut self) {
    let Some(draft) = self.surface_draft.clone() else {
      return;
    };
    // Keep rendering the just-applied draft values until the background
    // refresh (triggered by `reload_requested` once the task below finishes)
    // replaces it with a freshly loaded one. Clearing it here immediately
    // made every toggle/radio on the page flash back to its pre-edit state
    // for the duration of the apply + refresh round trip.
    let current_state = self.state.clone();
    self.apply(
      tr(self.lang, "control_center.surface_settings_applied").into(),
      move || match draft.surface {
        EffectSurface::Taskbar => {
          backend::set_taskbar_utility_group(draft.utility_group)?;
          backend::apply_taskbar_icons_and_format(
            draft.audio_player_enabled,
            draft.launcher_enabled,
            &draft.utility_widgets,
            draft.date_format,
            draft.time_seconds_enabled,
            draft.time_format,
          )?;
          backend::apply_surface_effects(
            draft.surface,
            draft.transparency_enabled,
            draft.transparency,
            draft.blur_enabled,
            draft.blur,
          )
        }
        EffectSurface::WidgetTelemetry => {
          backend::apply_widget_telemetry(
            draft.widget_enabled,
            &draft.widget_blocks,
            &draft.widget_order,
          )?;
          backend::apply_surface_effects(
            draft.surface,
            draft.transparency_enabled,
            draft.transparency,
            draft.blur_enabled,
            draft.blur,
          )
        }
        EffectSurface::ControlPanel => {
          backend::set_control_panel_enabled(draft.control_panel_enabled)?;
          let changes = ControlPanelCard::ALL
            .into_iter()
            .filter_map(|card| {
              (draft.control_panel_cards.enabled(card)
                != current_state.control_panel_cards.enabled(card))
              .then_some((card, draft.control_panel_cards.enabled(card)))
            })
            .collect::<Vec<_>>();
          if !changes.is_empty() {
            backend::set_control_panel_cards(changes)?;
          }
          backend::apply_surface_effects(
            draft.surface,
            draft.transparency_enabled,
            draft.transparency,
            draft.blur_enabled,
            draft.blur,
          )
        }
        EffectSurface::Terminal => backend::apply_surface_effects(
          draft.surface,
          draft.transparency_enabled,
          draft.transparency,
          true,
          0,
        ),
        EffectSurface::Launchers => backend::apply_surface_effects(
          draft.surface,
          draft.transparency_enabled,
          draft.transparency,
          true,
          0,
        ),
      },
    );
  }
  /// Applies the `toggle` operation while preserving the persistence and local-update contract. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn toggle(&mut self) {
    let value = if self.state.rounded { "0" } else { "1" };
    let enable = !self.state.rounded;
    self.apply(
      tr(self.lang, "control_center.border_settings_applied").into(),
      move || {
        backend::set_border("rounded", value).and_then(|_| {
          if enable {
            backend::set_border("rounding", "2")
          } else {
            Ok(())
          }
        })
      },
    );
  }
  /// Runs what activating `item` does: Enter/`→` (and Space, see
  /// [`Self::page_key`]) on any row, or Enter/Space on a toggle. Each effect is
  /// the one the item had before the single menu list (immediate jobs stay
  /// immediate, draft edits stay in the draft).
  fn activate(&mut self, item: Item) {
    match item {
      Item::Desktop => self.go(AppearancePage::Desktop),
      Item::ApplicationStyle => self.go(AppearancePage::ApplicationStyle),
      Item::Themes => self.go(AppearancePage::Themes),
      Item::Accent => self.go(AppearancePage::Accents),
      Item::Wallpaper => self.go(AppearancePage::Wallpapers),
      Item::Icons => self.go(AppearancePage::Icons),
      Item::Fonts => self.open_fonts = true,
      // Choosing a theme only sets the draft; `[ Apply ]` writes it.
      Item::IconTheme(index) => self.select_icon_theme(index),
      Item::Terminal => self.go(AppearancePage::Terminal),
      Item::Launcher => self.go(AppearancePage::Launchers),
      Item::Mode => self.go(AppearancePage::Mode),
      Item::OfficialThemes => self.go(AppearancePage::OfficialThemes),
      Item::CustomThemes => self.go(AppearancePage::CustomThemes),
      Item::ExportTheme => self.open_prompt(PromptGoal::ExportProfile),
      Item::ImportTheme => self.go(AppearancePage::ThemeImport),
      Item::ThemeCategory(category) => self.go(AppearancePage::ThemeFamilies { category }),
      Item::Family(index) => {
        // Sticky/Float is an independent mode (Appearance > Mode), no longer
        // part of theme selection: picking a family applies it directly,
        // under whichever mode is already active.
        let Some(entry) = self.state.official_themes.get(index) else {
          return;
        };
        let name = entry.id.clone();
        self.apply(
          tr(self.lang, "control_center.theme_applied").into(),
          move || backend::set_theme(&name),
        );
      }
      Item::CustomTheme(index) => self.apply_custom_theme(index),
      Item::ImportArchive(index) => self.inspect_import(index),
      Item::ModeSticky => self.apply_layout_mode("sticky"),
      Item::ModeFloat => self.apply_layout_mode("float"),
      Item::ChooseWallpaper => self.apply(
        tr(self.lang, "control_center.wallpaper_chooser_opened").into(),
        backend::choose_wallpaper,
      ),
      Item::Collection(collection) => self.go(AppearancePage::WallpaperModes { collection }),
      Item::WallpaperMode(mode) => {
        if let AppearancePage::WallpaperModes { collection } = self.page {
          self.go(AppearancePage::WallpaperItems { collection, mode });
        }
      }
      Item::WallpaperFile(index) => {
        let Some(entry) = self.state.wallpapers.get(index) else {
          return;
        };
        let path = entry.path.clone();
        self.apply(
          tr(self.lang, "control_center.wallpaper_applied").into(),
          move || backend::set_wallpaper(&path),
        );
      }
      Item::EditAccent => {
        self.page = AppearancePage::AccentEdit;
        self.prompt_buffer = self.state.accent.clone();
        self.prompt_error = None;
      }
      Item::ResetAccent => self.apply(tr(self.lang, "control_center.accent_reset").into(), || {
        backend::set_accent("--theme-default")
      }),
      Item::TerminalTransparency => self.apply_transparency_toggle(EffectSurface::Terminal),
      Item::TerminalTransparencyValue => self.go(AppearancePage::TerminalTransparency),
      Item::LauncherTransparency => self.apply_transparency_toggle(EffectSurface::Launchers),
      Item::LauncherTransparencyValue => self.go(AppearancePage::TransparencySurface {
        surface: EffectSurface::Launchers,
      }),
      Item::EffectValue | Item::SectionValue => self.open_prompt(PromptGoal::DraftValue),
      Item::BlurField(field) => self.open_prompt(PromptGoal::BlurValue(field)),
      Item::Apply if self.page == AppearancePage::Blur => self.apply_blur_changes(),
      Item::Apply if self.page.is_animations() => self.apply_animations_changes(),
      Item::Apply if Self::effect_spec(self.page).is_some() => self.apply_effect_changes(),
      Item::Apply => self.apply_surface_changes(),
      Item::Animations => self.edit_animations(|draft| draft.enabled = !draft.enabled),
      Item::AnimationsCustomCurves => self.go(AppearancePage::AnimationsCustomCurves),
      Item::AnimationsGroup(group) => self.go(AppearancePage::AnimationsGroup { group }),
      Item::AnimationsCurve(curve) => self.go(AppearancePage::AnimationsCurve { curve }),
      Item::AnimationsLeaf(leaf) => self.go(AppearancePage::AnimationsLeaf { leaf }),
      Item::AnimationLeafEnabled(leaf) => self.edit_animations(|draft| {
        let values = draft.values.leaf_mut(leaf);
        values.enabled = !values.enabled;
      }),
      Item::AnimationLeafStyle(_) | Item::AnimationLeafCurve(_) => {
        self.step_animation_choice(item, 1);
      }
      Item::AnimationField(field) => self.open_prompt(PromptGoal::AnimationValue(field)),
      Item::BlurEnabled => self.edit_blur(|draft| draft.enabled = !draft.enabled),
      Item::TaskbarPosition => self.go(AppearancePage::TaskbarPosition),
      Item::TaskbarSpaces => self.go(AppearancePage::TaskbarSpaces),
      Item::WindowSpacesInner => self.go(AppearancePage::WindowSpacesInner),
      Item::WindowSpacesOuter => self.go(AppearancePage::WindowSpacesOuter),
      Item::GeneralBorders => self.go(AppearancePage::GeneralBorders),
      Item::EdgeThickness => self.go(AppearancePage::EdgeThickness),
      Item::Position(position) => {
        let position = position.value();
        self.apply(
          tr(self.lang, "control_center.taskbar_position_changed").into(),
          move || backend::set_waybar_position(position),
        );
      }
      Item::Spacing(goal) => self.open_prompt(goal),
      Item::Rounded => self.toggle(),
      Item::Rounding if self.state.rounded => self.open_prompt(PromptGoal::Rounding),
      Item::Rounding => {}
      Item::Thickness => self.open_prompt(PromptGoal::Thickness),
      Item::TaskbarTransparency => self.go(AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        section: SurfaceSection::Transparency,
      }),
      Item::TaskbarIcons => self.go(AppearancePage::TaskbarIcons),
      Item::TaskbarDate => self.go(AppearancePage::TaskbarDate),
      Item::TaskbarTime => self.go(AppearancePage::TaskbarTime),
      Item::AudioPlayer => self.edit_draft(|draft| {
        draft.audio_player_enabled = !draft.audio_player_enabled;
      }),
      Item::LauncherIcon => self.go(AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        section: SurfaceSection::Launcher,
      }),
      Item::LauncherEnabled => self.edit_draft(|draft| {
        draft.launcher_enabled = !draft.launcher_enabled;
      }),
      Item::ChooseLauncherIcon => self.apply(
        tr(self.lang, "control_center.taskbar_launcher_icon_chosen").into(),
        backend::choose_launcher_icon,
      ),
      Item::UtilityWidget(widget) => self.edit_draft(|draft| {
        let enabled = !draft.utility_widgets.enabled(widget);
        draft.utility_widgets.set(widget, enabled);
      }),
      Item::Utilities => self.go(AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        section: SurfaceSection::UtilityIcons,
      }),
      Item::DateFormats => self.go(AppearancePage::TaskbarDateFormat),
      Item::DateFormat(format) => self.edit_draft(|draft| draft.date_format = format),
      Item::Seconds => self.edit_draft(|draft| {
        draft.time_seconds_enabled = !draft.time_seconds_enabled;
      }),
      Item::TimeFormats => self.go(AppearancePage::TaskbarTimeFormat),
      Item::TimeFormat(format) => self.edit_draft(|draft| draft.time_format = format),
      Item::UtilityGroup(mode) => self.edit_draft(|draft| draft.utility_group = mode),
      Item::SurfaceEnabled => match self.page {
        AppearancePage::WidgetTelemetry => self.edit_draft(|draft| {
          draft.widget_enabled = !draft.widget_enabled;
        }),
        AppearancePage::ControlPanel => self.edit_draft(|draft| {
          draft.control_panel_enabled = !draft.control_panel_enabled;
        }),
        _ => {}
      },
      Item::Sessions => self.open_section(SurfaceSection::Sessions),
      Item::SurfaceTransparency => self.open_section(SurfaceSection::Transparency),
      Item::TelemetryBlock(block) => self.edit_draft(|draft| {
        let enabled = !draft.widget_blocks.enabled(block);
        draft.widget_blocks.set(block, enabled);
      }),
      Item::PanelCard(card) => self.edit_draft(|draft| {
        let enabled = !draft.control_panel_cards.enabled(card);
        draft.control_panel_cards.set(card, enabled);
      }),
      Item::SectionEnabled => {
        if matches!(self.page, AppearancePage::SurfaceSection { .. }) {
          self.edit_draft(|draft| draft.transparency_enabled = !draft.transparency_enabled);
        }
      }
    }
  }

  /// `←/→` (and `+/-`, `h/l`) on a value row with a step: changes the draft
  /// value by `delta`, bounded to 0–100.
  fn adjust(&mut self, item: Item, delta: i32) {
    match (item, self.page) {
      (Item::BlurField(field), AppearancePage::Blur) => {
        self.edit_blur(|draft| draft.values.adjust(field, delta));
      }
      (Item::AnimationField(field), _) => {
        self.edit_animations(|draft| draft.values.adjust(field, delta));
      }
      (Item::AnimationLeafStyle(_) | Item::AnimationLeafCurve(_), _) => {
        self.step_animation_choice(item, delta);
      }
      (Item::EffectValue, _) => {
        self.effect_draft = Some((self.effect_editor_value() + delta).clamp(0, 100));
      }
      (Item::SectionValue, _) => {
        self.adjust_section_value(|value| *value = (*value + delta).clamp(0, 100));
      }
      _ => {}
    }
  }

  /// Edits the percentage of the open transparency section draft.
  fn adjust_section_value(&mut self, edit: impl FnOnce(&mut i32)) {
    if matches!(self.page, AppearancePage::SurfaceSection { .. }) {
      self.edit_draft(|draft| edit(&mut draft.transparency));
    }
  }

  fn edit_draft(&mut self, edit: impl FnOnce(&mut SurfaceDraft)) {
    if let Some(draft) = self.surface_draft_mut() {
      edit(draft);
    }
  }

  /// Moves the focused Widget Telemetry block one position up or down in the
  /// draft order. Called by the router on `Shift+Up`/`Shift+Down` before
  /// falling back to [`Self::handle`], since key modifiers are not threaded
  /// through that generic entry point. Returns `false` when the focused row
  /// is not a telemetry block or the move would go past either end, so the
  /// caller can fall through to normal key handling.
  pub fn move_focused_telemetry_block(&mut self, move_down: bool) -> bool {
    if self.page
      != (AppearancePage::SurfaceSection {
        surface: EffectSurface::WidgetTelemetry,
        section: SurfaceSection::Sessions,
      })
    {
      return false;
    }
    let rows = self.rows();
    let Some(Item::TelemetryBlock(block)) = self.menu.selected_id(&rows) else {
      return false;
    };
    let Some(draft) = self.surface_draft_mut() else {
      return false;
    };
    let Some(index) = draft
      .widget_order
      .iter()
      .position(|candidate| *candidate == block)
    else {
      return false;
    };
    let target = if move_down {
      index + 1
    } else {
      index.wrapping_sub(1)
    };
    if target >= draft.widget_order.len() {
      return false;
    }
    draft.widget_order.swap(index, target);
    let rows = self.rows();
    self.menu.normalize(&rows);
    self.menu.select(&rows, &Item::TelemetryBlock(block));
    true
  }

  /// Moves the focused Control Panel card one position up or down, applied
  /// immediately via the package's own `move` primitive — the same one its
  /// real drag-and-drop panel already calls — rather than the draft+Apply
  /// flow used for enable/disable. Returns `false` when the focused row is
  /// not a panel card, a previous move is still applying, or the move would
  /// go past either visible end, so the caller can fall through to normal
  /// key handling.
  pub fn move_focused_control_panel_card(&mut self, move_down: bool) -> bool {
    if self.page
      != (AppearancePage::SurfaceSection {
        surface: EffectSurface::ControlPanel,
        section: SurfaceSection::Sessions,
      })
      || self.action.is_some()
    {
      return false;
    }
    let rows = self.rows();
    let Some(Item::PanelCard(card)) = self.menu.selected_id(&rows) else {
      return false;
    };
    let visible = self
      .state
      .control_panel_order
      .iter()
      .copied()
      .filter(|candidate| self.state.control_panel_cards.available(*candidate))
      .collect::<Vec<_>>();
    let Some(visible_index) = visible.iter().position(|candidate| *candidate == card) else {
      return false;
    };
    let target_visible = if move_down {
      visible_index + 1
    } else {
      visible_index.wrapping_sub(1)
    };
    let Some(neighbor) = visible.get(target_visible).copied() else {
      return false;
    };
    let Some(current_index) = self
      .state
      .control_panel_order
      .iter()
      .position(|candidate| *candidate == card)
    else {
      return false;
    };
    let Some(target_index) = self
      .state
      .control_panel_order
      .iter()
      .position(|candidate| *candidate == neighbor)
    else {
      return false;
    };
    // Optimistic local swap so the row moves immediately; the background
    // refresh triggered by `apply` reconciles with the script's own state.
    self
      .state
      .control_panel_order
      .swap(current_index, target_index);
    self.apply(
      tr(self.lang, "control_center.surface_settings_applied").into(),
      move || backend::move_control_panel_card(card, target_index),
    );
    let rows = self.rows();
    self.menu.normalize(&rows);
    self.menu.select(&rows, &Item::PanelCard(card));
    true
  }

  /// Opens a section of the current surface (Widget Telemetry or Control
  /// Panel), keeping its draft.
  fn open_section(&mut self, section: SurfaceSection) {
    if let Some(surface) = Self::surface_for_page(self.page) {
      self.go(AppearancePage::SurfaceSection { surface, section });
    }
  }

  fn apply_custom_theme(&mut self, index: usize) {
    let Some(custom) = self.state.custom_themes.get(index).cloned() else {
      return;
    };
    let lang = self.lang;
    self.apply_result(move || {
      backend::apply_custom_theme(&custom).map(|report| {
        if report.wallpaper_missing {
          format!(
            "{}: {}",
            tr(lang, "control_center.theme_profile_applied"),
            tr(lang, "control_center.theme_profile_wallpaper_missing")
          )
        } else {
          tr(lang, "control_center.theme_profile_applied").to_string()
        }
      })
    });
  }

  fn inspect_import(&mut self, index: usize) {
    let Some(path) = self.import_archives.get(index).cloned() else {
      return;
    };
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.theme_profile_importing").into(),
    });
    self.action = Some(self.manager.spawn(move |_| {
      let (name, duplicate) = backend::inspect_theme_profile(&path)?;
      Ok(JobData::ImportReady {
        path,
        name,
        duplicate,
      })
    }));
  }

  fn apply_layout_mode(&mut self, variant: &'static str) {
    self.apply(
      tr(self.lang, "control_center.layout_mode_applied").into(),
      move || backend::set_layout_mode(variant),
    );
  }

  /// Flips the Terminal or Launcher transparency immediately, keeping its
  /// stored percentage.
  fn apply_transparency_toggle(&mut self, surface: EffectSurface) {
    let (enabled, transparency) = if surface == EffectSurface::Terminal {
      (
        !self.state.terminal_transparency_enabled,
        self.state.terminal_transparency,
      )
    } else {
      (
        !self.state.launcher_transparency_enabled,
        self.state.launcher_transparency,
      )
    };
    self.apply(
      tr(self.lang, "control_center.transparency_applied").into(),
      move || backend::apply_surface_effects(surface, enabled, transparency, true, 0),
    );
  }
  /// Executes the `open_prompt` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn open_prompt(&mut self, goal: PromptGoal) {
    self.prompt_back = Some(self.page);
    self.page = AppearancePage::Prompt { goal };
    self.prompt_buffer.clear();
    self.prompt_error = None;
  }
  /// Executes the `prompt_key` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn prompt_key(&mut self, key: KeyCode) -> bool {
    if self.page == AppearancePage::AccentEdit {
      match key {
        KeyCode::Enter if self.action.is_none() => {
          if let Some(color) = normalize_hex_color(&self.prompt_buffer) {
            self.apply(
              tr(self.lang, "control_center.accent_applied").into(),
              move || backend::set_accent(&color),
            );
            self.go(AppearancePage::Accents);
          } else {
            self.prompt_error = Some(tr(self.lang, "control_center.accent_invalid").into());
          }
        }
        KeyCode::Esc | KeyCode::Left => self.go(AppearancePage::Accents),
        KeyCode::Backspace | KeyCode::Delete => {
          self.prompt_buffer.pop();
          self.prompt_error = None;
        }
        KeyCode::Char('#') if self.prompt_buffer.is_empty() => self.prompt_buffer.push('#'),
        KeyCode::Char(c)
          if c.is_ascii_hexdigit() && self.prompt_buffer.trim_start_matches('#').len() < 6 =>
        {
          self.prompt_buffer.push(c.to_ascii_uppercase());
          self.prompt_error = None;
        }
        _ => {}
      }
      return false;
    }
    match key {
      KeyCode::Enter if self.action.is_none() => {
        let AppearancePage::Prompt { goal } = self.page else {
          return false;
        };
        let value = self.prompt_buffer.trim().to_string();
        if let PromptGoal::BlurValue(field) = goal {
          self.commit_blur_value(field, &value);
          return false;
        }
        if let PromptGoal::AnimationValue(field) = goal {
          self.commit_animation_value(field, &value);
          return false;
        }
        if goal == PromptGoal::ExportProfile {
          let back = self.prompt_back.take().unwrap_or(AppearancePage::Themes);
          if value.is_empty() {
            self.prompt_error =
              Some(tr(self.lang, "control_center.theme_profile_invalid_name").into());
            self.prompt_back = Some(back);
            return false;
          }
          let message = tr(self.lang, "control_center.theme_profile_exported").to_string();
          self.apply_result(move || {
            let path = backend::export_theme_profile(&value)?;
            Ok(format!("{message}: {}", path.display()))
          });
          self.go(back);
          return false;
        }
        let (min, max) = goal.range();
        let valid = value
          .parse::<i32>()
          .ok()
          .is_some_and(|v| (min..=max).contains(&v));
        if !valid {
          self.prompt_error = Some(format!(
            "{} ({}–{})",
            tr(self.lang, "control_center.enter_a_valid_integer"),
            min,
            max
          ));
          return false;
        }
        let back = self.prompt_back.take().unwrap_or(AppearancePage::Home);
        if goal == PromptGoal::DraftValue {
          if let Ok(value) = value.parse::<i32>() {
            self.set_draft_value(back, value);
          }
          return false;
        }
        let key_name = goal.key();
        let message_key = if matches!(goal, PromptGoal::Rounding) {
          "control_center.border_settings_applied"
        } else if matches!(goal, PromptGoal::Thickness) {
          "control_center.thickness_applied"
        } else {
          "control_center.spacing_applied"
        };
        self.apply(tr(self.lang, message_key).into(), move || match goal {
          PromptGoal::Rounding | PromptGoal::Thickness => backend::set_border(key_name, &value),
          _ => backend::set_spacing(key_name, &value),
        });
        self.go(back);
        false
      }
      KeyCode::Esc | KeyCode::Left => {
        let back = self.prompt_back.take().unwrap_or(AppearancePage::Home);
        self.go(back);
        false
      }
      KeyCode::Char(c)
        if matches!(
          self.page,
          AppearancePage::Prompt {
            goal: PromptGoal::ExportProfile | PromptGoal::ImportProfile
          }
        ) && !c.is_control()
          && self.prompt_buffer.len() < 512 =>
      {
        self.prompt_buffer.push(c);
        false
      }
      KeyCode::Char(c)
        if c.is_ascii_digit() && self.prompt_buffer.len() < self.prompt_digit_limit() =>
      {
        self.prompt_buffer.push(c);
        false
      }
      KeyCode::Char('.') if self.decimal_prompt() && !self.prompt_buffer.contains('.') => {
        self.prompt_buffer.push('.');
        false
      }
      KeyCode::Char('-')
        if self
          .animation_prompt_field()
          .is_some_and(AnimationField::allows_negative)
          && self.prompt_buffer.is_empty() =>
      {
        self.prompt_buffer.push('-');
        false
      }
      KeyCode::Backspace => {
        self.prompt_buffer.pop();
        false
      }
      _ => false,
    }
  }
  /// The Blur field whose value the open prompt is typing, if any.
  fn blur_prompt_field(&self) -> Option<BlurField> {
    match self.page {
      AppearancePage::Prompt {
        goal: PromptGoal::BlurValue(field),
      } => Some(field),
      _ => None,
    }
  }

  /// The Animations field whose value the open prompt is typing, if any.
  fn animation_prompt_field(&self) -> Option<AnimationField> {
    match self.page {
      AppearancePage::Prompt {
        goal: PromptGoal::AnimationValue(field),
      } => Some(field),
      _ => None,
    }
  }

  /// Whether the open prompt types a decimal value: Blur parameters and the
  /// Animations curve parameters.
  fn decimal_prompt(&self) -> bool {
    self.blur_prompt_field().is_some()
      || self
        .animation_prompt_field()
        .is_some_and(|field| !field.is_integer())
  }

  /// Longest text a numeric prompt accepts. Decimal values need room for a
  /// value such as `0.900000`.
  fn prompt_digit_limit(&self) -> usize {
    if self.decimal_prompt() { 8 } else { 3 }
  }

  /// Writes a typed Blur parameter into the Blur draft and returns to the page
  /// with its row selected. Nothing is applied; an out-of-range value keeps
  /// the prompt open with the accepted bounds.
  fn commit_blur_value(&mut self, field: BlurField, text: &str) {
    let Some(value) = field.parse(text) else {
      let (minimum, maximum) = field.bounds();
      self.prompt_error = Some(format!(
        "{} ({}–{})",
        tr(self.lang, "control_center.enter_a_valid_number"),
        field.format(minimum),
        field.format(maximum)
      ));
      return;
    };
    self.prompt_back = None;
    self.go(AppearancePage::Blur);
    self.edit_blur(|draft| draft.values.set(field, value));
    let rows = self.rows();
    self.menu.normalize(&rows);
    self.menu.select(&rows, &Item::BlurField(field));
  }

  /// Writes the Blur draft. The switch and the parameters go to the session
  /// script, which reloads the compositor. The page stays open.
  fn apply_blur_changes(&mut self) {
    if self.action.is_some() {
      return;
    }
    let Some(draft) = self.blur_draft.take() else {
      return;
    };
    let enabled_changed = draft.enabled != self.state.blur;
    let values_changed = draft.values != self.state.blur_values;
    if !enabled_changed && !values_changed {
      return;
    }
    self.apply(
      tr(self.lang, "control_center.blur_applied").into(),
      move || {
        if enabled_changed {
          backend::set_blur(draft.enabled)?;
        }
        if values_changed {
          backend::set_blur_values(draft.values)?;
        }
        Ok(())
      },
    );
  }

  /// Writes a typed percentage into the draft of `page` and returns to it
  /// with the value row selected. Nothing is applied.
  fn set_draft_value(&mut self, page: AppearancePage, value: i32) {
    self.go(page);
    let item = if Self::effect_spec(page).is_some() {
      self.effect_draft = Some(value);
      Item::EffectValue
    } else {
      self.adjust_section_value(|current| *current = value);
      Item::SectionValue
    };
    let rows = self.rows();
    self.menu.normalize(&rows);
    self.menu.select(&rows, &item);
  }

  /// Whether typed characters currently go to a prompt field (theme name,
  /// file path, gaps, borders) or to the accent HEX editor, so `q`/`?` must
  /// not act as the global quit/help keys. Same condition [`Self::handle`]
  /// uses to route keys to `prompt_key`.
  pub fn captures_text(&self) -> bool {
    self.prompt_back.is_some() || self.page == AppearancePage::AccentEdit || self.icon_searching
  }

  /// Processes one key press. Returns `true` when the user leaves the
  /// Appearance Home and the router should go back to the Control Center.
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if self.page == AppearancePage::Icons
      && self.confirm.is_none()
      && let Some(leave) = self.handle_icon_key(key)
    {
      return leave;
    }
    if self.captures_text() {
      return self.prompt_key(key);
    }
    if let Some((confirmation, state)) = &mut self.confirm {
      let confirmation = *confirmation;
      match state.handle(key) {
        ConfirmOutcome::Pending => {}
        ConfirmOutcome::Cancelled => {
          self.confirm = None;
          self.cancel_confirmation(confirmation);
        }
        ConfirmOutcome::Confirmed => {
          self.confirm = None;
          return self.resolve_confirmation(confirmation);
        }
      }
      return false;
    }
    let rows = self.rows();
    let mut menu = self.menu;
    menu.normalize(&rows);
    let key = Self::page_key(menu.selected_kind(&rows), key);
    let event = menu.handle(key, &rows, usize::from(self.list_height));
    // While a load or an action runs, only moving and going back are
    // allowed (plus opening a section from the Home), as before.
    let busy = self.job.is_some() || self.action.is_some();
    if busy
      && !matches!(event, MenuEvent::Moved | MenuEvent::Back)
      && !(self.page == AppearancePage::Home && matches!(event, MenuEvent::Activate(_)))
    {
      return false;
    }
    self.menu = menu;
    match event {
      MenuEvent::Back => return self.back(),
      MenuEvent::Activate(item) | MenuEvent::Toggle(item) | MenuEvent::Confirm(item) => {
        self.activate(item);
      }
      MenuEvent::Adjust(item, delta) => self.adjust(item, delta),
      MenuEvent::Moved => {}
      MenuEvent::None => self.page_shortcut(key, &rows),
    }
    false
  }

  /// Appearance keeps two key aliases on top of the shared menu:
  ///
  /// - Space activates Action, Submenu, Choice and Destructive rows and opens
  ///   the editor of a Value row without a step, as it always did here (the
  ///   shared menu only uses Space for toggles);
  /// - `+`/`l` and `-`/`h` adjust a Value row with a step, like `→`/`←`.
  fn page_key(selected: Option<RowKind>, key: KeyCode) -> KeyCode {
    match (selected, key) {
      (
        Some(
          RowKind::Action
          | RowKind::Submenu
          | RowKind::Choice { .. }
          | RowKind::Destructive
          | RowKind::Value { step: None },
        ),
        KeyCode::Char(' '),
      ) => KeyCode::Enter,
      (Some(RowKind::Value { step: Some(_) }), KeyCode::Char('+' | 'l')) => KeyCode::Right,
      (Some(RowKind::Value { step: Some(_) }), KeyCode::Char('-' | 'h')) => KeyCode::Left,
      _ => key,
    }
  }

  /// One-key shortcuts that are not part of the menu: `r` reloads, and the
  /// theme pages keep `e` (export), `i` (import) and `d` (delete).
  fn page_shortcut(&mut self, key: KeyCode, rows: &[Row<Item>]) {
    let theme_page = matches!(
      self.page,
      AppearancePage::Themes | AppearancePage::CustomThemes
    );
    match key {
      KeyCode::Char('r') => self.refresh(),
      KeyCode::Char('e') if theme_page => self.open_prompt(PromptGoal::ExportProfile),
      KeyCode::Char('i') if theme_page => self.go(AppearancePage::ThemeImport),
      KeyCode::Char('d') if self.page == AppearancePage::CustomThemes => {
        if let Some(Item::CustomTheme(index)) = self.menu.selected_id(rows)
          && let Some(theme) = self.state.custom_themes.get(index).cloned()
        {
          self.delete_theme = Some(theme);
          self.confirm = Some((Confirmation::DeleteTheme, ConfirmState::new()));
        }
      }
      _ => {}
    }
  }

  /// Executes the `back` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn back(&mut self) -> bool {
    if self.back_discards_draft() {
      self.confirm = Some((Confirmation::DiscardDraft, ConfirmState::new()));
      return false;
    }
    self.navigate_back()
  }

  /// Whether going back from this page would drop an unapplied draft: the
  /// effect editors always drop theirs, and a surface draft lives until its
  /// top page (Taskbar, Widget Telemetry, Control Panel) is left. While
  /// `Apply` runs, the draft is already on its way and nothing is lost.
  fn back_discards_draft(&self) -> bool {
    if self.action.is_some() {
      return false;
    }
    if self.page == AppearancePage::Icons {
      return self.icon_changed();
    }
    if self.page == AppearancePage::Animations {
      return self.animations_draft_has_changes();
    }
    // Sub-pages of Animations keep the draft; only leaving its root drops it.
    if self.page.is_animations() {
      return false;
    }
    if self.page == AppearancePage::Blur {
      return self.blur_draft_has_changes();
    }
    if Self::effect_spec(self.page).is_some() {
      return self.effect_draft_has_changes();
    }
    matches!(
      self.page,
      AppearancePage::Taskbar | AppearancePage::WidgetTelemetry | AppearancePage::ControlPanel
    ) && self.surface_draft_has_changes()
  }

  /// Runs the confirmed decision. Returns `true` when Appearance is left.
  fn resolve_confirmation(&mut self, confirmation: Confirmation) -> bool {
    match confirmation {
      Confirmation::DiscardDraft => self.navigate_back(),
      Confirmation::DeleteTheme => {
        if let Some(theme) = self.delete_theme.take() {
          self.apply(
            tr(self.lang, "control_center.theme_profile_deleted").into(),
            move || backend::delete_custom_theme(&theme),
          );
        }
        self.go(AppearancePage::Themes);
        false
      }
      Confirmation::ReplaceImport => {
        self.start_pending_import();
        self.go(AppearancePage::Themes);
        false
      }
    }
  }

  /// Cancelling a theme confirmation returns to the Themes page, as the old
  /// confirmation pages did; cancelling a discard keeps the page and draft.
  fn cancel_confirmation(&mut self, confirmation: Confirmation) {
    match confirmation {
      Confirmation::DiscardDraft => {}
      Confirmation::DeleteTheme => {
        self.delete_theme = None;
        self.go(AppearancePage::Themes);
      }
      Confirmation::ReplaceImport => {
        self.pending_import = None;
        self.pending_import_name = None;
        self.go(AppearancePage::Themes);
      }
    }
  }

  /// Goes to the parent page, dropping the drafts the parent does not keep.
  fn navigate_back(&mut self) -> bool {
    if self.page == AppearancePage::Home {
      return true;
    }
    match self.page {
      AppearancePage::Home => unreachable!(),
      // Both groups are Home cards of the principal crate, so leaving them
      // returns straight to the Control Center Home instead of this crate's
      // root (which only the CLI `appearance` route reaches).
      AppearancePage::Desktop | AppearancePage::ApplicationStyle => {
        return true;
      }
      AppearancePage::Themes
      | AppearancePage::Wallpapers
      | AppearancePage::Icons
      | AppearancePage::Accents
      | AppearancePage::Mode => {
        self.go(AppearancePage::Desktop);
      }
      AppearancePage::Terminal => self.go(AppearancePage::ApplicationStyle),
      AppearancePage::OfficialThemes => self.go(AppearancePage::Themes),
      AppearancePage::ThemeFamilies { .. } => {
        self.go(AppearancePage::OfficialThemes);
      }
      AppearancePage::CustomThemes => self.go(AppearancePage::Themes),
      AppearancePage::WallpaperModes { .. } => self.go(AppearancePage::Wallpapers),
      AppearancePage::WallpaperItems { collection, .. } => {
        self.go(AppearancePage::WallpaperModes { collection });
      }
      AppearancePage::ThemeImport => self.go(AppearancePage::Themes),
      AppearancePage::TaskbarPosition | AppearancePage::TaskbarSpaces => {
        self.go(AppearancePage::Taskbar);
      }
      // Entry points reached directly from a Home (the Hyprland category or
      // the Control Center Home itself, both in the crate principal);
      // there is no parent page left inside this crate, so leaving goes
      // back to whichever page opened them.
      AppearancePage::WindowSpaces | AppearancePage::Borders => {
        return true;
      }
      AppearancePage::Animations => {
        self.animations_draft = None;
        return true;
      }
      AppearancePage::AnimationsCustomCurves | AppearancePage::AnimationsGroup { .. } => {
        self.go(AppearancePage::Animations);
      }
      AppearancePage::AnimationsCurve { .. } => {
        self.go(AppearancePage::AnimationsCustomCurves);
      }
      AppearancePage::AnimationsLeaf { leaf } => {
        self.go(AppearancePage::AnimationsGroup {
          group: leaf.group(),
        });
      }
      // Same kind of entry point, but these three carry a surface draft
      // that otherwise would never be cleared (previously `go(Home)` did
      // that implicitly; see `back_discards_draft`).
      AppearancePage::Taskbar | AppearancePage::WidgetTelemetry | AppearancePage::ControlPanel => {
        self.surface_draft = None;
        return true;
      }
      AppearancePage::WindowSpacesInner | AppearancePage::WindowSpacesOuter => {
        self.go(AppearancePage::WindowSpaces);
      }
      AppearancePage::GeneralBorders | AppearancePage::EdgeThickness => {
        self.go(AppearancePage::Borders);
      }
      AppearancePage::TaskbarIcons | AppearancePage::TaskbarDate | AppearancePage::TaskbarTime => {
        self.go(AppearancePage::Taskbar);
      }
      AppearancePage::TaskbarDateFormat => self.go(AppearancePage::TaskbarDate),
      AppearancePage::TaskbarTimeFormat => self.go(AppearancePage::TaskbarTime),
      AppearancePage::AccentEdit => self.go(AppearancePage::Accents),
      // Unreachable page kept until the Phase 4 cleanup (see rows.rs).
      AppearancePage::Transparency => self.go(AppearancePage::Home),
      // Entry point reached from the Hyprland category (crate principal);
      // like the other effect editors, leaving drops the unapplied value.
      AppearancePage::Blur => {
        self.blur_draft = None;
        return true;
      }
      AppearancePage::TerminalTransparency => {
        self.effect_draft = None;
        self.go(AppearancePage::Terminal);
      }
      AppearancePage::Launchers => self.go(AppearancePage::ApplicationStyle),
      AppearancePage::TransparencySurface {
        surface: EffectSurface::Launchers,
      } => {
        self.effect_draft = None;
        self.go(AppearancePage::Launchers);
      }
      AppearancePage::TransparencySurface { .. } => {
        self.effect_draft = None;
        self.go(AppearancePage::Transparency);
      }
      AppearancePage::SurfaceSection { surface, section } => {
        self.go(match (surface, section) {
          (EffectSurface::Taskbar, SurfaceSection::UtilityIcons) => AppearancePage::TaskbarIcons,
          (EffectSurface::Taskbar, _) => AppearancePage::Taskbar,
          (EffectSurface::WidgetTelemetry, _) => AppearancePage::WidgetTelemetry,
          (EffectSurface::ControlPanel, _) => AppearancePage::ControlPanel,
          (EffectSurface::Terminal, _) => AppearancePage::Terminal,
          (EffectSurface::Launchers, _) => AppearancePage::Launchers,
        });
      }
      AppearancePage::Prompt { .. } => {}
    }
    false
  }
  /// The Animations settings as stored, before any unapplied edit.
  fn animations_saved(&self) -> AnimationSettings {
    AnimationSettings {
      enabled: self.state.animations,
      values: self.state.animation_values,
    }
  }

  /// The Animations settings as shown: the draft when there is one, otherwise
  /// the stored state. Every Animations page reads the same draft.
  fn animations_editor(&self) -> AnimationSettings {
    self
      .animations_draft
      .unwrap_or_else(|| self.animations_saved())
  }

  /// Whether the Animations draft differs from the stored settings.
  fn animations_draft_has_changes(&self) -> bool {
    self
      .animations_draft
      .is_some_and(|draft| draft != self.animations_saved())
  }

  /// Changes the Animations draft; nothing is written until `Apply`.
  fn edit_animations(&mut self, edit: impl FnOnce(&mut AnimationSettings)) {
    let mut draft = self.animations_editor();
    edit(&mut draft);
    self.animations_draft = Some(draft);
  }

  /// Moves the style or the curve of a leaf by `delta` choices, wrapping around.
  fn step_animation_choice(&mut self, item: Item, delta: i32) {
    match item {
      Item::AnimationLeafStyle(leaf) => self.edit_animations(|draft| {
        let values = draft.values.leaf_mut(leaf);
        values.style = cycle(leaf.styles(), values.style, delta);
      }),
      Item::AnimationLeafCurve(leaf) => self.edit_animations(|draft| {
        let values = draft.values.leaf_mut(leaf);
        values.curve = cycle(&AnimationCurve::ALL, values.curve, delta);
      }),
      _ => {}
    }
  }

  /// Writes a typed animation value into the Animations draft and returns to
  /// the page it was opened from, with its row selected. Nothing is applied;
  /// an out-of-range value keeps the prompt open with the accepted bounds.
  fn commit_animation_value(&mut self, field: AnimationField, text: &str) {
    let Some(value) = field.parse(text) else {
      let (minimum, maximum) = field.bounds();
      self.prompt_error = Some(format!(
        "{} ({}–{})",
        tr(self.lang, "control_center.enter_a_valid_number"),
        field.format(minimum),
        field.format(maximum)
      ));
      return;
    };
    let back = self
      .prompt_back
      .take()
      .unwrap_or(AppearancePage::Animations);
    self.go(back);
    self.edit_animations(|draft| draft.values.set(field, value));
    let rows = self.rows();
    self.menu.normalize(&rows);
    self.menu.select(&rows, &Item::AnimationField(field));
  }

  /// Writes the Animations draft. The global switch and the values go to the
  /// session script, which reloads the compositor. The page stays open.
  fn apply_animations_changes(&mut self) {
    if self.action.is_some() {
      return;
    }
    let Some(draft) = self.animations_draft.take() else {
      return;
    };
    let enabled_changed = draft.enabled != self.state.animations;
    let values_changed = draft.values != self.state.animation_values;
    if !enabled_changed && !values_changed {
      return;
    }
    self.apply(
      tr(self.lang, "control_center.animations_applied").into(),
      move || {
        if enabled_changed {
          backend::set_animations(draft.enabled)?;
        }
        if values_changed {
          backend::set_animation_values(draft.values)?;
        }
        Ok(())
      },
    );
  }
  fn effect_spec(page: AppearancePage) -> Option<(&'static str, EffectSurface)> {
    match page {
      AppearancePage::TerminalTransparency => Some(("transparency", EffectSurface::Terminal)),
      AppearancePage::TransparencySurface {
        surface: EffectSurface::Launchers,
      } => Some(("transparency", EffectSurface::Launchers)),
      AppearancePage::TransparencySurface { surface } => Some(("transparency", surface)),
      _ => None,
    }
  }

  fn effect_value(&self, kind: &str, surface: EffectSurface) -> i32 {
    match (kind, surface) {
      ("transparency", EffectSurface::Taskbar) => self.state.taskbar_transparency,
      ("transparency", EffectSurface::ControlPanel) => self.state.control_panel_transparency,
      ("transparency", EffectSurface::WidgetTelemetry) => self.state.widget_telemetry_transparency,
      ("blur", EffectSurface::Taskbar) => self.state.taskbar_blur,
      ("blur", EffectSurface::ControlPanel) => self.state.control_panel_blur,
      ("blur", EffectSurface::WidgetTelemetry) => self.state.widget_telemetry_blur,
      ("transparency", EffectSurface::Terminal) => self.state.terminal_transparency,
      ("transparency", EffectSurface::Launchers) => self.state.launcher_transparency,
      _ => 0,
    }
  }

  fn effect_editor_value(&self) -> i32 {
    Self::effect_spec(self.page)
      .map(|(kind, surface)| {
        self
          .effect_draft
          .unwrap_or_else(|| self.effect_value(kind, surface))
      })
      .unwrap_or(0)
  }

  fn apply_effect_changes(&mut self) {
    let Some((kind, surface)) = Self::effect_spec(self.page) else {
      return;
    };
    let value = self.effect_editor_value();
    let terminal_transparency_enabled = self.state.terminal_transparency_enabled;
    let launcher_transparency_enabled = self.state.launcher_transparency_enabled;
    self.effect_draft = None;
    let back = AppearancePage::Transparency;
    self.apply(
      tr(self.lang, "control_center.effect_value_applied").into(),
      move || {
        if kind == "transparency" && surface == EffectSurface::Terminal {
          backend::apply_surface_effects(surface, terminal_transparency_enabled, value, true, 0)
        } else if kind == "transparency" && surface == EffectSurface::Launchers {
          backend::apply_surface_effects(surface, launcher_transparency_enabled, value, true, 0)
        } else {
          backend::set_effect_value(kind, surface, value)
        }
      },
    );
    self.go(
      if kind == "transparency" && surface == EffectSurface::Terminal {
        AppearancePage::Terminal
      } else if kind == "transparency" && surface == EffectSurface::Launchers {
        AppearancePage::Launchers
      } else {
        back
      },
    );
  }
  /// Executes the `prompt_label` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn prompt_label(&self, goal: PromptGoal) -> &'static str {
    match goal {
      PromptGoal::ExportProfile => "control_center.theme_profile_export_name",
      PromptGoal::ImportProfile => "control_center.theme_profile_import_path",
      PromptGoal::WaybarTop => "control_center.top",
      PromptGoal::WaybarLeft => "control_center.left",
      PromptGoal::WaybarRight => "control_center.right",
      PromptGoal::WaybarBottom => "control_center.bottom",
      PromptGoal::GapsIn => "control_center.inner_gap",
      PromptGoal::GapsOutTop => "control_center.outer_gap_top",
      PromptGoal::GapsOutLeft => "control_center.outer_gap_left",
      PromptGoal::GapsOutRight => "control_center.outer_gap_right",
      PromptGoal::GapsOutBottom => "control_center.outer_gap_bottom",
      PromptGoal::Rounding | PromptGoal::Thickness => "control_center.value",
      PromptGoal::DraftValue => "control_center.value",
      PromptGoal::BlurValue(field) => field.label_key(),
      PromptGoal::AnimationValue(field) => field.label_key(),
    }
  }
  /// Executes the `breadcrumb` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn breadcrumb(&self) -> String {
    self.breadcrumb_of(self.page)
  }

  fn breadcrumb_of(&self, page: AppearancePage) -> String {
    let root = HomeCardId::Appearance.label(self.lang);
    // Window Spaces, Animations, Blur and Borders are owned by the Hyprland
    // domain: they only exist behind the Hyprland router (crate principal),
    // even though they are implemented in this crate for reuse. Their
    // breadcrumb reflects that ownership instead of this crate's own root.
    // Hyprland is listed under the Desktop & Input card on the principal
    // Home; Taskbar, Control Center and Widget Telemetry are listed under
    // Appearance, so their breadcrumbs start from the Appearance root.
    let desktop_input_root = HomeCardId::DesktopInput.label(self.lang);
    let hyprland_root = format!(
      "{desktop_input_root} > {}",
      tr(self.lang, "control_center.hyprland")
    );
    let taskbar_root = format!("{root} > {}", tr(self.lang, "control_center.taskbar"));
    let control_panel_root = format!("{root} > {}", tr(self.lang, "control_center.control_panel"));
    let widget_telemetry_root = format!(
      "{root} > {}",
      tr(self.lang, "control_center.widget_telemetry")
    );
    // Theme, Appearance Mode, Accent Color and Wallpaper live under the
    // Desktop group; Terminal and Launcher live under Application Style.
    let desktop_root = format!("{root} > {}", tr(self.lang, "control_center.desktop_style"));
    let style_root = format!(
      "{root} > {}",
      tr(self.lang, "control_center.application_style")
    );
    match page {
      AppearancePage::Home => root.into(),
      AppearancePage::Desktop => desktop_root,
      AppearancePage::ApplicationStyle => style_root,
      AppearancePage::Themes => {
        format!("{desktop_root} > {}", tr(self.lang, "control_center.theme"))
      }
      AppearancePage::OfficialThemes => format!(
        "{desktop_root} > {} > {}",
        tr(self.lang, "control_center.theme"),
        tr(self.lang, "control_center.theme_profile_official")
      ),
      AppearancePage::ThemeImport => format!(
        "{desktop_root} > {} > {}",
        tr(self.lang, "control_center.theme"),
        tr(self.lang, "control_center.theme_profile_import")
      ),
      AppearancePage::ThemeFamilies { category } => format!(
        "{desktop_root} > {} > {} > {}",
        tr(self.lang, "control_center.theme"),
        tr(self.lang, "control_center.theme_profile_official"),
        tr(self.lang, theme_category_label_key(category))
      ),
      AppearancePage::CustomThemes => format!(
        "{desktop_root} > {} > {}",
        tr(self.lang, "control_center.theme"),
        tr(self.lang, "control_center.theme_profile_custom")
      ),
      AppearancePage::Mode => format!(
        "{desktop_root} > {}",
        tr(self.lang, "control_center.appearance_mode")
      ),
      AppearancePage::Wallpapers => {
        format!(
          "{desktop_root} > {}",
          tr(self.lang, "control_center.wallpapers")
        )
      }
      AppearancePage::Icons => {
        format!("{desktop_root} > {}", tr(self.lang, "control_center.icons"))
      }
      AppearancePage::WallpaperModes { collection } => format!(
        "{desktop_root} > {} > {}",
        tr(self.lang, "control_center.wallpapers"),
        tr(self.lang, collection.label_key())
      ),
      AppearancePage::WallpaperItems { collection, mode } => format!(
        "{desktop_root} > {} > {} > {}",
        tr(self.lang, "control_center.wallpapers"),
        tr(self.lang, collection.label_key()),
        tr(self.lang, mode.label_key())
      ),
      AppearancePage::Accents => format!(
        "{desktop_root} > {}",
        tr(self.lang, "control_center.highlight_color")
      ),
      AppearancePage::AccentEdit => format!(
        "{desktop_root} > {} > {}",
        tr(self.lang, "control_center.highlight_color"),
        tr(self.lang, "control_center.edit_highlight_color")
      ),
      // Unreachable page kept until the Phase 4 cleanup (see rows.rs); its
      // former parent ("Effects") no longer exists.
      AppearancePage::Transparency => {
        format!("{root} > {}", tr(self.lang, "control_center.transparency"))
      }
      // Entered directly from the Hyprland category (crate principal); no
      // intermediate "Effects" level in this crate anymore.
      AppearancePage::Blur => format!("{hyprland_root} > {}", tr(self.lang, "control_center.blur")),
      AppearancePage::Terminal => format!(
        "{style_root} > {}",
        tr(self.lang, "control_center.terminal")
      ),
      AppearancePage::TerminalTransparency => format!(
        "{style_root} > {} > {}",
        tr(self.lang, "control_center.terminal"),
        tr(self.lang, "control_center.transparency")
      ),
      AppearancePage::Launchers => format!(
        "{style_root} > {}",
        tr(self.lang, "control_center.launcher")
      ),
      AppearancePage::TransparencySurface { surface } => format!(
        "{root} > {} > {} > {}",
        tr(self.lang, "control_center.effects"),
        tr(self.lang, "control_center.transparency"),
        tr(self.lang, surface.label_key())
      ),
      AppearancePage::TaskbarPosition => format!(
        "{taskbar_root} > {}",
        tr(self.lang, "control_center.position")
      ),
      AppearancePage::TaskbarSpaces => format!(
        "{taskbar_root} > {}",
        tr(self.lang, "control_center.spaces")
      ),
      AppearancePage::Taskbar => taskbar_root.clone(),
      AppearancePage::TaskbarIcons => {
        format!("{taskbar_root} > {}", tr(self.lang, "control_center.icons"))
      }
      AppearancePage::TaskbarDate => {
        format!("{taskbar_root} > {}", tr(self.lang, "control_center.date"))
      }
      AppearancePage::TaskbarDateFormat => format!(
        "{taskbar_root} > {} > {}",
        tr(self.lang, "control_center.date"),
        tr(self.lang, "control_center.format")
      ),
      AppearancePage::TaskbarTime => {
        format!("{taskbar_root} > {}", tr(self.lang, "control_center.time"))
      }
      AppearancePage::TaskbarTimeFormat => format!(
        "{taskbar_root} > {} > {}",
        tr(self.lang, "control_center.time"),
        tr(self.lang, "control_center.format")
      ),
      AppearancePage::WidgetTelemetry => widget_telemetry_root.clone(),
      AppearancePage::ControlPanel => control_panel_root.clone(),
      AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        section: SurfaceSection::UtilityIcons,
      } => format!(
        "{taskbar_root} > {} > {}",
        tr(self.lang, "control_center.icons"),
        tr(self.lang, "control_center.utilities")
      ),
      AppearancePage::SurfaceSection {
        surface: EffectSurface::Taskbar,
        section: SurfaceSection::Launcher,
      } => format!(
        "{taskbar_root} > {} > {}",
        tr(self.lang, "control_center.icons"),
        tr(self.lang, "control_center.launcher")
      ),
      // `surface` is always Taskbar/ControlPanel/WidgetTelemetry here:
      // `surface_for_page()` only maps those three pages to a surface, and
      // `open_section()` requires the current page to already resolve to one
      // before constructing `SurfaceSection`, so Terminal/Launchers never
      // reach this arm.
      AppearancePage::SurfaceSection { surface, section } => {
        let surface_root = match surface {
          EffectSurface::Taskbar => taskbar_root.as_str(),
          EffectSurface::ControlPanel => control_panel_root.as_str(),
          EffectSurface::WidgetTelemetry => widget_telemetry_root.as_str(),
          EffectSurface::Terminal | EffectSurface::Launchers => root,
        };
        format!(
          "{surface_root} > {}",
          tr(
            self.lang,
            match section {
              SurfaceSection::Launcher => "control_center.launcher",
              SurfaceSection::UtilityIcons => "control_center.taskbar_utility_group",
              SurfaceSection::Sessions => "control_center.sessions",
              SurfaceSection::Transparency => "control_center.transparency",
            }
          )
        )
      }
      // Entry points reached from the Hyprland category (crate principal).
      AppearancePage::WindowSpaces => {
        format!(
          "{hyprland_root} > {}",
          tr(self.lang, "control_center.window_spaces")
        )
      }
      AppearancePage::WindowSpacesInner => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.window_spaces"),
        tr(self.lang, "control_center.inner")
      ),
      AppearancePage::WindowSpacesOuter => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.window_spaces"),
        tr(self.lang, "control_center.outer")
      ),
      AppearancePage::Animations => {
        format!(
          "{hyprland_root} > {}",
          tr(self.lang, "control_center.animations")
        )
      }
      AppearancePage::AnimationsCustomCurves => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.animations"),
        tr(self.lang, "control_center.animations_custom_curves")
      ),
      AppearancePage::AnimationsCurve { curve } => format!(
        "{hyprland_root} > {} > {} > {}",
        tr(self.lang, "control_center.animations"),
        tr(self.lang, "control_center.animations_custom_curves"),
        tr(self.lang, curve.label_key())
      ),
      AppearancePage::AnimationsGroup { group } => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.animations"),
        tr(self.lang, group.label_key())
      ),
      AppearancePage::AnimationsLeaf { leaf } => format!(
        "{hyprland_root} > {} > {} > {}",
        tr(self.lang, "control_center.animations"),
        tr(self.lang, leaf.group().label_key()),
        tr(self.lang, leaf.label_key())
      ),
      AppearancePage::Borders => format!(
        "{hyprland_root} > {}",
        tr(self.lang, "control_center.borders")
      ),
      AppearancePage::GeneralBorders => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.borders"),
        tr(self.lang, "control_center.rounded")
      ),
      AppearancePage::EdgeThickness => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.borders"),
        tr(self.lang, "control_center.thickness")
      ),
      AppearancePage::Prompt { goal } => format!(
        "{} > {}",
        self.breadcrumb_for_prompt(goal),
        tr(self.lang, self.prompt_label(goal))
      ),
    }
  }
  /// Breadcrumb of the page a numeric prompt was opened from; the prompt
  /// page itself appends the field's own label (see `breadcrumb_of`).
  fn breadcrumb_for_prompt(&self, _goal: PromptGoal) -> String {
    self.breadcrumb_of(self.prompt_back.unwrap_or(AppearancePage::Home))
  }
  /// Footer of the current page. List pages derive it from the selected
  /// row's kind (`argvus_tui::hints`); the text editors list their own keys
  /// with the same translated action names.
  fn hints(&self, rows: &[Row<Item>]) -> String {
    let label = |key: &str| tr(self.lang, key);
    if self.confirm.is_some() {
      return confirm_hints(self.lang);
    }
    match self.page {
      AppearancePage::AccentEdit => [
        ("#/0-9/A-F", label("control_center.hex_color")),
        ("Enter", label("control_center.apply")),
        ("Esc", label("control_center.hint.back")),
      ]
      .map(|(keys, action)| format!("{keys} {action}"))
      .join(FOOTER_GAP),
      AppearancePage::Prompt { goal } => {
        let mut segments = Vec::new();
        if !matches!(goal, PromptGoal::ExportProfile | PromptGoal::ImportProfile) {
          segments.push(format!("0-9 {}", label("control_center.hint.edit")));
        }
        segments.push(format!("Enter {}", label("control_center.hint.confirm")));
        segments.push(format!("Esc {}", label("control_center.hint.back")));
        segments.join(FOOTER_GAP)
      }
      _ => {
        let mut extra = Vec::new();
        if self.page == AppearancePage::CustomThemes {
          extra.push(("d", label("control_center.delete")));
        }
        if matches!(
          self.page,
          AppearancePage::Themes | AppearancePage::CustomThemes
        ) {
          extra.push(("e", label("control_center.export")));
          extra.push(("i", label("control_center.import")));
        }
        let movable_row = matches!(
          self.menu.selected_id(rows),
          Some(Item::TelemetryBlock(_) | Item::PanelCard(_))
        );
        if movable_row
          && matches!(
            self.page,
            AppearancePage::SurfaceSection {
              surface: EffectSurface::WidgetTelemetry | EffectSurface::ControlPanel,
              section: SurfaceSection::Sessions,
            }
          )
        {
          extra.push(("⇧↑/⇧↓", label("control_center.move_block")));
        }
        hints(
          self.lang,
          &HintContext {
            row: self.menu.selected_kind(rows),
            can_go_back: true,
            refresh: true,
            extra: &extra,
            ..HintContext::default()
          },
        )
      }
    }
  }

  /// Renders the current page with the semantic theme.
  pub fn draw(&mut self, frame: &mut Frame) {
    let rows = self.rows();
    self.menu.normalize(&rows);
    // The Icons page draws its own footer (status left, shortcuts right).
    let icons_page = self.page == AppearancePage::Icons;
    let hints = if icons_page {
      String::new()
    } else {
      self.hints(&rows)
    };
    let area = shell(
      frame,
      frame.area(),
      &self.theme,
      self.lang,
      &self.breadcrumb(),
      &hints,
    );
    match self.page {
      AppearancePage::AccentEdit => self.draw_accent_editor(frame, area),
      AppearancePage::Prompt { goal } => self.draw_prompt(frame, area, goal),
      AppearancePage::Icons => self.draw_icons(frame, area, &rows),
      _ => {
        self.list_height = area.height;
        draw_menu(
          frame,
          area,
          &self.theme,
          &rows,
          &mut self.menu,
          MenuStyle {
            icons: AppConfig::icons_enabled(),
          },
        );
      }
    }
    if let Some((confirmation, state)) = &self.confirm {
      self.draw_confirmation(frame, area, *confirmation, state);
    }
    if icons_page {
      self.draw_icons_footer(frame, area);
      return;
    }
    if let Some(message) = &self.status {
      status(frame, area, &self.theme, message);
    }
    if self.status_loading {
      status(
        frame,
        area,
        &self.theme,
        &StatusMessage {
          kind: StatusKind::Info,
          text: tr(self.lang, "control_center.loading_appearance").into(),
        },
      );
    }
  }

  fn draw_confirmation(
    &self,
    frame: &mut Frame,
    area: Rect,
    confirmation: Confirmation,
    state: &ConfirmState,
  ) {
    let label = |key: &str| tr(self.lang, key);
    // The theme dialogs name the theme above the description.
    let named = |name: &str, description: &str| format!("{name}\n{}", label(description));
    let (title, message, confirm, danger) = match confirmation {
      Confirmation::DiscardDraft => (
        "control_center.discard_changes_title",
        label("control_center.discard_changes_description").to_string(),
        "control_center.discard",
        false,
      ),
      Confirmation::DeleteTheme => (
        "control_center.theme_profile_delete_title",
        named(
          self
            .delete_theme
            .as_ref()
            .map(|theme| theme.name.as_str())
            .unwrap_or_default(),
          "control_center.theme_profile_delete_description",
        ),
        "control_center.theme_profile_delete",
        true,
      ),
      Confirmation::ReplaceImport => (
        "control_center.theme_profile_duplicate_title",
        named(
          self.pending_import_name.as_deref().unwrap_or_default(),
          "control_center.theme_profile_duplicate_description",
        ),
        "control_center.theme_profile_duplicate_replace",
        true,
      ),
    };
    let dialog = ConfirmDialog {
      title: label(title),
      message: &message,
      confirm: label(confirm),
      cancel: label("control_center.cancel"),
      danger,
      deadline: None,
    };
    draw_confirm(frame, area, &self.theme, dialog, state);
  }

  /// Renders `draw_prompt` while respecting the current domain state and semantic theme. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn draw_prompt(&mut self, frame: &mut Frame, area: Rect, goal: PromptGoal) {
    let label = tr(self.lang, self.prompt_label(goal));
    let chunks = Layout::vertical([
      Constraint::Length(3),
      Constraint::Length(1),
      Constraint::Length(1),
      Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("{label}: {}", self.prompt_buffer),
        Style::new()
          .fg(self.theme.selected_foreground)
          .bg(self.theme.selected_background),
      ))),
      chunks[0],
    );
    // The keys are in the footer; this line only states what is accepted.
    let text = self.prompt_error.clone().unwrap_or_else(|| {
      if matches!(goal, PromptGoal::ExportProfile) {
        return tr(self.lang, "control_center.theme_profile_export_name").to_string();
      }
      let (min, max) = goal.range();
      format!("{min}..{max}")
    });
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        text,
        Style::new()
          .fg(if self.prompt_error.is_some() {
            self.theme.error
          } else {
            self.theme.foreground
          })
          .add_modifier(Modifier::DIM),
      ))),
      chunks[1],
    );
    if matches!(goal, PromptGoal::ExportProfile) {
      let output = backend::preview_theme_profile_path(&self.prompt_buffer)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "—".into());
      frame.render_widget(
        Paragraph::new(format!(
          "{}: {}",
          tr(self.lang, "control_center.theme_profile_output"),
          output
        ))
        .style(Style::new().fg(self.theme.muted)),
        chunks[2],
      );
    }
  }

  fn draw_accent_editor(&mut self, frame: &mut Frame, area: Rect) {
    let color = self.prompt_buffer.parse::<HexColor>().ok();
    let chunks = Layout::vertical([
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(
      Paragraph::new(tr(self.lang, "control_center.preview")),
      chunks[0],
    );
    let preview_style = color
      .map(|color| {
        Style::new()
          .bg(ratatui::style::Color::Rgb(color.r, color.g, color.b))
          .fg(color.foreground())
      })
      .unwrap_or_else(|| Style::new().fg(self.theme.error));
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("  {}  ", self.prompt_buffer),
        preview_style,
      ))),
      chunks[1],
    );
    frame.render_widget(
      Paragraph::new(tr(self.lang, "control_center.hex_color")),
      chunks[2],
    );
    frame.render_widget(
      Paragraph::new(Line::from(Span::styled(
        format!("> {}", self.prompt_buffer),
        Style::new()
          .fg(self.theme.selected_foreground)
          .bg(self.theme.selected_background),
      ))),
      chunks[3],
    );
    if let Some(error) = &self.prompt_error {
      frame.render_widget(
        Paragraph::new(Span::styled(error, Style::new().fg(self.theme.error))),
        chunks[4],
      );
    }
  }
}

#[cfg(test)]
mod tests;
