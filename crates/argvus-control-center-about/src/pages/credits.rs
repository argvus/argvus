//! Implements `credits` responsibilities in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::app::App;
use crate::i18n::tr;

use super::{ARGVUS_URL, Doc, Row, WILLIAM_CANIN_URL, simple_doc};

/// Executes the `doc` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn doc(app: &App, width: usize, selected: usize) -> Doc<'static> {
  let lang = app.lang;
  let rows = vec![
    Row::Section(tr(lang, "control_center.lead_developer").to_string()),
    Row::Para(
      tr(
        lang,
        "control_center.argvus_is_designed_and_developed_by_william_c_canin",
      )
      .to_string(),
    ),
    Row::Link {
      label: WILLIAM_CANIN_URL.to_string(),
      url: WILLIAM_CANIN_URL.to_string(),
    },
    Row::Spacer,
    Row::Section(tr(lang, "control_center.project").to_string()),
    Row::Link {
      label: ARGVUS_URL.to_string(),
      url: ARGVUS_URL.to_string(),
    },
    Row::Spacer,
    Row::Section(tr(lang, "control_center.contributions").to_string()),
    Row::Para(
      tr(
        lang,
        "control_center.we_thank_the_entire_argvus_community_and_all_contributors_who_helped_w",
      )
      .to_string(),
    ),
    Row::Spacer,
    Row::Section(tr(lang, "control_center.technologies").to_string()),
    Row::Para(
      tr(
        lang,
        "control_center.built_on_hyprland_wayland_quickshell_rust_vala_and_the_free_libraries_",
      )
      .to_string(),
    ),
  ];

  simple_doc(&rows, &app.theme, width, selected)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::App;

  #[test]
  /// Executes the `credits_have_author_and_project_links` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn credits_have_author_and_project_links() {
    let app = App::test();
    let doc = doc(&app, 100, 0);
    assert!(doc.actions.iter().any(|a| a.url == WILLIAM_CANIN_URL));
    assert!(doc.actions.iter().any(|a| a.url == ARGVUS_URL));
  }
}
