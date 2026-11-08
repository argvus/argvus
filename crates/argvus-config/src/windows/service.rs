use super::model::{self, Registry};
use super::*;
use std::fs::{self, OpenOptions};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::time::Instant;

const PARKING: &str = "special:argvus-minimized";

fn query(name: &str) -> ConfigResult<Value> {
  Ok(serde_json::from_str(&hyprland_ipc::send(&format!(
    "j/{name}"
  ))?)?)
}
fn bar_mapped() -> ConfigResult<bool> {
  bar_mapped_on(None)
}

fn bar_mapped_on(name: Option<&str>) -> ConfigResult<bool> {
  let layers = query("layers")?;
  Ok(layers.as_object().is_some_and(|outputs| {
    outputs
      .iter()
      .filter(|(output, _)| name.is_none_or(|name| name == output.as_str()))
      .any(|(_, output)| {
        output["levels"].as_object().is_some_and(|levels| {
          levels.values().any(|entries| {
            entries.as_array().is_some_and(|entries| {
              entries
                .iter()
                .any(|entry| entry["namespace"] == "argvus-window-bar")
            })
          })
        })
      })
  }))
}

fn clients() -> ConfigResult<Vec<Value>> {
  query("clients")?
    .as_array()
    .cloned()
    .ok_or_else(|| invalid("invalid client list"))
}
fn monitors() -> ConfigResult<Vec<Value>> {
  query("monitors")?
    .as_array()
    .cloned()
    .ok_or_else(|| invalid("invalid monitor list"))
}

/// Byte escapes also handle non-ASCII workspace names and control characters;
/// JSON string escapes are not interchangeable with Lua string escapes.
fn quote(text: &str) -> String {
  let mut quoted = String::from("\"");
  for byte in text.bytes() {
    quoted.push_str(&format!("\\{byte:03}"));
  }
  quoted.push('"');
  quoted
}

fn on_window(target: &str, statements: &str) -> ConfigResult<()> {
  address(target)?;
  // A restore may focus implicitly while moving/fullscreening a window, before
  // the final explicit focus. Suppress warps for every targeted operation and
  // restore the user's option even when a dispatcher fails.
  hyprland_ipc::eval(&format!(
    "local w=assert(hl.get_window({}), 'window closed'); local previous=hl.get_config('cursor.no_warps'); hl.config({{cursor={{no_warps=true}}}}); local ok,err=pcall(function() {statements} end); hl.config({{cursor={{no_warps=previous}}}}); if not ok then error(err) end",
    quote(&format!("address:{target}"))
  ))?;
  Ok(())
}

struct Service {
  registry: Registry,
  maximized: Registry,
  watchers: Vec<UnixStream>,
  ready: Option<Instant>,
  last_snapshot: String,
  labels: Value,
  suspended: bool,
}

impl Service {
  fn reconcile(&mut self) -> ConfigResult<Vec<Value>> {
    let windows = clients()?;
    let previous_maximized = self.maximized.len();
    self.maximized.retain(|target, origin| {
      windows
        .iter()
        .any(|window| window["address"] == *target && model::same_window(origin, window))
    });
    if self.maximized.len() != previous_maximized {
      model::save_named(&self.maximized, "maximized.json")?;
    }
    let previous = self.registry.len();
    self.registry.retain(|target, origin| {
      windows
        .iter()
        .any(|window| window["address"] == *target && model::same_window(origin, window))
    });
    if self.registry.len() != previous {
      model::save(&self.registry)?;
    }
    Ok(windows)
  }

