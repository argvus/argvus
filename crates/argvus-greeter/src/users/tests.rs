use super::*;
use std::fs::Permissions;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::sync::atomic::{AtomicU32, Ordering};

fn temp_accounts_dir(label: &str) -> PathBuf {
  static COUNTER: AtomicU32 = AtomicU32::new(0);
  let dir = std::env::temp_dir().join(format!(
    "argvus-greeter-avatar-{}-{}-{}",
    label,
    std::process::id(),
    COUNTER.fetch_add(1, Ordering::Relaxed),
  ));
  fs::create_dir_all(dir.join("icons")).unwrap();
  fs::create_dir_all(dir.join("users")).unwrap();
  dir
}

fn temp_home_dir(label: &str) -> PathBuf {
  let dir = std::env::temp_dir().join(format!(
    "argvus-greeter-home-{}-{}",
    label,
    std::process::id(),
  ));
  let _ = fs::remove_dir_all(&dir);
  fs::create_dir_all(&dir).unwrap();
  dir
}

fn running_as_root() -> bool {
  fs::metadata("/proc/self")
    .map(|meta| meta.uid())
    .unwrap_or(0)
    == 0
}

#[test]
fn avatar_found_in_icons_directory() {
  let dir = temp_accounts_dir("icons-hit");
  let icon = dir.join("icons").join("alice");
  fs::write(&icon, b"png-bytes").unwrap();

  assert_eq!(avatar_in_accounts(&dir, "alice"), Some(icon));
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_avatar_returns_none() {
  let dir = temp_accounts_dir("missing");
  assert_eq!(avatar_in_accounts(&dir, "carol"), None);
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn nonexistent_accounts_directory_returns_none() {
  assert_eq!(
    avatar_in_accounts(Path::new("/nonexistent/argvus-test"), "alice"),
    None
  );
}

#[test]
fn icon_reference_in_users_file_is_used_as_fallback() {
  let dir = temp_accounts_dir("icon-ref");
  let referenced = dir.join("custom-bob.png");
  fs::write(&referenced, b"png-bytes").unwrap();
  fs::write(
    dir.join("users").join("bob"),
    format!(
      "[User]\nIcon={}\nSystemAccount=false\n",
      referenced.display()
    ),
  )
  .unwrap();

  assert_eq!(avatar_in_accounts(&dir, "bob"), Some(referenced));
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn icon_reference_to_missing_file_is_ignored() {
  let dir = temp_accounts_dir("icon-ref-missing");
  fs::write(
    dir.join("users").join("bob"),
    "[User]\nIcon=/does/not/exist.png\n",
  )
  .unwrap();

  assert_eq!(avatar_in_accounts(&dir, "bob"), None);
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn empty_or_malformed_icon_lines_are_ignored() {
  let dir = temp_accounts_dir("icon-ref-empty");
  let referenced = dir.join("custom-carol.png");
  fs::write(&referenced, b"png-bytes").unwrap();
  fs::write(
    dir.join("users").join("carol"),
    format!("Icon=   \nIcon={}\n", referenced.display()),
  )
  .unwrap();

  assert_eq!(avatar_in_accounts(&dir, "carol"), Some(referenced));
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn directory_instead_of_icon_file_is_ignored() {
  let dir = temp_accounts_dir("icon-dir");
  fs::create_dir_all(dir.join("icons").join("dave")).unwrap();

  assert_eq!(avatar_in_accounts(&dir, "dave"), None);
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unreadable_icon_file_falls_back_to_icon_reference() {
  if running_as_root() {
    return;
  }
  let dir = temp_accounts_dir("unreadable-icon");
  let unreadable = dir.join("icons").join("erin");
  fs::write(&unreadable, b"png-bytes").unwrap();
  fs::set_permissions(&unreadable, Permissions::from_mode(0o000)).unwrap();

  let referenced = dir.join("custom-erin.png");
  fs::write(&referenced, b"png-bytes").unwrap();
  fs::write(
    dir.join("users").join("erin"),
    format!("[User]\nIcon={}\n", referenced.display()),
  )
  .unwrap();

  assert_eq!(avatar_in_accounts(&dir, "erin"), Some(referenced));
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unreadable_users_file_returns_none() {
  if running_as_root() {
    return;
  }
  let dir = temp_accounts_dir("unreadable-users");
  let users_file = dir.join("users").join("frank");
  fs::write(&users_file, "[User]\nIcon=/tmp/x.png\n").unwrap();
  fs::set_permissions(&users_file, Permissions::from_mode(0o000)).unwrap();

  assert_eq!(avatar_in_accounts(&dir, "frank"), None);
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn usernames_with_valid_special_characters_resolve() {
  let dir = temp_accounts_dir("special-chars");
  let icon = dir.join("icons").join("user.name-1_x");
  fs::write(&icon, b"png-bytes").unwrap();

  assert_eq!(avatar_in_accounts(&dir, "user.name-1_x"), Some(icon));
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn switching_users_resolves_distinct_avatars() {
  let dir = temp_accounts_dir("switch");
  let alice_icon = dir.join("icons").join("alice");
  let bob_icon = dir.join("icons").join("bob");
  fs::write(&alice_icon, b"alice-png").unwrap();
  fs::write(&bob_icon, b"bob-png").unwrap();

  assert_eq!(avatar_in_accounts(&dir, "alice"), Some(alice_icon));
  assert_eq!(avatar_in_accounts(&dir, "bob"), Some(bob_icon));
  assert_ne!(
    avatar_in_accounts(&dir, "alice"),
    avatar_in_accounts(&dir, "bob")
  );
  fs::remove_dir_all(dir).unwrap();
}

#[test]
fn face_file_in_home_is_resolved() {
  let home = temp_home_dir("face-hit");
  let face = home.join(FACE_FILENAME);
  fs::write(&face, b"png-bytes").unwrap();

  assert_eq!(avatar_in_home(&home, "alice"), Some(face));
  fs::remove_dir_all(home).unwrap();
}

#[test]
fn missing_face_file_returns_none() {
  let home = temp_home_dir("face-missing");
  assert_eq!(avatar_in_home(&home, "alice"), None);
  fs::remove_dir_all(home).unwrap();
}

#[test]
fn face_takes_priority_over_accounts_service() {
  let accounts = temp_accounts_dir("face-priority");
  let home = temp_home_dir("face-priority");
  let icon = accounts.join("icons").join("alice");
  fs::write(&icon, b"accounts-png").unwrap();
  let face = home.join(FACE_FILENAME);
  fs::write(&face, b"face-png").unwrap();

  let entry = PasswdEntry {
    username: "alice".to_string(),
    gecos: String::new(),
    uid: 1000,
    home: home.clone(),
    shell: "/bin/bash".to_string(),
  };
  assert_eq!(avatar_for_in(Path::new(&accounts), &entry), Some(face));
  fs::remove_dir_all(accounts).unwrap();
  fs::remove_dir_all(home).unwrap();
}

#[test]
fn accounts_service_is_used_when_no_face_exists() {
  let accounts = temp_accounts_dir("face-fallback");
  let home = temp_home_dir("face-fallback");
  let icon = accounts.join("icons").join("bob");
  fs::write(&icon, b"accounts-png").unwrap();

  let entry = PasswdEntry {
    username: "bob".to_string(),
    gecos: String::new(),
    uid: 1001,
    home: home.clone(),
    shell: "/bin/bash".to_string(),
  };
  assert_eq!(avatar_for_in(Path::new(&accounts), &entry), Some(icon));
  fs::remove_dir_all(accounts).unwrap();
  fs::remove_dir_all(home).unwrap();
}

#[test]
fn symlinked_face_file_is_ignored() {
  let home = temp_home_dir("face-symlink");
  let target = std::env::temp_dir().join(format!("face-target-{}", std::process::id()));
  fs::write(&target, b"secret").unwrap();
  std::os::unix::fs::symlink(&target, home.join(FACE_FILENAME)).unwrap();

  assert_eq!(avatar_in_home(&home, "alice"), None);
  fs::remove_dir_all(home).unwrap();
  fs::remove_file(target).unwrap();
}

#[test]
fn directory_named_face_is_ignored() {
  let home = temp_home_dir("face-dir");
  fs::create_dir_all(home.join(FACE_FILENAME)).unwrap();

  assert_eq!(avatar_in_home(&home, "alice"), None);
  fs::remove_dir_all(home).unwrap();
}

#[test]
fn unreadable_face_file_returns_none() {
  if running_as_root() {
    return;
  }
  let home = temp_home_dir("face-unreadable");
  let face = home.join(FACE_FILENAME);
  fs::write(&face, b"png-bytes").unwrap();
  fs::set_permissions(&face, Permissions::from_mode(0o000)).unwrap();

  assert_eq!(avatar_in_home(&home, "alice"), None);
  let _ = fs::remove_dir_all(home);
}

#[test]
fn unsafe_usernames_never_reach_the_filesystem() {
  let home = temp_home_dir("face-unsafe");
  fs::write(home.join(FACE_FILENAME), b"png-bytes").unwrap();

  for username in ["", "..", ".", "a/b", "traversal/../x"] {
    assert_eq!(
      avatar_in_home(&home, username),
      None,
      "username {username:?} must be rejected"
    );
  }
  // Even with a `.face` present, nothing was read through a path join.
  fs::remove_dir_all(home).unwrap();
}

#[test]
fn empty_home_directory_field_still_falls_back_to_accounts_service() {
  let accounts = temp_accounts_dir("empty-home-fallback");
  let icon = accounts.join("icons").join("carol");
  fs::write(&icon, b"accounts-png").unwrap();

  let entry = PasswdEntry {
    username: "carol".to_string(),
    gecos: String::new(),
    uid: 1002,
    home: PathBuf::new(),
    shell: "/bin/bash".to_string(),
  };
  assert_eq!(avatar_for_in(Path::new(&accounts), &entry), Some(icon));
  fs::remove_dir_all(accounts).unwrap();
}
