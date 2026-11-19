//! Header rendering of the About page.
use argvus_tui::chrome::draw_control_center_header;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::i18n::tr;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  let title = breadcrumb(app);
  draw_control_center_header(
    frame,
    area,
    &app.theme,
    &title,
    app
      .is_loaded()
      .then(|| (tr(app.lang, "control_center.version"), app.argvus_version())),
  );
}

/// Builds `Tools & Diagnostics > About ARGVUS > <tab>`.
///
/// The card and item keys are the ones `home/catalog.rs` uses for the About entry, so the
/// path matches the Home from every entry point (Home, search and CLI route).
pub fn breadcrumb(app: &App) -> String {
  let lang = app.lang;
  format!(
    "{} > {} > {}",
    tr(lang, "control_center.card_tools"),
    tr(lang, "control_center.about_argvus"),
    app.active_tab.label(lang),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::Tab;
  use crate::i18n::Lang;

  #[test]
  fn breadcrumb_shows_home_card_item_and_tab() {
    let mut app = App::test();
    app.lang = Lang::for_locale("en-US");
    assert_eq!(
      breadcrumb(&app),
      "Tools & Diagnostics > About ARGVUS > System & Hardware"
    );
    app.goto_tab(Tab::Credits);
    assert!(breadcrumb(&app).ends_with("> About ARGVUS > Credits"));
  }

  #[test]
  fn breadcrumb_is_translated() {
    let app = App::test();
    assert_eq!(
      breadcrumb(&app),
      "Ferramentas e diagnóstico > Sobre o ARGVUS > Sistema e hardware"
    );
  }
}
