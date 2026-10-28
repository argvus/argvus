//! Keyboard-binding defaults and overrides of the Settings pages.
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use argvus_i18n::Lang;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binding {
  pub id: String,
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
  let Ok(raw) = fs::read_to_string(manifest_path()) else {
    return Vec::new();
  };
  let Ok(manifest) = serde_json::from_str::<Manifest>(&raw) else {
    return Vec::new();
  };
  let canonical_shortcuts = load_canonical_shortcuts();
  manifest
    .bindings
    .into_iter()
    .map(|mut b| {
      b.keys = normalize_keys(&b.keys).unwrap_or(b.keys);
      match canonical_shortcuts
        .as_ref()
        .and_then(|shortcuts| shortcuts.get(&config_key_for_binding(&b.id)))
      {
        Some(serde_json::Value::String(keys)) => {
          b.keys = normalize_keys(keys).unwrap_or(b.keys);
          b.enabled = true;
        }
        Some(serde_json::Value::Null) => b.enabled = false,
        _ => b.enabled = true,
      };
      b
    })
    .collect()
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

fn load_canonical_shortcuts() -> Option<serde_json::Map<String, serde_json::Value>> {
  let output = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["get", "/keyboard_shortcuts", "--effective"])
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  serde_json::from_slice::<serde_json::Value>(&output.stdout)
    .ok()?
    .as_object()
    .cloned()
}

fn persist_canonical_shortcut(id: &str, shortcut: Option<&str>) -> Result<(), String> {
  let pointer = format!("/keyboard_shortcuts/{}", config_key_for_binding(id));
  let value = shortcut
    .map(|keys| serde_json::to_string(keys).map_err(|error| error.to_string()))
    .transpose()?
    .unwrap_or_else(|| "null".into());
  let status = argvus_control_center_core::process::command("argvus")
    .arg("config")
    .args(["set", &pointer, &value])
    .status()
    .map_err(|error| format!("failed to persist shortcut: {error}"))?;
  if !status.success() {
    return Err("argvus config rejected the shortcut".into());
  }
  Ok(())
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
      "pageup" => "Page_Up".into(),
      "pagedown" => "Page_Down".into(),
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
      b.id != id
        && b.enabled
        && b.configurable
        && current == requested
        && b.context == context
        && b.input == input
    })
    .map(|b| b.id.clone())
    .collect()
}
pub fn save_override(
  id: &str,
  keys: Option<String>,
  enabled: Option<bool>,
  _bindings: &[Binding],
) -> Result<(), String> {
  let default = defaults()
    .into_iter()
    .find(|b| b.id == id)
    .ok_or("unknown keybinding")?;
  let normalized_keys = keys.as_deref().map(normalize_keys).transpose()?;
  let shortcut = if enabled == Some(false) {
    None
  } else {
    Some(normalized_keys.as_deref().unwrap_or(&default.keys))
  };
  persist_canonical_shortcut(id, shortcut)?;
  project_canonical_config()
}
pub fn restore_all(bindings: &[Binding]) -> Result<(), String> {
  for binding in defaults()
    .into_iter()
    .filter(|binding| binding.configurable)
  {
    persist_canonical_shortcut(&binding.id, Some(&binding.keys))?;
  }
  project_canonical_config()?;
  generate(bindings)
}
pub fn restore(id: &str, bindings: &[Binding]) -> Result<(), String> {
  let default = defaults()
    .into_iter()
    .find(|binding| binding.id == id)
    .ok_or("unknown keybinding")?;
  persist_canonical_shortcut(id, Some(&default.keys))?;
  project_canonical_config()?;
  generate(bindings)
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
    let description = cheatsheet_description(language, binding)
      .unwrap_or_else(|| language.tr(&binding.description_key));
    let description = if description == binding.description_key {
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
