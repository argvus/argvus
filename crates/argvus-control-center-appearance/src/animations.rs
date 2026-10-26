//! Hyprland animation parameters edited on the Animations pages.
//!
//! The values are stored by `argvus-config` under `effects.animation_*` and
//! reach the compositor through `argvus-hyprland`. This module owns the leaves,
//! their bounds, steps, display format and parsing, so the pages, the backend
//! and the tests agree. The bounds must match `ANIMATION_SETTINGS` in
//! `argvus effects-toggle` and the schema in `argvus-config`.

/// Animation leaves the pages expose. Only leaves that change what Hyprland 0.56.2
/// draws are listed. `windows`, `layers` and `workspaces` are parents that the
/// compositor never reads on their own, so their values would have no effect.
/// Window resizing shares `windowsMove`, and the popup fades, shadow and glow
/// gradient angles and the zoom fade are not exposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationLeaf {
  Opening,
  Closing,
  Moving,
  WorkspaceIn,
  WorkspaceOut,
  SpecialWorkspace,
  LayersIn,
  LayersOut,
  Fade,
  FadeIn,
  FadeOut,
  Border,
  BorderAngle,
  Zoom,
  MonitorAdded,
}

impl AnimationLeaf {
  pub const ALL: [Self; 15] = [
    Self::Opening,
    Self::Closing,
    Self::Moving,
    Self::WorkspaceIn,
    Self::WorkspaceOut,
    Self::SpecialWorkspace,
    Self::LayersIn,
    Self::LayersOut,
    Self::Fade,
    Self::FadeIn,
    Self::FadeOut,
    Self::Border,
    Self::BorderAngle,
    Self::Zoom,
    Self::MonitorAdded,
  ];

