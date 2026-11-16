//! The eight layout combinations, composed through the existing domain backends.
use crate::{
  ConfigDocument, ConfigError, ConfigResult, ConfigStore, config_lock,
  window_mode::{self, WindowMode},
  windows,
};
use argvus_theme_core::layout::Variant;
use std::{
  fs::{self, OpenOptions},
  os::unix::fs::OpenOptionsExt,
  process::Command,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarPosition {
  Top,
  Bottom,
}
impl TaskbarPosition {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Top => "top",
      Self::Bottom => "bottom",
    }
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutPreset {
  pub variant: Variant,
  pub windows: WindowMode,
  pub position: TaskbarPosition,
}
impl LayoutPreset {
  pub const ALL: [Self; 8] = [
    Self::new(Variant::Sticky, WindowMode::Tile, TaskbarPosition::Top),
    Self::new(Variant::Sticky, WindowMode::Tile, TaskbarPosition::Bottom),
    Self::new(Variant::Sticky, WindowMode::Floating, TaskbarPosition::Top),
    Self::new(
      Variant::Sticky,
      WindowMode::Floating,
      TaskbarPosition::Bottom,
    ),
    Self::new(Variant::Spaced, WindowMode::Tile, TaskbarPosition::Top),
    Self::new(Variant::Spaced, WindowMode::Tile, TaskbarPosition::Bottom),
    Self::new(Variant::Spaced, WindowMode::Floating, TaskbarPosition::Top),
    Self::new(
      Variant::Spaced,
      WindowMode::Floating,
      TaskbarPosition::Bottom,
    ),
  ];
  const fn new(variant: Variant, windows: WindowMode, position: TaskbarPosition) -> Self {
    Self {
      variant,
      windows,
      position,
    }
  }
  pub fn from_document(document: &ConfigDocument) -> ConfigResult<Self> {
    let variant = match document
      .get("/layout/variant")
      .and_then(|value| value.as_str())
    {
      None => Variant::Sticky,
      Some(value) => Variant::parse(value).ok_or_else(|| invalid("unknown layout variant"))?,
    };
    let position = match document
      .get("/layout/taskbar/position")
      .and_then(|value| value.as_str())
    {
      None | Some("top") => TaskbarPosition::Top,
      Some("bottom") => TaskbarPosition::Bottom,
      Some(_) => return Err(invalid("unknown taskbar position")),
    };
    Ok(Self {
      variant,
      windows: window_mode::from_document(document)?,
      position,
    })
  }
  pub fn index(self) -> usize {
    Self::ALL
      .iter()
      .position(|preset| *preset == self)
      .unwrap_or(0)
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
  Applied,
  Pending,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
  Geometry(Variant),
  Position(TaskbarPosition),
  Windows(WindowMode),
}
fn invalid(message: impl Into<String>) -> ConfigError {
  ConfigError::Invalid(message.into())
}

// Geometry resets the bar to Top. Position must follow it, and Floating must
// observe the final bar edge when it creates the opposite application bar.
fn apply_steps(
  preset: LayoutPreset,
  mut execute: impl FnMut(Step) -> ConfigResult<()>,
) -> ConfigResult<()> {
  for step in [
    Step::Geometry(preset.variant),
    Step::Position(preset.position),
    Step::Windows(preset.windows),
  ] {
    execute(step)?;
  }
  Ok(())
}

/// Applies a complete preset. Partial changes are left visible and can be retried.
/// The preset lock serializes this coordinator, while each backend retains its
/// own lock. Holding the theme or window lock here would deadlock child work.
pub fn apply(preset: LayoutPreset) -> ConfigResult<ApplyOutcome> {
  let store = ConfigStore::from_environment()?;
  let root = store
    .path
    .parent()
    .ok_or(ConfigError::MissingConfigDirectory)?
    .join("data/internal");
  fs::create_dir_all(&root)?;
  let lock = OpenOptions::new()
    .create(true)
    .truncate(false)
    .read(true)
    .write(true)
    .mode(0o600)
    .open(root.join("layout-preset.lock"))?;
  config_lock::lock_file(&lock)?;
  let _guard = config_lock::LockGuard(&lock);
  let before = LayoutPreset::from_document(&store.load_effective()?)?;
  let paths = crate::session_paths::SessionPaths::from_environment()
    .ok_or_else(|| invalid("configuration paths unavailable"))?;
  let displayed = crate::theme_build::displayed_theme(&paths);
  let rendered_spaced = displayed.ends_with("-spaced") || displayed.ends_with("-float");
  // A previous attempt may have saved the mode before rendering failed.
  let retry_marker = root.join("layout-preset.pending");
  let geometry_changed = retry_marker.exists()
    || before.variant != preset.variant
    || rendered_spaced != (preset.variant == Variant::Spaced);
  // Preserve retry intent when saving succeeded but a runtime step failed.
  fs::write(&retry_marker, "pending\n")?;
  let mut window_status = None;
  apply_steps(preset, |step| {
    match step {
      Step::Geometry(variant) => {
        if !geometry_changed {
          return Ok(());
        }
        // Commit the final edge before rendering so the session restarts each bar once.
        {
          let _theme_lock = crate::theme_apply::lock_mode_preparation()?;
          store.apply_layout_preset_geometry(variant.as_str(), preset.position.as_str())?;
        }
        let document = store.load_effective()?;
        let theme = document
          .get("/appearance/theme")
          .and_then(|value| value.as_str())
          .unwrap_or("argvus-dark");
        let output = Command::new("argvus")
          .args(["theme-switch", theme])
          .env("ARGVUS_MODE_SWITCH", "1")
          .env("ARGVUS_SELECTOR_APPLY", "1")
          .output()?;
        if !output.status.success() {
          return Err(invalid(format!(
            "layout geometry failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
          )));
        }
      }
      Step::Position(position) => {
        if !geometry_changed && before.position != position {
          windows::set_position(position.as_str())?;
        }
      }
      Step::Windows(mode) => window_status = Some(window_mode::set(mode)?),
    }
    Ok(())
  })?;
  let outcome = completion(
    preset,
    LayoutPreset::from_document(&store.load_effective()?)?,
    window_status.ok_or_else(|| invalid("window application status unavailable"))?,
  )?;
  if outcome == ApplyOutcome::Applied {
    fs::remove_file(retry_marker)?;
  }
  Ok(outcome)
}

fn completion(
  expected: LayoutPreset,
  saved: LayoutPreset,
  status: window_mode::Status,
) -> ConfigResult<ApplyOutcome> {
  if saved != expected {
    return Err(invalid(
      "layout preference changed during application; read current state and retry",
    ));
  }
  if status.available
    && let Some(error) = status.error
  {
    return Err(invalid(format!(
      "layout saved, window application failed: {error}"
    )));
  }
  if !status.available || status.pending || status.applied != Some(expected.windows) {
    Ok(ApplyOutcome::Pending)
  } else {
    Ok(ApplyOutcome::Applied)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn all_combinations_are_unique_and_round_trip() {
    let mut seen = Vec::new();
    for (index, preset) in LayoutPreset::ALL.into_iter().enumerate() {
      assert!(!seen.contains(&preset));
      seen.push(preset);
      assert_eq!(preset.index(), index);
      let mut document = ConfigDocument::default();
      document
        .set("/layout/variant", preset.variant.as_str().into())
        .unwrap();
      document
        .set("/hyprland/window_mode", preset.windows.as_str().into())
        .unwrap();
      document
        .set("/layout/taskbar/position", preset.position.as_str().into())
        .unwrap();
      assert_eq!(LayoutPreset::from_document(&document).unwrap(), preset);
      if preset.variant == Variant::Spaced {
        document.sections.get_mut("layout").unwrap()["variant"] = "float".into();
        assert_eq!(LayoutPreset::from_document(&document).unwrap(), preset);
      }
    }
  }
  #[test]
  fn geometry_precedes_position_and_window_application_for_every_preset() {
    for preset in LayoutPreset::ALL {
      let mut steps = Vec::new();
      apply_steps(preset, |step| {
        steps.push(step);
        Ok(())
      })
      .unwrap();
      assert_eq!(
        steps,
        [
          Step::Geometry(preset.variant),
          Step::Position(preset.position),
          Step::Windows(preset.windows)
        ]
      );
    }
  }
  #[test]
  fn failures_stop_before_later_steps() {
    for failed in 0..3 {
      let mut count = 0;
      assert!(
        apply_steps(LayoutPreset::ALL[7], |_| {
          let current = count;
          count += 1;
          if current == failed {
            Err(invalid("simulated failure"))
          } else {
            Ok(())
          }
        })
        .is_err()
      );
      assert_eq!(count, failed + 1);
    }
  }
  #[test]
  fn saved_preferences_do_not_imply_runtime_success() {
    for preset in LayoutPreset::ALL {
      let mut status = window_mode::Status {
        preferred: preset.windows,
        applied: None,
        available: false,
        pending: true,
        error: None,
      };
      assert_eq!(
        completion(preset, preset, status.clone()).unwrap(),
        ApplyOutcome::Pending
      );
      status.available = true;
      status.applied = Some(preset.windows);
      status.pending = false;
      assert_eq!(
        completion(preset, preset, status.clone()).unwrap(),
        ApplyOutcome::Applied
      );
      status.error = Some("IPC failure".into());
      assert!(completion(preset, preset, status.clone()).is_err());
      assert!(completion(preset, LayoutPreset::ALL[(preset.index() + 1) % 8], status).is_err());
    }
  }
}
