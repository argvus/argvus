//! Implements `donate` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::app::App;
use crate::i18n::tr;

use super::{DONATE_URL, Doc, Row, simple_doc};

/// Executes the `doc` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn doc(app: &App, width: usize, selected: usize) -> Doc<'static> {
  let lang = app.lang;
  let rows = vec![
    Row::Section(tr(lang, "control_center.support_the_development").to_string()),
    Row::Para(
      tr(
        lang,
        "control_center.argvus_is_a_free_and_open_source_project_if_you_like_the_desktop_consi",
      )
      .to_string(),
    ),
    Row::Spacer,
    Row::Link {
      label: DONATE_URL.to_string(),
      url: DONATE_URL.to_string(),
    },
    Row::Spacer,
    Row::Para(tr(lang, "control_center.thank_you_for_your_care_and_support").to_string()),
  ];

  simple_doc(&rows, &app.theme, width, selected)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::App;

  #[test]
  /// Executes the `donate_has_link_to_support` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn donate_has_link_to_support() {
    let app = App::test();
    let doc = doc(&app, 100, 0);
    assert!(doc.actions.iter().any(|a| a.url == DONATE_URL));
  }
}
