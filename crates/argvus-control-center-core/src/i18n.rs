//! Internationalization integration.
pub use argvus_i18n::{Lang, na, tr};

pub fn label(key: &str) -> &'static str {
  tr(Lang::detect(), key)
}
