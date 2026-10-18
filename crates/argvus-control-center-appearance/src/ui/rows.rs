//! Menu rows of every Appearance page.
//!
//! Each entry is identified by an [`Item`] instead of its position on the
//! screen, and the item carries its own icon, so pages never pick glyphs by
//! index or splice them into labels.
use super::{AppearanceApp, THEME_CATEGORIES, current_family_label, family_indices};
use crate::animations::{
  AnimationCurve, AnimationField, AnimationGroup, AnimationLeaf, CurveParam,
};
use crate::blur::BlurField;
use crate::model::{
  AppearancePage, ControlPanelCard, EffectSurface, IconThemeSource, PromptGoal, SurfaceSection,
  TaskbarDateFormat, TaskbarPosition, TaskbarTimeFormat, TaskbarUtilityGroupMode,
  TaskbarUtilityWidget, WallpaperCollection, WallpaperMode, WidgetTelemetryBlock, accent_label,
};
use argvus_i18n::tr;
use argvus_theme::discovery::ThemeCategory;
use argvus_tui::{
  icons,
  menu::{Row, draft_actions},
};

/// Stable identity of an Appearance menu entry.
///
/// List entries carry the identity of the data they stand for (an index into
/// the loaded state or a model enum), never their position on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Item {
  // Home
  Desktop,
  ApplicationStyle,
  Themes,
  Accent,
  Wallpaper,
  Terminal,
  Launcher,
  Mode,
  Icons,
  /// Opens the Fonts page, which the host app owns (settings crate).
  Fonts,
  // Themes
  OfficialThemes,
  CustomThemes,
  ExportTheme,
  ImportTheme,
  ThemeCategory(ThemeCategory),
  /// Index into `AppearanceState::official_themes`.
  Family(usize),
  /// Index into `AppearanceState::custom_themes`.
  CustomTheme(usize),
  /// Index into `AppearanceApp::import_archives`.
  ImportArchive(usize),
  ModeSticky,
  ModeFloat,
  // Icons
  /// Index into `AppearanceState::icon_themes`.
  IconTheme(usize),
  // Wallpapers
  ChooseWallpaper,
  Collection(WallpaperCollection),
  WallpaperMode(WallpaperMode),
  /// Index into `AppearanceState::wallpapers`.
  WallpaperFile(usize),
  // Accent
  EditAccent,
  ResetAccent,
  // Terminal, Launcher
  TerminalTransparency,
  TerminalTransparencyValue,
  LauncherTransparency,
  LauncherTransparencyValue,
  /// The value row of an effect editor page.
  EffectValue,
  /// "Enable" on the Animations page (entry point from Hyprland). Like every
  /// Animations value, it edits the draft until `Apply`.
  Animations,
  /// "Custom curves" on the Animations page.
  AnimationsCustomCurves,
  /// One animation category on the Animations page.
  AnimationsGroup(AnimationGroup),
  /// One curve under "Custom curves".
  AnimationsCurve(AnimationCurve),
  /// One animation leaf under a category.
  AnimationsLeaf(AnimationLeaf),
  /// "Enable" of an animation leaf that has one.
  AnimationLeafEnabled(AnimationLeaf),
  /// The style row of an animation leaf (`←/→` or Enter cycles it).
  AnimationLeafStyle(AnimationLeaf),
  /// The curve row of an animation leaf (`←/→` or Enter cycles it).
  AnimationLeafCurve(AnimationLeaf),
  /// A numeric animation row: speed, pop-in percentage or a curve parameter.
  AnimationField(AnimationField),
  /// "Enable" on the Blur page (entry point from Hyprland; global blur). It
  /// edits the draft, like every Blur value, until `Apply`.
  BlurEnabled,
  /// One Blur parameter row (size, passes, brightness, ...).
  BlurField(BlurField),
  // Taskbar position/spaces, window spaces and borders
  TaskbarPosition,
  TaskbarSpaces,
  WindowSpacesInner,
  WindowSpacesOuter,
  GeneralBorders,
  EdgeThickness,
  Position(TaskbarPosition),
  Spacing(PromptGoal),
  Rounded,
  Rounding,
  Thickness,
  // Draft pages
  TaskbarTransparency,
  TaskbarIcons,
  TaskbarDate,
  TaskbarTime,
  AudioPlayer,
  LauncherIcon,
  LauncherEnabled,
  ChooseLauncherIcon,
  UtilityWidget(TaskbarUtilityWidget),
  Utilities,
  DateFormats,
  DateFormat(TaskbarDateFormat),
  Seconds,
  TimeFormats,
  TimeFormat(TaskbarTimeFormat),
  UtilityGroup(TaskbarUtilityGroupMode),
  /// "Enable" on Widget Telemetry and Control Panel.
  SurfaceEnabled,
  Sessions,
  SurfaceTransparency,
  TelemetryBlock(WidgetTelemetryBlock),
  PanelCard(ControlPanelCard),
  /// "Enable" on a transparency/blur section.
  SectionEnabled,
  /// The percentage row of a transparency/blur section.
  SectionValue,
  /// `Apply` at the end of every draft page.
  Apply,
}

