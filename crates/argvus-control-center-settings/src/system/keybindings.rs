//! Keyboard-binding defaults and overrides of the Settings pages.
pub use argvus_config_core::custom_shortcuts::{CustomShortcut, ShortcutAction};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use argvus_control_center_core::process::{ProcessRequest, ProcessRunner, SystemProcessRunner};
use argvus_i18n::Lang;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binding {
  pub id: String,
  #[serde(skip)]
  pub custom: Option<CustomShortcut>,
  pub category: String,
  pub description_key: String,
  pub keys: String,
  pub action: String,
  #[serde(default)]
  pub context: String,
  #[serde(default)]
  pub input: String,
  #[serde(default = "default_true")]
  pub configurable: bool,
  #[serde(default)]
  pub flags: Vec<String>,
  #[serde(default)]
  pub enabled: bool,
}
fn default_true() -> bool {
  true
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
  version: u32,
  bindings: Vec<Binding>,
}
pub fn manifest_path() -> PathBuf {
  std::env::var_os("ARGVUS_SYSTEM_CONFIG")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/usr/share/argvus"))
    .join("hyprland/keybindings.json")
}
fn generated_cheatsheet_path() -> PathBuf {
  crate::config::paths::argvus_data_home().join("generated/hypr/keybindings.txt")
}

