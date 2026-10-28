//! Keyboard shortcut editing: capturing a key combination, conflicts with other bindings and the cheatsheet sections.

use super::*;

impl App {
  pub(super) fn edited_binding(&self) -> Option<&keybindings::Binding> {
    self
      .keybinding_edit_id
      .as_deref()
      .and_then(|id| self.keybindings.iter().find(|b| b.id == id))
  }

  pub(super) fn captured_shortcut(&self) -> String {
    let mut parts = Vec::new();
    for (enabled, modifier) in self
      .keybinding_editor_modifiers
      .into_iter()
      .zip(["CTRL", "ALT", "SHIFT", "SUPER"])
    {
      if enabled {
        parts.push(modifier);
      }
    }
    parts.push(self.keybinding_editor_key.as_str());
    keybindings::display_keys(&parts.join(" + "))
  }

  pub fn begin_keybinding_capture(&mut self) {
    if let Some(keys) = self
      .edited_binding()
      .or_else(|| self.selected_keybinding())
      .map(|binding| binding.keys.clone())
    {
      let parts: Vec<&str> = keys.split(" + ").collect();
      self.keybinding_super_required = parts.contains(&"SUPER");
      self.keybinding_editor_modifiers = [
        parts.contains(&"CTRL"),
        parts.contains(&"ALT"),
        parts.contains(&"SHIFT"),
        parts.contains(&"SUPER"),
      ];
      self.keybinding_editor_key = parts.last().copied().unwrap_or_default().to_string();
    }
    self.keybinding_editor_field = 0;
    self.keybinding_capturing = true;
    self.keybinding_detected = false;
    self.success(tr(self.lang, "control_center.keybindings_press_key").to_string());
  }

  pub fn is_keybinding_capturing(&self) -> bool {
    self.keybinding_capturing
  }

  pub fn has_keybinding_conflict(&self) -> bool {
    self.keybinding_conflict.is_some()
  }

  pub fn keybinding_conflict_state(&self) -> Option<(&str, &str, Vec<String>)> {
    self
      .keybinding_conflict
      .as_ref()
      .map(|(id, keys, conflicts)| (id.as_str(), keys.as_str(), conflicts.clone()))
  }

  pub fn keybinding_label_for(&self, id: &str) -> String {
    self
      .keybindings
      .iter()
      .find(|binding| binding.id == id)
      .map(|binding| keybinding_label(self.lang, binding))
      .unwrap_or_else(|| id.to_string())
  }

  pub fn cancel_keybinding_conflict(&mut self) {
    self.keybinding_conflict = None;
  }

