use std::path::PathBuf;

use crate::actions::{ActionKind, Actions};
use crate::config::Config;
use crate::device::Device;
use crate::i18n;
use crate::util;

pub struct MenuEntry {
  pub kind: ActionKind,
  pub device_index: usize,
  pub label: String,
}

pub struct Menu<'a> {
  config: &'a Config,
}

impl<'a> Menu<'a> {
  pub fn new(config: &'a Config) -> Self {
    Menu { config }
  }

  // Two-level rofi/dmenu navigation: pick a device, then pick an action for
  // that device (submenu). Pressing Enter or the Right arrow on a device row
  // opens its submenu; Esc or the Left arrow in the submenu goes back.
  pub async fn run(&self, devices: &[Device]) -> i32 {
    let valid: Vec<usize> = (0..devices.len())
      .filter(|&i| !device_entries(devices, i).is_empty())
      .collect();
    if valid.is_empty() {
      util::notify(
        "argvus-removable-devices",
        i18n::tr(
          "No removable storage devices",
          "Nenhum dispositivo de armazenamento removível",
        ),
        false,
      );
      return 1;
    }

    let devices_prompt = i18n::tr("Removable devices", "Dispositivos removíveis");

    loop {
      let rows: Vec<String> = valid
        .iter()
        .map(|&i| submenu_device_label(&devices[i]))
        .collect();
      let cmd = self.menu_cmd(devices_prompt);
      let Some((output, _error)) = util::run_capture(&cmd, &rows) else {
        // Cancelled at the device list (dmenu returns non-zero).
        return 1;
      };
      let selection = util::trim(&output);
      if selection.is_empty() {
        return 1;
      }
      let Some(row) = rows.iter().position(|r| r == selection) else {
        return 1;
      };
      let index = valid[row];

      let entries = device_entries(devices, index);
      let lines: Vec<String> = entries.iter().map(|e| e.label.clone()).collect();
      let sub_cmd = self.menu_cmd(&submenu_prompt(&devices[index]));
      let Some((output, _error)) = util::run_capture(&sub_cmd, &lines) else {
        // Cancelled inside the submenu: go back to the device list.
        continue;
      };
      let selection = util::trim(&output);
      if selection.is_empty() {
        continue;
      }
      for e in &entries {
        if e.label == selection {
          let actions = Actions::new(self.config);
          return actions.perform(e.kind, &devices[e.device_index]).await;
        }
      }
    }
  }

  pub async fn run_devices(&self, devices: &[Device]) -> i32 {
    if devices.is_empty() {
      util::notify(
        "argvus-removable-devices",
        i18n::tr(
          "No removable storage devices",
          "Nenhum dispositivo de armazenamento removível",
        ),
        false,
      );
      return 1;
    }

    let entries: Vec<String> = devices.iter().map(device_label).collect();
    let prompt = i18n::tr("Removable devices", "Dispositivos removíveis");
    let cmd = self.menu_cmd(prompt);
    let Some((output, _error)) = util::run_capture(&cmd, &entries) else {
      return 1;
    };
    let selection = util::trim(&output);
    if selection.is_empty() {
      return 1;
    }

    for (i, entry) in entries.iter().enumerate() {
      if entry == selection {
        let actions = Actions::new(self.config);
        return actions.perform(ActionKind::Open, &devices[i]).await;
      }
    }
    1
  }

  // Base menu command: the configured launcher, the resolved ARGVUS rofi
  // theme (same `-config` used by every other themed rofi app), and the user
  // flags minus any `-p` prompt (each pass sets its own). Arrow navigation is
  // wired for rofi only.
  fn menu_cmd(&self, prompt: &str) -> String {
    let mut cmd = self.config.menu.clone();
    if self.config.menu == "rofi" {
      let flags = self.config.menu_flags.split_ascii_whitespace();
      let has_config = flags.clone().any(|t| t == "-config");
      if !has_config && let Some(cfg) = rofi_config_path() {
        cmd.push_str(" -config ");
        cmd.push_str(&util::shell_quote(&cfg));
      }
      cmd.push_str(" -kb-move-char-back \"Control+b\" -kb-move-char-forward \"Control+f\" -kb-accept-entry \"Return,KP_Enter,Right\" -kb-cancel \"Escape,Left\"");
    }
    let flags = strip_flag_pair(&self.config.menu_flags, "-p");
    if !flags.is_empty() {
      cmd.push(' ');
      cmd.push_str(&flags);
    }
    cmd.push_str(" -p ");
    cmd.push_str(&util::shell_quote(prompt));
    cmd
  }
}