  /// Name used in the `effects.animation_<key>_*` config keys and in the
  /// script arguments.
  pub const fn key(self) -> &'static str {
    match self {
      Self::Opening => "opening",
      Self::Closing => "closing",
      Self::Moving => "moving",
      Self::WorkspaceIn => "workspace_in",
      Self::WorkspaceOut => "workspace_out",
      Self::SpecialWorkspace => "special_workspace",
      Self::LayersIn => "layers_in",
      Self::LayersOut => "layers_out",
      Self::Fade => "fade",
      Self::FadeIn => "fade_in",
      Self::FadeOut => "fade_out",
      Self::Border => "border",
      Self::BorderAngle => "border_angle",
      Self::Zoom => "zoom",
      Self::MonitorAdded => "monitor_added",
    }
  }

  /// Translation key of the row label.
  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Opening => "control_center.animation_opening",
      Self::Closing => "control_center.animation_closing",
      Self::Moving => "control_center.animation_moving",
      Self::WorkspaceIn => "control_center.animation_workspace_in",
      Self::WorkspaceOut => "control_center.animation_workspace_out",
      Self::SpecialWorkspace => "control_center.animation_special_workspace",
      Self::LayersIn => "control_center.animation_layers_in",
      Self::LayersOut => "control_center.animation_layers_out",
      Self::Fade => "control_center.animation_fade",
      Self::FadeIn => "control_center.animation_fade_in",
      Self::FadeOut => "control_center.animation_fade_out",
      Self::Border => "control_center.animation_border",
      Self::BorderAngle => "control_center.animation_border_angle",
      Self::Zoom => "control_center.animation_zoom",
      Self::MonitorAdded => "control_center.animation_monitor_added",
    }
  }

  /// The category page that lists the leaf.
  pub const fn group(self) -> AnimationGroup {
    match self {
      Self::Opening | Self::Closing | Self::Moving => AnimationGroup::Windows,
      Self::WorkspaceIn | Self::WorkspaceOut | Self::SpecialWorkspace => AnimationGroup::Workspaces,
      Self::LayersIn | Self::LayersOut | Self::Fade | Self::FadeIn | Self::FadeOut => {
        AnimationGroup::LayersPopups
      }
      Self::Border | Self::BorderAngle => AnimationGroup::BordersEffects,
      Self::Zoom | Self::MonitorAdded => AnimationGroup::OtherAnimations,
    }
  }

  /// Whether the leaf page has an "Enabled" switch.
  pub const fn has_enabled(self) -> bool {
    matches!(
      self,
      Self::Moving
        | Self::Fade
        | Self::FadeIn
        | Self::FadeOut
        | Self::Border
        | Self::BorderAngle
        | Self::Zoom
        | Self::MonitorAdded
    )
  }

  /// Whether the leaf page has a "Style" row.
  pub const fn has_style(self) -> bool {
    matches!(
      self,
      Self::Opening
        | Self::Closing
        | Self::WorkspaceIn
        | Self::WorkspaceOut
        | Self::SpecialWorkspace
        | Self::LayersIn
        | Self::LayersOut
    )
  }

  /// Whether the leaf page has a "Pop-in percentage" row (window opening and
  /// closing only).
  pub const fn has_popin(self) -> bool {
    matches!(self, Self::Opening | Self::Closing)
  }

  /// The styles Hyprland 0.56.2 accepts for the leaf, in display order. Empty
  /// for leaves without a Style row.
  pub const fn styles(self) -> &'static [AnimationStyle] {
    match self {
      Self::Opening | Self::Closing => &WINDOW_STYLES,
      Self::LayersIn | Self::LayersOut => &LAYER_STYLES,
      Self::WorkspaceIn | Self::WorkspaceOut | Self::SpecialWorkspace => &WORKSPACE_STYLES,
      _ => &[],
    }
  }

  /// Values written when the config has no usable entry. They are the
  /// Hyprland 0.56.2 stock values, or the ARGVUS values for the leaves the
  /// stock config does not define.
  pub const fn default_values(self) -> LeafValues {
    LeafValues {
      enabled: true,
      style: self.default_style(),
      speed: self.default_speed(),
      curve: AnimationCurve::Default,
      popin: 87,
    }
  }

  const fn default_style(self) -> AnimationStyle {
    match self {
      Self::Opening | Self::Closing => AnimationStyle::Popin,
      _ => AnimationStyle::Slide,
    }
  }

  const fn default_speed(self) -> f64 {
    match self {
      Self::Opening => 4.1,
      Self::Closing => 1.49,
      Self::Moving => 4.79,
      Self::WorkspaceIn => 1.21,
      Self::WorkspaceOut => 1.94,
      Self::SpecialWorkspace => 5.0,
      Self::LayersIn => 4.0,
      Self::LayersOut => 1.5,
      Self::Fade => 3.03,
      Self::FadeIn => 1.73,
      Self::FadeOut => 1.46,
      Self::Border => 5.39,
      Self::BorderAngle => 5.0,
      Self::Zoom => 7.0,
      Self::MonitorAdded => 5.0,
    }
  }
}

/// The categories of the Animations page. Each one has its own page that
/// lists its leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationGroup {
  Windows,
  Workspaces,
  LayersPopups,
  BordersEffects,
  OtherAnimations,
}

impl AnimationGroup {
  pub const ALL: [Self; 5] = [
    Self::Windows,
    Self::Workspaces,
    Self::LayersPopups,
    Self::BordersEffects,
    Self::OtherAnimations,
  ];

  /// Translation key of the category row.
  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Windows => "control_center.animations_windows",
      Self::Workspaces => "control_center.animations_workspaces",
      Self::LayersPopups => "control_center.animations_layers_popups",
      Self::BordersEffects => "control_center.animations_borders_effects",
      Self::OtherAnimations => "control_center.animations_other",
    }
  }

  /// The leaves listed under this category, in display order.
  pub fn leaves(self) -> impl Iterator<Item = AnimationLeaf> {
    AnimationLeaf::ALL
      .into_iter()
      .filter(move |leaf| leaf.group() == self)
  }
}

/// Visual style of a leaf animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationStyle {
  Slide,
  Popin,
  Fade,
  SlideFade,
  SlideVert,
  SlideFadeVert,
  Gnomed,
}

/// Styles of the window open and close leaves. The slide variants are left out
/// because Hyprland draws them as pop-in.
const WINDOW_STYLES: [AnimationStyle; 3] = [
  AnimationStyle::Slide,
  AnimationStyle::Popin,
  AnimationStyle::Gnomed,
];

