//! Native ARGVUS calculator: profile engines are independent of the GTK UI.
mod conversion;
mod engine;
mod financial;
mod history;
mod preferences;
mod programming;
mod ui;

use gtk::prelude::*;
use relm4::gtk;
use std::{cell::RefCell, rc::Rc};

fn text(key: &str) -> &'static str {
  argvus_i18n::tr(
    argvus_i18n::Lang::for_domain(&argvus_i18n::user_locale(), "calculator"),
    key,
  )
}
fn main() -> gtk::glib::ExitCode {
  let application = gtk::Application::builder()
    .application_id("org.argvus.Calculator")
    .build();
  let retained = Rc::new(RefCell::new(None::<Rc<ui::Calculator>>));
  application.connect_activate(move |application| {
    if let Some(calculator) = retained
      .borrow()
      .as_ref()
      .filter(|calculator| calculator.window.is_visible())
    {
      calculator.window.present();
      return;
    }
    let calculator =
      ui::Calculator::new(application, history::path().ok(), preferences::path().ok());
    calculator.window.present();
    *retained.borrow_mut() = Some(calculator);
  });
  application.run()
}
