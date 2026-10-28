//! Header rendering of the About page.
use argvus_tui::chrome::{Header, draw_header};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::i18n::tr;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
  let title = breadcrumb(app);
  draw_header(
    frame,
    area,
    &app.theme,
    Header {
      title: &title,
      // Hidden while loading, so the header never flashes "N/A".
      version: app.is_loaded().then(|| app.argvus_version()),
      version_label: tr(app.lang, "control_center.version"),
    },
  );
}

/// Builds `Control Center > Tools & Diagnostics > About ARGVUS > <tab>`.
///
/// The card and item keys are the ones `home/catalog.rs` uses for the About entry, so the
/// path matches the Home from every entry point (Home, search and CLI route).
pub fn breadcrumb(app: &App) -> String {
  let lang = app.lang;
  format!(
    "{} > {} > {} > {}",
    tr(lang, "control_center.argvus_control_center"),
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
      "Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware"
    );
    app.goto_tab(Tab::Credits);
    assert!(breadcrumb(&app).ends_with("> About ARGVUS > Credits"));
  }

  #[test]
  fn breadcrumb_is_translated() {
    let app = App::test();
    assert_eq!(
      breadcrumb(&app),
      "Central de Controle > Ferramentas e diagnóstico > Sobre o ARGVUS > Sistema e hardware"
    );
  }
}
