//! External theme and effect adapters run after a projection, and the plan of which ones a change needs.

use super::*;

/// Packaged defaults root. `ARGVUS_SYSTEM_CONFIG` lets tests and DESTDIR
/// checks point the projection at another tree.
pub(super) fn system_config_root() -> PathBuf {
  env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(session_paths::PACKAGED_SYSTEM_CONFIG))
}

/// Runs the component adapters and reports whether any of them ran.
pub(super) fn run_component_adapters(
  document: &ConfigDocument,
  paths: &ArgvusPaths,
  changed_sections: &[&str],
  appearance_visual_changed: bool,
) -> ConfigResult<bool> {
  let theme = crate::accent::effective_theme(
    document
      .get(json_pointers::APPEARANCE_THEME)
      .and_then(Value::as_str)
      .unwrap_or("argvus-dark"),
  );
  let theme = theme.as_str();
  // A running switch applies the theme itself and holds the theme lock. A
  // projection started by that switch (through its configuration reload) must
  // not start another switch, which would wait on that lock. The environment
  // variable covers callers that set it; the marker covers the reload, because
  // systemd does not pass the caller's environment to it.
  let theme_transaction = env::var("ARGVUS_THEME_SWITCH").ok().as_deref() == Some("1")
    || crate::theme_apply::transaction_in_progress();
  let plan = plan_adapters(&AdapterInputs {
    changed_sections,
    appearance_visual_changed,
    theme_transaction,
    theme_materialized: theme_outputs_present(paths, theme),
    effects_materialized: effects_outputs_present(paths, theme),
  });
  if plan.theme_switch {
    timed("project.theme-switch", || run_theme_switch(theme, paths))?;
  }
  if plan.effects_toggle {
    timed("project.effects-toggle", || {
      run_adapter_command(
        Command::new("argvus").arg("effects-toggle").arg("apply"),
        Path::new("argvus effects-toggle"),
        paths,
      )
    })?;
  }
  Ok(plan.theme_switch || plan.effects_toggle)
}

/// What decides whether the theme and effects adapters run in a projection.
pub(super) struct AdapterInputs<'a> {
  pub(super) changed_sections: &'a [&'a str],
  pub(super) appearance_visual_changed: bool,
  /// A theme switch is already running and owns the theme lock.
  pub(super) theme_transaction: bool,
  /// The files the theme switch writes for the active theme are in place.
  pub(super) theme_materialized: bool,
  /// The effects file of the active theme is in place.
  pub(super) effects_materialized: bool,
}

/// The adapters a projection runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AdapterPlan {
  pub(super) theme_switch: bool,
  pub(super) effects_toggle: bool,
}

/// Decides which adapters run. A change of their own section always runs
/// them. With nothing changed, they run only to repair missing outputs: a
/// stable login must not repeat the theme switch, which costs seconds.
pub(super) fn plan_adapters(inputs: &AdapterInputs<'_>) -> AdapterPlan {
  if inputs.theme_transaction {
    return AdapterPlan {
      theme_switch: false,
      effects_toggle: false,
    };
  }
  let repairs_only = inputs.changed_sections.is_empty();
  AdapterPlan {
    theme_switch: inputs.appearance_visual_changed || (repairs_only && !inputs.theme_materialized),
    effects_toggle: inputs.changed_sections.contains(&"effects")
      || (repairs_only && !inputs.effects_materialized),
  }
}

/// Whether the user copies that the theme switch materializes exist. It reuses
/// the session preparation check, built on this store's directories, so both
/// callers agree on what "applied" means. Without a resolvable environment
/// the outputs are treated as missing, which keeps the safe behavior.
pub(super) fn theme_outputs_present(paths: &ArgvusPaths, theme: &str) -> bool {
  projection_session_paths(paths)
    .is_some_and(|session_paths| crate::session_prepare::theme_materialized(&session_paths, theme))
}

/// Whether the effects file of the theme is in place. The file is named by the
/// theme the switch renders (`<id>-float` on Float), so the same identity as
/// the theme check is used; a bare id would never match and the adapter would
/// repeat on every login.
pub(super) fn effects_outputs_present(paths: &ArgvusPaths, theme: &str) -> bool {
  let Some(session_paths) = projection_session_paths(paths) else {
    return false;
  };
  let Some(built_theme) = crate::session_prepare::built_theme(&session_paths, theme) else {
    return false;
  };
  crate::effects::theme_effects_path(
    &crate::effects::effects_state_home(&paths.data),
    &built_theme,
  )
  .is_file()
}

/// The session paths of this store's directories, as the session preparation
/// reads them. `None` without a resolvable environment, which the callers treat
/// as missing outputs.
fn projection_session_paths(paths: &ArgvusPaths) -> Option<crate::session_paths::SessionPaths> {
  let config_home = paths.root.parent()?;
  let environment_paths = crate::session_paths::SessionPaths::from_environment()?;
  Some(crate::session_paths::SessionPaths {
    config_home: config_home.to_path_buf(),
    data_home: paths.data.clone(),
    system_config: environment_paths.system_config,
  })
}

pub(super) fn run_adapter(
  path: &Path,
  arguments: &[&str],
  paths: &ArgvusPaths,
) -> ConfigResult<()> {
  if !path.is_file() {
    return Ok(());
  }
  run_adapter_command(Command::new("sh").arg(path).args(arguments), path, paths)
}

/// Runs the theme switch through the `argvus theme-switch` route, which is
/// argvus-config's `theme-apply`.
pub(super) fn run_theme_switch(theme: &str, paths: &ArgvusPaths) -> ConfigResult<()> {
  run_adapter_command(
    Command::new("argvus").arg("theme-switch").arg(theme),
    Path::new("argvus theme-switch"),
    paths,
  )
}

pub(super) fn run_adapter_command(
  command: &mut Command,
  path: &Path,
  paths: &ArgvusPaths,
) -> ConfigResult<()> {
  let status = command
    .env("ARGVUS_NO_RUNTIME", "1")
    .env("ARGVUS_PROJECTING", "1")
    .env(
      "ARGVUS_CONFIG_HOME",
      paths.root.parent().unwrap_or(Path::new("")),
    )
    .status()?;
  if status.success() {
    Ok(())
  } else {
    Err(ConfigError::Invalid(format!(
      "projection adapter {} exited with {status}",
      path.display()
    )))
  }
}

pub(super) fn remove_legacy_markers(root: &Path) -> ConfigResult<()> {
  // These files are still consumed by the Lua compositor and shell
  // adapters. They are derived compatibility projections, not competing
  // sources of truth, so do not delete them until all consumers read data/.
  let path = root.join("data/.accent-custom");
  if path.exists() {
    fs::remove_file(path)?;
  }
  Ok(())
}
