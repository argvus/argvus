use argvus_i18n::{I18n, TextDirection};
use std::collections::HashMap;

const MESSAGES: &[(&str, &str)] = &[
  ("title", "Disk Usage Analyzer"),
  ("directory", "Directory"),
  ("browse", "Browse"),
  (
    "picker_help",
    "Space: select current directory  •  q / Esc: cancel",
  ),
  ("browse_title", "Select a directory"),
  ("browse_error", "Cannot open directory chooser"),
  (
    "manager_no_selection",
    "The file manager cannot return a directory; enter the path manually",
  ),
  ("browsing", "Choose a directory in the open dialog"),
  ("option_one_file_system", "Stay on one filesystem"),
  ("option_exclude_caches", "Exclude cache directories"),
  ("option_exclude_kernfs", "Exclude virtual filesystems"),
  ("option_apparent_size", "Show apparent size"),
  ("option_si", "Use decimal units"),
  ("option_itemcount", "Show item counts"),
  ("options", "Options"),
  ("results", "Results"),
  ("analyze", "Analyze"),
  ("initial", "Select Analyze to scan the directory"),
  ("small", "Enlarge terminal to at least 40 × 22. Esc exits."),
  (
    "footer_main",
    "Tab: next  •  arrows: move  •  Enter: select  •  Esc: quit",
  ),
  ("footer_results", "Esc: back  •  d: delete  •  q: quit"),
  ("stopped", "Analysis stopped"),
  ("empty", "Enter a directory"),
  ("not_directory", "Path is not a directory"),
  ("inaccessible", "Cannot access directory"),
  ("unreadable", "Cannot read directory"),
  (
    "missing_ncdu",
    "Cannot start ncdu; install ncdu 2.9.2 or newer",
  ),
  ("child_exited", "ncdu exited"),
  ("kitty_exited", "Kitty exited"),
  (
    "missing_kitty",
    "Cannot start Kitty; install the kitty package",
  ),
];

pub struct Messages {
  translations: HashMap<&'static str, String>,
  right_to_left: bool,
}
impl Messages {
  pub fn load() -> Self {
    let catalog = I18n::new("du").ok();
    let values = MESSAGES
      .iter()
      .map(|&(key, fallback)| {
        let translated = catalog
          .as_ref()
          .map(|catalog| catalog.tr(key))
          .filter(|text| !text.is_empty() && text != key)
          .unwrap_or_else(|| fallback.to_owned());
        (key, translated)
      })
      .collect();
    Self {
      translations: values,
      right_to_left: catalog
        .as_ref()
        .is_some_and(|catalog| catalog.direction() == TextDirection::RightToLeft),
    }
  }
  pub fn get(&self, key: &'static str) -> &str {
    self
      .translations
      .get(key)
      .map(String::as_str)
      .unwrap_or(key)
  }
  pub fn is_rtl(&self) -> bool {
    self.right_to_left
  }
}