  fn snapshot(&mut self) -> ConfigResult<Value> {
    let mut windows = self.reconcile()?;
    let outputs = monitors()?;
    let document = crate::ConfigStore::from_environment()?.load_effective()?;
    if document.get("/layout/variant").and_then(Value::as_str) == Some("spaced") && mode_applied() {
      let changed = self.apply_spacing(&windows, &outputs)?;
      if changed {
        windows = clients()?;
      }
    }
    let active = query("activewindow")?["address"].clone();
    let entries:Vec<_>=windows.iter().filter(|window|window["mapped"]==true).filter_map(|window| {
      let target=window["address"].as_str().unwrap_or("");let origin=self.registry.get(target);
      let output=origin.map(|origin|origin.monitor.clone()).filter(|name|outputs.iter().any(|output|output["name"]==*name))
        .or_else(||outputs.iter().find(|output|output["id"]==window["monitor"]).and_then(|output|output["name"].as_str()).map(str::to_owned))
        .unwrap_or_else(||outputs.first().and_then(|output|output["name"].as_str()).unwrap_or("").into());
      let monitor = outputs.iter().find(|monitor| monitor["name"] == output)?;
      if !model::listed(window, origin, monitor) { return None; }
      Some(json!({"address":target,"title":window["title"],"app_id":window["class"],"output":output,
        "workspace":origin.map(|origin|origin.workspace.clone()).unwrap_or_else(||model::workspace(window)),
        "active":window["address"]==active,"minimized":origin.is_some(),"maximized":window["fullscreen"]==1 || self.maximized.contains_key(target)}))
    }).collect();
    let position = document
      .get("/layout/taskbar/position")
      .and_then(Value::as_str)
      .unwrap_or("top");
    Ok(
      json!({"ready":self.ready.is_some_and(|ready|ready.elapsed()<Duration::from_secs(5)) && bar_mapped()?,"labels":self.labels,"floating":mode_applied(),"recovering":!self.registry.is_empty(),"position":position,"windows":entries}),
    )
  }