/// Styles of the layer leaves. The slide variants are left out because Hyprland
/// draws them as a plain slide.
const LAYER_STYLES: [AnimationStyle; 3] = [
  AnimationStyle::Slide,
  AnimationStyle::Popin,
  AnimationStyle::Fade,
];

/// Styles of the workspace leaves. Hyprland refuses `popin` and `gnomed` here.
const WORKSPACE_STYLES: [AnimationStyle; 5] = [
  AnimationStyle::Slide,
  AnimationStyle::Fade,
  AnimationStyle::SlideFade,
  AnimationStyle::SlideVert,
  AnimationStyle::SlideFadeVert,
];

impl AnimationStyle {
  pub const ALL: [Self; 7] = [
    Self::Slide,
    Self::Popin,
    Self::Fade,
    Self::SlideFade,
    Self::SlideVert,
    Self::SlideFadeVert,
    Self::Gnomed,
  ];

  /// Hyprland style name, also used in the config and the script arguments.
  pub const fn name(self) -> &'static str {
    match self {
      Self::Slide => "slide",
      Self::Popin => "popin",
      Self::Fade => "fade",
      Self::SlideFade => "slidefade",
      Self::SlideVert => "slidevert",
      Self::SlideFadeVert => "slidefadevert",
      Self::Gnomed => "gnomed",
    }
  }

  /// Translation key of the style name.
  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Slide => "control_center.animation_style_slide",
      Self::Popin => "control_center.animation_style_popin",
      Self::Fade => "control_center.animation_style_fade",
      Self::SlideFade => "control_center.animation_style_slidefade",
      Self::SlideVert => "control_center.animation_style_slidevert",
      Self::SlideFadeVert => "control_center.animation_style_slidefadevert",
      Self::Gnomed => "control_center.animation_style_gnomed",
    }
  }

  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|style| style.name() == name)
  }
}

/// Motion curve of a leaf. `Default` is the ARGVUS default Bézier; the other
/// four are the curves listed under "Custom curves".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationCurve {
  Default,
  Bezier,
  Spring,
  Linear,
  EaseOut,
}

impl AnimationCurve {
  pub const ALL: [Self; 5] = [
    Self::Default,
    Self::Bezier,
    Self::Spring,
    Self::Linear,
    Self::EaseOut,
  ];

  /// The curves that have an editor under "Custom curves", in display order.
  pub const CUSTOM: [Self; 4] = [Self::Bezier, Self::Spring, Self::Linear, Self::EaseOut];

  pub const fn name(self) -> &'static str {
    match self {
      Self::Default => "default",
      Self::Bezier => "bezier",
      Self::Spring => "spring",
      Self::Linear => "linear",
      Self::EaseOut => "ease_out",
    }
  }

  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Default => "control_center.animation_curve_default",
      Self::Bezier => "control_center.animation_curve_bezier",
      Self::Spring => "control_center.animation_curve_spring",
      Self::Linear => "control_center.animation_curve_linear",
      Self::EaseOut => "control_center.animation_curve_ease_out",
    }
  }

  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|curve| curve.name() == name)
  }
}

/// Editable parameters of the custom curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveParam {
  BezierX1,
  BezierY1,
  BezierX2,
  BezierY2,
  SpringMass,
  SpringStiffness,
  SpringDampening,
}

impl CurveParam {
  pub const ALL: [Self; 7] = [
    Self::BezierX1,
    Self::BezierY1,
    Self::BezierX2,
    Self::BezierY2,
    Self::SpringMass,
    Self::SpringStiffness,
    Self::SpringDampening,
  ];