pub fn cheatsheet_description(lang: Lang, binding: &Binding) -> Option<String> {
  let portuguese = lang.locale().starts_with("pt");
  if let Some(number) = binding.id.strip_prefix("workspace.switch.") {
    return Some(if portuguese {
      format!("Área de trabalho {number}")
    } else {
      format!("Workspace {number}")
    });
  }
  if let Some(number) = binding.id.strip_prefix("workspace.move.") {
    return Some(if portuguese {
      format!("Mover janela para a área de trabalho {number}")
    } else {
      format!("Move window to desktop {number}")
    });
  }
  if let Some(number) = binding.id.strip_prefix("project.slot_") {
    return Some(if portuguese {
      format!("Projeto {number}")
    } else {
      format!("Project {number}")
    });
  }
  let description = match binding.id.as_str() {
    "window.toggle_floating" => (
      "Enable/Disable Floating Window",
      "Ativa/Desativa Janela flutuante",
    ),
    "focus.left" | "focus.right" | "focus.up" | "focus.down" => (
      "Focus tiled window by direction",
      "Focar janela tileada por direção",
    ),
    "focus.cycle_left" | "focus.cycle_right" => (
      "Cycle focus between windows (tiled and floating)",
      "Ciclar foco entre janelas (tileadas e flutuantes)",
    ),
    "workspace.next"
    | "workspace.previous"
    | "workspace.next_mouse"
    | "workspace.previous_mouse" => (
      "Moving between work areas",
      "Circular entre áreas de trabalho",
    ),
    "resize.enter" => (
      "Enter window resize mode (floating window only)",
      "Entrar modo redimensionar janela (apenas flutuante)",
    ),
    "resize.right" | "resize.left" | "resize.up" | "resize.down" => ("Resize", "Redimensionar"),
    "resize.move_right" | "resize.move_left" | "resize.move_up" | "resize.move_down" => {
      ("Move window", "Mover janela")
    }
    "resize.cancel_escape" | "resize.cancel_return" => {
      ("Exit resize mode", "Sair modo redimensionar")
    }
    "window.drag_mouse" => (
      "Drag window (floating window only)",
      "Arrastar janela (apenas flutuante)",
    ),
    "window.resize_mouse" => (
      "Window resize mode (floating window only)",
      "Modo redimensionar janela (apenas flutuante)",
    ),
    "window.fullscreen" => ("Full screen window", "Tela cheia"),
    "window.maximize" => ("Maximize window (toggle)", "Maximizar janela (toggle)"),
    "window.toggle_split" => (
      "Split vertical/horizontal toggle",
      "Alternar split vertical/horizontal",
    ),
    "window.group_tabs" => ("Group/Ungroup into tabs", "Agrupar/Desagrupar em abas"),
    "window.next_tab" => ("Navigate between tabs", "Navegar entre abas"),
    id if id.starts_with("window.move_") => ("Move window", "Mover janela"),
    "window.close" => ("Close window", "Fechar janela"),
    id if id.starts_with("workspace.move.") => (
      "Move window to desktop",
      "Mover janela para a área de trabalho",
    ),
    "screenshot.region" => ("Capture selected area", "Capturar área selecionada"),
    "screenshot.window" => ("Capture focused window", "Capturar janela em foco"),
    "screenshot.fullscreen" => ("Capture entire screen", "Capturar tela inteira"),
    "record.toggle" => (
      "Start/Pause/Resume screen recording",
      "Iniciar/Pausar/Retomar gravação de tela",
    ),
    "record.stop" => (
      "Stop and save screen recording",
      "Parar e salvar gravação de tela",
    ),
    "widget.sidebar" | "widget.sidebar_mouse" => (
      "Open/Close notification sidebar",
      "Abre/Fecha sidebar de notificações",
    ),
    "widget.taskbar_toggle" => ("Toggle Waybar top", "Oculta/Mostra Waybar top"),
    "appearance.wallpaper" => (
      "Open Control Center wallpapers",
      "Abre os wallpapers do Control Center",
    ),
    "appearance.theme" => ("Open theme selector", "Abre o seletor de temas"),
    "appearance.layout_mode" => (
      "Open Sticky/Float mode picker",
      "Abre o seletor de modo Sticky/Float",
    ),
    "session.idle_timeout" => (
      "Choose inactivity lock timeout",
      "Escolhe o tempo de bloqueio por inatividade",
    ),
    "session.keep_awake" => ("Toggle Keep Awake", "Ativa/Desativa manter acordado"),
    "widget.weather" => (
      "Configure weather location",
      "Configura a localização do clima",
    ),
    "appearance.brightness" => ("Open brightness selector", "Abre o seletor de brilho"),
    "appearance.mode" => (
      "Toggle between the ARGVUS Dark and ARGVUS Light themes",
      "Alterna entre os temas ARGVUS Dark e ARGVUS Light",
    ),
    "appearance.animations" => ("Toggle visual effects", "Ativa/Desativa efeitos visuais"),
    "system.about" => ("Open About ARGVUS", "Abrir Sobre o ARGVUS"),
    "session.lock" => ("Lock system", "Bloquear sistema"),
    "session.dpms" => (
      "Turn monitor off/on (not suspend system)",
      "Desligar/Ligar monitor (não é suspender)",
    ),
    "session.exit" => ("Exit system", "Sair do sistema"),
    "session.reload" => ("Reload Hyprland", "Recarregar Hyprland"),
    "widget.waybar_top" | "widget.waybar_bottom" => (
      "Move taskbar to top/bottom",
      "Mover barra de tarefas para topo/inferior",
    ),
    "session.volume_up" => ("Volume up", "Aumentar volume"),
    "session.volume_down" => ("Volume down", "Diminuir volume"),
    "session.mute" => ("Mute", "Silenciar"),
    "session.brightness_up" => ("Brightness up", "Aumentar brilho"),
    "session.brightness_down" => ("Brightness down", "Diminuir brilho"),
    "session.play_pause" => ("Play/Pause", "Reproduzir/Pausar"),
    "session.next_track" => ("Next track", "Próxima faixa"),
    "session.previous_track" => ("Previous track", "Faixa anterior"),
    "session.stop_track" => ("Stop track", "Parar faixa"),
    "app.terminal" => ("Terminal", "Terminal"),
    "app.scratchpad" => ("Terminal scratchpad", "Terminal suspenso (scratchpad)"),
    "app.file_manager" => ("File Manager", "Gerenciador de arquivos"),
    "app.removable_devices" => ("Removable devices menu", "Menu de dispositivos removíveis"),
    "app.focus" => (
      "ARGVUS Focus (Pomodoro and focus sessions)",
      "ARGVUS Focus (Pomodoro e sessões de foco)",
    ),
    "app.clock" => (
      "ARGVUS Clock (clock, timers and alarms)",
      "ARGVUS Clock (relógio, temporizadores e alarmes)",
    ),
    "app.browser" => ("Default Browser", "Navegador padrão"),
    "app.launcher" => ("Program Launcher", "Lançador de programas"),
    "app.calculator" => ("Calculator", "Calculadora"),
    "app.projects" => (
      "Project list (argvus projects)",
      "Lista de projetos (argvus projects)",
    ),
    "app.ssh" => (
      "SSH host picker (argvus ssh)",
      "Seletor de hosts SSH (argvus ssh)",
    ),
    "app.snippets" => (
      "Snippet picker (argvus snippets)",
      "Seletor de snippets (argvus snippets)",
    ),
    "app.git" => (
      "Project picker, then git TUI (argvus git)",
      "Seletor de projetos e TUI de git (argvus git)",
    ),
    "system.kitty_cheatsheet" => ("Kitty Cheatsheets", "Cheatsheets do Kitty"),
    "system.hyprland_cheatsheet" => ("Hyprland Cheatsheets", "Cheatsheets do Hyprland"),
    "system.clipboard" => ("Clipboard History", "Histórico no Clipboard"),
    "system.clipboard_clear" => ("Clear Clipboard History", "Apagar histórico do Clipboard"),
    "system.color_picker" => ("Color Picker", "Seletor de cores (Color Picker)"),
    "system.emoji_picker" => ("Emoji Picker", "Emoji Picker"),
    "system.control_center" => (
      "Default apps selector (argvus-default-apps)",
      "Seletor de aplicativos padrão (argvus-default-apps)",
    ),
    _ => return None,
  };
  Some(if portuguese {
    description.1.to_string()
  } else {
    description.0.to_string()
  })
}
pub fn load() -> Vec<Binding> {
  let canonical = load_canonical_value("/keyboard_shortcuts");
  let mut bindings = defaults();
  for binding in &mut bindings {
    binding.keys = normalize_keys(&binding.keys).unwrap_or_else(|_| binding.keys.clone());
    match canonical
      .as_ref()
      .and_then(|value| value.get(config_key_for_binding(&binding.id)))
    {
      Some(serde_json::Value::String(keys)) => {
        binding.keys = normalize_keys(keys).unwrap_or_else(|_| binding.keys.clone());
        binding.enabled = true;
      }
      Some(serde_json::Value::Null) => binding.enabled = false,
      _ => binding.enabled = true,
    }
  }
  let custom = load_canonical_value("/hyprland/custom_shortcuts");
  if let Some(custom) = custom
    && let Ok(shortcuts) =
      serde_json::from_value::<std::collections::BTreeMap<String, CustomShortcut>>(custom)
  {
    for (id, shortcut) in shortcuts {
      bindings.push(custom_binding(id, shortcut));
    }
  }
  bindings
}

