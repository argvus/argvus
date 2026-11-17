//! Persistent, contextual history and reuse.
use super::*;

impl Calculator {
  pub(super) fn add_history(self: &Rc<Self>, record: &history::Record) {
    let profile = record
      .context
      .as_ref()
      .map_or(Profile::Basic, |context| context.preferences.profile);
    let description = record
      .context
      .as_ref()
      .map(|context| match profile {
        Profile::Financial => text(context.preferences.financial.key()).to_string(),
        Profile::Conversion => format!(
          "{} → {}",
          context
            .preferences
            .category
            .units()
            .get(context.preferences.from)
            .map_or("?", |unit| unit.symbol),
          context
            .preferences
            .category
            .units()
            .get(context.preferences.to)
            .map_or("?", |unit| unit.symbol)
        ),
        _ => String::new(),
      })
      .unwrap_or_default();
    let button = gtk::Button::with_label(&format!(
      "{} · {} · {} = {}",
      text(profile.key()),
      description,
      record.expression,
      record.result
    ));
    if let Some(child) = button.child()
      && let Ok(label) = child.downcast::<gtk::Label>()
    {
      label.set_wrap(true);
      label.set_max_width_chars(40);
    }
    button.set_hexpand(true);
    let weak = Rc::downgrade(self);
    let record = record.clone();
    button.connect_clicked(move |_| {
      if let Some(state) = weak.upgrade() {
        let mut preferences = record
          .context
          .as_ref()
          .map_or_else(Preferences::default, |context| context.preferences.clone());
        if preferences.profile == Profile::Conversion {
          std::mem::swap(&mut preferences.from, &mut preferences.to);
        }
        state.apply_preferences(preferences);
        if let Some(context) = &record.context {
          for (field, value) in state.parameters.iter().zip(&context.fields) {
            field.set_text(value);
          }
        }
        state.navigate("calculator");
        state.show_result(&record.result);
      }
    });
    self.history_box.prepend(&button);
    let mut child = self.history_box.first_child();
    let mut count = 0;
    while let Some(widget) = child {
      child = widget.next_sibling();
      count += 1;
      if count > 100 {
        self.history_box.remove(&widget);
      }
    }
  }
  pub(super) fn load_history(self: &Rc<Self>) {
    match self
      .history_path
      .as_deref()
      .map(history::load)
      .unwrap_or_else(|| Ok(Vec::new()))
    {
      Ok(records) => {
        for record in &records {
          self.add_history(record);
        }
        *self.records.borrow_mut() = records;
      }
      Err(error) => {
        *self.load_error.borrow_mut() = Some("error.history");
        self.storage_error("error.history", error);
      }
    }
  }
  pub(super) fn clear_history(&self) {
    if let Some(path) = &self.history_path
      && let Err(error) = history::save(path, &[])
    {
      self.storage_error("error.history", error);
      return;
    }
    self.records.borrow_mut().clear();
    while let Some(child) = self.history_box.first_child() {
      self.history_box.remove(&child);
    }
  }
}
