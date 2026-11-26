//! A native GTK file chooser for the embedded Calendar. It runs without a
//! popup session, so the compositor can focus it as an independent window.

use gtk::{gio, glib, prelude::*};

use crate::config::Settings;
use crate::error::{ArgvusError, Result};
use crate::i18n::{I18n, Language};

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum FilePicker {
  Import,
  Export,
}

impl FilePicker {
  pub fn run(self, settings: &Settings) -> Result<()> {
    gtk::init().map_err(|error| ArgvusError::Service(error.to_string()))?;

    let language = Language::resolve(&settings.locale.language);
    gtk::Widget::set_default_direction(if language.is_rtl() {
      gtk::TextDirection::Rtl
    } else {
      gtk::TextDirection::Ltr
    });
    let i18n = I18n::new(language);
    let title = match self {
      Self::Import => i18n.extra("morphing.import"),
      Self::Export => i18n.extra("morphing.export"),
    };
    let dialog = gtk::FileDialog::builder().title(title).modal(false).build();
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("iCalendar (*.ics)"));
    filter.add_suffix("ics");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(Some(&filter));
    let parent = None::<&gtk::Window>;
    let selection = match self {
      Self::Import => dialog.open_future(parent),
      Self::Export => {
        dialog.set_initial_name(Some("calendar.ics"));
        dialog.save_future(parent)
      }
    };
    match glib::MainContext::default().block_on(selection) {
      Ok(file) => println!("{}", file.uri()),
      Err(error) if error.matches(gtk::DialogError::Dismissed) => {}
      Err(error) => return Err(ArgvusError::Service(error.to_string())),
    }
    Ok(())
  }
}