pub fn custom_binding(id: String, mut shortcut: CustomShortcut) -> Binding {
  shortcut.keys = normalize_keys(&shortcut.keys).unwrap_or(shortcut.keys);
  Binding {
    id,
    category: "custom".into(),
    description_key: shortcut.description.clone(),
    keys: shortcut.keys.clone(),
    enabled: shortcut.enabled,
    action: match &shortcut.action {
      ShortcutAction::Command { command } => command.clone(),
      ShortcutAction::Dispatcher {
        dispatcher,
        arguments,
      } => format!(
        "{dispatcher} {}",
        arguments
          .as_ref()
          .map(ToString::to_string)
          .unwrap_or_default()
      ),
    },
    context: String::new(),
    input: String::new(),
    configurable: true,
    flags: Vec::new(),
    custom: Some(shortcut),
  }
}

fn load_canonical_value(pointer: &str) -> Option<serde_json::Value> {
  let request = ProcessRequest::new("argvus")
    .arg("config")
    .arg("get")
    .arg(pointer)
    .arg("--effective")
    .timeout(Duration::from_secs(10));
  let output = SystemProcessRunner.run(&request).ok()?;
  if output.status != Some(0) {
    return None;
  }
  serde_json::from_slice(&output.stdout).ok()
}

/// One patch saves the target and disables every conflicting shortcut under the store lock.
pub fn persist_custom(
  id: &str,
  shortcut: &CustomShortcut,
  conflicts: &[Binding],
) -> Result<(), String> {
  persist_custom_with(&SystemProcessRunner, id, shortcut, conflicts)
}

