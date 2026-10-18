//! Translation loading and normalization of greetd prompt messages.
//!
//! greetd messages originate outside the application and vary slightly by
//! PAM/backend implementation. Normalizing known messages here keeps the UI
//! independent from English prompt wording while preserving unknown text.

use anyhow::Result;
use argvus_i18n::I18n;
use std::sync::Arc;

/// Loads the shared `greeter` translation catalog once for the application.
pub fn load() -> Result<Arc<I18n>> {
  Ok(Arc::new(I18n::new("greeter")?))
}

/// Converts common greetd/PAM messages into stable translation keys.
///
/// Unknown messages are returned unchanged because authentication backends may
/// provide useful administrator- or site-specific information.
pub fn auth_message(i18n: &I18n, message: &str) -> String {
  // Punctuation and case are presentation details, not part of the lookup
  // contract, so remove them before matching known phrases.
  let normalized = message
    .trim()
    .trim_end_matches([':', '.'])
    .trim()
    .to_lowercase();

  let key = match normalized.as_str() {
    "password" => Some("password"),
    "authentication failed" | "login incorrect" => Some("authentication_failed"),
    "no active authentication prompt" => Some("no_active_auth_prompt"),
    "greetd_sock is not set" => Some("greetd_socket_missing"),
    "greetd requested authentication after startsession" => {
      Some("authentication_requested_after_session")
    }
    _ => None,
  };

  key.map_or_else(|| message.to_owned(), |key| i18n.tr(key))
}