impl Item {
  /// The item's own glyph. Entries of homogeneous data lists (theme
  /// families, files, formats, blocks, cards, widgets) carry none, so the
  /// icon column disappears on those pages.
  pub(super) fn icon(self) -> Option<&'static str> {
    let glyph = match self {
      Self::Desktop => icons::WALLPAPER,
      Self::Fonts => icons::FONTS,
      Self::ApplicationStyle => icons::TERMINAL,
      Self::Themes => icons::PALETTE,
      Self::Accent => icons::ACCENT,
      Self::Wallpaper => icons::WALLPAPER,
      Self::Terminal => icons::TERMINAL,
      Self::Launcher => icons::LAUNCHER,
      Self::Mode => icons::THEME_MODE,
      Self::Icons => icons::APPS,
      Self::OfficialThemes => icons::PALETTE,
      Self::CustomThemes => icons::EDIT,
      Self::ExportTheme => icons::EXPORT,
      Self::ImportTheme => icons::IMPORT,
      Self::ThemeCategory(_) => icons::THEME_MODE,
      Self::ModeSticky => icons::WINDOW_STICKY,
      Self::ModeFloat => icons::WINDOW_FLOAT,
      Self::ChooseWallpaper => icons::FOLDER,
      Self::Collection(_) | Self::WallpaperMode(_) => icons::IMAGE,
      Self::EditAccent => icons::EDIT,
      Self::ResetAccent => icons::RESTORE,
      Self::TerminalTransparency
      | Self::TerminalTransparencyValue
      | Self::LauncherTransparency
      | Self::LauncherTransparencyValue
      | Self::TaskbarTransparency
      | Self::SurfaceTransparency => icons::OPACITY,
      Self::Animations
      | Self::AnimationsCustomCurves
      | Self::AnimationsCurve(_)
      | Self::AnimationsLeaf(_)
      | Self::AnimationLeafEnabled(_)
      | Self::AnimationLeafStyle(_)
      | Self::AnimationLeafCurve(_)
      | Self::AnimationField(_) => icons::ANIMATION,
      Self::AnimationsGroup(group) => match group {
        AnimationGroup::Windows => icons::WINDOW_RULES,
        AnimationGroup::Workspaces => icons::GAP,
        AnimationGroup::LayersPopups => icons::LAYOUT,
        AnimationGroup::BordersEffects => icons::BORDER,
        AnimationGroup::OtherAnimations => icons::EFFECT,
      },
      Self::BlurEnabled | Self::BlurField(_) => icons::BLUR,
      Self::TaskbarPosition => icons::TASKBAR,
      Self::TaskbarSpaces | Self::WindowSpacesInner | Self::WindowSpacesOuter => icons::GAP,
      Self::GeneralBorders => icons::BORDER,
      Self::EdgeThickness | Self::Thickness => icons::THICKNESS,
      Self::Position(TaskbarPosition::Top) => icons::ARROW_UP,
      Self::Position(TaskbarPosition::Bottom) => icons::ARROW_DOWN,
      Self::Spacing(goal) => return spacing_icon(goal),
      Self::Rounded | Self::Rounding => icons::ROUNDED,
      Self::TaskbarIcons => icons::APPS,
      Self::TaskbarDate | Self::DateFormats => icons::CALENDAR,
      Self::TaskbarTime | Self::TimeFormats => icons::CLOCK,
      Self::AudioPlayer => icons::MUSIC,
      Self::LauncherIcon | Self::LauncherEnabled => icons::LAUNCHER,
      Self::ChooseLauncherIcon => icons::FOLDER,
      Self::Utilities => icons::WIDGET,
      Self::Seconds => icons::TIMER,
      Self::Sessions => icons::LAYOUT,
      Self::Family(_)
      | Self::CustomTheme(_)
      | Self::ImportArchive(_)
      | Self::WallpaperFile(_)
      | Self::IconTheme(_)
      | Self::UtilityWidget(_)
      | Self::DateFormat(_)
      | Self::TimeFormat(_)
      | Self::UtilityGroup(_)
      | Self::TelemetryBlock(_)
      | Self::PanelCard(_) => return None,
      // `draft_actions` sets the shared confirmation glyph.
      Self::Apply => icons::APPLY,
      // These depend on the page; see `AppearanceApp::page_icon`.
      Self::EffectValue | Self::SurfaceEnabled | Self::SectionEnabled | Self::SectionValue => {
        return None;
      }
    };
    Some(glyph)
  }
}