fn persist_custom_with(
  runner: &impl ProcessRunner,
  id: &str,
  shortcut: &CustomShortcut,
  conflicts: &[Binding],
) -> Result<(), String> {
  shortcut.validate().map_err(|error| error.to_string())?;
  if conflicts.iter().any(|binding| !binding.configurable) {
    return Err("protected keybinding".into());
  }
  let mut patch = serde_json::Map::new();
  patch.insert(
    format!("/hyprland/custom_shortcuts/{id}"),
    serde_json::to_value(shortcut).map_err(|error| error.to_string())?,
  );
  for binding in conflicts {
    let (pointer, value) = if binding.custom.is_some() {
      (
        format!("/hyprland/custom_shortcuts/{}/enabled", binding.id),
        serde_json::Value::Bool(false),
      )
    } else {
      (
        format!(
          "/keyboard_shortcuts/{}",
          config_key_for_binding(&binding.id)
        ),
        serde_json::Value::Null,
      )
    };
    patch.insert(pointer, value);
  }
  persist_patch_with(runner, patch)
}

fn persist_patch(patch: serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
  persist_patch_with(&SystemProcessRunner, patch)
}

fn persist_patch_with(
  runner: &impl ProcessRunner,
  patch: serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
  run_config(
    runner,
    ProcessRequest::new("argvus")
      .arg("config")
      .arg("patch")
      .arg("-")
      .stdin(serde_json::Value::Object(patch).to_string().into_bytes()),
  )
}

fn run_config(runner: &impl ProcessRunner, request: ProcessRequest) -> Result<(), String> {
  let output = runner
    .run(&request.timeout(Duration::from_secs(15)))
    .map_err(|error| error.to_string())?;
  if output.status == Some(0) && !output.timed_out && !output.cancelled {
    Ok(())
  } else {
    Err(format!(
      "configuration command failed: {}",
      String::from_utf8_lossy(&output.stderr).trim()
    ))
  }
}

pub fn delete_custom(id: &str) -> Result<(), String> {
  if !id.starts_with("custom_")
    || !id.chars().all(|character| {
      character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    })
  {
    return Err("not a custom shortcut".into());
  }
  run_config(
    &SystemProcessRunner,
    ProcessRequest::new("argvus")
      .arg("config")
      .arg("unset")
      .arg(format!("/hyprland/custom_shortcuts/{id}")),
  )
}

/// A successful save remains saved when applying to the running session fails.
pub fn apply_saved() -> Result<(), String> {
  project_canonical_config()?;
  generate(&load())
}

fn config_key_for_binding(id: &str) -> String {
  match id {
    "window.close" => "close_window",
    "window.drag_mouse" => "drag_window__floating_window_only",
    "window.maximize" => "maximize_window__toggle",
    "app.browser" => "open_browser",
    "system.about" => "open_about",
    "appearance.theme" => "open_theme_selector",
    _ => return id.replace(['.', '-'], "_"),
  }
  .into()
}

