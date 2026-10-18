//! The facts of the system block: the operating system, the session, the window
//! manager, the display, the CPU and the GPU. The rules that read and shape them
//! live here; the binary runs the commands and reads the files.

/// The `PRETTY_NAME` of `/etc/os-release`, without its quotes. Empty when absent.
pub fn pretty_name(os_release: &str) -> String {
  os_release
    .lines()
    .find_map(|line| line.strip_prefix("PRETTY_NAME="))
    .unwrap_or_default()
    .replace('"', "")
}

/// The session type named by the environment: `Wayland` or `X11`, or `None`.
pub fn session_type(
  session_type: Option<&str>,
  wayland_display: bool,
  display: bool,
) -> Option<&'static str> {
  match session_type {
    Some("wayland") => Some("Wayland"),
    Some("x11") => Some("X11"),
    _ if wayland_display => Some("Wayland"),
    _ if display => Some("X11"),
    _ => None,
  }
}

/// The window manager line: `NAME VERSION (SESSION)`, or `NAME (SESSION)` when
/// the version is not known.
pub fn window_manager(name: &str, version: Option<&str>, session: &str) -> String {
  match version.filter(|version| !version.is_empty()) {
    Some(version) => format!("{name} {version} ({session})"),
    None => format!("{name} ({session})"),
  }
}

/// The active mode and physical size of a monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
  pub width: u32,
  pub height: u32,
  pub refresh_hz: u32,
  pub physical_width_mm: u32,
  pub physical_height_mm: u32,
}

/// The first monitor of `hyprctl monitors -j`.
pub fn first_monitor(json: &str) -> Option<Monitor> {
  let monitors: serde_json::Value = serde_json::from_str(json).ok()?;
  let monitor = monitors.as_array()?.first()?;
  let number = |key: &str| monitor.get(key).and_then(serde_json::Value::as_f64);
  Some(Monitor {
    width: number("width")? as u32,
    height: number("height")? as u32,
    refresh_hz: number("refreshRate")?.round_ties_even() as u32,
    physical_width_mm: number("physicalWidth").unwrap_or(0.0) as u32,
    physical_height_mm: number("physicalHeight").unwrap_or(0.0) as u32,
  })
}

/// The diagonal of a display in inches, rounded. Without a physical size there
/// is no diagonal.
pub fn diagonal_inches(width_mm: u32, height_mm: u32) -> Option<u32> {
  if width_mm == 0 || height_mm == 0 {
    return None;
  }
  let (width, height) = (f64::from(width_mm), f64::from(height_mm));
  Some(((width * width + height * height).sqrt() / 25.4).round_ties_even() as u32)
}

/// A display of 21 inches or more counts as external; smaller ones as laptop.
pub fn is_external(inches: u32) -> bool {
  inches >= 21
}

/// The CPU model name from `lscpu`, without the trademark marks and the word
/// `Core`, with single spaces.
pub fn cpu_model(lscpu: &str) -> String {
  let model = lscpu
    .lines()
    .find(|line| line.contains("Model name"))
    .and_then(|line| line.split_once(':'))
    .map(|(_, value)| value.trim())
    .unwrap_or_default();
  let cleaned = model
    .replace("(R)", "")
    .replace("(TM)", "")
    .replace("Core", "");
  cleaned
    .split(' ')
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

/// The GPU name from `lspci`: the bracketed name of the first display controller
/// when it has one, otherwise its whole description.
pub fn gpu_name(lspci: &str) -> String {
  let description = lspci
    .lines()
    .find(|line| line.contains("VGA compatible controller") || line.contains("3D controller"))
    .and_then(|line| line.split(": ").nth(1))
    .unwrap_or_default();
  match (description.rfind('['), description.rfind(']')) {
    (Some(start), Some(end)) if start < end => description[start + 1..end].to_string(),
    _ => description.to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::{
    Monitor, cpu_model, diagonal_inches, first_monitor, gpu_name, pretty_name, session_type,
    window_manager,
  };

  #[test]
  fn os_name_drops_the_quotes() {
    let os_release = "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\n";
    assert_eq!(pretty_name(os_release), "Arch Linux");
    assert_eq!(pretty_name("NAME=x\n"), "");
  }

  #[test]
  fn session_prefers_the_declared_type() {
    assert_eq!(session_type(Some("x11"), true, false), Some("X11"));
    assert_eq!(session_type(None, true, true), Some("Wayland"));
    assert_eq!(session_type(None, false, true), Some("X11"));
    assert_eq!(session_type(None, false, false), None);
    assert_eq!(
      window_manager("Hyprland", Some("0.56.2"), "Wayland"),
      "Hyprland 0.56.2 (Wayland)"
    );
    assert_eq!(
      window_manager("Hyprland", None, "Wayland"),
      "Hyprland (Wayland)"
    );
  }

  #[test]
  fn monitor_json_gives_the_first_monitor() {
    let json = r#"[{"width":1920,"height":1080,"refreshRate":74.973,"physicalWidth":520,"physicalHeight":310,"focused":true}]"#;
    assert_eq!(
      first_monitor(json),
      Some(Monitor {
        width: 1920,
        height: 1080,
        refresh_hz: 75,
        physical_width_mm: 520,
        physical_height_mm: 310,
      })
    );
    assert_eq!(first_monitor("[]"), None);
  }

  #[test]
  fn diagonal_rounds_to_whole_inches() {
    assert_eq!(diagonal_inches(520, 310), Some(24));
    assert_eq!(diagonal_inches(0, 310), None);
  }

  #[test]
  fn cpu_name_is_cleaned() {
    let lscpu =
      "Architecture: x86_64\nModel name:            Intel(R) Core(TM) i7-10750H CPU @ 2.60GHz\n";
    assert_eq!(cpu_model(lscpu), "Intel i7-10750H CPU @ 2.60GHz");
  }

  #[test]
  fn gpu_name_prefers_the_bracketed_name() {
    let lspci = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics\n\
                     01:00.0 3D controller: NVIDIA Corporation GA106M [GeForce RTX 3060 Mobile]\n";
    assert_eq!(gpu_name(lspci), "Intel Corporation UHD Graphics");
    assert_eq!(
      gpu_name("01:00.0 3D controller: NVIDIA Corporation GA106M [GeForce RTX 3060 Mobile]\n"),
      "GeForce RTX 3060 Mobile"
    );
  }
}
