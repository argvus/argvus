//! Running a row: toggles, value steps, reordering cards and blocks, themes, layout and transparency.

use super::*;

impl AppearanceApp {
  pub(super) fn toggle(&mut self) {
    let enable = !self.state.rounded;
    self.apply(
      tr(self.lang, "control_center.border_settings_applied").into(),
      move || backend::set_rounded(enable),
    );
  }

  /// Runs what activating `item` does: Enter/`→` (and Space, see
  /// [`Self::page_key`]) on any row, or Enter/Space on a toggle. Each effect is
  /// the one the item had before the single menu list (immediate jobs stay
  /// immediate, draft edits stay in the draft).
  pub(super) fn activate(&mut self, item: Item) {
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
      Item::TaskbarHeight => self.go(AppearancePage::TaskbarHeight),
      Item::TaskbarHeightOption(height) => self.edit_draft(|draft| draft.height = height),
      Item::TaskbarBorder => self.go(AppearancePage::TaskbarBorder),
      Item::BorderEnabled => self.edit_draft(|draft| {
        draft.border_enabled = !draft.border_enabled;
      }),
      // Thickness is disabled while the border is off, so only an enabled
      // border opens its page.
      Item::BorderThickness => {
        if self
          .surface_draft()
          .is_some_and(|draft| draft.border_enabled)
        {
          self.go(AppearancePage::TaskbarBorderThickness);
        }
      }
      Item::BorderThicknessOption(thickness) => {
        self.edit_draft(|draft| draft.border_thickness = thickness);
      }
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
  pub(super) fn adjust(&mut self, item: Item, delta: i32) {
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
  pub(super) fn adjust_section_value(&mut self, edit: impl FnOnce(&mut i32)) {
    if matches!(self.page, AppearancePage::SurfaceSection { .. }) {
      self.edit_draft(|draft| edit(&mut draft.transparency));
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
  pub(super) fn open_section(&mut self, section: SurfaceSection) {
    if let Some(surface) = Self::surface_for_page(self.page) {
      self.go(AppearancePage::SurfaceSection { surface, section });
    }
  }

  pub(super) fn apply_custom_theme(&mut self, index: usize) {
    let Some(custom) = self.state.custom_themes.get(index).cloned() else {
      return;
    };
    let lang = self.lang;
    self.apply_result(move || {
      backend::apply_custom_theme(&custom)
        .map(|()| tr(lang, "control_center.theme_profile_applied").to_string())
    });
  }

  pub(super) fn inspect_import(&mut self, index: usize) {
    let Some(path) = self.import_archives.get(index).cloned() else {
      return;
    };
    self.status = Some(StatusMessage {
      kind: StatusKind::Info,
      text: tr(self.lang, "control_center.theme_profile_importing").into(),
    });
    self.action = Some(self.manager.spawn(move |_| {
      let (name, duplicate) = backend::inspect_theme_archive(&path)?;
      Ok(JobData::ImportReady {
        path,
        name,
        duplicate,
      })
    }));
  }

  pub(super) fn apply_layout_mode(&mut self, variant: &'static str) {
    self.apply(
      tr(self.lang, "control_center.layout_mode_applied").into(),
      move || backend::set_layout_mode(variant),
    );
  }

  /// Flips the Terminal or Launcher transparency immediately, keeping its
  /// stored percentage.
  pub(super) fn apply_transparency_toggle(&mut self, surface: EffectSurface) {
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
}
