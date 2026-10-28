//! Transparency, blur and animation values of each surface, read from the effects command and applied through it.

use super::commands::{read_first_or_default, run_effects, run_effects_output};
use super::*;

/// Reads one independent visual state from the shared session contract.
pub(super) fn effect_state(component: &str) -> bool {
  let path = argvus_data_home().join("state").join(component);
  match read_first_or_default(&path).as_deref() {
    Some("enabled") => true,
    Some("disabled") => false,
    _ => matches!(
      run_effects_output(&[component, "status"])
        .as_deref()
        .map(str::trim),
      Some("enabled")
    ),
  }
}

pub(super) fn effect_value(kind: &str, surface: EffectSurface) -> i32 {
  run_effects_output(&[&format!("{kind}-value"), surface.key(), "get"])
    .and_then(|value| value.trim().parse::<i32>().ok())
    .filter(|value| (0..=100).contains(value))
    .unwrap_or(50)
}

/// Reads the Hyprland animation values from the session script. A missing
/// script or config falls back to the defaults.
pub(super) fn animation_values() -> AnimationValues {
  run_effects_output(&["animations-settings", "get"])
    .map(|output| AnimationValues::from_script_output(&output))
    .unwrap_or_default()
}

/// Reads the Hyprland blur parameters from the session script. A missing
/// script or config falls back to the defaults.
pub(super) fn blur_values() -> BlurValues {
  run_effects_output(&["blur-settings", "get"])
    .map(|output| BlurValues::from_script_output(&output))
    .unwrap_or_default()
}

pub(super) fn effect_enabled(kind: &str, surface: EffectSurface) -> bool {
  run_effects_output(&["effect-enabled", surface.key(), kind])
    .is_some_and(|value| value.trim() == "enabled")
}

/// Surfaces whose transparency (and, for some, blur) `load_page` reads.
pub(super) const SURFACES_WITH_EFFECTS: [EffectSurface; 5] = [
  EffectSurface::Taskbar,
  EffectSurface::ControlPanel,
  EffectSurface::WidgetTelemetry,
  EffectSurface::Terminal,
  EffectSurface::Launchers,
];

/// The global effect switches and the Hyprland blur parameters.
pub(super) struct GlobalEffects {
  pub(super) animations: bool,
  pub(super) transparency: bool,
  pub(super) blur: bool,
  pub(super) blur_values: BlurValues,
}

/// One surface's transparency, and its blur when the surface has one.
pub(super) struct SurfaceEffects {
  pub(super) transparency: i32,
  pub(super) transparency_enabled: bool,
  pub(super) blur: Option<(i32, bool)>,
}

pub(super) fn read_global_effects() -> GlobalEffects {
  std::thread::scope(|scope| {
    let blur_values = scope.spawn(blur_values);
    let animations = scope.spawn(|| effect_state("animations"));
    let transparency = scope.spawn(|| effect_state("transparency"));
    let blur = effect_state("blur");
    GlobalEffects {
      animations: animations.join().unwrap_or_default(),
      transparency: transparency.join().unwrap_or_default(),
      blur,
      blur_values: blur_values.join().unwrap_or_default(),
    }
  })
}

/// Reads one surface. Terminal and launchers have no blur of their own.
pub(super) fn read_surface_effects(surface: EffectSurface) -> SurfaceEffects {
  let has_blur = !matches!(surface, EffectSurface::Terminal | EffectSurface::Launchers);
  SurfaceEffects {
    transparency: effect_value("transparency", surface),
    transparency_enabled: effect_enabled("transparency", surface),
    blur: has_blur.then(|| {
      (
        effect_value("blur", surface),
        effect_enabled("blur", surface),
      )
    }),
  }
}

