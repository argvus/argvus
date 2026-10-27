//! Connection profiles of NetworkManager as the editor sees them: the kinds,
//! the settings of each kind, the values being edited and their checks, and
//! the translation to and from `nmcli`. Nothing here runs a process.

mod draft;
mod kind;
pub mod nmcli;
pub mod schema;
pub mod validate;

pub use draft::{ConnectionDraft, DraftError, PeerField, SecretState, WireGuardPeer};
pub use kind::{ConnectionKind, KindGroup};
