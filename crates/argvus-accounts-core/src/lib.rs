//! `argvus-accounts-core` — account and avatar management for the Argvus desktop.
//!
//! The crate is the single source of truth for Argvus account metadata:
//!
//! - user/group discovery via NSS (`getpwnam`, `getgrent`, `getgrouplist`);
//! - display names stored in the standard GECOS full-name field;
//! - avatars stored as normalized 256x256 PNG files at `$HOME/.face`
//!   (mode 0644, owned by the user, deployed atomically);
//! - an authorization layer ([`permissions::AuthorizationProvider`]) ready for
//!   future polkit integration.
//!
//! # Consuming the avatar (e.g. from a greeter)
//!
//! ```no_run
//! use argvus_accounts_core::{get_user_by_name, avatar};
//!
//! # fn main() -> argvus_accounts_core::Result<()> {
//! let user = get_user_by_name("ghost")?.expect("user exists");
//! if let Some(path) = avatar::find_avatar(&user.home) {
//!     let bytes = std::fs::read(path)?;
//!     // `bytes` is always a regular PNG image, at most 256x256.
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

pub mod account;
pub mod autologin;
pub mod avatar;
pub mod error;
pub mod groups;
pub mod metadata;
pub mod passwd;
pub mod password;
pub mod permissions;
pub mod validation;

pub use account::{AccountManager, PrivilegeOps, ShadowOps, UserView};
pub use avatar::{AVATAR_FILENAME, avatar_path, find_avatar};
pub use error::{Error, Result, SystemSource};
pub use passwd::UserInfo;
pub use permissions::{
  Action, ActorIdentity, AllowAllProvider, AuthContext, AuthorizationProvider,
  UnixAuthorizationProvider,
};

/// Looks up a local user by name through NSS. Re-exported for convenience.
pub fn get_user_by_name(name: &str) -> Result<Option<UserInfo>> {
  passwd::get_user_by_name(name)
}
pub mod admin;
