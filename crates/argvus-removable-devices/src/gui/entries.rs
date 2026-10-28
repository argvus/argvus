//! Menu rows built from the devices: titles, state words, action labels and tooltips.

use super::*;

pub(super) struct DeviceEntry {
  pub(super) kind: ActionKind,
  pub(super) device: Device,
}

// Mirror the action set offered by the rofi mode, filtered by device state.
pub(super) fn device_entries(d: &Device) -> Vec<DeviceEntry> {
  let can_mount = !(d.mounted || (d.encrypted && d.locked));
  let mut out = Vec::new();
  if !d.encrypted || !d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Open,
      device: d.clone(),
    });
  }
  if can_mount {
    out.push(DeviceEntry {
      kind: ActionKind::Mount,
      device: d.clone(),
    });
  }
  if d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Unmount,
      device: d.clone(),
    });
  }
  if d.encrypted && d.locked {
    out.push(DeviceEntry {
      kind: ActionKind::Unlock,
      device: d.clone(),
    });
  }
  if d.encrypted && !d.locked && d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Lock,
      device: d.clone(),
    });
  }
  if d.ejectable {
    out.push(DeviceEntry {
      kind: ActionKind::Eject,
      device: d.clone(),
    });
  }
  if d.can_power_off {
    out.push(DeviceEntry {
      kind: ActionKind::PowerOff,
      device: d.clone(),
    });
  }
  if d.mounted {
    out.push(DeviceEntry {
      kind: ActionKind::Copy,
      device: d.clone(),
    });
  }
  out
}

pub(super) fn action_label(kind: ActionKind) -> &'static str {
  let (en, pt) = match kind {
    ActionKind::Open => ("Open", "Abrir"),
    ActionKind::Mount => ("Mount", "Montar"),
    ActionKind::Unmount => ("Unmount", "Desmontar"),
    ActionKind::Eject => ("Eject", "Ejetar"),
    ActionKind::PowerOff => ("Power Off", "Desligar"),
    ActionKind::Lock => ("Lock", "Bloquear"),
    ActionKind::Unlock => ("Unlock", "Desbloquear"),
    ActionKind::Copy => ("Copy path", "Copiar caminho"),
  };
  i18n::tr(en, pt)
}

pub(super) fn action_tooltip(kind: ActionKind) -> &'static str {
  let (en, pt) = match kind {
    ActionKind::Open => (
      "Open in the configured file manager",
      "Abrir no gerenciador de arquivos configurado",
    ),
    ActionKind::Mount => ("Mount this device", "Montar este dispositivo"),
    ActionKind::Unmount => ("Unmount this device", "Desmontar este dispositivo"),
    ActionKind::Eject => (
      "Unmount and eject this device",
      "Desmontar e ejetar este dispositivo",
    ),
    ActionKind::PowerOff => (
      "Safely power off this drive",
      "Desligar esta unidade com segurança",
    ),
    ActionKind::Lock => (
      "Lock this encrypted volume",
      "Bloquear este volume criptografado",
    ),
    ActionKind::Unlock => (
      "Unlock this encrypted volume",
      "Desbloquear este volume criptografado",
    ),
    ActionKind::Copy => ("Copy the mount path", "Copiar o ponto de montagem"),
  };
  i18n::tr(en, pt)
}

pub(super) fn device_title(d: &Device) -> String {
  let (en, pt) = state_words(d.encrypted && d.locked, d.mounted);
  let state = i18n::tr(en, pt);
  if d.capacity > 0 {
    format!("{} · {} · {}", d.name, text::human_bytes(d.capacity), state)
  } else {
    format!("{} · {}", d.name, state)
  }
}

pub(super) fn state_words(locked: bool, mounted: bool) -> (&'static str, &'static str) {
  if locked {
    ("locked", "bloqueado")
  } else if mounted {
    ("mounted", "montado")
  } else {
    ("available", "disponível")
  }
}
