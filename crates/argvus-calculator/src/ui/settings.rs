//! Profile selection and keyboard-first settings.
use super::*;

impl Calculator {
  pub(super) fn rebuild_profiles(&self) {
    clear_list(&self.profiles);
    let active = self.preferences.borrow().profile;
    for profile in Profile::ALL {
      append_row(
        &self.profiles,
        &format!(
          "[{}] {}",
          if active == profile { "x" } else { " " },
          text(profile.key())
        ),
      );
    }
  }
  pub(super) fn rebuild_settings(&self) {
    let selected = self.settings.selected_row().map_or(0, |row| row.index());
    clear_list(&self.settings);
    let preferences = self.preferences.borrow();
    let mut actions = Vec::new();
    let mut add = |action, label: String| {
      append_row(&self.settings, &label);
      actions.push(action);
    };
    add(
      Action::Profile,
      format!("{}: {}", text("profile"), text(preferences.profile.key())),
    );
    match preferences.profile {
      Profile::Scientific => add(
        Action::Radians,
        format!(
          "{}: {}",
          text("angle.units"),
          text(if preferences.radians {
            "angle.radians"
          } else {
            "angle.degrees"
          })
        ),
      ),
      Profile::Programming => {
        add(
          Action::Base,
          format!(
            "{}: {}",
            text("program.base"),
            preferences.programming.base.radix()
          ),
        );
        add(
          Action::Bits,
          format!("{}: {}", text("program.bits"), preferences.programming.bits),
        );
        add(
          Action::Signed,
          text(if preferences.programming.signed {
            "program.signed"
          } else {
            "program.unsigned"
          })
          .into(),
        );
      }
      Profile::Financial => add(Action::Financial, text(preferences.financial.key()).into()),
      Profile::Conversion => {
        add(Action::Category, text(preferences.category.key()).into());
        add(
          Action::From,
          format!(
            "{}: {}",
            text("conversion.from"),
            preferences.category.units()[preferences.from].symbol
          ),
        );
        add(
          Action::To,
          format!(
            "{}: {}",
            text("conversion.to"),
            preferences.category.units()[preferences.to].symbol
          ),
        );
      }
      Profile::Basic => {}
    }
    add(Action::Copy, text("action.copy").into());
    add(Action::History, text("history").into());
    *self.actions.borrow_mut() = actions;
    if let Some(row) = self.settings.row_at_index(selected) {
      self.settings.select_row(Some(&row));
      if self.pages.visible_child_name().as_deref() == Some("menu") {
        row.grab_focus();
      }
    }
  }
  pub(super) fn activate(self: &Rc<Self>, action: Action) {
    let mut preferences = self.preferences.borrow().clone();
    let word = if matches!(action, Action::Base | Action::Bits | Action::Signed)
      && !self.entry.text().is_empty()
    {
      match programming::word(&self.entry.text(), preferences.programming) {
        Ok(value) => Some(value),
        Err(_) => {
          self.error("error.syntax");
          return;
        }
      }
    } else {
      None
    };
    match action {
      Action::Profile => {
        self.navigate("profiles");
        return;
      }
      Action::Copy => {
        self.entry.clipboard().set_text(&self.entry.text());
        return;
      }
      Action::History => {
        self.navigate("history");
        return;
      }
      Action::Radians => preferences.radians = !preferences.radians,
      Action::Base => {
        preferences.programming.base = match preferences.programming.base {
          programming::Base::Binary => programming::Base::Octal,
          programming::Base::Octal => programming::Base::Decimal,
          programming::Base::Decimal => programming::Base::Hexadecimal,
          programming::Base::Hexadecimal => programming::Base::Binary,
        };
      }
      Action::Bits => {
        preferences.programming.bits = match preferences.programming.bits {
          8 => 16,
          16 => 32,
          32 => 64,
          _ => 8,
        }
      }
      Action::Signed => preferences.programming.signed = !preferences.programming.signed,
      Action::Financial => {
        preferences.financial = cycle(preferences.financial, &financial::Operation::ALL)
      }
      Action::Category => {
        preferences.category = cycle(preferences.category, &conversion::Category::ALL);
        preferences.from = 0;
        preferences.to = 1;
      }
      Action::From => {
        preferences.from = (preferences.from + 1) % preferences.category.units().len()
      }
      Action::To => preferences.to = (preferences.to + 1) % preferences.category.units().len(),
    }
    if let Some(word) = word {
      self.entry.set_text(&preferences.programming.format(word));
      self.entry.set_position(-1);
    }
    self.apply_preferences(preferences);
  }
}
