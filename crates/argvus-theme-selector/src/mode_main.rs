//! Visual Sticky/Spaced picker; application uses the shared mode backend.
use argvus_theme_selector::mode_ui as ui;

use argvus_i18n::I18n;
use gtk::{glib, prelude::*};
use std::sync::Arc;

fn main() -> glib::ExitCode {
  if std::env::args()
    .skip(1)
    .any(|argument| argument != "--help")
  {
    eprintln!("usage: argvus layout-mode-menu");
    return glib::ExitCode::FAILURE;
  }
  if std::env::args().any(|argument| argument == "--help") {
    println!("usage: argvus layout-mode-menu");
    return glib::ExitCode::SUCCESS;
  }
  let i18n = match I18n::new("appearance") {
    Ok(i18n) => Arc::new(i18n),
    Err(error) => {
      eprintln!("argvus-layout-mode-menu: {error}");
      return glib::ExitCode::FAILURE;
    }
  };
  let application = gtk::Application::builder()
    .application_id("sh.argvus.LayoutModeSelector")
    .build();
  application.connect_activate(move |app| {
    if let Some(window) = app.active_window() {
      window.present();
    } else {
      ui::open(app, Arc::clone(&i18n));
    }
  });
  application.run_with_args::<&str>(&[])
}
