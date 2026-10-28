//! Unsaved edits of the surfaces, effects, blur and animations pages: what changed and how Save applies it.

use super::*;

/// Whether applying `draft` over the saved `current` values recreates the
/// surface: restarts a Waybar bar or the Control Panel. Only then does the
/// `Apply` show the splash. Styles, borders, blur and transparency reload in
/// place.
pub(super) fn recreates_surface(draft: &SurfaceDraft, current: &SurfaceDraft) -> bool {
  match draft.surface {
    // Waybar re-reads its JSONC only on a restart; the border is CSS.
    EffectSurface::Taskbar => {
      draft.height != current.height
        || draft.utility_group != current.utility_group
        || draft.audio_player_enabled != current.audio_player_enabled
        || draft.launcher_enabled != current.launcher_enabled
        || draft.utility_widgets != current.utility_widgets
        || draft.date_format != current.date_format
        || draft.time_seconds_enabled != current.time_seconds_enabled
        || draft.time_format != current.time_format
    }
    EffectSurface::WidgetTelemetry => {
      draft.widget_enabled != current.widget_enabled
        || draft.widget_blocks != current.widget_blocks
        || draft.widget_order != current.widget_order
    }
    // A hidden panel is not recreated by a card change.
    EffectSurface::ControlPanel => {
      draft.control_panel_enabled != current.control_panel_enabled
        || (draft.control_panel_enabled && draft.control_panel_cards != current.control_panel_cards)
    }
    EffectSurface::Terminal | EffectSurface::Launchers => false,
  }
}

pub(super) fn apply_draft_effects(draft: &SurfaceDraft) -> Result<(), String> {
  backend::apply_surface_effects(
    draft.surface,
    draft.transparency_enabled,
    draft.transparency,
    draft.blur_enabled,
    draft.blur,
  )
}

/// The taskbar part of `Apply`: the styles and effects first, which the bar
/// reloads in place, then the utility group, height and icons, written to the
/// JSONC that Waybar reads only when `recreate` restarts it once at the end.
pub(super) fn apply_taskbar_draft(draft: &SurfaceDraft, recreate: bool) -> Result<(), String> {
  apply_draft_effects(draft)?;
  backend::set_taskbar_shape(draft.height, draft.border_enabled, draft.border_thickness)?;
  if !recreate {
    return Ok(());
  }
  backend::set_taskbar_utility_group(draft.utility_group)?;
  backend::apply_taskbar_icons_and_format(
    draft.audio_player_enabled,
    draft.launcher_enabled,
    &draft.utility_widgets,
    draft.date_format,
    draft.time_seconds_enabled,
    draft.time_format,
  )?;
  backend::reload_taskbar()
}

/// The Control Panel part of `Apply`: the effects reach the running panel by
/// the configuration reload; the cards and the switch are written next, and
/// the panel is restarted once, by the switch when it changes or else here.
pub(super) fn apply_control_panel_draft(
  draft: &SurfaceDraft,
  current: &SurfaceDraft,
) -> Result<(), String> {
  apply_draft_effects(draft)?;
  let changes = ControlPanelCard::ALL
    .into_iter()
    .filter_map(|card| {
      let enabled = draft.control_panel_cards.enabled(card);
      (enabled != current.control_panel_cards.enabled(card)).then_some((card, enabled))
    })
    .collect::<Vec<_>>();
  backend::set_control_panel_cards(&changes)?;
  if draft.control_panel_enabled != current.control_panel_enabled {
    // `master set` restarts or stops the panel.
    return backend::set_control_panel_enabled(draft.control_panel_enabled);
  }
  if draft.control_panel_enabled && !changes.is_empty() {
    return backend::reload_control_panel();
  }
  Ok(())
}

impl AppearanceApp {
  /// Whether the surface draft differs from the loaded state.
  pub(super) fn surface_draft_has_changes(&self) -> bool {
    self
      .surface_draft
      .as_ref()
      .is_some_and(|draft| *draft != self.make_surface_draft(draft.surface))
  }

