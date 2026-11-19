//! Branded title for the main pages of the time popups.

use gtk::prelude::*;

/// Switches between the application name and a contextual page title.
pub struct PopupTitle {
  pub widget: gtk::Stack,
  page: gtk::Label,
}

impl PopupTitle {
  pub fn new(name: &str) -> Self {
    let branded = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    branded.set_direction(gtk::TextDirection::Ltr);
    branded.set_halign(gtk::Align::Start);
    branded.set_valign(gtk::Align::Center);
    branded.add_css_class("popup-title");
    branded.add_css_class("popup-branded-title");

    let brand = gtk::Label::new(Some("ARGVUS"));
    brand.add_css_class("popup-brand");
    branded.append(&brand);
    branded.append(&gtk::Label::new(Some(name)));

    let page = gtk::Label::builder()
      .xalign(0.0)
      .css_classes(["popup-title"])
      .build();
    let widget = gtk::Stack::builder()
      .hexpand(true)
      .hhomogeneous(false)
      .vhomogeneous(false)
      .build();
    widget.add_named(&branded, Some("application"));
    widget.add_named(&page, Some("page"));
    Self { widget, page }
  }

  pub fn set_page(&self, title: Option<&str>) {
    if let Some(title) = title {
      self.page.set_label(title);
      self.widget.set_visible_child_name("page");
    } else {
      self.widget.set_visible_child_name("application");
    }
  }
}