fn project_canonical_config() -> Result<(), String> {
  argvus_control_center_core::config::reload_argvus_config_service()
}
pub fn normalize_keys(value: &str) -> Result<String, String> {
  let mut parts: Vec<String> = value
    .split('+')
    .map(str::trim)
    .filter(|v| !v.is_empty())
    .map(|v| match v.to_ascii_lowercase().as_str() {
      "super" | "win" | "meta" => "SUPER".into(),
      "ctrl" | "control" => "CTRL".into(),
      "alt" => "ALT".into(),
      "shift" => "SHIFT".into(),
      "enter" | "return" => "Return".into(),
      "esc" | "escape" => "Escape".into(),
      "space" => "Space".into(),
      "tab" => "Tab".into(),
      "left" | "leftarrow" => "left".into(),
      "right" | "rightarrow" => "right".into(),
      "up" | "uparrow" => "up".into(),
      "down" | "downarrow" => "down".into(),
      "pageup" | "page_up" => "Page_Up".into(),
      "pagedown" | "page_down" => "Page_Down".into(),
      "backspace" => "BackSpace".into(),
      "delete" => "Delete".into(),
      "insert" => "Insert".into(),
      "home" => "Home".into(),
      "end" => "End".into(),
      "/" | "slash" => "slash".into(),
      "-" | "minus" => "minus".into(),
      "." | "period" => "period".into(),
      "," | "comma" => "comma".into(),
      ";" | "semicolon" => "semicolon".into(),
      "'" | "apostrophe" => "apostrophe".into(),
      "[" | "bracketleft" => "bracketleft".into(),
      "]" | "bracketright" => "bracketright".into(),
      "\\" | "backslash" => "backslash".into(),
      "`" | "grave" => "grave".into(),
      "=" | "equal" => "equal".into(),
      "print" => "Print".into(),
      "xf86audioraisevolume" => "XF86AudioRaiseVolume".into(),
      "xf86audiolowervolume" => "XF86AudioLowerVolume".into(),
      "xf86audiomute" => "XF86AudioMute".into(),
      "xf86audionext" => "XF86AudioNext".into(),
      "xf86audioplay" => "XF86AudioPlay".into(),
      "xf86audioprev" => "XF86AudioPrev".into(),
      "xf86audiostop" => "XF86AudioStop".into(),
      "xf86monbrightnessdown" => "XF86MonBrightnessDown".into(),
      "xf86monbrightnessup" => "XF86MonBrightnessUp".into(),
      v => {
        if v.starts_with("mouse:") || v.starts_with("XF86") || v == "Print" {
          v.to_string()
        } else if v.len() == 1
          || (v.len() >= 2
            && v.starts_with('f')
            && v[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)))
        {
          v.to_ascii_uppercase()
        } else {
          v.to_string()
        }
      }
    })
    .collect();
  let key = parts.pop().ok_or("shortcut needs a key")?;
  if ["CTRL", "ALT", "SHIFT", "SUPER"].contains(&key.as_str()) {
    return Err("shortcut needs a non-modifier key".into());
  }
  let mut mods = parts;
  mods.sort_by_key(|v| {
    ["SUPER", "CTRL", "ALT", "SHIFT"]
      .iter()
      .position(|m| m == v)
      .unwrap_or(99)
  });
  mods.dedup();
  Ok(if mods.is_empty() {
    key
  } else {
    format!("{} + {key}", mods.join(" + "))
  })
}

pub fn display_key(key: &str) -> String {
  match key {
    "slash" => "/".into(),
    "minus" => "-".into(),
    "period" => ".".into(),
    "comma" => ",".into(),
    "semicolon" => ";".into(),
    "apostrophe" => "'".into(),
    "bracketleft" => "[".into(),
    "bracketright" => "]".into(),
    "backslash" => "\\".into(),
    "grave" => "`".into(),
    "equal" => "=".into(),
    "left" => "Left Arrow".into(),
    "right" => "Right Arrow".into(),
    "up" => "Up Arrow".into(),
    "down" => "Down Arrow".into(),
    "Return" => "Enter".into(),
    "Escape" => "Escape".into(),
    "Page_Up" => "Page Up".into(),
    "Page_Down" => "Page Down".into(),
    "BackSpace" => "Backspace".into(),
    "XF86AudioLowerVolume" => "Volume Down".into(),
    "XF86AudioMute" => "Mute".into(),
    "XF86AudioNext" => "Next Track".into(),
    "XF86AudioPlay" => "Play/Pause".into(),
    "XF86AudioPrev" => "Previous Track".into(),
    "XF86AudioStop" => "Stop".into(),
    "XF86MonBrightnessDown" => "Brightness Down".into(),
    "XF86MonBrightnessUp" => "Brightness Up".into(),
    other => other.into(),
  }
}
pub fn conflicts(bindings: &[Binding], id: &str, keys: &str) -> Vec<String> {
  conflicts_in_context(bindings, id, keys, "", "")
}
pub fn conflicts_in_context(
  bindings: &[Binding],
  id: &str,
  keys: &str,
  context: &str,
  input: &str,
) -> Vec<String> {
  bindings
    .iter()
    .filter(|b| {
      let current = normalize_keys(&b.keys).ok();
      let requested = normalize_keys(keys).ok();
      b.id != id && b.enabled && current == requested && b.context == context && b.input == input
    })
    .map(|b| b.id.clone())
    .collect()
}
/// Persists an official/custom edit and all conflict changes in one transaction.
pub fn persist_binding(
  id: &str,
  keys: Option<String>,
  enabled: bool,
  bindings: &[Binding],
  conflicts: &[Binding],
) -> Result<(), String> {
  if bindings
    .iter()
    .find(|binding| binding.id == id)
    .is_some_and(|binding| !binding.configurable)
    || conflicts.iter().any(|binding| !binding.configurable)
  {
    return Err("protected keybinding".into());
  }
  if let Some(mut shortcut) = bindings
    .iter()
    .find(|binding| binding.id == id)
    .and_then(|binding| binding.custom.clone())
  {
    if let Some(keys) = keys {
      shortcut.keys = normalize_keys(&keys)?;
    }
    shortcut.enabled = enabled;
    return persist_custom(id, &shortcut, conflicts);
  }
  let default = defaults()
    .into_iter()
    .find(|binding| binding.id == id)
    .ok_or("unknown keybinding")?;
  let keys = normalize_keys(keys.as_deref().unwrap_or(&default.keys))?;
  let mut patch = serde_json::Map::new();
  patch.insert(
    format!("/keyboard_shortcuts/{}", config_key_for_binding(id)),
    if enabled {
      keys.into()
    } else {
      serde_json::Value::Null
    },
  );
  for binding in conflicts {
    if binding.custom.is_some() {
      patch.insert(
        format!("/hyprland/custom_shortcuts/{}/enabled", binding.id),
        false.into(),
      );
    } else {
      patch.insert(
        format!(
          "/keyboard_shortcuts/{}",
          config_key_for_binding(&binding.id)
        ),
        serde_json::Value::Null,
      );
    }
  }
  persist_patch(patch)
}