/// Arrows name the side a spacing value applies to; the inner gap uses the
/// gap glyph.
fn spacing_icon(goal: PromptGoal) -> Option<&'static str> {
  match goal {
    PromptGoal::WaybarTop | PromptGoal::GapsOutTop => Some(icons::ARROW_UP),
    PromptGoal::WaybarLeft | PromptGoal::GapsOutLeft => Some(icons::ARROW_LEFT),
    PromptGoal::WaybarRight | PromptGoal::GapsOutRight => Some(icons::ARROW_RIGHT),
    PromptGoal::WaybarBottom | PromptGoal::GapsOutBottom => Some(icons::ARROW_DOWN),
    PromptGoal::GapsIn => Some(icons::GAP),
    PromptGoal::Rounding => Some(icons::ROUNDED),
    PromptGoal::Thickness => Some(icons::THICKNESS),
    PromptGoal::ExportProfile
    | PromptGoal::ImportProfile
    | PromptGoal::DraftValue
    | PromptGoal::BlurValue(_)
    | PromptGoal::AnimationValue(_) => None,
  }
}

/// Attaches the item's icon, when it has one.
fn with_icon(row: Row<Item>, icon: Option<&'static str>) -> Row<Item> {
  match icon {
    Some(glyph) => row.icon(glyph),
    None => row,
  }
}

fn percent(value: i32) -> String {
  format!("{value}%")
}