pub(super) fn store_surface_effects(
  state: &mut AppearanceState,
  surface: EffectSurface,
  effects: SurfaceEffects,
) {
  let (blur, blur_enabled) = effects.blur.unwrap_or((0, true));
  match surface {
    EffectSurface::Taskbar => {
      state.taskbar_transparency = effects.transparency;
      state.taskbar_transparency_enabled = effects.transparency_enabled;
      state.taskbar_blur = blur;
      state.taskbar_blur_enabled = blur_enabled;
    }
    EffectSurface::ControlPanel => {
      state.control_panel_transparency = effects.transparency;
      state.control_panel_transparency_enabled = effects.transparency_enabled;
      state.control_panel_blur = blur;
      state.control_panel_blur_enabled = blur_enabled;
    }
    EffectSurface::WidgetTelemetry => {
      state.widget_telemetry_transparency = effects.transparency;
      state.widget_telemetry_transparency_enabled = effects.transparency_enabled;
      state.widget_telemetry_blur = blur;
      state.widget_telemetry_blur_enabled = blur_enabled;
    }
    EffectSurface::Terminal => {
      state.terminal_transparency = effects.transparency;
      state.terminal_transparency_enabled = effects.transparency_enabled;
    }
    EffectSurface::Launchers => {
      state.launcher_transparency = effects.transparency;
      state.launcher_transparency_enabled = effects.transparency_enabled;
    }
  }
}

/// Writes the animations switch and values that changed in one call, so
/// `Apply` reloads the configuration once. `argvus config effects` validates
/// each value again.
pub fn apply_animations(
  enabled: Option<bool>,
  values: Option<AnimationValues>,
) -> Result<(), String> {
  let arguments = settings_arguments(enabled, values.map(|values| values.script_arguments()));
  apply_settings("animations-settings", &arguments)
}

/// The `name=value` arguments of an effects `…-settings set`: the master
/// switch first, when it changed, then the values, when they changed.
pub(super) fn settings_arguments(
  enabled: Option<bool>,
  values: Option<Vec<String>>,
) -> Vec<String> {
  let switch =
    enabled.map(|enabled| format!("enabled={}", if enabled { "enabled" } else { "disabled" }));
  switch
    .into_iter()
    .chain(values.unwrap_or_default())
    .collect()
}

pub(super) fn apply_settings(command: &str, arguments: &[String]) -> Result<(), String> {
  if arguments.is_empty() {
    return Ok(());
  }
  let mut request = vec![command, "set"];
  request.extend(arguments.iter().map(String::as_str));
  run_effects(&request)
}

pub fn set_effect_value(kind: &str, surface: EffectSurface, value: i32) -> Result<(), String> {
  if !matches!(kind, "transparency" | "blur") || !(0..=100).contains(&value) {
    return Err("invalid effect value".into());
  }
  if kind == "transparency" && surface == EffectSurface::Terminal {
    return apply_surface_effects(surface, true, value, true, 0);
  }
  let value = value.to_string();
  run_effects(&[&format!("{kind}-value"), surface.key(), "set", &value])
}

/// Writes the blur switch and parameters that changed in one call, so `Apply`
/// reloads the configuration once. `argvus config effects` validates each
/// value again.
pub fn apply_blur(enabled: Option<bool>, values: Option<BlurValues>) -> Result<(), String> {
  let arguments = settings_arguments(enabled, values.map(BlurValues::script_arguments));
  apply_settings("blur-settings", &arguments)
}

pub fn apply_surface_effects(
  surface: EffectSurface,
  transparency_enabled: bool,
  transparency: i32,
  blur_enabled: bool,
  blur: i32,
) -> Result<(), String> {
  if !(0..=100).contains(&transparency) || !(0..=100).contains(&blur) {
    return Err("invalid effect value".into());
  }
  let args = [
    "surface-apply".to_string(),
    surface.key().to_string(),
    if transparency_enabled {
      "enabled"
    } else {
      "disabled"
    }
    .into(),
    transparency.to_string(),
    if blur_enabled { "enabled" } else { "disabled" }.into(),
    blur.to_string(),
  ];
  let references = args.iter().map(String::as_str).collect::<Vec<_>>();
  run_effects(&references)
}
