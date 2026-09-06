mod i18n;
mod system;
mod theme;
mod ui;

use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("io.github.argvus.About")
        .build();

    app.connect_activate(ui::build);
    app.run()
}
