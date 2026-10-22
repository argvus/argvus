//! Error types shared across the library.

use thiserror::Error;

/// Convenient result alias used throughout `argvus-accounts-core`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Source detail attached to [`Error::SystemOperationFailed`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SystemSource {
  /// An external command existed but terminated unsuccessfully.
  #[error("{command} failed ({status}): {stderr}")]
  CommandFailed {
    /// Program name that was executed.
    command: String,
    /// Exit status description.
    status: String,
    /// Trimmed standard error output.
    stderr: String,
  },
  /// An external command required by the operation was not found.
  #[error("required system tool not found: {0}")]
  CommandNotFound(String),
}

/// All errors produced by `argvus-accounts-core`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
  /// The requested user does not exist on the system.
  #[error("user not found: {0}")]
  UserNotFound(String),
  /// The requested group does not exist on the system.
  #[error("group not found: {0}")]
  GroupNotFound(String),
  /// A username failed validation.
  #[error("invalid username: '{0}'")]
  InvalidUsername(String),
  /// A group name failed validation.
  #[error("invalid group name: '{0}'")]
  InvalidGroupName(String),
  /// A display name failed validation.
  #[error("invalid display name: {0}")]
  InvalidDisplayName(String),
  /// The current identity is not allowed to perform the action.
  #[error("permission denied: {0}")]
  PermissionDenied(String),
  /// The supplied image is not a decodable picture.
  #[error("invalid image: {0}")]
  InvalidImage(String),
  /// The supplied image file exceeds the accepted size limit.
  #[error("image too large: {actual} bytes, maximum allowed is {max} bytes")]
  ImageTooLarge {
    /// Observed size in bytes.
    actual: u64,
    /// Configured maximum in bytes.
    max: u64,
  },
  /// The image uses a format that is not accepted as an avatar.
  #[error("unsupported image format: {0}")]
  UnsupportedImageFormat(String),
  /// No avatar is currently set for the user.
  #[error("avatar not found for user: {0}")]
  AvatarNotFound(String),
  /// A membership insertion was requested for an existing member.
  #[error("user '{user}' is already a member of group '{group}'")]
  AlreadyMember {
    /// Target username.
    user: String,
    /// Target group.
    group: String,
  },
  /// A membership removal targeted a non-member.
  #[error("user '{user}' is not a member of group '{group}'")]
  NotMember {
    /// Target username.
    user: String,
    /// Target group.
    group: String,
  },
  /// Misuse of the API or contradictory request parameters.
  #[error("invalid operation: {0}")]
  InvalidOperation(String),
  /// A chosen password failed validation.
  #[error("invalid password: {0}")]
  InvalidPassword(String),
  /// A system-level operation (NSS, external tools) went wrong.
  #[error("{context}")]
  SystemOperationFailed {
    /// Human-readable context of the failure.
    context: String,
    /// Optional lower-level detail.
    #[source]
    source: Option<SystemSource>,
  },
  /// The target user has no usable home directory.
  #[error("home directory missing or unusable for user: {0}")]
  HomeDirUnusable(String),
  /// Unwrapped I/O failure.
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  /// Image decoding/encoding failure.
  #[error("image processing failed: {0}")]
  Image(#[from] image::ImageError),
}

impl Error {
  /// Builds a [`Error::SystemOperationFailed`] without a nested source.
  pub fn system(context: impl Into<String>) -> Self {
    Error::SystemOperationFailed {
      context: context.into(),
      source: None,
    }
  }

  /// Builds a [`Error::SystemOperationFailed`] for a failing external tool.
  pub fn command_failed(command: &str, status: &str, stderr: &str) -> Self {
    let mut context = format!("external tool '{command}' failed ({status})");
    let stderr = stderr.trim();
    if !stderr.is_empty() {
      context.push_str(": ");
      context.push_str(stderr);
    }
    Error::SystemOperationFailed {
      context,
      source: Some(SystemSource::CommandFailed {
        command: command.to_string(),
        status: status.to_string(),
        stderr: stderr.trim().to_string(),
      }),
    }
  }

  /// Builds a [`Error::SystemOperationFailed`] for a missing external tool.
  pub fn command_not_found(tool: &str) -> Self {
    Error::SystemOperationFailed {
      context: "cannot run required system tool".to_string(),
      source: Some(SystemSource::CommandNotFound(tool.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn messages_are_user_facing() {
    let err = Error::UserNotFound("ghost".to_string());
    assert_eq!(err.to_string(), "user not found: ghost");

    let err = Error::AlreadyMember {
      user: "ghost".into(),
      group: "wheel".into(),
    };
    assert_eq!(
      err.to_string(),
      "user 'ghost' is already a member of group 'wheel'"
    );
  }
}
