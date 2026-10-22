//! Command-line interface definition.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// `argvus-accounts` command line.
#[derive(Parser, Debug)]
#[command(
  name = "argvus accounts",
  version,
  about = "Local user account and avatar manager for the Argvus desktop",
  after_help = "Self-service: 'self' subcommands act on your own account. \
                  Privileged operations elevate automatically via polkit \
                  (pkexec); use sudo if you prefer a terminal prompt."
)]
pub struct Cli {
  /// Print additional diagnostic information to stderr.
  #[arg(long, global = true)]
  pub verbose: bool,

  #[command(subcommand)]
  pub command: Command,
}

/// Top-level subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
  /// Structured frontend API. Mutations require pkexec; requests use stdin.
  Manage,
  /// List local users.
  List {
    /// Include system accounts (uid < 1000).
    #[arg(long)]
    all: bool,
  },
  /// Show detailed information about a user.
  Show {
    /// Username to inspect.
    user: String,
  },
  /// Change a user's display name (never the Unix username).
  Name {
    /// Target username.
    user: String,
    /// New display name.
    #[arg(value_name = "DISPLAY_NAME")]
    name: String,
  },
  /// Set or remove a user's avatar.
  Avatar {
    /// Target username.
    user: String,
    /// Source image file (PNG, JPEG or WebP; format is auto-detected).
    #[arg(
      value_name = "IMAGE",
      required_unless_present = "remove",
      conflicts_with = "remove"
    )]
    image: Option<PathBuf>,
    /// Remove the current avatar instead of setting one.
    #[arg(long)]
    remove: bool,
  },
  /// Query or change group membership.
  Groups {
    /// Target username.
    user: String,
    /// Groups to add (administrative).
    #[arg(long, value_name = "GROUP")]
    add: Vec<String>,
    /// Groups to remove (administrative).
    #[arg(long, value_name = "GROUP")]
    remove: Vec<String>,
  },
  /// Change an account password.
  ///
  /// The current, new and confirmation passwords are read from standard
  /// input, one per line, and never appear in the argument list. Changing your
  /// own password requires the current one as proof. Administrators may reset
  /// another user's password; the current password line is then ignored and
  /// may be empty.
  Passwd {
    /// Target username.
    user: String,
  },
  /// Operate on your own account.
  #[command(name = "self")]
  SelfAccount {
    #[command(subcommand)]
    action: Option<SelfAction>,
  },
}

/// Self-service subcommands.
#[derive(Subcommand, Debug)]
pub enum SelfAction {
  /// Show your own account information.
  Show,
  /// Set or remove your own avatar.
  Avatar {
    /// Source image file (PNG, JPEG or WebP; format is auto-detected).
    #[arg(
      value_name = "IMAGE",
      required_unless_present = "remove",
      conflicts_with = "remove"
    )]
    image: Option<PathBuf>,
    /// Remove your current avatar instead of setting one.
    #[arg(long)]
    remove: bool,
  },
  /// Change your own display name (elevates automatically via polkit).
  Name {
    /// New display name.
    #[arg(value_name = "DISPLAY_NAME")]
    name: String,
  },
  /// List your own groups.
  Groups,
}