  /// Name used in the `effects.animation_<name>` config key and in the script
  /// arguments.
  pub const fn name(self) -> &'static str {
    match self {
      Self::BezierX1 => "bezier_x1",
      Self::BezierY1 => "bezier_y1",
      Self::BezierX2 => "bezier_x2",
      Self::BezierY2 => "bezier_y2",
      Self::SpringMass => "spring_mass",
      Self::SpringStiffness => "spring_stiffness",
      Self::SpringDampening => "spring_dampening",
    }
  }

  pub const fn label_key(self) -> &'static str {
    match self {
      Self::BezierX1 => "control_center.animation_bezier_x1",
      Self::BezierY1 => "control_center.animation_bezier_y1",
      Self::BezierX2 => "control_center.animation_bezier_x2",
      Self::BezierY2 => "control_center.animation_bezier_y2",
      Self::SpringMass => "control_center.animation_spring_mass",
      Self::SpringStiffness => "control_center.animation_spring_stiffness",
      Self::SpringDampening => "control_center.animation_spring_dampening",
    }
  }

  /// Inclusive bounds accepted for the parameter.
  pub const fn bounds(self) -> (f64, f64) {
    match self {
      Self::BezierX1 | Self::BezierX2 => (0.0, 1.0),
      Self::BezierY1 | Self::BezierY2 => (-1.0, 2.0),
      Self::SpringMass => (0.5, 10.0),
      Self::SpringStiffness => (0.5, 2000.0),
      Self::SpringDampening => (0.5, 200.0),
    }
  }

  /// Value written when the config has no usable entry.
  pub const fn default_value(self) -> f64 {
    match self {
      Self::BezierX1 => 0.23,
      Self::BezierY1 => 1.0,
      Self::BezierX2 => 0.32,
      Self::BezierY2 => 1.0,
      Self::SpringMass => 1.0,
      Self::SpringStiffness => 238.1191,
      Self::SpringDampening => 24.21279333,
    }
  }

  /// Amount added by one `←/→` step.
  pub const fn step(self) -> f64 {
    match self {
      Self::BezierX1 | Self::BezierX2 | Self::BezierY1 | Self::BezierY2 => 0.05,
      Self::SpringMass => 0.1,
      Self::SpringStiffness => 10.0,
      Self::SpringDampening => 1.0,
    }
  }
}

/// A numeric value of the Animations pages: a leaf's speed or pop-in
/// percentage, or a custom curve parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationField {
  Speed(AnimationLeaf),
  Popin(AnimationLeaf),
  Param(CurveParam),
}

impl AnimationField {
  /// Translation key of the row label.
  pub const fn label_key(self) -> &'static str {
    match self {
      Self::Speed(_) => "control_center.animation_speed",
      Self::Popin(_) => "control_center.animation_popin",
      Self::Param(param) => param.label_key(),
    }
  }

  pub const fn bounds(self) -> (f64, f64) {
    match self {
      Self::Speed(_) => (1.0, 10.0),
      Self::Popin(_) => (1.0, 100.0),
      Self::Param(param) => param.bounds(),
    }
  }

  pub const fn is_integer(self) -> bool {
    matches!(self, Self::Popin(_))
  }

  /// Whether the prompt accepts a minus sign for this field.
  pub fn allows_negative(self) -> bool {
    self.bounds().0 < 0.0
  }

  pub const fn step(self) -> f64 {
    match self {
      Self::Speed(_) => 0.1,
      Self::Popin(_) => 5.0,
      Self::Param(param) => param.step(),
    }
  }

  /// Clamps `value` to the bounds and rounds it to the precision of the field.
  pub fn normalize(self, value: f64) -> f64 {
    clamp_number(value, self.bounds(), self.is_integer())
  }

  /// Parses text typed in the prompt or read from the config. Returns `None`
  /// for anything that is not a number inside the bounds.
  pub fn parse(self, text: &str) -> Option<f64> {
    parse_number(text, self.bounds(), self.is_integer())
  }

  /// Display form. Parsing it back gives the same value.
  pub fn format(self, value: f64) -> String {
    format!("{value}")
  }
}

/// Rounds to the field precision (whole numbers, or eight decimals, which keeps
/// the Hyprland stock spring values exact) and clamps to `bounds`, so repeated
/// `←/→` presses do not accumulate floating-point drift.
fn clamp_number(value: f64, (minimum, maximum): (f64, f64), integer: bool) -> f64 {
  let rounded = if integer {
    value.round()
  } else {
    (value * 1e8).round() / 1e8
  };
  rounded.clamp(minimum, maximum)
}

