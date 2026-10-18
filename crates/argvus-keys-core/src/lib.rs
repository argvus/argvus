//! Keys for ARGVUS: the SSH key pairs (names, OpenSSH output and the key store)
//! and the GPG keys (rules, gpg operations).
//!
//! Passphrases never pass through this crate or any process argument: when a
//! step needs one, `ssh-keygen` or `ssh-add` asks on the terminal it runs in.

pub mod gpg;
pub mod gpg_ops;
pub mod keys;
pub mod store;

pub use keys::{KeyRow, KeyType, Loaded, valid_comment, valid_name};
pub use store::{KeyError, agent_socket, ssh_dir};
