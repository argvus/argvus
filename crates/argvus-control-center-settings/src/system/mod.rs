//! Module declarations for the `system` subsystem of the Settings pages.
pub mod command;
pub mod date;
pub mod fonts;
pub mod gpg_keys;
pub mod host;
pub mod input;
pub mod keybindings;
pub mod keyboard;
pub mod locale;
mod pacman_repositories;
pub mod privileged;
pub mod projects;
pub mod ratbag;
pub mod snippets;
pub mod ssh_keys;
pub mod time;
pub mod window_rules;

/// Reject option-like values before passing untrusted operands to system tools.
pub(super) fn validate_plain_arg(value: &str) -> Result<(), String> {
  if value.is_empty() || value.starts_with('-') || value.chars().any(char::is_control) {
    return Err("invalid command operand".into());
  }
  Ok(())
}
