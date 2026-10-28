//! Domain state and models consumed by the UI of the Session page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPage {
  Home,
  Components,
  Autostart,
  Diagnostics,
  Logs,
  LogDetail(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
  Running,
  Stopped,
  Failed,
}

impl ComponentStatus {
  pub fn running(&self) -> bool {
    matches!(self, Self::Running)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
  pub id: String,
  pub display_name: String,
  pub description: String,
  pub process: Vec<String>,
  pub unit: Option<String>,
  pub essential: bool,
  pub status: ComponentStatus,
  pub pid: Option<u32>,
}

impl Component {
  pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
      && id.len() <= 64
      && id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutostartEntry {
  /// Desktop entry ID (basename of the .desktop file).
  pub id: String,
  pub name: String,
  pub command: String,
  /// Whether the entry comes from a system directory being shadowed locally.
  pub from_system: bool,
  pub enabled: bool,
  /// Absolute path of the effective entry file.
  pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsEntry {
  pub ok: bool,
  pub warn: bool,
  pub label: String,
  pub detail: String,
}

impl DiagnosticsEntry {
  pub fn good(label: impl Into<String>, detail: impl Into<String>) -> Self {
    Self {
      ok: true,
      warn: false,
      label: label.into(),
      detail: detail.into(),
    }
  }

  pub fn warning(label: impl Into<String>, detail: impl Into<String>) -> Self {
    Self {
      ok: true,
      warn: true,
      label: label.into(),
      detail: detail.into(),
    }
  }

  pub fn bad(label: impl Into<String>, detail: impl Into<String>) -> Self {
    Self {
      ok: false,
      warn: false,
      label: label.into(),
      detail: detail.into(),
    }
  }
}

/// The list of components ARGVUS starts through `argvus-sessionctl`. The
/// process names keep status detection working even when a unit has no PID.
pub fn manifest() -> Vec<Component> {
  vec![
    component(
      "argvus-session",
      "Sessão ARGVUS",
      "Gerenciador de sessão do ARGVUS",
      vec!["session"],
      None,
      true,
    ),
    component(
      "shell",
      "Shell",
      "Shell / app launcher do ARGVUS",
      vec!["quickshell"],
      None,
      true,
    ),
    component(
      "waybar",
      "Waybar",
      "Barra de status",
      vec!["waybar"],
      None,
      false,
    ),
    component(
      "keyboard-layout",
      "Layout de teclado",
      "Aplicação do layout de teclado",
      vec![],
      Some("keyboard-layout.service"),
      true,
    ),
    component(
      "polkit-agent",
      "Agente polkit",
      "Autorizações gráficas",
      vec!["polkit-kde-authentication-agent", "xdg-polkit-agent"],
      None,
      true,
    ),
    component(
      "secret-service",
      "Secret Service",
      "Provedor de senhas (gnome-keyring)",
      vec!["gnome-keyring-daemon"],
      Some("argvus-secret-service.service"),
      false,
    ),
    component(
      "clipboard",
      "Área de transferência",
      "Histórico de clipboard",
      vec!["cliphist", "copyq"],
      None,
      false,
    ),
    component(
      "notification",
      "Notificações",
      "Serviço de notificações",
      vec!["swaync", "dunst", "mako"],
      None,
      false,
    ),
    component(
      "network-agent",
      "Agente de rede",
      "Menu e gerente de rede",
      vec!["nm-applet", "networkmanager"],
      None,
      false,
    ),
    component(
      "bluetooth-agent",
      "Agente de Bluetooth",
      "Applet de Bluetooth",
      vec!["blueman-applet"],
      None,
      false,
    ),
    component(
      "hypridle",
      "Hypridle",
      "Gerenciador de idle (energia)",
      vec!["hypridle"],
      None,
      false,
    ),
    component(
      "hyprpaper",
      "Hyprpaper",
      "Papéis de parede",
      vec!["hyprpaper"],
      None,
      false,
    ),
  ]
}

fn component(
  id: &str,
  display_name: &str,
  description: &str,
  process: Vec<&str>,
  unit: Option<&str>,
  essential: bool,
) -> Component {
  Component {
    id: id.into(),
    display_name: display_name.into(),
    description: description.into(),
    process: process.into_iter().map(Into::into).collect(),
    unit: unit.map(Into::into),
    essential,
    status: ComponentStatus::Stopped,
    pid: None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn manifest_contains_essential_components() {
    let components = manifest();
    assert!(
      components
        .iter()
        .any(|c| c.id == "argvus-session" && c.essential)
    );
    assert!(components.iter().any(|c| c.id == "shell" && c.essential));
    assert!(components.iter().all(|c| Component::valid_id(&c.id)));
    assert!(components.iter().any(|c| {
      c.id == "secret-service" && c.unit.as_deref() == Some("argvus-secret-service.service")
    }));
  }

  #[test]
  fn component_ids_are_strict() {
    assert!(Component::valid_id("keyboard-layout"));
    assert!(Component::valid_id("polkit-agent"));
    assert!(!Component::valid_id("bad id"));
    assert!(!Component::valid_id("bad/id"));
  }

  #[test]
  fn diagnostics_entry_has_three_kinds() {
    assert!(DiagnosticsEntry::good("a", "b").ok);
    assert!(DiagnosticsEntry::warning("a", "b").warn);
    assert!(!DiagnosticsEntry::bad("a", "b").ok);
  }
}