  pub fn replace_keybinding_conflict(&mut self) {
    let Some((id, keys, conflicts)) = self.keybinding_conflict.take() else {
      return;
    };
    for old_id in conflicts {
      let _ = keybindings::save_override(&old_id, None, Some(false), &self.keybindings);
    }
    if let Err(error) = keybindings::save_override(&id, Some(keys), Some(true), &self.keybindings) {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.success(tr(self.lang, "control_center.keybindings_replaced").to_string());
    if self.page() == Page::KeybindingCapture {
      self.navigation.back();
    }
    self.navigation.back();
  }

  pub fn capture_keybinding(&mut self, key: crossterm::event::KeyEvent) {
    if key.code == crossterm::event::KeyCode::Esc {
      self.keybinding_capturing = false;
      self.keybinding_detected = false;
      return;
    }
    if self.keybinding_detected {
      if key.code == crossterm::event::KeyCode::Enter {
        self.apply_keybinding_editor();
      }
      return;
    }
    if key.code == crossterm::event::KeyCode::Enter && self.keybinding_editor_key.is_empty() {
      self.keybinding_editor_key = "Return".into();
    } else if key.code == crossterm::event::KeyCode::Tab && self.keybinding_editor_key.is_empty() {
      self.keybinding_editor_key = "Tab".into();
    } else if key.code == crossterm::event::KeyCode::Backspace
      && self.keybinding_editor_key.is_empty()
    {
      self.keybinding_editor_key = "BackSpace".into();
    } else {
      self.capture_keybinding_key(key);
    }
    if !self.keybinding_editor_key.is_empty() {
      self.keybinding_detected = true;
      self.keybinding_capturing = false;
    }
  }

  pub(super) fn capture_keybinding_key(&mut self, key: crossterm::event::KeyEvent) {
    if key.code == crossterm::event::KeyCode::Enter {
      self.apply_keybinding_editor();
      return;
    }
    self.keybinding_editor_modifiers = [
      key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL),
      key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
      key
        .modifiers
        .contains(crossterm::event::KeyModifiers::SHIFT),
      self.keybinding_super_required
        || key
          .modifiers
          .contains(crossterm::event::KeyModifiers::SUPER),
    ];
    let raw = match key.code {
      crossterm::event::KeyCode::Char(c) => match c {
        '?' => "/".into(),
        '_' => "-".into(),
        '+' => "=".into(),
        ':' => ";".into(),
        '"' => "'".into(),
        '{' => "[".into(),
        '}' => "]".into(),
        '|' => "\\".into(),
        '~' => "`".into(),
        '<' => ",".into(),
        '>' => ".".into(),
        other => other.to_string(),
      },
      crossterm::event::KeyCode::Enter => "Return".into(),
      crossterm::event::KeyCode::Tab => "Tab".into(),
      crossterm::event::KeyCode::Backspace => "BackSpace".into(),
      crossterm::event::KeyCode::Left => "left".into(),
      crossterm::event::KeyCode::Right => "right".into(),
      crossterm::event::KeyCode::Up => "up".into(),
      crossterm::event::KeyCode::Down => "down".into(),
      crossterm::event::KeyCode::Esc => "Escape".into(),
      crossterm::event::KeyCode::Delete => "Delete".into(),
      crossterm::event::KeyCode::Insert => "Insert".into(),
      crossterm::event::KeyCode::Home => "Home".into(),
      crossterm::event::KeyCode::End => "End".into(),
      crossterm::event::KeyCode::PageUp => "Page_Up".into(),
      crossterm::event::KeyCode::PageDown => "Page_Down".into(),
      crossterm::event::KeyCode::F(n) => format!("F{n}"),
      _ => return,
    };
    self.keybinding_editor_key = raw;
  }

  pub(super) fn apply_keybinding_editor(&mut self) {
    let mut parts = Vec::new();
    for (enabled, modifier) in self
      .keybinding_editor_modifiers
      .into_iter()
      .zip(["CTRL", "ALT", "SHIFT", "SUPER"])
    {
      if enabled {
        parts.push(modifier);
      }
    }
    if self.keybinding_editor_key.is_empty() {
      return;
    }
    parts.push(self.keybinding_editor_key.as_str());
    let Ok(keys) = keybindings::normalize_keys(&parts.join(" + ")) else {
      return;
    };
    let binding = self.edited_binding().or_else(|| self.selected_keybinding());
    let Some(binding) = binding else {
      return;
    };
    let conflicts = keybindings::conflicts_in_context(
      &self.keybindings,
      &binding.id,
      &keys,
      &binding.context,
      &binding.input,
    );
    if !conflicts.is_empty() {
      self.keybinding_conflict = Some((binding.id.clone(), keys, conflicts));
      self.confirm_focus = ConfirmState::new();
      self.keybinding_capturing = false;
      return;
    }
    let id = binding.id.clone();
    if let Err(error) = keybindings::save_override(&id, Some(keys), Some(true), &self.keybindings) {
      self.fail(error);
      return;
    }
    self.keybindings = keybindings::load();
    self.keybinding_capturing = false;
    self.success(tr(self.lang, "control_center.keybindings_applied").to_string());
    if self.page() == Page::KeybindingCapture {
      self.navigation.back();
    }
    self.navigation.back();
  }

  pub fn keybinding_editor_state(&self) -> ([bool; 4], &str, usize) {
    (
      self.keybinding_editor_modifiers,
      &self.keybinding_editor_key,
      self.keybinding_editor_field,
    )
  }
}