/// Parses `text` as a finite number inside `bounds`. Whole-number fields
/// reject fractions.
fn parse_number(text: &str, (minimum, maximum): (f64, f64), integer: bool) -> Option<f64> {
  let value: f64 = text.trim().parse().ok()?;
  let in_bounds = value.is_finite() && (minimum..=maximum).contains(&value);
  let shape_ok = !integer || value.fract() == 0.0;
  (in_bounds && shape_ok).then_some(value)
}

/// The entry `delta` steps away from `current` in `all`, wrapping around.
pub fn cycle<T: Copy + PartialEq>(all: &[T], current: T, delta: i32) -> T {
  let index = all.iter().position(|item| *item == current).unwrap_or(0);
  let length = all.len() as i32;
  all[(index as i32 + delta).rem_euclid(length) as usize]
}

/// The persisted settings of one animation leaf.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeafValues {
  pub enabled: bool,
  pub style: AnimationStyle,
  pub speed: f64,
  pub curve: AnimationCurve,
  pub popin: i32,
}

/// Every persisted animation value except the global switch, indexed by
/// [`AnimationLeaf`] (same order as `AnimationLeaf::ALL`) and [`CurveParam`]
/// (same order as `CurveParam::ALL`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationValues {
  leaves: [LeafValues; 15],
  params: [f64; 7],
}

impl Default for AnimationValues {
  fn default() -> Self {
    Self {
      leaves: AnimationLeaf::ALL.map(AnimationLeaf::default_values),
      params: CurveParam::ALL.map(CurveParam::default_value),
    }
  }
}

impl AnimationValues {
  pub fn leaf(&self, leaf: AnimationLeaf) -> LeafValues {
    self.leaves[leaf as usize]
  }

  pub fn leaf_mut(&mut self, leaf: AnimationLeaf) -> &mut LeafValues {
    &mut self.leaves[leaf as usize]
  }

  pub fn get(&self, field: AnimationField) -> f64 {
    match field {
      AnimationField::Speed(leaf) => self.leaf(leaf).speed,
      AnimationField::Popin(leaf) => f64::from(self.leaf(leaf).popin),
      AnimationField::Param(param) => self.params[param as usize],
    }
  }

  pub fn set(&mut self, field: AnimationField, value: f64) {
    let value = field.normalize(value);
    match field {
      AnimationField::Speed(leaf) => self.leaf_mut(leaf).speed = value,
      AnimationField::Popin(leaf) => self.leaf_mut(leaf).popin = value as i32,
      AnimationField::Param(param) => self.params[param as usize] = value,
    }
  }

  /// Moves `field` by `steps` of its own step size, within its bounds.
  pub fn adjust(&mut self, field: AnimationField, steps: i32) {
    let next = self.get(field) + f64::from(steps) * field.step();
    self.set(field, next);
  }

  /// `name=value` pairs for `argvus effects-toggle animations-settings set`. The
  /// global switch is not included; `backend::apply_animations` adds it.
  pub fn script_arguments(&self) -> Vec<String> {
    let mut arguments = Vec::new();
    for leaf in AnimationLeaf::ALL {
      let values = self.leaf(leaf);
      let key = leaf.key();
      if leaf.has_enabled() {
        arguments.push(format!("{key}_enabled={}", values.enabled));
      }
      if leaf.has_style() {
        arguments.push(format!("{key}_style={}", values.style.name()));
      }
      arguments.push(format!("{key}_speed={}", values.speed));
      arguments.push(format!("{key}_curve={}", values.curve.name()));
      if leaf.has_popin() {
        arguments.push(format!("{key}_popin={}", values.popin));
      }
    }
    for param in CurveParam::ALL {
      arguments.push(format!(
        "{}={}",
        param.name(),
        self.get(AnimationField::Param(param))
      ));
    }
    arguments
  }

  /// Reads `name=value` lines from `argvus effects-toggle animations-settings get`.
  /// Missing or invalid entries keep their defaults.
  pub fn from_script_output(output: &str) -> Self {
    let mut values = Self::default();
    for line in output.lines() {
      let Some((name, raw)) = line.split_once('=') else {
        continue;
      };
      values.apply_script_entry(name.trim(), raw.trim());
    }
    values
  }

