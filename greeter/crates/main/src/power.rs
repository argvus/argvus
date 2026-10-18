//! Login-screen power actions backed by systemd-logind over D-Bus.
//!
//! Keeping these calls behind a small enum prevents UI code from depending on
//! D-Bus method names and gives failures a single typed error boundary.

use argvus_i18n::I18n;
use thiserror::Error;
use zbus::blocking::{Connection, Proxy};

/// Actions exposed by the Greeter power menu.
#[derive(Debug, Clone, Copy)]
pub enum PowerAction {
  Shutdown,
  Restart,
  Suspend,
}

impl PowerAction {
  /// Returns the localized label for this action.
  pub fn label(self, i18n: &I18n) -> String {
    match self {
      Self::Shutdown => i18n.tr("power.shutdown"),
      Self::Restart => i18n.tr("power.restart"),
      Self::Suspend => i18n.tr("power.suspend"),
    }
  }
}

/// Errors that can occur before logind receives a power request.
#[derive(Debug, Error)]
pub enum PowerError {
  #[error("could not connect to system bus: {0}")]
  Bus(#[from] zbus::Error),
}

/// Requests a system power transition through the system D-Bus.
///
/// The boolean argument asks logind to interactively authenticate according to
/// its policy when required; the Greeter does not implement authorization.
pub fn request(action: PowerAction) -> Result<(), PowerError> {
  // A system-bus connection is required because these operations affect the
  // whole machine rather than only the greeter session.
  let connection = Connection::system()?;
  // The well-known logind manager object owns the standard power methods.
  let proxy = Proxy::new(
    &connection,
    "org.freedesktop.login1",
    "/org/freedesktop/login1",
    "org.freedesktop.login1.Manager",
  )?;

  match action {
    PowerAction::Shutdown => proxy.call::<_, _, ()>("PowerOff", &(true))?,
    PowerAction::Restart => proxy.call::<_, _, ()>("Reboot", &(true))?,
    PowerAction::Suspend => proxy.call::<_, _, ()>("Suspend", &(true))?,
  }

  Ok(())
}
