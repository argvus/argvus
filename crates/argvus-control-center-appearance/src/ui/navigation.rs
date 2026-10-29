//! Moving between appearance pages, going back with or without a draft, and the breadcrumb of each page.

use super::*;

impl AppearanceApp {
  /// Enters `page` from outside Appearance (the Home, search, a CLI route or
  /// another domain). It goes through the same path as in-page navigation: a
  /// load still running for another page is cancelled instead of delaying this
  /// one, and the page's draft is rebuilt from the state already known, so a
  /// page reopened within its refresh window shows its values right away.
  pub fn open(&mut self, page: AppearancePage) {
    self.go(page);
  }

  pub(super) fn ensure_page(&mut self) {
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

  pub(super) fn go(&mut self, page: AppearancePage) {
    // A load for another page is dropped: its result would only be replaced,
    // and while it runs `refresh` cannot start the load of `page`. This also
    // covers a load left running when the host returned to its Home.
    if self.job_page != Some(page)
      && let Some(job) = self.job.take()
    {
      job.cancel();
      self.job_page = None;
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

  pub(super) fn back(&mut self) -> bool {
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
  pub(super) fn back_discards_draft(&self) -> bool {
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
    if self.page == AppearancePage::BordersShadow {
      return self.shadow_draft_has_changes();
    }
    if Self::effect_spec(self.page).is_some() {
      return self.effect_draft_has_changes();
    }
    matches!(
      self.page,
      AppearancePage::Taskbar | AppearancePage::WidgetTelemetry | AppearancePage::ControlPanel
    ) && self.surface_draft_has_changes()
  }

  /// Goes to the parent page, dropping the drafts the parent does not keep.
  pub(super) fn navigate_back(&mut self) -> bool {
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
      AppearancePage::TaskbarPosition
      | AppearancePage::TaskbarSpaces
      | AppearancePage::TaskbarHeight
      | AppearancePage::TaskbarBorder => {
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
      // The Shadow draft is dropped on the way back, as the discard prompt warns.
      AppearancePage::BordersShadow => {
        self.shadow_draft = None;
        self.go(AppearancePage::Borders);
      }
      AppearancePage::TaskbarIcons | AppearancePage::TaskbarDate | AppearancePage::TaskbarTime => {
        self.go(AppearancePage::Taskbar);
      }
      AppearancePage::TaskbarDateFormat => self.go(AppearancePage::TaskbarDate),
      AppearancePage::TaskbarTimeFormat => self.go(AppearancePage::TaskbarTime),
      AppearancePage::TaskbarBorderThickness => self.go(AppearancePage::TaskbarBorder),
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
          (
            EffectSurface::Taskbar,
            SurfaceSection::Focus | SurfaceSection::UtilityIcons | SurfaceSection::Launcher,
          ) => AppearancePage::TaskbarIcons,
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

  pub(super) fn breadcrumb(&self) -> String {
    self.breadcrumb_of(self.page)
  }

  pub(super) fn breadcrumb_of(&self, page: AppearancePage) -> String {
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
      AppearancePage::TaskbarHeight => format!(
        "{taskbar_root} > {}",
        tr(self.lang, "control_center.height")
      ),
      AppearancePage::TaskbarBorder => format!(
        "{taskbar_root} > {}",
        tr(self.lang, "control_center.border")
      ),
      AppearancePage::TaskbarBorderThickness => format!(
        "{taskbar_root} > {} > {}",
        tr(self.lang, "control_center.border"),
        tr(self.lang, "control_center.thickness")
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
        section: SurfaceSection::Focus,
      } => format!(
        "{taskbar_root} > {} > {}",
        tr(self.lang, "control_center.icons"),
        tr(self.lang, "control_center.taskbar_focus_icon")
      ),
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
              SurfaceSection::Focus => "control_center.taskbar_focus_icon",
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
      AppearancePage::BordersShadow => format!(
        "{hyprland_root} > {} > {}",
        tr(self.lang, "control_center.borders"),
        tr(self.lang, "control_center.shadow")
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
  pub(super) fn breadcrumb_for_prompt(&self, _goal: PromptGoal) -> String {
    self.breadcrumb_of(self.prompt_back.unwrap_or(AppearancePage::Home))
  }
}