  fn apply_script_entry(&mut self, name: &str, raw: &str) {
    if let Some(param) = CurveParam::ALL
      .into_iter()
      .find(|param| param.name() == name)
    {
      if let Some(value) = AnimationField::Param(param).parse(raw) {
        self.set(AnimationField::Param(param), value);
      }
      return;
    }
    for leaf in AnimationLeaf::ALL {
      // `fade_in_speed` also starts with `fade`; the suffix check below skips
      // it for the `fade` leaf and lets `fade_in` claim it.
      let Some(suffix) = name
        .strip_prefix(leaf.key())
        .and_then(|rest| rest.strip_prefix('_'))
      else {
        continue;
      };
      match suffix {
        "enabled" if leaf.has_enabled() => {
          if let Ok(enabled) = raw.parse::<bool>() {
            self.leaf_mut(leaf).enabled = enabled;
          }
        }
        "style" if leaf.has_style() => {
          // A style Hyprland refuses for this leaf keeps the default.
          let style = AnimationStyle::from_name(raw).filter(|style| leaf.styles().contains(style));
          if let Some(style) = style {
            self.leaf_mut(leaf).style = style;
          }
        }
        "curve" => {
          if let Some(curve) = AnimationCurve::from_name(raw) {
            self.leaf_mut(leaf).curve = curve;
          }
        }
        "speed" => {
          if let Some(value) = AnimationField::Speed(leaf).parse(raw) {
            self.set(AnimationField::Speed(leaf), value);
          }
        }
        "popin" if leaf.has_popin() => {
          if let Some(value) = AnimationField::Popin(leaf).parse(raw) {
            self.set(AnimationField::Popin(leaf), value);
          }
        }
        _ => continue,
      }
      return;
    }
  }
}

/// What the Animations pages edit: the global switch and the values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationSettings {
  pub enabled: bool,
  pub values: AnimationValues,
}

