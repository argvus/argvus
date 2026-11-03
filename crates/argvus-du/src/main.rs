mod i18n;
mod superfile;
mod theme;
mod ui;
mod window;
use std::{
  env,
  process::{Command, ExitCode},
};

fn main() -> ExitCode {
  let args: Vec<_> = env::args_os().collect();
  let result = match args.get(1).and_then(|arg| arg.to_str()) {
    Some("--ui") => ui::run(),
    Some("--directory-picker") if args.len() >= 4 => {
      ui::run_directory_picker(&args[2], &args[3], &args[4..])
    }
    _ => launch(),
  };
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("argvus-du: {error}");
      if env::var_os("WAYLAND_DISPLAY").is_some() || env::var_os("DISPLAY").is_some() {
        let _ = Command::new("notify-send")
          .args(["ARGVUS Disk Usage Analyzer", &error])
          .status();
      }
      ExitCode::FAILURE
    }
  }
}
fn launch() -> Result<(), String> {
  let executable = env::current_exe().map_err(|e| e.to_string())?;
  let messages = i18n::Messages::load();
  let title = format!("ARGVUS {}", messages.get("title"));
  let mut command = Command::new("kitty");
  // This dedicated themed interface needs colors even if the session exports
  // NO_COLOR for ordinary command-line output. The parent environment is untouched.
  command.env_remove("NO_COLOR");
  let user_config = env::var_os("XDG_CONFIG_HOME")
    .map(std::path::PathBuf::from)
    .or_else(|| env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
    .map(|p| p.join("kitty/kitty.conf"));
  if let Some(path) = user_config.filter(|p| p.is_file()) {
    command.arg("--config").arg(path);
  }
  // Use the same generated ARGVUS terminal profile as argvus-ai. The
  // dedicated policy below still has the final word on window behaviour.
  if let Ok(output) = Command::new("argvus")
    .args(["terminal", "--print-tui-config"])
    .output()
    && output.status.success()
    && let Ok(path) = String::from_utf8(output.stdout)
    && std::path::Path::new(path.trim()).is_file()
  {
    command.arg("--config").arg(path.trim());
  }
  let config = env::var_os("ARGVUS_DU_KITTY_CONFIG")
    .unwrap_or_else(|| "/usr/share/argvus/du/kitty.conf".into());
  let mut child = command
    .arg("--config")
    .arg(config)
    .args(["--class", "argvus-du", "--title"])
    .arg(title)
    .arg(executable)
    .arg("--ui")
    .spawn()
    .map_err(|e| format!("{}: {e}", messages.get("missing_kitty")))?;
  let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
  window::place(
    child.id(),
    None,
    Some(window::MEMORY_POPUP_SIZE),
    stop.clone(),
  );
  let status = child.wait().map_err(|e| e.to_string());
  stop.store(true, std::sync::atomic::Ordering::Relaxed);
  let status = status?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("{}: {status}", messages.get("kitty_exited")))
  }
}
