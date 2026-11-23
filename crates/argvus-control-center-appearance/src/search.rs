//! Stable search routes for pages exposed by Appearance.
use crate::animations::{AnimationCurve, AnimationGroup, AnimationLeaf};
use crate::model::{
  AppearancePage, EffectSurface, SurfaceSection, WallpaperCollection, WallpaperMode,
};
use argvus_theme::discovery::ThemeCategory;

pub const SEARCH_PAGES: &[(&str, AppearancePage)] = &[
  ("appearance/desktop_style", AppearancePage::Desktop),
  (
    "appearance/application_style",
    AppearancePage::ApplicationStyle,
  ),
  ("appearance/summary", AppearancePage::Home),
  ("appearance/themes", AppearancePage::Themes),
  ("appearance/wallpapers", AppearancePage::Wallpapers),
  ("appearance/icons", AppearancePage::Icons),
  ("appearance/accents", AppearancePage::Accents),
  ("appearance/taskbar", AppearancePage::Taskbar),
  (
    "appearance/taskbar-position",
    AppearancePage::TaskbarPosition,
  ),
  ("appearance/taskbar-icons", AppearancePage::TaskbarIcons),
  ("appearance/taskbar-date", AppearancePage::TaskbarDate),
  ("appearance/taskbar-time", AppearancePage::TaskbarTime),
  ("appearance/taskbar-height", AppearancePage::TaskbarHeight),
  ("appearance/taskbar-border", AppearancePage::TaskbarBorder),
  (
    "appearance/widget-telemetry",
    AppearancePage::WidgetTelemetry,
  ),
  ("appearance/control-panel", AppearancePage::ControlPanel),
  ("appearance/official-themes", AppearancePage::OfficialThemes),
  ("appearance/custom-themes", AppearancePage::CustomThemes),
  ("appearance/theme-import", AppearancePage::ThemeImport),
  ("appearance/mode", AppearancePage::Mode),
  ("appearance/accent-edit", AppearancePage::AccentEdit),
  ("appearance/terminal", AppearancePage::Terminal),
  (
    "appearance/terminal-transparency",
    AppearancePage::TerminalTransparency,
  ),
  ("appearance/launchers", AppearancePage::Launchers),
  (
    "appearance/animations-custom-curves",
    AppearancePage::AnimationsCustomCurves,
  ),
  ("appearance/animations", AppearancePage::Animations),
  ("appearance/blur", AppearancePage::Blur),
  ("appearance/taskbar-spaces", AppearancePage::TaskbarSpaces),
  ("appearance/taskbar-modules", AppearancePage::TaskbarModules),
  (
    "appearance/taskbar-workspaces",
    AppearancePage::TaskbarWorkspaces,
  ),
  (
    "appearance/taskbar-date-format",
    AppearancePage::TaskbarDateFormat,
  ),
  (
    "appearance/taskbar-time-format",
    AppearancePage::TaskbarTimeFormat,
  ),
  (
    "appearance/taskbar-border-thickness",
    AppearancePage::TaskbarBorderThickness,
  ),
  (
    "appearance/window-spaces-inner",
    AppearancePage::WindowSpacesInner,
  ),
  (
    "appearance/window-spaces-outer",
    AppearancePage::WindowSpacesOuter,
  ),
  ("appearance/borders", AppearancePage::Borders),
  ("appearance/general-borders", AppearancePage::GeneralBorders),
  ("appearance/edge-thickness", AppearancePage::EdgeThickness),
  ("appearance/borders-shadow", AppearancePage::BordersShadow),
  (
    "appearance/wallpapers-abstract",
    AppearancePage::WallpaperModes {
      collection: WallpaperCollection::Abstract,
    },
  ),
  (
    "appearance/wallpapers-abstract-dark",
    AppearancePage::WallpaperItems {
      collection: WallpaperCollection::Abstract,
      mode: WallpaperMode::Dark,
    },
  ),
  (
    "appearance/wallpapers-abstract-light",
    AppearancePage::WallpaperItems {
      collection: WallpaperCollection::Abstract,
      mode: WallpaperMode::Light,
    },
  ),
  (
    "appearance/wallpapers-landscape",
    AppearancePage::WallpaperModes {
      collection: WallpaperCollection::Landscape,
    },
  ),
  (
    "appearance/wallpapers-landscape-dark",
    AppearancePage::WallpaperItems {
      collection: WallpaperCollection::Landscape,
      mode: WallpaperMode::Dark,
    },
  ),
  (
    "appearance/wallpapers-landscape-light",
    AppearancePage::WallpaperItems {
      collection: WallpaperCollection::Landscape,
      mode: WallpaperMode::Light,
    },
  ),
  (
    "appearance/themes-dark",
    AppearancePage::ThemeFamilies {
      category: ThemeCategory::Dark,
    },
  ),
  (
    "appearance/themes-light",
    AppearancePage::ThemeFamilies {
      category: ThemeCategory::Light,
    },
  ),
  (
    "hyprland/animation-leaf-opening",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Opening,
    },
  ),
  (
    "hyprland/animation-leaf-closing",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Closing,
    },
  ),
  (
    "hyprland/animation-leaf-moving",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Moving,
    },
  ),
  (
    "hyprland/animation-leaf-workspace-in",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::WorkspaceIn,
    },
  ),
  (
    "hyprland/animation-leaf-workspace-out",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::WorkspaceOut,
    },
  ),
  (
    "hyprland/animation-leaf-special-workspace",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::SpecialWorkspace,
    },
  ),
  (
    "hyprland/animation-leaf-layers-in",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::LayersIn,
    },
  ),
  (
    "hyprland/animation-leaf-layers-out",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::LayersOut,
    },
  ),
  (
    "hyprland/animation-leaf-fade",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Fade,
    },
  ),
  (
    "hyprland/animation-leaf-fade-in",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::FadeIn,
    },
  ),
  (
    "hyprland/animation-leaf-fade-out",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::FadeOut,
    },
  ),
  (
    "hyprland/animation-leaf-border",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Border,
    },
  ),
  (
    "hyprland/animation-leaf-border-angle",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::BorderAngle,
    },
  ),
  (
    "hyprland/animation-leaf-zoom",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Zoom,
    },
  ),
  (
    "hyprland/animation-leaf-monitor-added",
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::MonitorAdded,
    },
  ),
  (
    "hyprland/animation-group-workspaces",
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::Workspaces,
    },
  ),
  (
    "hyprland/animation-group-layers-popups",
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::LayersPopups,
    },
  ),
  (
    "hyprland/animation-group-borders-effects",
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::BordersEffects,
    },
  ),
  (
    "hyprland/animation-group-other-animations",
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::OtherAnimations,
    },
  ),
  (
    "hyprland/animation-curve-bezier",
    AppearancePage::AnimationsCurve {
      curve: AnimationCurve::Bezier,
    },
  ),
  (
    "hyprland/animation-curve-spring",
    AppearancePage::AnimationsCurve {
      curve: AnimationCurve::Spring,
    },
  ),
  (
    "hyprland/animation-curve-linear",
    AppearancePage::AnimationsCurve {
      curve: AnimationCurve::Linear,
    },
  ),
  (
    "hyprland/animation-curve-ease-out",
    AppearancePage::AnimationsCurve {
      curve: AnimationCurve::EaseOut,
    },
  ),
  (
    "appearance/taskbar-launcher",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Launcher,
    },
  ),
  (
    "appearance/taskbar-utility-icons",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::UtilityIcons,
    },
  ),
  (
    "appearance/taskbar-transparency",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::Taskbar,
      section: SurfaceSection::Transparency,
    },
  ),
  (
    "appearance/control-panel-sessions",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::ControlPanel,
      section: SurfaceSection::Sessions,
    },
  ),
  (
    "appearance/control-panel-transparency",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::ControlPanel,
      section: SurfaceSection::Transparency,
    },
  ),
  (
    "appearance/widget-telemetry-sessions",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::WidgetTelemetry,
      section: SurfaceSection::Sessions,
    },
  ),
  (
    "appearance/widget-telemetry-transparency",
    AppearancePage::SurfaceSection {
      surface: EffectSurface::WidgetTelemetry,
      section: SurfaceSection::Transparency,
    },
  ),
];
