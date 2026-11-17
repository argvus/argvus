//! Profile dispatch and inline-result presentation.
use super::*;

impl Calculator {
  pub(super) fn calculate(self: &Rc<Self>) {
    let preferences = self.preferences.borrow().clone();
    if preferences.profile == Profile::Conversion && self.conversion_result.get() {
      return;
    }
    let expression = self.entry.text().to_string();
    let fields = std::array::from_fn(|index| self.parameters[index].text().to_string());
    let result = match preferences.profile {
      Profile::Basic | Profile::Scientific => engine::evaluate(
        &expression,
        if preferences.radians {
          Angle::Radians
        } else {
          Angle::Degrees
        },
      )
      .map(format_result),
      Profile::Programming => programming::evaluate(&expression, preferences.programming),
      Profile::Financial => {
        let operation = preferences.financial;
        let parse = |index: usize| engine::evaluate(&fields[index], Angle::Degrees);
        engine::evaluate(&expression, Angle::Degrees).and_then(|principal| {
          let rate = parse(0)?;
          let periods = if operation == financial::Operation::Change {
            1.0
          } else {
            parse(1)?
          };
          let payment =
            if [financial::Operation::Present, financial::Operation::Future].contains(&operation) {
              parse(2)?
            } else {
              0.0
            };
          financial::calculate(operation, principal, rate, periods, payment).map(format_result)
        })
      }
      Profile::Conversion => engine::evaluate(&expression, Angle::Degrees)
        .and_then(|value| {
          conversion::calculate(
            value,
            preferences.category,
            preferences.from,
            preferences.to,
          )
        })
        .map(format_result),
    };
    match result {
      Ok(result) => {
        self.show_result(&result);
        self
          .conversion_result
          .set(preferences.profile == Profile::Conversion);
        let record = history::Record {
          expression,
          result,
          context: Some(history::Context {
            preferences,
            fields,
          }),
        };
        self.add_history(&record);
        let mut records = self.records.borrow_mut();
        records.push(record);
        if records.len() > 100 {
          records.remove(0);
        }
        if let Some(path) = &self.history_path
          && let Err(error) = history::save(path, &records)
        {
          self.storage_error("error.history", error);
        }
      }
      Err(error) => self.error(match error {
        Error::Syntax => "error.syntax",
        Error::Domain => "error.domain",
        Error::Limit => "error.limit",
      }),
    }
  }
  pub(super) fn show_result(&self, result: &str) {
    self.entry.set_text(result);
    self.entry.grab_focus_without_selecting();
    self.entry.select_region(-1, -1);
    self.entry.set_position(-1);
  }
}
