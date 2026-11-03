//! Best-effort placement of the exact window created by a child process.
use std::{
  process::Command,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  thread,
  time::Duration,
};

// Match the floating memory popup created by `argvus taskbar-actions --mem`.
pub(crate) const MEMORY_POPUP_SIZE: (u32, u32) = (1399, 920);

pub fn place(
  pid: u32,
  class: Option<&'static str>,
  preferred_size: Option<(u32, u32)>,
  stop: Arc<AtomicBool>,
) {
  if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
    return;
  }
  thread::spawn(move || {
    for attempt in 0..80 {
      if stop.load(Ordering::Relaxed) {
        return;
      }
      thread::sleep(Duration::from_millis(100));
      let Ok(output) = Command::new("hyprctl").args(["-j", "clients"]).output() else {
        return;
      };
      if !output.status.success() {
        continue;
      }
      // Prefer the new process; only use a unique class after a delay
      // for D-Bus-activated managers whose surface has another PID.
      let fallback = if attempt >= 40 { class } else { None };
      let Some(address) = find_window(&output.stdout, pid, fallback) else {
        continue;
      };
      let expression = format!("hl.dsp.focus({{ window = 'address:{address}' }})");
      if !dispatch(&expression) {
        continue;
      }
      if stop.load(Ordering::Relaxed) {
        return;
      }
      if !dispatch("hl.dsp.window.float({ action = 'enable' })") {
        return;
      }
      if let (Ok(clients), Ok(monitors)) = (
        Command::new("hyprctl").args(["-j", "clients"]).output(),
        Command::new("hyprctl").args(["-j", "monitors"]).output(),
      ) && let Some((width, height)) =
        target_size(&clients.stdout, &monitors.stdout, &address, preferred_size)
      {
        let _ = dispatch(&format!(
          "hl.dsp.window.resize({{ x = {width}, y = {height} }})"
        ));
      }
      for expression in ["hl.dsp.window.center({})", "hl.dsp.window.bring_to_top({})"] {
        if stop.load(Ordering::Relaxed) {
          return;
        }
        let _ = dispatch(expression);
      }
      return;
    }
  });
}
fn dispatch(expression: &str) -> bool {
  Command::new("hyprctl")
    .args(["dispatch", expression])
    .output()
    .is_ok_and(|output| {
      let error = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
      output.status.success() && !error.contains("warning:") && !error.contains("error:")
    })
}
fn target_size(
  clients: &[u8],
  monitors: &[u8],
  address: &str,
  preferred_size: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
  let clients: Vec<serde_json::Value> = serde_json::from_slice(clients).ok()?;
  let monitors: Vec<serde_json::Value> = serde_json::from_slice(monitors).ok()?;
  let client = clients
    .iter()
    .find(|client| client.get("address").and_then(|value| value.as_str()) == Some(address))?;
  let monitor_id = client.get("monitor")?.as_i64()?;
  let monitor = monitors
    .iter()
    .find(|monitor| monitor.get("id").and_then(|id| id.as_i64()) == Some(monitor_id))?;
  let scale = monitor.get("scale")?.as_f64()?;
  if !scale.is_finite() || scale <= 0.0 {
    return None;
  }
  let mut width = monitor.get("width")?.as_f64()?;
  let mut height = monitor.get("height")?.as_f64()?;
  if matches!(
    monitor.get("transform").and_then(|v| v.as_u64()),
    Some(1 | 3 | 5 | 7)
  ) {
    std::mem::swap(&mut width, &mut height);
  }
  let reserved = monitor.get("reserved")?.as_array()?;
  let side = |i: usize| {
    reserved
      .get(i)
      .and_then(|v| v.as_f64())
      .unwrap_or(0.0)
      .max(0.0)
  };
  let available_width = (width / scale - side(1) - side(3)).floor().max(1.0) as u32;
  let available_height = (height / scale - side(0) - side(2)).floor().max(1.0) as u32;
  let Some((preferred_width, preferred_height)) = preferred_size else {
    let size = client.get("size")?.as_array()?;
    let current_width = u32::try_from(size.first()?.as_u64()?).ok()?;
    let current_height = u32::try_from(size.get(1)?.as_u64()?).ok()?;
    if current_width <= available_width && current_height <= available_height {
      return None;
    }
    return Some((
      current_width.min(available_width),
      current_height.min(available_height),
    ));
  };
  let target = (
    preferred_width.min(available_width),
    preferred_height.min(available_height),
  );
  let size = client.get("size")?.as_array()?;
  let current = (
    u32::try_from(size.first()?.as_u64()?).ok()?,
    u32::try_from(size.get(1)?.as_u64()?).ok()?,
  );
  if current == target {
    return None;
  }
  Some(target)
}
fn find_window(data: &[u8], pid: u32, class: Option<&str>) -> Option<String> {
  let clients: Vec<serde_json::Value> = serde_json::from_slice(data).ok()?;
  let address = |client: &serde_json::Value| {
    let value = client.get("address")?.as_str()?;
    let hex = value.strip_prefix("0x")?;
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
      return None;
    }
    Some(value.to_owned())
  };
  if let Some(client) = clients
    .iter()
    .find(|client| client.get("pid").and_then(|v| v.as_u64()) == Some(u64::from(pid)))
  {
    return address(client);
  }
  let class = class?;
  let mut matches = clients
    .iter()
    .filter(|client| client.get("class").and_then(|v| v.as_str()) == Some(class));
  let found = matches.next()?;
  if matches.next().is_some() {
    return None;
  }
  address(found)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn pid_takes_precedence_over_class_and_multiple_instances() {
    let clients = br#"[{"pid":9,"class":"argvus-du","address":"0x123"},{"pid":10,"class":"argvus-du","address":"0x456"},{"pid":11,"class":"argvus-du-picker","address":"0x789"}]"#;
    assert_eq!(
      find_window(clients, 10, Some("argvus-du")).as_deref(),
      Some("0x456")
    );
    assert_eq!(find_window(clients, 88, Some("argvus-du")), None);
    assert_eq!(
      find_window(clients, 88, Some("argvus-du-picker")).as_deref(),
      Some("0x789")
    );
  }
  #[test]
  fn clamps_to_scaled_monitor_area_only_when_necessary() {
    let clients = br#"[{"address":"0x123","monitor":3,"size":[1300,900]}]"#;
    let monitors =
      br#"[{"id":3,"width":1920,"height":1080,"scale":2,"transform":0,"reserved":[20,10,30,10]}]"#;
    assert_eq!(
      target_size(clients, monitors, "0x123", None),
      Some((940, 490))
    );
    let rotated =
      br#"[{"id":3,"width":1920,"height":1080,"scale":2,"transform":1,"reserved":[0,0,0,0]}]"#;
    assert_eq!(
      target_size(clients, rotated, "0x123", None),
      Some((540, 900))
    );
    assert_eq!(target_size(clients, monitors, "0x999", None), None);
    let small = br#"[{"address":"0x123","monitor":3,"size":[500,400]}]"#;
    assert_eq!(target_size(small, monitors, "0x123", None), None);
  }
  #[test]
  fn applies_memory_popup_size_and_clamps_it_to_available_monitor_area() {
    let clients = br#"[{"address":"0x123","monitor":3,"size":[800,600]}]"#;
    let large_monitor =
      br#"[{"id":3,"width":2560,"height":1440,"scale":1,"transform":0,"reserved":[0,0,0,0]}]"#;
    assert_eq!(
      target_size(clients, large_monitor, "0x123", Some(MEMORY_POPUP_SIZE)),
      Some((1399, 920))
    );
    let small_monitor =
      br#"[{"id":3,"width":1920,"height":1080,"scale":2,"transform":0,"reserved":[20,10,30,10]}]"#;
    assert_eq!(
      target_size(clients, small_monitor, "0x123", Some(MEMORY_POPUP_SIZE)),
      Some((940, 490))
    );
  }
  #[test]
  fn rejects_invalid_addresses() {
    assert_eq!(
      find_window(br#"[{"pid":1,"address":"0x1'bad"}]"#, 1, None),
      None
    );
  }
}
