//! Adapter to the compositor-owned workspace navigation policy.
use super::*;

fn request(arguments: &[String], root: &Path) -> ConfigResult<String> {
  let (direction, source) = match arguments {
    [direction] => (direction.as_str(), "focused"),
    [direction, flag, source] if flag == "--monitor" => (direction.as_str(), source.as_str()),
    _ => {
      return Err(ConfigError::Invalid(
        "usage: argvus config workspace-cycle next|previous [--monitor focused|cursor]".into(),
      ));
    }
  };
  let offset = match direction {
    "next" => 1,
    "previous" => -1,
    _ => return Err(ConfigError::Invalid("invalid workspace direction".into())),
  };
  if !matches!(source, "focused" | "cursor") {
    return Err(ConfigError::Invalid(
      "invalid workspace monitor source".into(),
    ));
  }
  let path = serde_json::to_string(&root.join("hyprland/lua/workspaces.lua").to_string_lossy())?;
  Ok(format!(
    "local ok, policy = pcall(dofile, {path}); if not ok then error(policy) end; policy.cycle({offset}, '{source}')"
  ))
}

pub(super) fn run(arguments: &[String]) -> ExitCode {
  let root = env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(session_paths::PACKAGED_SYSTEM_CONFIG));
  match request(arguments, &root).and_then(|lua| crate::hyprland_ipc::eval(&lua)) {
    Ok(_) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-config: {error}");
      ExitCode::from(1)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
  }
  #[test]
  fn defaults_to_focused_monitor() {
    let lua = request(&args(&["next"]), Path::new("/usr/share/argvus")).unwrap();
    assert!(lua.ends_with("policy.cycle(1, 'focused')"));
  }
  #[test]
  fn accepts_previous_at_cursor_and_escapes_paths() {
    let lua = request(
      &args(&["previous", "--monitor", "cursor"]),
      Path::new("/tmp/a\"b"),
    )
    .unwrap();
    assert!(lua.ends_with("policy.cycle(-1, 'cursor')"));
    assert!(lua.contains("a\\\"b"));
  }
  #[test]
  fn rejects_invalid_requests_before_ipc() {
    for values in [
      vec![],
      vec!["other"],
      vec!["next", "--monitor", "bad"],
      vec!["next", "extra"],
    ] {
      assert!(request(&args(&values), Path::new("/usr/share/argvus")).is_err());
    }
  }
}