  /// Whether the effect editor value differs from the stored one.
  pub(super) fn effect_draft_has_changes(&self) -> bool {
    match (Self::effect_spec(self.page), self.effect_draft) {
      (Some((kind, surface)), Some(value)) => value != self.effect_value(kind, surface),
      _ => false,
    }
  }

  /// The Blur page as stored, before any unapplied edit.
  pub(super) fn blur_saved(&self) -> BlurSettings {
    BlurSettings {
      enabled: self.state.blur,
      values: self.state.blur_values,
    }
  }

  /// The Blur page as shown: the draft when there is one, otherwise the stored state.
  pub(super) fn blur_editor(&self) -> BlurSettings {
    self.blur_draft.unwrap_or_else(|| self.blur_saved())
  }

  /// Whether the Blur draft differs from the stored settings.
  pub(super) fn blur_draft_has_changes(&self) -> bool {
    self
      .blur_draft
      .is_some_and(|draft| draft != self.blur_saved())
  }

  /// Changes the Blur draft; nothing is written until `Apply`.
  pub(super) fn edit_blur(&mut self, edit: impl FnOnce(&mut BlurSettings)) {
    let mut draft = self.blur_editor();
    edit(&mut draft);
    self.blur_draft = Some(draft);
  }

  /// Whether the current page has an unapplied draft, which enables `Apply`.
  pub(super) fn has_pending_changes(&self) -> bool {
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
  pub(super) fn is_draft_page(page: AppearancePage) -> bool {
    page == AppearancePage::Blur
      || page.is_animations()
      || Self::surface_for_page(page).is_some()
      || Self::effect_spec(page).is_some()
  }

  pub(super) fn surface_for_page(page: AppearancePage) -> Option<EffectSurface> {
    match page {
      AppearancePage::Taskbar
      | AppearancePage::TaskbarIcons
      | AppearancePage::TaskbarDate
      | AppearancePage::TaskbarDateFormat
      | AppearancePage::TaskbarTime
      | AppearancePage::TaskbarTimeFormat
      | AppearancePage::TaskbarHeight
      | AppearancePage::TaskbarBorder
      | AppearancePage::TaskbarBorderThickness => Some(EffectSurface::Taskbar),
      AppearancePage::WidgetTelemetry => Some(EffectSurface::WidgetTelemetry),
      AppearancePage::ControlPanel => Some(EffectSurface::ControlPanel),
      AppearancePage::SurfaceSection { surface, .. } => Some(surface),
      _ => None,
    }
  }

  pub(super) fn make_surface_draft(&self, surface: EffectSurface) -> SurfaceDraft {
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
      height: self.state.taskbar_height,
      border_enabled: self.state.taskbar_border_enabled,
      border_thickness: self.state.taskbar_border_thickness,
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

  pub(super) fn surface_draft(&self) -> Option<&SurfaceDraft> {
    self.surface_draft.as_ref()
  }

  pub(super) fn surface_draft_mut(&mut self) -> Option<&mut SurfaceDraft> {
    self.surface_draft.as_mut()
  }

  pub(super) fn apply_surface_changes(&mut self) {
    let Some(draft) = self.surface_draft.clone() else {
      return;
    };
    // Keep rendering the just-applied draft values until the background
    // refresh (triggered by `reload_requested` once the task below finishes)
    // replaces it with a freshly loaded one. Clearing it here immediately
    // made every toggle/radio on the page flash back to its pre-edit state
    // for the duration of the apply + refresh round trip.
    let current = self.make_surface_draft(draft.surface);
    let recreate = recreates_surface(&draft, &current);
    self.apply(
      tr(self.lang, "control_center.surface_settings_applied").into(),
      move || {
        // A surface that is recreated gets the splash before the first write;
        // it comes down when the guard drops after the last step, failed or
        // not, so no change is visible before it.
        let _overlay = backend::ApplyOverlay::start_if(recreate);
        match draft.surface {
          EffectSurface::Taskbar => apply_taskbar_draft(&draft, recreate),
          EffectSurface::WidgetTelemetry => {
            apply_draft_effects(&draft)?;
            if !recreate {
              return Ok(());
            }
            backend::apply_widget_telemetry(
              draft.widget_enabled,
              &draft.widget_blocks,
              &draft.widget_order,
            )
          }
          EffectSurface::ControlPanel => apply_control_panel_draft(&draft, &current),
          EffectSurface::Terminal | EffectSurface::Launchers => backend::apply_surface_effects(
            draft.surface,
            draft.transparency_enabled,
            draft.transparency,
            true,
            0,
          ),
        }
      },
    );
  }

  pub(super) fn edit_draft(&mut self, edit: impl FnOnce(&mut SurfaceDraft)) {
    if let Some(draft) = self.surface_draft_mut() {
      edit(draft);
    }
  }

  /// Writes a typed Blur parameter into the Blur draft and returns to the page
  /// with its row selected. Nothing is applied; an out-of-range value keeps
  /// the prompt open with the accepted bounds.
  pub(super) fn commit_blur_value(&mut self, field: BlurField, text: &str) {
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
  pub(super) fn apply_blur_changes(&mut self) {
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
        backend::apply_blur(
          enabled_changed.then_some(draft.enabled),
          values_changed.then_some(draft.values),
        )
      },
    );
  }

  /// Writes a typed percentage into the draft of `page` and returns to it
  /// with the value row selected. Nothing is applied.
  pub(super) fn set_draft_value(&mut self, page: AppearancePage, value: i32) {
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

  /// The Animations settings as stored, before any unapplied edit.
  pub(super) fn animations_saved(&self) -> AnimationSettings {
    AnimationSettings {
      enabled: self.state.animations,
      values: self.state.animation_values,
    }
  }

  /// The Animations settings as shown: the draft when there is one, otherwise
  /// the stored state. Every Animations page reads the same draft.
  pub(super) fn animations_editor(&self) -> AnimationSettings {
    self
      .animations_draft
      .unwrap_or_else(|| self.animations_saved())
  }

  /// Whether the Animations draft differs from the stored settings.
  pub(super) fn animations_draft_has_changes(&self) -> bool {
    self
      .animations_draft
      .is_some_and(|draft| draft != self.animations_saved())
  }

  /// Changes the Animations draft; nothing is written until `Apply`.
  pub(super) fn edit_animations(&mut self, edit: impl FnOnce(&mut AnimationSettings)) {
    let mut draft = self.animations_editor();
    edit(&mut draft);
    self.animations_draft = Some(draft);
  }

  /// Moves the style or the curve of a leaf by `delta` choices, wrapping around.
  pub(super) fn step_animation_choice(&mut self, item: Item, delta: i32) {
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
  pub(super) fn commit_animation_value(&mut self, field: AnimationField, text: &str) {
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
  pub(super) fn apply_animations_changes(&mut self) {
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
        backend::apply_animations(
          enabled_changed.then_some(draft.enabled),
          values_changed.then_some(draft.values),
        )
      },
    );
  }

  pub(super) fn effect_spec(page: AppearancePage) -> Option<(&'static str, EffectSurface)> {
    match page {
      AppearancePage::TerminalTransparency => Some(("transparency", EffectSurface::Terminal)),
      AppearancePage::TransparencySurface {
        surface: EffectSurface::Launchers,
      } => Some(("transparency", EffectSurface::Launchers)),
      AppearancePage::TransparencySurface { surface } => Some(("transparency", surface)),
      _ => None,
    }
  }

  pub(super) fn effect_value(&self, kind: &str, surface: EffectSurface) -> i32 {
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

  pub(super) fn effect_editor_value(&self) -> i32 {
    Self::effect_spec(self.page)
      .map(|(kind, surface)| {
        self
          .effect_draft
          .unwrap_or_else(|| self.effect_value(kind, surface))
      })
      .unwrap_or(0)
  }

  pub(super) fn apply_effect_changes(&mut self) {
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
        // A value reloads in place: no surface is recreated, so no splash.
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
}