pub fn save_override(
  id: &str,
  keys: Option<String>,
  enabled: Option<bool>,
  bindings: &[Binding],
) -> Result<(), String> {
  persist_binding(id, keys, enabled.unwrap_or(true), bindings, &[])?;
  apply_saved()
}

/// Restoring defaults must not silently reactivate an official binding over a custom one.
pub fn restoration_conflicts(bindings: &[Binding], id: Option<&str>) -> bool {
  defaults()
    .iter()
    .filter(|binding| binding.configurable && id.is_none_or(|id| binding.id == id))
    .any(|default| {
      bindings.iter().any(|binding| {
        binding.custom.is_some()
          && binding.enabled
          && binding.context == default.context
          && binding.input == default.input
          && normalize_keys(&binding.keys).ok() == normalize_keys(&default.keys).ok()
      })
    })
}

pub fn restore_all(bindings: &[Binding]) -> Result<(), String> {
  if restoration_conflicts(bindings, None) {
    return Err("custom shortcuts conflict with official defaults".into());
  }
  let patch = defaults()
    .into_iter()
    .filter(|binding| binding.configurable)
    .map(|binding| {
      (
        format!(
          "/keyboard_shortcuts/{}",
          config_key_for_binding(&binding.id)
        ),
        binding.keys.into(),
      )
    })
    .collect();
  persist_patch(patch)?;
  apply_saved()
}

