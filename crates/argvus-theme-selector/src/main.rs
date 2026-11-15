//! Independent visual theme selector. Theme application stays in argvus config.
mod model;
mod preview;
mod render;
mod ui;

use argvus_i18n::I18n;
use gtk::{glib, prelude::*};
use std::sync::Arc;

fn main() -> glib::ExitCode {
  if std::env::args()
    .skip(1)
    .any(|argument| argument != "--help")
  {
    eprintln!("usage: argvus theme-selector");
    return glib::ExitCode::FAILURE;
  }
  if std::env::args().any(|argument| argument == "--help") {
    println!("usage: argvus theme-selector");
    return glib::ExitCode::SUCCESS;
  }
  let i18n = match I18n::new("appearance") {
    Ok(i18n) => Arc::new(i18n),
    Err(error) => {
      eprintln!("argvus-theme-selector: {error}");
      return glib::ExitCode::FAILURE;
    }
  };
  let application = gtk::Application::builder()
    .application_id("sh.argvus.ThemeSelector")
    .build();
  application.connect_activate(move |app| {
    if let Some(window) = app.active_window() {
      window.present();
      ui::refocus(&window);
    } else {
      ui::open(app, Arc::clone(&i18n));
    }
  });
  application.run_with_args::<&str>(&[])
}