pub(super) fn keybinding_is_cheatsheet_entry(binding: &keybindings::Binding) -> bool {
  !matches!(
    binding.id.as_str(),
    "resize.right"
      | "resize.left"
      | "resize.up"
      | "resize.down"
      | "resize.move_right"
      | "resize.move_left"
      | "resize.move_up"
      | "resize.move_down"
      | "resize.cancel_escape"
      | "resize.cancel_return"
  )
}

pub(super) fn keybinding_section(lang: Lang, binding: &keybindings::Binding) -> String {
  let software = matches!(
    binding.id.as_str(),
    id if id.starts_with("app.")
      || matches!(
        id,
        "system.kitty_cheatsheet"
          | "system.hyprland_cheatsheet"
          | "system.clipboard"
          | "system.clipboard_clear"
          | "system.color_picker"
          | "system.emoji_picker"
          | "system.control_center"
      )
  );
  let media = matches!(
    binding.id.as_str(),
    "session.volume_up"
      | "session.volume_down"
      | "session.mute"
      | "session.brightness_up"
      | "session.brightness_down"
      | "session.play_pause"
      | "session.next_track"
      | "session.previous_track"
      | "session.stop_track"
  );
  let section_key = if media {
    "control_center.keybindings.section.media"
  } else if software {
    "control_center.keybindings.section.software"
  } else {
    "control_center.keybindings.section.desktop"
  };
  tr(lang, section_key).to_string()
}

pub(super) fn keybinding_section_order(binding: &keybindings::Binding) -> usize {
  if matches!(
    binding.id.as_str(),
    "session.volume_up"
      | "session.volume_down"
      | "session.mute"
      | "session.brightness_up"
      | "session.brightness_down"
      | "session.play_pause"
      | "session.next_track"
      | "session.previous_track"
      | "session.stop_track"
  ) {
    return 2;
  }
  if binding.id.starts_with("app.")
    || matches!(
      binding.id.as_str(),
      "system.kitty_cheatsheet"
        | "system.hyprland_cheatsheet"
        | "system.clipboard"
        | "system.clipboard_clear"
        | "system.color_picker"
        | "system.emoji_picker"
        | "system.control_center"
    )
  {
    1
  } else {
    0
  }
}

pub(super) fn keybinding_label(lang: Lang, binding: &keybindings::Binding) -> String {
  if let Some(description) = keybindings::cheatsheet_description(lang, binding) {
    return description;
  }
  let translated = tr(lang, &binding.description_key);
  if translated != binding.description_key {
    translated.to_string()
  } else {
    let label = binding
      .id
      .rsplit('.')
      .next()
      .unwrap_or(&binding.id)
      .split('_')
      .map(|part| {
        let mut chars = part.chars();
        match chars.next() {
          Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
          None => String::new(),
        }
      })
      .collect::<Vec<_>>()
      .join(" ");
    match label.as_str() {
      "Terminal" => "Open terminal".into(),
      "Browser" => "Open default browser".into(),
      "File Manager" => "Open file manager".into(),
      "Launcher" => "Open application launcher".into(),
      "Close" => "Close window".into(),
      "Maximize" => "Maximize window".into(),
      "Next" => "Next workspace".into(),
      "Previous" => "Previous workspace".into(),
      _ => label,
    }
  }
}

pub(super) fn keybinding_category(lang: Lang, category: &str) -> String {
  let key = format!("control_center.keybindings.category.{category}");
  let translated = tr(lang, &key);
  if translated == key {
    category.to_string()
  } else {
    translated.to_string()
  }
}

pub(super) fn keybinding_category_order(category: &str) -> usize {
  [
    "applications",
    "windows",
    "navigation",
    "workspaces",
    "system",
    "session",
    "media",
    "screenshots",
    "appearance",
    "widgets",
  ]
  .iter()
  .position(|value| *value == category)
  .unwrap_or(usize::MAX)
}
