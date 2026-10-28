//! Running the helper scripts, `argvus` routes and the effects command, and reading small state files.

use super::*;

pub(super) fn script(name: &str) -> PathBuf {
  let project = match name {
    "hypr-wallpaper-pick.sh"
    | "taskbar-right-2-mode.sh"
    | "taskbar-widgets-mode.sh"
    | "brightness-switch.sh"
    | "layout-mode-switch.sh"
    | "icon-switch.sh" => "appearance",
    "hyprlock-theme.sh" => "lock",
    "spaces-switch.sh" | "borders-switch.sh" => "hyprland",
    _ => "session",
  };
  system_config_root().join(project).join("sh").join(name)
}

/// Returns the Control Panel-owned preference helper without duplicating its
/// persistence rules in the Control Center.
pub(super) fn control_panel_cards_script() -> PathBuf {
  system_config_root()
    .join("control-panel")
    .join("sh")
    .join("cards-config.sh")
}

pub(super) fn run_script(script_path: &Path, args: &[&str]) -> Result<(), String> {
  run_script_with_env(script_path, args, &[])
}

pub(super) fn run_script_with_env(
  script_path: &Path,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  if !script_path.is_file() {
    return Err(format!("script não encontrado: {}", script_path.display()));
  }
  let request = ProcessRequest::new("sh").arg(script_path.to_string_lossy().to_string());
  run_process(
    request,
    &script_path.display().to_string(),
    args,
    environment,
  )
}

/// Runs an `argvus <route>` command: the same entry point the keybindings and
/// the menus use, so the route alone decides the implementation.
pub(crate) fn run_argvus_route(
  route: &str,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  let request = ProcessRequest::new("argvus").arg(route);
  run_process(request, &format!("argvus {route}"), args, environment)
}

/// Runs `argvus effects-toggle` with the arguments: the visual effects route.
pub(super) fn run_effects(arguments: &[&str]) -> Result<(), String> {
  run_argvus_route("effects-toggle", arguments, &[])
}

/// Reads from `argvus effects-toggle`, falling back to `None` when it fails.
pub(super) fn run_effects_output(arguments: &[&str]) -> Option<String> {
  run_argvus_route_output("effects-toggle", arguments)
}

/// Validates the arguments, applies the environment, runs the command and
/// reports its stderr when it fails.
pub(super) fn run_process(
  mut request: ProcessRequest,
  label: &str,
  args: &[&str],
  environment: &[(&str, &str)],
) -> Result<(), String> {
  for argument in args {
    if argument.is_empty()
      || argument
        .chars()
        .any(|character| character.is_control() || character == '\n')
    {
      return Err("invalid script argument".into());
    }
    request = request.arg(*argument);
  }
  for (name, value) in environment {
    request = request.env(*name, *value);
  }
  let output = SystemProcessRunner
    .run(&request)
    .map_err(|error| error.to_string())?;
  if output.status.is_none_or(|status| status != 0) {
    let stderr = terminal_text(&String::from_utf8_lossy(&output.stderr))
      .trim()
      .to_string();
    return Err(if stderr.is_empty() {
      format!("{label} falhou")
    } else {
      stderr
    });
  }
  Ok(())
}

pub(super) fn run_script_output(script_path: &Path, args: &[&str]) -> Option<String> {
  if !script_path.is_file() {
    return None;
  }
  run_output(
    ProcessRequest::new("sh").arg(script_path.to_string_lossy().to_string()),
    args,
  )
}

/// Runs an `argvus <route>` command and returns its stdout, like `run_script_output`.
pub(super) fn run_argvus_route_output(route: &str, args: &[&str]) -> Option<String> {
  run_output(ProcessRequest::new("argvus").arg(route), args)
}

/// Runs the prepared request with validated arguments. A failed run or an
/// invalid argument gives `None`, so readers fall back to their defaults.
pub(super) fn run_output(mut request: ProcessRequest, args: &[&str]) -> Option<String> {
  for argument in args {
    if argument.is_empty()
      || argument
        .chars()
        .any(|character| character.is_control() || character == '\n')
    {
      return None;
    }
    request = request.arg(*argument);
  }
  let output = SystemProcessRunner
    .run(&request.timeout(Duration::from_secs(5)))
    .ok()?;
  if output.status.is_none_or(|status| status != 0) {
    return None;
  }
  Some(terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

pub(super) fn read_first(path: &Path, fallback: &str) -> String {
  fs::read_to_string(path)
    .ok()
    .and_then(|content| content.lines().next().map(str::to_string))
    .filter(|line| !line.trim().is_empty())
    .unwrap_or_else(|| fallback.to_string())
}

pub(super) fn read_first_or_default(path: &Path) -> Option<String> {
  fs::read_to_string(path)
    .ok()
    .and_then(|content| content.lines().next().map(str::to_string))
    .map(|line| line.trim().to_string())
}
