//! Clipboard and focus-aware keyboard delivery.
use std::{
  env, fs, io,
  path::PathBuf,
  process::Command,
  thread,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

fn query(command: &str) -> Option<serde_json::Value> {
  let output = Command::new("hyprctl")
    .args(["-j", command])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  serde_json::from_slice(&output.stdout).ok()
}

pub fn active_window() -> Option<String> {
  query("activewindow")?["address"]
    .as_str()
    .filter(|address| !address.is_empty())
    .map(str::to_owned)
}

pub fn picker_size() -> (i32, i32) {
  const DEFAULT: (i32, i32) = (1120, 590);
  let Some(monitors) = query("monitors") else {
    return DEFAULT;
  };
  let Some(monitor) = monitors.as_array().and_then(|items| {
    items
      .iter()
      .find(|item| item["focused"].as_bool() == Some(true))
  }) else {
    return DEFAULT;
  };
  let scale = monitor["scale"].as_f64().unwrap_or(1.0);
  let width = monitor["width"].as_f64().unwrap_or(f64::from(DEFAULT.0));
  let height = monitor["height"].as_f64().unwrap_or(f64::from(DEFAULT.1));
  let rotated = monitor["transform"].as_u64().unwrap_or_default() % 2 == 1;
  let (width, height) = if rotated {
    (height, width)
  } else {
    (width, height)
  };
  let available = |pixels: f64, default: i32| ((pixels * 0.92 / scale) as i32).clamp(1, default);
  (available(width, DEFAULT.0), available(height, DEFAULT.1))
}

pub fn deliver(sequence: &str, insert: bool, target: Option<&str>) -> Result<(), &'static str> {
  if !insert {
    return run("wl-copy", sequence).map_err(|_| "copy_error");
  }
  let target = target.ok_or("focus_error")?;
  let mut focused = false;
  for _ in 0..12 {
    if active_window().as_deref() == Some(target) {
      focused = true;
      break;
    }
    thread::sleep(Duration::from_millis(25));
  }
  if !focused {
    return Err("focus_error");
  }
  let stamp = suppression_stamp();
  let result = if active_window().as_deref() == Some(target) {
    crate::clipboard::paste(sequence, || active_window().as_deref() == Some(target))
  } else {
    Err("focus_error")
  };
  if let Some((path, value)) = stamp {
    thread::sleep(Duration::from_secs(1));
    if fs::read_to_string(&path).as_deref().ok() == Some(&value) {
      let _ = fs::remove_file(path);
    }
  }
  result
}

fn run(program: &str, sequence: &str) -> io::Result<()> {
  let status = Command::new(program).args(["--", sequence]).status()?;
  if status.success() {
    Ok(())
  } else {
    Err(io::Error::other(status.to_string()))
  }
}

fn suppression_stamp() -> Option<(PathBuf, String)> {
  let cache = env::var_os("ARGVUS_CACHE_HOME")
    .map(PathBuf::from)
    .or_else(|| {
      let base = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|home| home.join(".cache")))?;
      Some(base.join("argvus"))
    })?;
  let directory = cache.join("hypr");
  fs::create_dir_all(&directory).ok()?;
  let path = directory.join("keyboard-layout-notify.suppress");
  let value = format!(
    "{}\n",
    SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs()
  );
  fs::write(&path, &value).ok()?;
  Some((path, value))
}