// Actions offered for a single device (the submenu rows). Open is available for
// any unlocked/mountable device so a mounted drive can be opened too.
fn device_entries(devices: &[Device], index: usize) -> Vec<MenuEntry> {
  let d = &devices[index];
  let mut entries: Vec<MenuEntry> = Vec::new();
  let can_mount = !(d.mounted || d.encrypted && d.locked);
  if can_mount || d.mounted {
    entries.push(MenuEntry {
      kind: ActionKind::Open,
      device_index: index,
      label: i18n::tr("Open", "Abrir").to_string(),
    });
  }
  if can_mount {
    entries.push(MenuEntry {
      kind: ActionKind::Mount,
      device_index: index,
      label: i18n::tr("Mount", "Montar").to_string(),
    });
  }
  if d.mounted {
    entries.push(MenuEntry {
      kind: ActionKind::Unmount,
      device_index: index,
      label: i18n::tr("Unmount", "Desmontar").to_string(),
    });
  }
  if d.encrypted && d.locked {
    entries.push(MenuEntry {
      kind: ActionKind::Unlock,
      device_index: index,
      label: i18n::tr("Unlock", "Desbloquear").to_string(),
    });
  }
  if d.encrypted && !d.locked && d.mounted {
    entries.push(MenuEntry {
      kind: ActionKind::Lock,
      device_index: index,
      label: i18n::tr("Lock", "Bloquear").to_string(),
    });
  }
  if d.ejectable {
    entries.push(MenuEntry {
      kind: ActionKind::Eject,
      device_index: index,
      label: i18n::tr("Eject", "Ejetar").to_string(),
    });
  }
  if d.can_power_off {
    entries.push(MenuEntry {
      kind: ActionKind::PowerOff,
      device_index: index,
      label: i18n::tr("Power Off", "Desligar").to_string(),
    });
  }
  if d.mounted {
    entries.push(MenuEntry {
      kind: ActionKind::Copy,
      device_index: index,
      label: i18n::tr("Copy path", "Copiar caminho").to_string(),
    });
  }
  entries
}

fn submenu_device_label(d: &Device) -> String {
  format!("{}  >", device_short_name(d))
}

fn submenu_prompt(d: &Device) -> String {
  device_short_name(d)
}

fn device_short_name(d: &Device) -> String {
  let name = util::trim(&d.name);
  if name.is_empty() {
    d.block.clone()
  } else {
    name.to_string()
  }
}

fn device_label(d: &Device) -> String {
  let state = i18n::tr(
    if d.encrypted && d.locked {
      "locked"
    } else if d.mounted {
      "mounted"
    } else {
      "available"
    },
    if d.encrypted && d.locked {
      "bloqueado"
    } else if d.mounted {
      "montado"
    } else {
      "disponível"
    },
  );

  let mut details: Vec<String> = Vec::new();
  if !d.filesystem.is_empty() {
    details.push(d.filesystem.clone());
  }
  if d.capacity > 0 {
    details.push(util::human_bytes(d.capacity));
  }
  details.push(state.to_string());
  if !d.mount_point.is_empty() {
    details.push(d.mount_point.clone());
  }

  format!("{}  ·  {}", d.name, details.join(" · "))
}