pub fn restore(id: &str, bindings: &[Binding]) -> Result<(), String> {
  if restoration_conflicts(bindings, Some(id)) {
    return Err("custom shortcuts conflict with official defaults".into());
  }
  let default = defaults()
    .into_iter()
    .find(|binding| binding.id == id)
    .ok_or("unknown keybinding")?;
  save_override(id, Some(default.keys), Some(true), bindings)
}
pub fn generate(bindings: &[Binding]) -> Result<(), String> {
  let mut cheatsheet =
    String::from("ARGVUS effective Hyprland shortcuts\n===================================\n\n");
  let language = argvus_i18n::Lang::detect();
  for binding in bindings.iter().filter(|binding| binding.configurable) {
    let state = if binding.enabled {
      &display_keys(&binding.keys)
    } else {
      "Disabled"
    };
    let description = if let Some(custom) = &binding.custom {
      custom.description.clone()
    } else {
      cheatsheet_description(language, binding)
        .unwrap_or_else(|| language.tr(&binding.description_key))
    };
    let description = if binding.custom.is_none() && description == binding.description_key {
      binding
        .id
        .rsplit('.')
        .next()
        .unwrap_or(&binding.id)
        .replace('_', " ")
    } else {
      description
    };
    cheatsheet.push_str(&format!("{:<34} {}\n", state, description));
  }
  let cheat_path = generated_cheatsheet_path();
  if let Some(parent) = cheat_path.parent() {
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let cheat_tmp = cheat_path.with_extension("txt.tmp");
  fs::write(&cheat_tmp, cheatsheet).map_err(|e| e.to_string())?;
  fs::rename(cheat_tmp, cheat_path).map_err(|e| e.to_string())
}
pub fn display_keys(keys: &str) -> String {
  let mut modifiers = Vec::new();
  let mut key = "";
  for part in keys.split(" + ") {
    match part.to_ascii_uppercase().as_str() {
      "SUPER" | "CTRL" | "ALT" | "SHIFT" => modifiers.push(part.to_ascii_uppercase()),
      _ => key = part,
    }
  }
  modifiers.sort_by_key(|part| {
    ["SUPER", "CTRL", "ALT", "SHIFT"]
      .iter()
      .position(|m| m == part)
      .unwrap_or(99)
  });
  let mut output = modifiers
    .into_iter()
    .map(|part| {
      if part == "SHIFT" {
        "Shift".into()
      } else {
        part
      }
    })
    .collect::<Vec<String>>();
  if !key.is_empty() {
    output.push(display_key(key));
  }
  output.join(" + ")
}
fn defaults() -> Vec<Binding> {
  let Ok(raw) = fs::read_to_string(manifest_path()) else {
    return Vec::new();
  };
  serde_json::from_str::<Manifest>(&raw)
    .map(|m| m.bindings)
    .unwrap_or_default()
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn canonicalizes_modifiers() {
    assert_eq!(
      normalize_keys("shift + super + q").unwrap(),
      "SUPER + SHIFT + Q"
    );
  }
  #[test]
  fn rejects_empty() {
    assert!(normalize_keys("CTRL +").is_err());
  }
  #[test]
  fn deduplicates_modifiers() {
    assert_eq!(normalize_keys("SUPER + super + q").unwrap(), "SUPER + Q");
  }
  #[test]
  fn canonicalizes_symbols_and_special_keys() {
    for (raw, expected) in [
      ("/", "slash"),
      ("-", "minus"),
      (".", "period"),
      (",", "comma"),
      (";", "semicolon"),
      ("'", "apostrophe"),
      ("[", "bracketleft"),
      ("]", "bracketright"),
      ("\\", "backslash"),
      ("`", "grave"),
      ("=", "equal"),
      ("F1", "F1"),
      ("F12", "F12"),
      ("XF86AudioMute", "XF86AudioMute"),
      ("left", "left"),
      ("Return", "Return"),
    ] {
      assert_eq!(normalize_keys(raw).unwrap(), expected, "{raw}");
    }
  }
  #[test]
  fn display_keys_hides_internal_keysyms() {
    assert_eq!(display_keys("SUPER + SHIFT + slash"), "SUPER + Shift + /");
    assert_eq!(
      display_keys("SHIFT + CTRL + ALT + SUPER + slash"),
      "SUPER + CTRL + ALT + Shift + /"
    );
    assert_eq!(display_keys("SUPER + F1"), "SUPER + F1");
    assert_eq!(display_keys("XF86AudioMute"), "Mute");
  }
}

#[cfg(test)]
mod launcher_shortcut_tests {
  use super::*;

  fn binding(id: &str) -> Binding {
    Binding {
      custom: None,
      id: id.into(),
      category: "applications".into(),
      description_key: String::new(),
      keys: "SUPER + ALT + G".into(),
      action: String::new(),
      context: String::new(),
      input: String::new(),
      configurable: true,
      flags: Vec::new(),
      enabled: true,
    }
  }

  #[test]
  fn launcher_shortcuts_have_readable_descriptions_in_both_languages() {
    for id in ["app.projects", "app.ssh", "app.snippets", "app.git"] {
      let en = cheatsheet_description(Lang::for_locale("en-US"), &binding(id));
      let pt = cheatsheet_description(Lang::for_locale("pt-BR"), &binding(id));
      assert!(
        en.as_deref().is_some_and(|text| text.contains("argvus")),
        "{id}"
      );
      assert!(
        pt.as_deref().is_some_and(|text| text.contains("argvus")),
        "{id}"
      );
    }
  }
}

#[cfg(test)]
mod custom_tests {
  use super::*;
  use argvus_control_center_core::process::{ProcessError, ProcessOutput};
  use std::sync::Mutex;

  struct RecordingRunner {
    requests: Mutex<Vec<ProcessRequest>>,
    status: i32,
  }
  impl ProcessRunner for RecordingRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
      self.requests.lock().unwrap().push(request.clone());
      Ok(ProcessOutput {
        stdout: Vec::new(),
        stderr: b"save failed".to_vec(),
        status: Some(self.status),
        timed_out: false,
        cancelled: false,
      })
    }
  }

  fn shortcut() -> CustomShortcut {
    CustomShortcut {
      description: "Example".into(),
      keys: "SUPER + CTRL + E".into(),
      enabled: true,
      action: ShortcutAction::Command {
        command: "editor 'a b'".into(),
      },
    }
  }

  #[test]
  fn replacement_saves_the_custom_record_and_all_conflicts_in_one_request() {
    let runner = RecordingRunner {
      requests: Mutex::default(),
      status: 0,
    };
    let mut official = custom_binding("window.close".into(), shortcut());
    official.custom = None;
    let custom = custom_binding("custom_old".into(), shortcut());
    persist_custom_with(&runner, "custom_new", &shortcut(), &[official, custom]).unwrap();
    let requests = runner.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].args, ["config", "patch", "-"]);
    let patch: serde_json::Value =
      serde_json::from_slice(requests[0].stdin.as_ref().unwrap().as_bytes()).unwrap();
    assert!(patch["/keyboard_shortcuts/close_window"].is_null());
    assert_eq!(
      patch["/hyprland/custom_shortcuts/custom_old/enabled"],
      false
    );
    assert_eq!(
      patch["/hyprland/custom_shortcuts/custom_new"]["description"],
      "Example"
    );
    assert_eq!(
      patch["/hyprland/custom_shortcuts/custom_new"]["action"]["command"],
      "editor 'a b'"
    );
  }

  #[test]
  fn protected_conflicts_do_not_start_a_process_and_write_errors_are_returned() {
    let runner = RecordingRunner {
      requests: Mutex::default(),
      status: 1,
    };
    let mut protected = custom_binding("custom_old".into(), shortcut());
    protected.configurable = false;
    assert!(persist_custom_with(&runner, "custom_new", &shortcut(), &[protected]).is_err());
    assert!(runner.requests.lock().unwrap().is_empty());
    assert!(persist_custom_with(&runner, "custom_new", &shortcut(), &[]).is_err());
    assert_eq!(runner.requests.lock().unwrap().len(), 1);
  }

  #[test]
  fn custom_conflicts_respect_context_and_ignore_disabled_bindings() {
    let active = custom_binding("custom_active".into(), shortcut());
    let mut disabled = custom_binding("custom_disabled".into(), shortcut());
    disabled.enabled = false;
    let mut submap = custom_binding("custom_submap".into(), shortcut());
    submap.context = "resize".into();
    assert_eq!(
      conflicts(
        &[active, disabled, submap],
        "custom_new",
        "CTRL + SUPER + E"
      ),
      vec!["custom_active"]
    );
  }
}

#[cfg(test)]
mod custom_keys_tests {
  use super::*;
  #[test]
  fn special_and_media_keys_keep_their_canonical_names() {
    for (input, expected) in [
      ("Print", "Print"),
      ("CTRL + Page_Up", "CTRL + Page_Up"),
      ("ALT + Page_Down", "ALT + Page_Down"),
      ("XF86AudioRaiseVolume", "XF86AudioRaiseVolume"),
    ] {
      assert_eq!(normalize_keys(input).unwrap(), expected);
      argvus_config_core::custom_shortcuts::validate_keys(expected).unwrap();
    }
  }
}
