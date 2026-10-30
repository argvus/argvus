//! Help overlay text: the keys of the active page.

use super::*;

impl App {
  pub fn help_lines(&self) -> Vec<String> {
    let mut lines: Vec<&str> = vec![
      tr(self.lang, "control_center.global"),
      tr(self.lang, "control_center.q_quit"),
      tr(self.lang, "control_center.esc_back"),
      tr(self.lang, "control_center.close_help"),
      "",
    ];
    #[cfg(feature = "snapshot")]
    if self.route == Route::Snapshot {
      lines.push(self.snapshot.help_text());
    }
    if self.route == Route::Home {
      lines.extend([
        tr(self.lang, "control_center.menu"),
        tr(self.lang, "control_center.navigate"),
        tr(self.lang, "control_center.enter_open"),
        tr(self.lang, "control_center.search"),
        tr(self.lang, "control_center.s_configuration"),
      ]);
    }
    if self.route == Route::Config {
      lines.extend([
        tr(self.lang, "control_center.configuration"),
        tr(self.lang, "control_center.enter_space_toggle"),
        tr(self.lang, "control_center.esc_back_1dce0e"),
      ]);
    }
    if self.route == Route::Settings {
      lines.extend([
        tr(self.lang, "control_center.lists"),
        tr(self.lang, "control_center.navigate"),
        tr(self.lang, "control_center.enter_open_apply"),
        tr(self.lang, "control_center.search"),
      ]);
    }
    #[cfg(feature = "about")]
    if self.route == Route::About {
      lines.extend([
        tr(self.lang, "control_center.about_4cf69b"),
        tr(self.lang, "control_center.tabs"),
        tr(self.lang, "control_center.navigate_scroll"),
        tr(self.lang, "control_center.pgup_pgdn_scroll"),
        tr(self.lang, "control_center.enter_open_action"),
        tr(self.lang, "control_center.about_help_copy_report"),
      ]);
    }
    #[cfg(any(
      feature = "hardware",
      feature = "services",
      feature = "dev-services",
      feature = "network",
      feature = "audio",
      feature = "bluetooth",
      feature = "boot",
      feature = "packages",
      feature = "storage",
      feature = "diagnostics",
      feature = "power",
      feature = "session",
      feature = "displays"
    ))]
    {
      let mut domain = false;
      #[cfg(feature = "hardware")]
      {
        domain |= self.route == Route::Hardware;
      }
      #[cfg(feature = "services")]
      {
        domain |= self.route == Route::Services;
      }
      #[cfg(feature = "dev-services")]
      {
        domain |= self.route == Route::DevServices;
      }
      #[cfg(feature = "network")]
      {
        domain |= self.route == Route::Network;
      }
      #[cfg(feature = "audio")]
      {
        domain |= self.route == Route::Audio;
      }
      #[cfg(feature = "bluetooth")]
      {
        domain |= self.route == Route::Bluetooth;
      }
      #[cfg(feature = "boot")]
      {
        domain |= self.route == Route::Boot;
      }
      #[cfg(feature = "packages")]
      {
        domain |= self.route == Route::Packages;
      }
      #[cfg(feature = "storage")]
      {
        domain |= self.route == Route::Storage;
      }
      #[cfg(feature = "diagnostics")]
      {
        domain |= self.route == Route::Diagnostics;
      }
      #[cfg(feature = "power")]
      {
        domain |= self.route == Route::Power;
      }
      #[cfg(feature = "session")]
      {
        domain |= self.route == Route::Session;
      }
      #[cfg(feature = "displays")]
      {
        domain |= self.route == Route::Displays;
      }
      if domain {
        lines.extend([
          tr(self.lang, "control_center.navigate"),
          tr(self.lang, "control_center.enter_open"),
          tr(self.lang, "control_center.esc_back_bd56d7"),
          tr(self.lang, "control_center.r_refresh"),
        ]);
      }
    }
    lines.into_iter().map(str::to_string).collect()
  }
}