impl AppearanceApp {
  fn label(&self, key: &str) -> &'static str {
    tr(self.lang, key)
  }

  fn submenu(&self, item: Item, key: &str) -> Row<Item> {
    with_icon(Row::submenu(item, self.label(key)), item.icon())
  }

  fn toggle_row(&self, item: Item, key: &str, on: bool) -> Row<Item> {
    with_icon(Row::toggle(item, self.label(key), on), self.item_icon(item))
  }

  fn choice_row(&self, item: Item, label: impl Into<String>, current: bool) -> Row<Item> {
    with_icon(Row::choice(item, label, current), item.icon())
  }

  /// Icon of an item whose glyph depends on the page it is shown on.
  pub(super) fn item_icon(&self, item: Item) -> Option<&'static str> {
    match (item, self.page) {
      (Item::SurfaceEnabled, AppearancePage::WidgetTelemetry) => Some(icons::TELEMETRY),
      (Item::SurfaceEnabled, AppearancePage::ControlPanel) => Some(icons::CONTROL_PANEL),
      (Item::SectionEnabled | Item::SectionValue, _) => Some(icons::OPACITY),
      (Item::EffectValue, _) => Some(icons::OPACITY),
      _ => item.icon(),
    }
  }

  /// Rows of the current page, in display order. Draft pages end with the
  /// `Apply` group, enabled only while the draft has unapplied changes.
  pub(super) fn rows(&self) -> Vec<Row<Item>> {
    let mut rows = self.page_rows();
    if Self::is_draft_page(self.page) {
      let pending = self.has_pending_changes();
      let mut actions = draft_actions(Item::Apply, self.label("control_center.apply"), pending);
      if pending && let Some(apply) = actions.pop() {
        actions.push(apply.detail(self.label("control_center.draft_changed")));
      }
      rows.extend(actions);
    }
    rows
  }

  /// Rows of the page without the trailing `Apply` group.
  pub(super) fn page_rows(&self) -> Vec<Row<Item>> {
    match self.page {
      AppearancePage::Home => self.home_rows(),
      AppearancePage::Desktop => self.desktop_rows(),
      AppearancePage::ApplicationStyle => self.application_style_rows(),
      AppearancePage::Themes => vec![
        self.submenu(
          Item::OfficialThemes,
          "control_center.theme_profile_official",
        ),
        self.submenu(Item::CustomThemes, "control_center.theme_profile_custom"),
        with_icon(
          Row::action(
            Item::ExportTheme,
            self.label("control_center.theme_profile_export"),
          ),
          Item::ExportTheme.icon(),
        ),
        self.submenu(Item::ImportTheme, "control_center.theme_profile_import"),
      ],
      AppearancePage::OfficialThemes => THEME_CATEGORIES
        .into_iter()
        .map(|category| {
          let key = super::theme_category_label_key(category);
          self.submenu(Item::ThemeCategory(category), key)
        })
        .collect(),
      AppearancePage::ThemeFamilies { category } => {
        let current = self
          .state
          .theme
          .strip_suffix("-float")
          .unwrap_or(&self.state.theme);
        let locale = self.lang.locale();
        family_indices(category, &self.state.official_themes)
          .into_iter()
          .map(|index| {
            let entry = &self.state.official_themes[index];
            let label = entry
              .name_i18n
              .get(&locale)
              .cloned()
              .unwrap_or_else(|| entry.name.clone());
            let is_current = self.state.active_custom_theme.is_none() && current == entry.id;
            self.choice_row(Item::Family(index), label, is_current)
          })
          .collect()
      }
      AppearancePage::CustomThemes => self
        .state
        .custom_themes
        .iter()
        .enumerate()
        .map(|(index, theme)| {
          let is_current = self.state.active_custom_theme.as_deref() == Some(theme.id.as_str());
          self.choice_row(Item::CustomTheme(index), theme.name.clone(), is_current)
        })
        .collect(),
      AppearancePage::ThemeImport => self
        .import_archives
        .iter()
        .enumerate()
        .map(|(index, path)| {
          let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
          Row::action(Item::ImportArchive(index), name)
        })
        .collect(),
      AppearancePage::Mode => {
        let float = self.state.is_float_theme();
        vec![
          self.choice_row(
            Item::ModeSticky,
            self.label("control_center.theme_mode_sticky"),
            !float,
          ),
          self.choice_row(
            Item::ModeFloat,
            self.label("control_center.theme_mode_float"),
            float,
          ),
        ]
      }
      AppearancePage::Icons => self.icon_rows(),
      AppearancePage::Wallpapers => std::iter::once(with_icon(
        Row::action(
          Item::ChooseWallpaper,
          self.label("control_center.choose_image_from_home"),
        ),
        Item::ChooseWallpaper.icon(),
      ))
      .chain(
        WallpaperCollection::ALL
          .into_iter()
          .map(|collection| self.submenu(Item::Collection(collection), collection.label_key())),
      )
      .collect(),
      AppearancePage::WallpaperModes { .. } => WallpaperMode::ALL
        .into_iter()
        .map(|mode| self.submenu(Item::WallpaperMode(mode), mode.label_key()))
        .collect(),
      AppearancePage::WallpaperItems { collection, mode } => self
        .state
        .wallpapers
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.collection == collection && entry.mode == mode)
        .map(|(index, entry)| {
          let filename = std::path::Path::new(&entry.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&entry.path);
          let is_current = self.state.wallpaper_active.as_deref() == Some(entry.path.as_str());
          self.choice_row(Item::WallpaperFile(index), filename, is_current)
        })
        .collect(),
      AppearancePage::Accents => vec![
        with_icon(
          Row::value(
            Item::EditAccent,
            self.label("control_center.edit_highlight_color"),
            self.state.accent.clone(),
            None,
          ),
          Item::EditAccent.icon(),
        ),
        with_icon(
          Row::action(
            Item::ResetAccent,
            self.label("control_center.reset_to_theme_default"),
          ),
          Item::ResetAccent.icon(),
        ),
      ],
      // Unreachable page kept until the Phase 4 cleanup; its entries never
      // had an action, so they are read-only.
      AppearancePage::Transparency => std::iter::once(Row::info(
        self.label("control_center.transparency"),
        self.label(if self.state.transparency {
          "control_center.enabled"
        } else {
          "control_center.disabled"
        }),
      ))
      .chain(EffectSurface::ALL.into_iter().map(|surface| {
        Row::info(
          self.label(surface.label_key()),
          percent(self.effect_value("transparency", surface)),
        )
      }))
      .collect(),
      AppearancePage::Terminal => vec![
        self.toggle_row(
          Item::TerminalTransparency,
          "control_center.transparency",
          self.state.terminal_transparency_enabled,
        ),
        self
          .submenu(
            Item::TerminalTransparencyValue,
            "control_center.transparency",
          )
          .detail(percent(self.state.terminal_transparency)),
      ],
      AppearancePage::Launchers => vec![
        self.toggle_row(
          Item::LauncherTransparency,
          "control_center.transparency",
          self.state.launcher_transparency_enabled,
        ),
        self
          .submenu(
            Item::LauncherTransparencyValue,
            "control_center.transparency",
          )
          .detail(percent(self.state.launcher_transparency)),
      ],
      // Entered directly from the Hyprland category (crate principal). The
      // switch and every Hyprland blur parameter edit the draft; `Apply`
      // (added by `rows`) writes them in one step.
      AppearancePage::Blur => {
        let draft = self.blur_editor();
        let enable = self.toggle_row(Item::BlurEnabled, "control_center.enable", draft.enabled);
        let mut rows = vec![
          enable,
          Row::section(self.label("control_center.blur_values")),
        ];
        rows.extend(BlurField::ALL.map(|field| {
          self.value_row(
            Item::BlurField(field),
            field.label_key(),
            field.format(draft.values.get(field)),
            Some(1),
          )
        }));
        rows
      }
      // Entered directly from the Hyprland category (crate principal). The
      // switch, the custom curves and every leaf edit the shared Animations
      // draft; `Apply` (added by `rows`) writes it in one step.
      AppearancePage::Animations => {
        let draft = self.animations_editor();
        let mut rows = vec![
          self.toggle_row(Item::Animations, "control_center.enable", draft.enabled),
          Row::section(self.label("control_center.animations_options")),
          self.submenu(
            Item::AnimationsCustomCurves,
            "control_center.animations_custom_curves",
          ),
        ];
        rows.extend(
          AnimationGroup::ALL
            .map(|group| self.submenu(Item::AnimationsGroup(group), group.label_key())),
        );
        rows
      }
      AppearancePage::AnimationsCustomCurves => AnimationCurve::CUSTOM
        .into_iter()
        .map(|curve| self.submenu(Item::AnimationsCurve(curve), curve.label_key()))
        .collect(),
      AppearancePage::AnimationsCurve { curve } => self.animation_curve_rows(curve),
      AppearancePage::AnimationsGroup { group } => group
        .leaves()
        .map(|leaf| self.submenu(Item::AnimationsLeaf(leaf), leaf.label_key()))
        .collect(),
      AppearancePage::AnimationsLeaf { leaf } => self.animation_leaf_rows(leaf),
      AppearancePage::TerminalTransparency | AppearancePage::TransparencySurface { .. } => {
        vec![self.value_row(
          Item::EffectValue,
          "control_center.transparency",
          percent(self.effect_editor_value()),
          Some(5),
        )]
      }
      // Entered directly from the Hyprland category (crate principal);
      // groups the two border-related screens under one parent.
      AppearancePage::Borders => vec![
        self.submenu(Item::EdgeThickness, "control_center.thickness"),
        self.submenu(Item::GeneralBorders, "control_center.rounded"),
      ],
      AppearancePage::TaskbarPosition => [TaskbarPosition::Top, TaskbarPosition::Bottom]
        .into_iter()
        .map(|position| {
          let key = match position {
            TaskbarPosition::Top => "control_center.top",
            TaskbarPosition::Bottom => "control_center.bottom",
          };
          self.choice_row(
            Item::Position(position),
            self.label(key),
            self.state.waybar_pos == position,
          )
        })
        .collect(),
      AppearancePage::TaskbarSpaces => [
        (PromptGoal::WaybarTop, self.state.waybar_top),
        (PromptGoal::WaybarLeft, self.state.waybar_left),
        (PromptGoal::WaybarRight, self.state.waybar_right),
        (PromptGoal::WaybarBottom, self.state.waybar_bottom),
      ]
      .into_iter()
      .map(|(goal, value)| self.spacing_row(goal, value))
      .collect(),
      // Entered directly from the Hyprland category; groups the inner and
      // outer gaps under their own submenus.
      AppearancePage::WindowSpaces => vec![
        self.submenu(Item::WindowSpacesInner, "control_center.inner"),
        self.submenu(Item::WindowSpacesOuter, "control_center.outer"),
      ],
      AppearancePage::WindowSpacesInner => {
        vec![self.spacing_row(PromptGoal::GapsIn, self.state.gaps_in)]
      }
      AppearancePage::WindowSpacesOuter => [
        (PromptGoal::GapsOutTop, self.state.gaps_out_top),
        (PromptGoal::GapsOutLeft, self.state.gaps_out_left),
        (PromptGoal::GapsOutRight, self.state.gaps_out_right),
        (PromptGoal::GapsOutBottom, self.state.gaps_out_bottom),
      ]
      .into_iter()
      .map(|(goal, value)| self.spacing_row(goal, value))
      .collect(),
      AppearancePage::GeneralBorders => {
        let rounding = if self.state.rounded {
          self.state.rounding.to_string()
        } else {
          format!(
            "{} · {}",
            self.state.rounding,
            self.label("control_center.disabled")
          )
        };
        vec![
          self.toggle_row(Item::Rounded, "control_center.enable", self.state.rounded),
          // Rounding has no visible effect while Rounded is off, so the row
          // is disabled and skipped instead of opening a pointless prompt.
          self
            .value_row(Item::Rounding, "control_center.value", rounding, None)
            .enabled(self.state.rounded),
        ]
      }
      AppearancePage::EdgeThickness => vec![self.value_row(
        Item::Thickness,
        "control_center.value",
        self.state.thickness.to_string(),
        None,
      )],
      AppearancePage::Taskbar => vec![
        self.submenu(Item::TaskbarTransparency, "control_center.transparency"),
        self.submenu(Item::TaskbarPosition, "control_center.position"),
        self.submenu(Item::TaskbarSpaces, "control_center.spaces"),
        self.submenu(Item::TaskbarIcons, "control_center.icons"),
        self.submenu(Item::TaskbarDate, "control_center.date"),
        self.submenu(Item::TaskbarTime, "control_center.time"),
      ],
      AppearancePage::TaskbarIcons => {
        let draft = self.surface_draft();
        let mut rows = vec![
          self.toggle_row(
            Item::AudioPlayer,
            "control_center.taskbar_audio_player_view",
            draft.is_some_and(|draft| draft.audio_player_enabled),
          ),
          self.submenu(Item::LauncherIcon, "control_center.launcher"),
        ];
        rows.extend(TaskbarUtilityWidget::ALL.into_iter().map(|widget| {
          self.toggle_row(
            Item::UtilityWidget(widget),
            widget.label_key(),
            draft.is_some_and(|draft| draft.utility_widgets.enabled(widget)),
          )
        }));
        rows.push(self.submenu(Item::Utilities, "control_center.utilities"));
        rows
      }
      AppearancePage::TaskbarDate => vec![self.submenu(Item::DateFormats, "control_center.format")],
      AppearancePage::TaskbarDateFormat => TaskbarDateFormat::ALL
        .into_iter()
        .map(|format| {
          self.choice_row(
            Item::DateFormat(format),
            self.label(format.label_key()),
            self
              .surface_draft()
              .is_some_and(|draft| draft.date_format == format),
          )
        })
        .collect(),
      AppearancePage::TaskbarTime => vec![
        self.toggle_row(
          Item::Seconds,
          "control_center.seconds",
          self
            .surface_draft()
            .is_some_and(|draft| draft.time_seconds_enabled),
        ),
        self.submenu(Item::TimeFormats, "control_center.format"),
      ],
      AppearancePage::TaskbarTimeFormat => TaskbarTimeFormat::ALL
        .into_iter()
        .map(|format| {
          self.choice_row(
            Item::TimeFormat(format),
            self.label(format.label_key()),
            self
              .surface_draft()
              .is_some_and(|draft| draft.time_format == format),
          )
        })
        .collect(),
      AppearancePage::WidgetTelemetry => vec![
        self.toggle_row(
          Item::SurfaceEnabled,
          "control_center.enable",
          self
            .surface_draft()
            .is_some_and(|draft| draft.widget_enabled),
        ),
        self.submenu(Item::Sessions, "control_center.sessions"),
        self.submenu(Item::SurfaceTransparency, "control_center.transparency"),
      ],
      AppearancePage::ControlPanel => vec![
        self.toggle_row(
          Item::SurfaceEnabled,
          "control_center.enable",
          self
            .surface_draft()
            .is_some_and(|draft| draft.control_panel_enabled),
        ),
        self.submenu(Item::Sessions, "control_center.sessions"),
        self.submenu(Item::SurfaceTransparency, "control_center.transparency"),
      ],
      AppearancePage::SurfaceSection { surface, section } => self.section_rows(surface, section),
      AppearancePage::AccentEdit | AppearancePage::Prompt { .. } => Vec::new(),
    }
  }

  /// Appearance root: groups the Desktop and Application Style pages.
  fn home_rows(&self) -> Vec<Row<Item>> {
    vec![
      self.submenu(Item::Desktop, "control_center.desktop"),
      self.submenu(Item::ApplicationStyle, "control_center.application_style"),
    ]
  }

  /// Appearance > Desktop: theme, appearance mode, accent color and wallpaper.
  fn desktop_rows(&self) -> Vec<Row<Item>> {
    let mode = self.label(if self.state.is_float_theme() {
      "control_center.theme_mode_float"
    } else {
      "control_center.theme_mode_sticky"
    });
    let wallpaper = self
      .state
      .wallpaper_active
      .clone()
      .unwrap_or_else(|| self.label("control_center.none").to_string());
    let mut rows = vec![
      self
        .submenu(Item::Themes, "control_center.theme")
        .detail(format!(
          "{} [{mode}]",
          current_family_label(&self.state, &self.lang)
        )),
      self
        .submenu(Item::Mode, "control_center.appearance_mode")
        .detail(mode),
      self
        .submenu(Item::Accent, "control_center.highlight_color")
        .detail(accent_label(&self.state.accent)),
      self
        .submenu(Item::Wallpaper, "control_center.wallpaper")
        .detail(wallpaper),
      self
        .submenu(Item::Icons, "control_center.icons")
        .detail(self.state.icon_theme.clone()),
    ];
    if self.fonts_available {
      rows.push(self.submenu(Item::Fonts, "control_center.fonts"));
    }
    rows
  }

  /// Appearance > Desktop > Icons: one choice per installed icon theme.
  /// The configured theme is marked; when it is not installed, an info row
  /// says which theme is used instead. Info rows are not focusable.
  pub(super) fn icon_theme_rows(&self) -> Vec<Row<Item>> {
    if self.state.icon_themes.is_empty() {
      return vec![Row::info(
        self.label("control_center.icon_themes_empty"),
        String::new(),
      )];
    }

    let configured = self.state.icon_theme.as_str();
    // The mark follows the draft: a chosen theme is shown before it is applied.
    let marked = self.icon_draft.as_deref().unwrap_or(configured);
    let configured_installed = self
      .state
      .icon_themes
      .iter()
      .any(|entry| entry.directory_name == configured);
    let mut rows = Vec::new();
    if !configured_installed {
      rows.push(Row::info(
        self.label("control_center.icon_theme_missing_fallback"),
        configured.to_string(),
      ));
    }
    rows.extend(
      self
        .state
        .icon_themes
        .iter()
        .enumerate()
        .map(|(index, entry)| {
          let origin = match entry.source {
            IconThemeSource::System => self.label("control_center.icon_theme_source_system"),
            IconThemeSource::User => self.label("control_center.icon_theme_source_user"),
          };
          let detail = match entry.official {
            Some(_) => format!(
              "{} · {origin}",
              self.label("control_center.icon_theme_official")
            ),
            None => origin.to_string(),
          };
          self
            .choice_row(
              Item::IconTheme(index),
              entry.display_name.clone(),
              entry.directory_name == marked,
            )
            .detail(detail)
        }),
    );
    rows
  }

  /// Appearance > Application Style: terminal and launcher transparency.
  fn application_style_rows(&self) -> Vec<Row<Item>> {
    vec![
      self.submenu(Item::Terminal, "control_center.terminal"),
      self.submenu(Item::Launcher, "control_center.launcher"),
    ]
  }

  fn section_rows(&self, surface: EffectSurface, section: SurfaceSection) -> Vec<Row<Item>> {
    let draft = self.surface_draft();
    match section {
      SurfaceSection::Launcher => vec![
        self.toggle_row(
          Item::LauncherEnabled,
          "control_center.enable",
          draft.is_some_and(|draft| draft.launcher_enabled),
        ),
        with_icon(
          Row::action(
            Item::ChooseLauncherIcon,
            self.label("control_center.custom"),
          ),
          Item::ChooseLauncherIcon.icon(),
        )
        .detail(
          self
            .state
            .taskbar_launcher_custom_icon_path
            .clone()
            .unwrap_or_else(|| self.label("control_center.none").to_string()),
        ),
      ],
      SurfaceSection::UtilityIcons => [
        (
          TaskbarUtilityGroupMode::AlwaysExpanded,
          "control_center.taskbar_utility_group_always_expanded",
        ),
        (
          TaskbarUtilityGroupMode::Auto,
          "control_center.taskbar_utility_group_auto",
        ),
      ]
      .into_iter()
      .map(|(mode, key)| {
        self.choice_row(
          Item::UtilityGroup(mode),
          self.label(key),
          draft.is_some_and(|draft| draft.utility_group == mode),
        )
      })
      .collect(),
      SurfaceSection::Sessions => match surface {
        EffectSurface::WidgetTelemetry => draft
          .map(|draft| draft.widget_order.clone())
          .unwrap_or_else(|| WidgetTelemetryBlock::ALL.to_vec())
          .into_iter()
          .map(|block| {
            self.toggle_row(
              Item::TelemetryBlock(block),
              block.label_key(),
              draft.is_some_and(|draft| draft.widget_blocks.enabled(block)),
            )
          })
          .collect(),
        EffectSurface::ControlPanel => draft
          .map(|draft| {
            self
              .state
              .control_panel_order
              .clone()
              .into_iter()
              .filter(|card| draft.control_panel_cards.available(*card))
              .map(|card| {
                self.toggle_row(
                  Item::PanelCard(card),
                  card.label_key(),
                  draft.control_panel_cards.enabled(card),
                )
              })
              .collect()
          })
          .unwrap_or_default(),
        EffectSurface::Taskbar | EffectSurface::Terminal | EffectSurface::Launchers => Vec::new(),
      },
      SurfaceSection::Transparency => {
        let (enabled, value) = match draft {
          Some(draft) => (draft.transparency_enabled, draft.transparency),
          None => (false, 50),
        };
        vec![
          self.toggle_row(Item::SectionEnabled, "control_center.enable", enabled),
          self.value_row(
            Item::SectionValue,
            "control_center.value",
            percent(value),
            Some(5),
          ),
        ]
      }
    }
  }

  fn value_row(
    &self,
    item: Item,
    key: &str,
    text: impl Into<String>,
    step: Option<i32>,
  ) -> Row<Item> {
    with_icon(
      Row::value(item, self.label(key), text, step),
      self.item_icon(item),
    )
  }

  /// Rows of one custom curve. Bézier and Spring list their parameters;
  /// Linear and Ease-out are fixed curves, so they only show their points.
  fn animation_curve_rows(&self, curve: AnimationCurve) -> Vec<Row<Item>> {
    let draft = self.animations_editor();
    let params: &[CurveParam] = match curve {
      AnimationCurve::Bezier => &[
        CurveParam::BezierX1,
        CurveParam::BezierY1,
        CurveParam::BezierX2,
        CurveParam::BezierY2,
      ],
      AnimationCurve::Spring => &[
        CurveParam::SpringMass,
        CurveParam::SpringStiffness,
        CurveParam::SpringDampening,
      ],
      AnimationCurve::Default | AnimationCurve::Linear | AnimationCurve::EaseOut => &[],
    };
    let fixed_points = match curve {
      AnimationCurve::Linear => Some("(0, 0) (1, 1)"),
      AnimationCurve::EaseOut => Some("(0, 0) (0.58, 1)"),
      _ => None,
    };
    if let Some(points) = fixed_points {
      return vec![Row::info(
        self.label("control_center.animation_points"),
        points,
      )];
    }
    params
      .iter()
      .map(|param| {
        let field = AnimationField::Param(*param);
        self.value_row(
          Item::AnimationField(field),
          field.label_key(),
          field.format(draft.values.get(field)),
          Some(1),
        )
      })
      .collect()
  }

  /// Rows of one animation leaf, in the order of the approved layout: Enable
  /// (when the leaf has one), Style, Speed, Curve and Pop-in percentage.
  fn animation_leaf_rows(&self, leaf: AnimationLeaf) -> Vec<Row<Item>> {
    let draft = self.animations_editor();
    let values = draft.values.leaf(leaf);
    let mut rows = Vec::new();
    if leaf.has_enabled() {
      rows.push(self.toggle_row(
        Item::AnimationLeafEnabled(leaf),
        "control_center.enable",
        values.enabled,
      ));
    }
    if leaf.has_style() {
      rows.push(self.value_row(
        Item::AnimationLeafStyle(leaf),
        "control_center.animation_style",
        self.label(values.style.label_key()),
        Some(1),
      ));
    }
    let speed = AnimationField::Speed(leaf);
    rows.push(self.value_row(
      Item::AnimationField(speed),
      speed.label_key(),
      speed.format(draft.values.get(speed)),
      Some(1),
    ));
    rows.push(self.value_row(
      Item::AnimationLeafCurve(leaf),
      "control_center.animation_curve",
      self.label(values.curve.label_key()),
      Some(1),
    ));
    if leaf.has_popin() {
      let popin = AnimationField::Popin(leaf);
      rows.push(self.value_row(
        Item::AnimationField(popin),
        popin.label_key(),
        format!("{}%", popin.format(draft.values.get(popin))),
        Some(1),
      ));
    }
    rows
  }

  fn spacing_row(&self, goal: PromptGoal, value: i32) -> Row<Item> {
    let item = Item::Spacing(goal);
    self.value_row(item, self.prompt_label(goal), value.to_string(), None)
  }
}
