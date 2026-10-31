//! Preference pages: default apps, fonts, time zone, locales, keyboard, language and input settings.

use super::menu::setting_values;
use super::*;

impl App {
  pub(super) fn apply_app(&mut self, category: Category, index: usize) {
    let Some((display, binary)) = self
      .apps
      .installed(category)
      .get(index)
      .map(|app| (app.display.clone(), app.binary.clone()))
    else {
      return;
    };
    match self.apps.set_default(category, &binary) {
      Ok(()) => self.success(format!(
        "{}: {display}",
        tr(self.lang, "control_center.default_application_changed")
      )),
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn apply_font(&mut self, target: crate::config::fonts::FontTarget, index: usize) {
    let Some(font) = self.system_fonts.get(index).cloned() else {
      return;
    };
    match self.fonts.apply_font(target, &font, self.pending_size) {
      Ok(()) => self.success(format!(
        "{}: {} {}",
        tr(self.lang, "control_center.font_applied"),
        font.display_name(),
        self.pending_size
      )),
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn apply_font_setting(&mut self, setting: SettingKind, index: usize) {
    let Some(value) = setting_values(setting).get(index) else {
      return;
    };
    match self.fonts.apply_setting(setting, value) {
      Ok(()) => self.success(tr(self.lang, "control_center.setting_applied").to_string()),
      Err(error) => self.fail(error),
    }
  }

  /// Enter on a zone: makes it the draft. `Apply` writes it.
  pub(super) fn select_timezone(&mut self, index: usize) {
    if let Some(zone) = self.timezones.get(index) {
      self.timezone_draft = zone.clone();
    }
  }

  /// `Apply` of Time Zone: writes the user TZ environment file, then reloads
  /// the saved zone so the draft matches what was written.
  pub(super) fn apply_timezone(&mut self) {
    let zone = self.timezone_draft.clone();
    match time::set_timezone(&zone, &self.timezones) {
      Ok(()) => {
        self.refresh_time();
        self.load_timezone_draft();
        self.success(format!(
          "{}: {zone}",
          tr(self.lang, "control_center.time_zone_changed")
        ));
      }
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn apply_regional_locale(&mut self, index: usize) {
    let Some(value) = self.generated_locales.get(index).cloned() else {
      return;
    };
    self.apply_user_language(&value);
  }

  fn apply_user_language(&mut self, value: &str) {
    match locale::set_lang(value, &self.generated_locales) {
      Ok(()) => {
        self.lang = Lang::for_locale(value);
        self.success(tr(self.lang, "control_center.user_locale_applied").to_owned());
      }
      Err(error) => {
        eprintln!("{error}");
        let key = match error {
          locale::UserLocaleError::NotGenerated(_) => "control_center.user_locale_not_generated",
          _ => "control_center.user_locale_save_failed",
        };
        self.fail(tr(self.lang, key));
      }
    }
  }

  /// Marks or unmarks one `/etc/locale.gen` entry; nothing is written until
  /// `Apply`.
  pub(super) fn toggle_system_locale(&mut self, index: usize) {
    if let Some(entry) = self.locale_gen_entries.get(index) {
      let key = format!("{} {}", entry.locale, entry.encoding);
      if !self.selected_locales.remove(&key) {
        self.selected_locales.insert(key);
      }
    }
  }

  /// Enter on a layout: makes it the default in the draft. `Apply` writes it.
  pub(super) fn apply_keyboard_layout(&mut self, index: usize) {
    let Some(code) = self
      .keyboard_layouts
      .get(index)
      .map(|layout| layout.code.clone())
    else {
      return;
    };
    self.keyboard_draft.set_default(&code);
  }

  /// Space on a layout: enables or disables it in the draft. `Apply` writes it.
  pub(super) fn toggle_keyboard_layout(&mut self, index: usize) {
    let Some(code) = self
      .keyboard_layouts
      .get(index)
      .map(|layout| layout.code.clone())
    else {
      return;
    };
    if let Err(message) = self.keyboard_draft.toggle(&code) {
      self.fail(message);
    }
  }

  /// `Apply` of Keyboard > Layout: writes the draft in one backend call, then
  /// reloads the state so the draft matches what was written.
  pub(super) fn apply_keyboard_layout_draft(&mut self) {
    let draft = self.keyboard_draft.clone();
    match keyboard::set_x11_layouts(&draft.default, &draft.selected, &self.keyboard_layouts) {
      Ok(()) => {
        self.refresh_keyboard();
        self.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&self.keyboard_info);
        self.success(tr(self.lang, "control_center.keyboard_layouts_updated").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn apply_keyboard_variant(&mut self, index: usize) {
    let Some(variant) = self.keyboard_variants.get(index).cloned() else {
      return;
    };
    let layout = self.keyboard_info.default_layout.clone();
    match keyboard::set_x11_variant(&layout, &variant.code, &self.keyboard_variants) {
      Ok(()) => {
        self.refresh_keyboard();
        self.success(format!(
          "{}: {}",
          tr(self.lang, "control_center.variant_changed"),
          variant.description
        ));
      }
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn apply_console_keymap(&mut self, index: usize) {
    let Some(keymap) = self.console_keymaps.get(index).cloned() else {
      return;
    };
    match keyboard::set_console_keymap(&keymap, &self.console_keymaps) {
      Ok(()) => {
        self.refresh_keyboard();
        self.success(format!(
          "{}: {keymap}",
          tr(self.lang, "control_center.console_keymap_changed")
        ));
      }
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn apply_language(&mut self, index: usize) {
    let Some((locale, _, _)) = LANGUAGES.get(index) else {
      return;
    };
    let locale = match *locale {
      "pt-BR" => "pt_BR.UTF-8",
      _ => "en_US.UTF-8",
    };
    self.apply_user_language(locale);
  }

  /// Enter/Space on a Mouse & Touchpad row: toggles flip, values step up.
  pub(super) fn apply_input(&mut self, item: Item) {
    match item {
      Item::Ratbag(_) => self.input_cycle(item, 1),
      Item::Input(index @ (3 | 5 | 7..=11)) => {
        self.input_toggle_pending = Some(index);
        self.status = Some(Status::new(
          tr(self.lang, "control_center.input_applying").to_string(),
          StatusKind::Success,
        ));
      }
      Item::Input(_) => self.input_cycle(item, 1),
      _ => {}
    }
  }

  /// `←/→` (and `h/l`) on a Mouse & Touchpad row.
  pub(crate) fn input_cycle(&mut self, item: Item, direction: i8) {
    let selected = match item {
      Item::Ratbag(RatbagRow::Device) => return self.cycle_ratbag_device(direction),
      Item::Ratbag(RatbagRow::Profile) => {
        return self.queue_ratbag(crate::system::ratbag::Change::Profile(direction));
      }
      Item::Ratbag(RatbagRow::Dpi) => {
        return self.queue_ratbag(crate::system::ratbag::Change::Dpi(direction));
      }
      Item::Ratbag(RatbagRow::ReportRate) => {
        return self.queue_ratbag(crate::system::ratbag::Change::ReportRate(direction));
      }
      Item::Input(index) => index,
      _ => return,
    };
    let direction = direction.signum();
    let pending = self.input_pending.take();
    self.input_pending = Some(match pending {
      Some((pending_selected, pending_direction, _)) if pending_selected == selected => (
        selected,
        pending_direction.saturating_add(direction),
        std::time::Instant::now() + std::time::Duration::from_millis(100),
      ),
      _ => (
        selected,
        direction,
        std::time::Instant::now() + std::time::Duration::from_millis(100),
      ),
    });
    self.status = Some(Status::new(
      tr(self.lang, "control_center.input_applying").to_string(),
      StatusKind::Success,
    ));
  }
}