impl Default for AnimationSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      values: AnimationValues::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn defaults_match_the_values_shown_on_the_pages() {
    let values = AnimationValues::default();
    let opening = AnimationLeaf::Opening;
    assert_eq!(values.leaf(opening).speed, 4.1);
    assert_eq!(values.leaf(opening).style, AnimationStyle::Popin);
    assert_eq!(values.leaf(opening).popin, 87);
    assert_eq!(values.leaf(AnimationLeaf::Closing).speed, 1.49);
    assert_eq!(
      values.leaf(AnimationLeaf::WorkspaceIn).style,
      AnimationStyle::Slide
    );
    assert_eq!(
      values.leaf(AnimationLeaf::WorkspaceOut).style,
      AnimationStyle::Slide
    );
    assert_eq!(
      values.leaf(AnimationLeaf::LayersIn).style,
      AnimationStyle::Slide
    );
    assert_eq!(
      values.leaf(AnimationLeaf::LayersOut).style,
      AnimationStyle::Slide
    );
    assert_eq!(
      values.get(AnimationField::Param(CurveParam::SpringStiffness)),
      238.1191
    );
  }

  #[test]
  fn every_default_style_is_one_the_leaf_accepts() {
    let values = AnimationValues::default();
    for leaf in AnimationLeaf::ALL
      .into_iter()
      .filter(|leaf| leaf.has_style())
    {
      assert!(
        leaf.styles().contains(&values.leaf(leaf).style),
        "{} default style",
        leaf.key()
      );
    }
  }

  #[test]
  fn each_leaf_offers_only_the_styles_hyprland_accepts() {
    assert!(
      AnimationLeaf::Opening
        .styles()
        .contains(&AnimationStyle::Gnomed)
    );
    assert!(
      !AnimationLeaf::Opening
        .styles()
        .contains(&AnimationStyle::Fade)
    );
    assert!(
      AnimationLeaf::LayersIn
        .styles()
        .contains(&AnimationStyle::Fade)
    );
    assert!(
      !AnimationLeaf::LayersIn
        .styles()
        .contains(&AnimationStyle::Gnomed)
    );
    assert!(
      !AnimationLeaf::Opening
        .styles()
        .contains(&AnimationStyle::SlideFadeVert)
    );
    for leaf in [
      AnimationLeaf::WorkspaceIn,
      AnimationLeaf::WorkspaceOut,
      AnimationLeaf::SpecialWorkspace,
    ] {
      assert!(
        !leaf.styles().contains(&AnimationStyle::Popin),
        "{}",
        leaf.key()
      );
      assert!(
        !leaf.styles().contains(&AnimationStyle::Gnomed),
        "{}",
        leaf.key()
      );
    }
  }

  #[test]
  fn each_category_lists_its_leaves() {
    let all: usize = AnimationGroup::ALL
      .into_iter()
      .map(|group| group.leaves().count())
      .sum();
    assert_eq!(all, AnimationLeaf::ALL.len());
    assert_eq!(AnimationGroup::Windows.leaves().count(), 3);
    assert_eq!(AnimationGroup::LayersPopups.leaves().count(), 5);
  }

  #[test]
  fn adjust_steps_by_field_and_stays_in_bounds() {
    let mut values = AnimationValues::default();
    let speed = AnimationField::Speed(AnimationLeaf::Opening);
    values.adjust(speed, -100);
    assert_eq!(values.get(speed), 1.0);
    values.adjust(speed, 1000);
    assert_eq!(values.get(speed), 10.0);

    let y1 = AnimationField::Param(CurveParam::BezierY1);
    values.set(y1, -5.0);
    assert_eq!(values.get(y1), -1.0);
    values.set(y1, 5.0);
    assert_eq!(values.get(y1), 2.0);
  }

  #[test]
  fn parse_applies_the_bounds_and_the_shape() {
    let speed = AnimationField::Speed(AnimationLeaf::Opening);
    assert_eq!(speed.parse("0"), None);
    assert_eq!(speed.parse("11"), None);
    assert_eq!(speed.parse("2.5"), Some(2.5));
    assert_eq!(speed.parse(" 7 "), Some(7.0));
    assert_eq!(
      AnimationField::Popin(AnimationLeaf::Opening).parse("2.5"),
      None
    );

    let y1 = AnimationField::Param(CurveParam::BezierY1);
    assert_eq!(y1.parse("-0.5"), Some(-0.5));
    assert_eq!(y1.parse("-1.5"), None);
    assert_eq!(y1.parse("2.5"), None);
    assert_eq!(
      AnimationField::Param(CurveParam::BezierX1).parse("-0.1"),
      None
    );
    assert_eq!(y1.parse("nan"), None);
  }

  #[test]
  fn cycle_wraps_in_both_directions() {
    assert_eq!(
      cycle(AnimationLeaf::Opening.styles(), AnimationStyle::Popin, 1),
      AnimationStyle::Gnomed
    );
    assert_eq!(
      cycle(AnimationLeaf::Opening.styles(), AnimationStyle::Gnomed, 1),
      AnimationStyle::Slide
    );
    assert_eq!(
      cycle(&AnimationCurve::ALL, AnimationCurve::Default, -1),
      AnimationCurve::EaseOut
    );
  }

  #[test]
  fn script_round_trip_keeps_the_values() {
    let mut values = AnimationValues::default();
    values.leaf_mut(AnimationLeaf::Opening).style = AnimationStyle::Gnomed;
    values.leaf_mut(AnimationLeaf::Fade).enabled = false;
    values.leaf_mut(AnimationLeaf::FadeIn).speed = 9.5;
    values.leaf_mut(AnimationLeaf::Border).curve = AnimationCurve::Spring;
    values.set(AnimationField::Param(CurveParam::BezierY2), 1.25);
    let output = values.script_arguments().join("\n");
    assert_eq!(AnimationValues::from_script_output(&output), values);
  }

  #[test]
  fn fade_in_entries_do_not_leak_into_the_fade_leaf() {
    let values = AnimationValues::from_script_output("fade_in_speed=9.5\n");
    assert_eq!(values.leaf(AnimationLeaf::FadeIn).speed, 9.5);
    assert_eq!(values.leaf(AnimationLeaf::Fade).speed, 3.03);
  }

  #[test]
  fn invalid_script_entries_fall_back_to_defaults() {
    let values = AnimationValues::from_script_output(
      "opening_speed=abc\nopening_style=wobble\nclosing_popin=500\nunknown=3\nmoving_enabled=maybe\n",
    );
    assert_eq!(values, AnimationValues::default());
  }
}