  fn spacing() -> ConfigResult<([i64; 4], i64, i64)> {
    let gaps = query("getoption general:gaps_out")?;
    let mut gaps = layout::gaps(
      gaps["css"]
        .as_str()
        .ok_or_else(|| invalid("compositor gaps unavailable"))?,
    )?;
    if bar_mapped()? {
      match fs::read(runtime_path("bar.json")?) {
        Ok(bytes) => gaps = layout::compensate_secondary(gaps, &serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
      }
    }
    let border = query("getoption general:border_size")?["int"]
      .as_i64()
      .unwrap_or(1)
      .max(0);
    let title = query("getoption plugin:hyprbars:bar_height")?["int"]
      .as_i64()
      .unwrap_or(28)
      .max(0);
    Ok((gaps, border, title))
  }

  fn set_geometry(target: &str, at: [i64; 2], size: [i64; 2]) -> ConfigResult<()> {
    on_window(
      target,
      &format!(
        "hl.dispatch(hl.dsp.window.resize({{x={},y={},window=w}})); hl.dispatch(hl.dsp.window.move({{x={},y={},relative=false,window=w}}))",
        size[0], size[1], at[0], at[1]
      ),
    )
  }

  fn apply_spacing(&self, windows: &[Value], outputs: &[Value]) -> ConfigResult<bool> {
    let (gaps, border, title) = Self::spacing()?;
    let mut changed = false;
    for window in windows {
      let Some(target) = window["address"].as_str() else {
        continue;
      };
      if window["mapped"] != true
        || window["floating"] != true
        || window["hidden"] == true
        || window["fullscreen"].as_i64().unwrap_or(0) != 0
        || self.registry.contains_key(target)
      {
        continue;
      }
      let Some(output) = outputs
        .iter()
        .find(|output| output["id"] == window["monitor"])
      else {
        continue;
      };
      let (at, size) = layout::geometry(
        window,
        output,
        gaps,
        border,
        title,
        self.maximized.contains_key(target),
      );
      if window["at"] != json!(at) || window["size"] != json!(size) {
        Self::set_geometry(target, at, size)?;
        changed = true;
      }
    }
    Ok(changed)
  }

  fn maximize(&mut self, target: &str) -> ConfigResult<()> {
    if let Some(origin) = self.maximized.get(target) {
      let outputs = monitors()?;
      let output = outputs
        .iter()
        .find(|output| output["name"] == origin.monitor)
        .or_else(|| outputs.first())
        .ok_or_else(|| invalid("no monitor available"))?;
      let (at, size) = model::restored_geometry(origin, output);
      Self::set_geometry(target, at, size)?;
      self.maximized.remove(target);
      model::save_named(&self.maximized, "maximized.json")?;
      return Ok(());
    }
    let document = crate::ConfigStore::from_environment()?.load_effective()?;
    let windows = clients()?;
    let window = windows
      .iter()
      .find(|window| window["address"] == target)
      .ok_or_else(|| invalid("window closed"))?;
    if document.get("/layout/variant").and_then(Value::as_str) != Some("spaced")
      || window["floating"] != true
      || window["fullscreen"].as_i64().unwrap_or(0) != 0
    {
      return on_window(
        target,
        "hl.dispatch(hl.dsp.window.fullscreen({mode='maximized',action='toggle',window=w}))",
      );
    }
    let outputs = monitors()?;
    let output = outputs
      .iter()
      .find(|output| output["id"] == window["monitor"])
      .ok_or_else(|| invalid("window monitor unavailable"))?;
    let (gaps, border, title) = Self::spacing()?;
    let (at, size) = layout::geometry(window, output, gaps, border, title, true);
    self
      .maximized
      .insert(target.into(), model::capture(window, &outputs));
    model::save_named(&self.maximized, "maximized.json")?;
    Self::set_geometry(target, at, size)
  }

  fn publish(&mut self) -> ConfigResult<()> {
    let line = serde_json::to_string(&self.snapshot()?)?;
    if line != self.last_snapshot {
      self
        .watchers
        .retain_mut(|stream| writeln!(stream, "{line}").is_ok());
      self.last_snapshot = line;
    }
    Ok(())
  }

  fn minimize(&mut self, target: &str) -> ConfigResult<()> {
    if self.suspended || !mode_applied() {
      return Err(invalid("minimization is available only in Floating"));
    }
    if self
      .ready
      .is_none_or(|ready| ready.elapsed() >= Duration::from_secs(5))
      || !bar_mapped()?
    {
      return Err(invalid("window bar is not ready; window was not hidden"));
    }
    let windows = self.reconcile()?;
    let selected = windows
      .iter()
      .find(|window| window["address"] == target)
      .ok_or_else(|| invalid("window closed"))?;
    if self.registry.contains_key(target) {
      return Ok(());
    }
    let group = selected["grouped"].as_array().cloned().unwrap_or_default();
    let targets: Vec<_> = windows
      .iter()
      .filter(|window| window["address"] == target || group.contains(&window["address"]))
      .collect();
    let outputs = monitors()?;
    let output = outputs
      .iter()
      .find(|output| output["id"] == selected["monitor"])
      .and_then(|output| output["name"].as_str())
      .ok_or_else(|| invalid("window monitor is unavailable"))?;
    if !bar_mapped_on(Some(output))? {
      return Err(invalid("recovery bar is not mapped on this monitor"));
    }
    for window in &targets {
      let target = window["address"]
        .as_str()
        .ok_or_else(|| invalid("missing window address"))?;
      self
        .registry
        .entry(target.into())
        .or_insert_with(|| model::capture(window, &outputs));
    }
    model::save(&self.registry)?;
    for window in targets {
      let target = window["address"]
        .as_str()
        .ok_or_else(|| invalid("missing window address"))?;
      on_window(
        target,
        &format!(
          "hl.dispatch(hl.dsp.window.fullscreen_state({{internal=0,client=0,window=w}})); if w.pinned then hl.dispatch(hl.dsp.window.pin({{action='unset',window=w}})) end; hl.dispatch(hl.dsp.window.move({{workspace={},follow=false,window=w}}))",
          quote(PARKING)
        ),
      )?;
    }
    let after = clients()?;
    for window in after.iter().filter(|window| {
      self
        .registry
        .contains_key(window["address"].as_str().unwrap_or(""))
    }) {
      if model::workspace(window) != PARKING {
        return Err(invalid(
          "window could not be minimized; recovery record retained",
        ));
      }
    }
    Ok(())
  }

  fn restore_one(&mut self, target: &str) -> ConfigResult<()> {
    let Some(origin) = self.registry.get(target).cloned() else {
      return Ok(());
    };
    let windows = self.reconcile()?;
    let Some(window) = windows.iter().find(|window| window["address"] == target) else {
      return Ok(());
    };
    if !model::same_window(&origin, window) {
      return Err(invalid("window identity changed"));
    }
    let outputs = monitors()?;
    let output = outputs
      .iter()
      .find(|output| output["name"] == origin.monitor)
      .or_else(|| outputs.first())
      .ok_or_else(|| invalid("no monitor available"))?;
    let (at, size) = model::restored_geometry(&origin, output);
    let workspace = if origin.workspace == PARKING || origin.workspace.is_empty() {
      "1"
    } else {
      &origin.workspace
    };
    on_window(
      target,
      &format!(
        "hl.dispatch(hl.dsp.window.move({{monitor={},follow=false,window=w}})); hl.dispatch(hl.dsp.window.move({{workspace={},follow=false,window=w}})); if w.floating then hl.dispatch(hl.dsp.window.resize({{x={},y={},window=w}})); hl.dispatch(hl.dsp.window.move({{x={},y={},relative=false,window=w}})) end; hl.dispatch(hl.dsp.window.fullscreen_state({{internal={},client={},window=w}})); if {} and w.floating then hl.dispatch(hl.dsp.window.pin({{action='set',window=w}})) end",
        quote(
          output["name"]
            .as_str()
            .ok_or_else(|| invalid("invalid monitor name"))?
        ),
        quote(&model::workspace_selector(workspace)),
        size[0].max(1),
        size[1].max(1),
        at[0],
        at[1],
        origin.fullscreen,
        origin.fullscreen_client,
        origin.pinned
      ),
    )?;
    let after = clients()?;
    if after
      .iter()
      .any(|window| window["address"] == target && model::workspace(window) == PARKING)
    {
      return Err(invalid(
        "window restoration failed; recovery record retained",
      ));
    }
    self.registry.remove(target);
    model::save(&self.registry)?;
    Ok(())
  }

  fn action(&mut self, action: &str, target: Option<&str>) -> ConfigResult<Value> {
    match action {
      "list" => return self.snapshot(),
      "ready" => {
        if bar_mapped()? {
          self.ready = Some(Instant::now());
        }
      }
      "enable" => self.suspended = false,
      "prepare-tile" | "restore-all" => {
        if action == "prepare-tile" {
          self.suspended = true;
        }
        for target in self.registry.keys().cloned().collect::<Vec<_>>() {
          self.restore_one(&target)?;
        }
        if action == "prepare-tile" {
          for target in self.maximized.keys().cloned().collect::<Vec<_>>() {
            self.maximize(&target)?;
          }
        }
      }
      "minimize" => self.minimize(address(target.ok_or_else(|| invalid("missing address"))?)?)?,
      "restore" | "activate" | "maximize" | "close" => {
        let target = address(target.ok_or_else(|| invalid("missing address"))?)?;
        if action != "close" {
          // Restore every member of a minimized group before focusing it.
          let group = clients()?
            .into_iter()
            .find(|window| window["address"] == target)
            .and_then(|window| window["grouped"].as_array().cloned())
            .unwrap_or_default();
          for member in group.iter().filter_map(Value::as_str) {
            self.restore_one(member)?;
          }
          self.restore_one(target)?;
        }
        match action {
          "activate" | "restore" => on_window(target, "hl.dispatch(hl.dsp.focus({window=w}))")?,
          "maximize" => self.maximize(target)?,
          "close" => on_window(target, "hl.dispatch(hl.dsp.window.close({window=w}))")?,
          _ => {}
        }
      }
      _ => return Err(invalid("unknown window action")),
    }
    Ok(json!({"ok":true}))
  }
}

pub(super) fn run() -> ConfigResult<()> {
  let path = runtime_path("sock")?;
  fs::DirBuilder::new().recursive(true).mode(0o700).create(
    path
      .parent()
      .ok_or_else(|| invalid("missing runtime directory"))?,
  )?;
  let lock = OpenOptions::new()
    .create(true)
    .truncate(false)
    .read(true)
    .write(true)
    .open(runtime_path("lock")?)?;
  crate::config_lock::lock_file(&lock)?;
  let _guard = crate::config_lock::LockGuard(&lock);
  match fs::remove_file(&path) {
    Ok(()) => {}
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
    Err(error) => return Err(error.into()),
  }
  let listener = UnixListener::bind(&path)?;
  listener.set_nonblocking(true)?;
  let (sender, events) = mpsc::channel();
  let runtime = std::env::var_os("XDG_RUNTIME_DIR")
    .map(PathBuf::from)
    .ok_or_else(|| invalid("missing runtime"))?;
  let signature =
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| invalid("missing instance"))?;
  let event_path = hyprland_ipc::socket_path(&runtime, &signature)?.with_file_name(".socket2.sock");
  std::thread::spawn(move || {
    loop {
      match UnixStream::connect(&event_path) {
        Ok(stream) => {
          for line in BufReader::new(stream).lines() {
            match line {
              Ok(line) => {
                if sender.send(line).is_err() {
                  return;
                }
              }
              Err(_) => break,
            }
          }
        }
        Err(_) => std::thread::sleep(Duration::from_secs(1)),
      }
    }
  });
  let mut service = Service {
    registry: model::load()?,
    maximized: model::load_named("maximized.json")?,
    watchers: Vec::new(),
    ready: None,
    last_snapshot: String::new(),
    labels: labels()?,
    suspended: false,
  };
  let mut last_floating = mode_applied();
  let mut refreshed = Instant::now() - Duration::from_secs(2);
  loop {
    let mut dirty = false;
    let floating = mode_applied();
    if floating && !last_floating {
      service.suspended = false;
    }
    if floating != last_floating {
      dirty = true;
    }
    last_floating = floating;
    for event in events.try_iter() {
      dirty = true;
      if let Some(target) = event.strip_prefix("closewindow>>") {
        let target = format!("0x{}", target.trim().trim_start_matches("0x"));
        if service.registry.remove(&target).is_some() {
          model::save(&service.registry)?;
        }
        if service.maximized.remove(&target).is_some() {
          model::save_named(&service.maximized, "maximized.json")?;
        }
      }
      if let Some(arguments) = event.strip_prefix("minimized>>")
        && let Some((target, state)) = arguments.split_once(',')
      {
        let target = format!("0x{}", target.trim_start_matches("0x"));
        if let Err(error) = service.action(
          if state == "1" { "minimize" } else { "restore" },
          Some(&target),
        ) {
          eprintln!("argvus-windows: {error}");
        }
      }
    }
    match listener.accept() {
      Ok((mut stream, _)) => {
        dirty = true;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
        let mut line = String::new();
        let result = (|| -> ConfigResult<Value> {
          BufReader::new(&mut stream)
            .take(4096)
            .read_line(&mut line)?;
          let request: Value = serde_json::from_str(&line)?;
          let action = request["action"]
            .as_str()
            .ok_or_else(|| invalid("missing action"))?;
          if action == "watch" {
            return service.snapshot();
          }
          service.action(action, request["address"].as_str())
        })();
        let watching =
          serde_json::from_str::<Value>(&line).is_ok_and(|request| request["action"] == "watch");
        let reply = match result {
          Ok(reply) => reply,
          Err(error) => json!({"error":error.to_string()}),
        };
        if writeln!(stream, "{reply}").is_ok() && watching && reply.get("error").is_none() {
          service.watchers.push(stream);
        }
      }
      Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
      Err(error) => return Err(error.into()),
    }
    // Also handles mode changes made by configuration reload rather than CLI.
    if refreshed.elapsed() >= Duration::from_secs(1)
      && !mode_applied()
      && !service.registry.is_empty()
      && let Err(error) = service.action("restore-all", None)
    {
      eprintln!("argvus-windows: {error}");
    }
    if dirty || refreshed.elapsed() >= Duration::from_secs(1) {
      if let Err(error) = service.publish() {
        eprintln!("argvus-windows: {error}");
      }
      refreshed = Instant::now();
    }
    std::thread::sleep(Duration::from_millis(100));
  }
}

fn labels() -> ConfigResult<Value> {
  let i18n = argvus_i18n::I18n::new("taskbar").map_err(|error| invalid(error.to_string()))?;
  let mut labels = serde_json::Map::new();
  for key in ["restore", "minimize", "maximize", "close", "workspace"] {
    labels.insert(
      key.into(),
      Value::String(i18n.tr(&format!("windows.{key}"))),
    );
  }
  Ok(Value::Object(labels))
}