// Resolve the ARGVUS rofi config the same way scripts do via paths_config:
// XDG override, user ARGVUS copy, generated copy, then the system default.
fn rofi_config_path() -> Option<String> {
  let mut dirs: Vec<PathBuf> = Vec::new();
  if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME")
    && !xdg.is_empty()
  {
    dirs.push(PathBuf::from(xdg.to_string_lossy().into_owned()));
  }
  if let Some(home) = std::env::var_os("HOME")
    && !home.is_empty()
  {
    dirs.push(PathBuf::from(home.to_string_lossy().into_owned()).join(".config"));
  }
  let relative = [
    "rofi/config.rasi",
    "argvus/data/rofi/config.rasi",
    "argvus/data/generated/rofi/config.rasi",
    // Pre-`data/` location, kept so an unmigrated profile still resolves.
    "argvus/rofi/config.rasi",
  ];
  for dir in dirs {
    for rel in relative {
      let p = dir.join(rel);
      if p.is_file() {
        return p.to_str().map(String::from);
      }
    }
  }
  let system = PathBuf::from("/usr/share/argvus/launcher/config/config.rasi");
  if system.is_file() {
    return system.to_str().map(String::from);
  }
  None
}

// Remove a `--flag <value>` pair, keeping every other token intact. Values
// wrapped in quotes (single or double) are consumed as a single token.
fn strip_flag_pair(flags: &str, flag: &str) -> String {
  let tokens: Vec<&str> = flags.split_ascii_whitespace().collect();
  let mut out: Vec<&str> = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == flag {
      i += 1;
      if i < tokens.len() {
        let value = tokens[i];
        let quote = value.chars().next();
        if quote == Some('\'') || quote == Some('"') {
          let closing = quote.unwrap();
          loop {
            if i >= tokens.len() || value_is_closed(tokens[i], closing) {
              break;
            }
            i += 1;
          }
        }
      }
      i += 1;
      continue;
    }
    out.push(tokens[i]);
    i += 1;
  }
  out.join(" ")
}

fn value_is_closed(token: &str, quote: char) -> bool {
  token.ends_with(quote) && token.len() > 1
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::device::Device;

  fn dev(name: &str, mounted: bool) -> Device {
    Device {
      name: name.to_string(),
      block: "/dev/sdb1".to_string(),
      object_path: "/org/freedesktop/UDisks2/drives/X".to_string(),
      filesystem: "exfat".to_string(),
      mount_point: if mounted {
        "/run/media/user/".to_string() + name
      } else {
        String::new()
      },
      mounted,
      can_power_off: true,
      ejectable: true,
      ..Device::default()
    }
  }

  #[test]
  fn strip_prompt_pair_keeps_other_flags() {
    let out = strip_flag_pair("-dmenu -i -p Storage", "-p");
    assert_eq!(out, "-dmenu -i");
    let out = strip_flag_pair("-dmenu -p 'My Menu' -i -theme-str 'window {}'", "-p");
    assert_eq!(out, "-dmenu -i -theme-str 'window {}'");
    let out = strip_flag_pair("-dmenu -i", "-p");
    assert_eq!(out, "-dmenu -i");
  }

  #[test]
  fn submenu_label_uses_name_with_chevron() {
    assert_eq!(submenu_device_label(&dev("Files", false)), "Files  >");
    assert_eq!(submenu_prompt(&dev("Files", false)), "Files");
  }

  #[test]
  fn mounted_device_submenu_has_open_unmount_and_copy() {
    let entries = device_entries(&[dev("Files", true)], 0);
    let labels: Vec<&str> = entries.iter().map(|e| e.label.as_str()).collect();
    assert!(labels.contains(&"Open"));
    assert!(labels.contains(&"Unmount"));
    assert!(labels.contains(&"Copy path"));
    assert!(labels.contains(&"Power Off"));
    assert!(!labels.contains(&"Mount"));
    let open = entries.iter().find(|e| e.kind == ActionKind::Open).unwrap();
    assert_eq!(open.kind, ActionKind::Open);
  }

  #[test]
  fn unmounted_device_submenu_offers_mount() {
    let entries = device_entries(&[dev("Ventoy", false)], 0);
    let labels: Vec<&str> = entries.iter().map(|e| e.label.as_str()).collect();
    assert!(labels.contains(&"Open"));
    assert!(labels.contains(&"Mount"));
    assert!(!labels.contains(&"Unmount"));
    assert!(!labels.contains(&"Copy path"));
  }
}
