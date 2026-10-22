//! Hermetic integration tests.
//!
//! These tests never modify real system accounts or real home directories:
//! privilege operations are recorded by a mock backend and avatar deployment
//! is redirected to a temporary "home" via the manager's home override.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use argvus_accounts_core::account::PrivilegeOps;
use argvus_accounts_core::avatar::{self, AVATAR_FILENAME, MAX_SOURCE_FILE_BYTES, OUTPUT_SIZE};
use argvus_accounts_core::passwd;
use argvus_accounts_core::permissions::{AllowAllProvider, UnixAuthorizationProvider};
use argvus_accounts_core::{AccountManager, Error, Result};

/// Records every privileged operation instead of touching the system.
#[derive(Clone, Default)]
struct RecordingOps {
  calls: Arc<Mutex<Vec<String>>>,
}

impl RecordingOps {
  fn snapshot(&self) -> Vec<String> {
    self.calls.lock().expect("mutex").clone()
  }
}

impl PrivilegeOps for RecordingOps {
  fn set_display_name_admin(&self, user: &str, name: &str) -> Result<()> {
    self
      .calls
      .lock()
      .expect("mutex")
      .push(format!("admin-name:{user}:{name}"));
    Ok(())
  }

  fn set_own_display_name(&self, user: &str, name: &str) -> Result<()> {
    self
      .calls
      .lock()
      .expect("mutex")
      .push(format!("own-name:{user}:{name}"));
    Ok(())
  }

  fn add_to_group(&self, user: &str, group: &str) -> Result<()> {
    self
      .calls
      .lock()
      .expect("mutex")
      .push(format!("add:{user}:{group}"));
    Ok(())
  }

  fn remove_from_group(&self, user: &str, group: &str) -> Result<()> {
    self
      .calls
      .lock()
      .expect("mutex")
      .push(format!("remove:{user}:{group}"));
    Ok(())
  }

  fn set_password(&self, user: &str, new: &str) -> Result<()> {
    // Records the length only — never the secret itself.
    self
      .calls
      .lock()
      .expect("mutex")
      .push(format!("passwd:{user}:{}", new.len()));
    Ok(())
  }
}

fn fixture_png(width: u32, height: u32, color: [u8; 3]) -> PathBuf {
  let image = image::RgbImage::from_fn(width, height, |_, _| color.into());
  let dynamic = image::DynamicImage::ImageRgb8(image);
  let path = std::env::temp_dir().join(format!(
    "argvus-accounts-test-{}-{}.png",
    std::process::id(),
    color[0] as u32 * 1_000_000 + color[1] as u32 * 1000 + color[2] as u32
  ));
  dynamic
    .write_to(
      &mut std::fs::File::create(&path).expect("create fixture"),
      image::ImageFormat::Png,
    )
    .expect("encode fixture");
  path
}

fn temp_home(tag: &str) -> PathBuf {
  let dir = std::env::temp_dir().join(format!("argvus-accounts-home-{}-{tag}", std::process::id()));
  let _ = std::fs::remove_dir_all(&dir);
  std::fs::create_dir_all(&dir).expect("temp home");
  dir
}

fn me() -> String {
  AccountManager::current_username().expect("current user resolves")
}

#[test]
fn avatar_full_lifecycle_through_manager() {
  let me = me();
  let home = temp_home("lifecycle");
  let manager =
    AccountManager::with_components(Box::new(AllowAllProvider), Box::<RecordingOps>::default())
      .with_home_override(home.clone());

  assert!(manager.get_avatar(&me).unwrap().is_none());
  assert!(matches!(
    manager.remove_avatar(&me),
    Err(Error::AvatarNotFound(_))
  ));

  let first = fixture_png(320, 200, [255, 0, 0]);
  manager.set_avatar(&me, &first).expect("set avatar");
  std::fs::remove_file(&first).unwrap();

  let deployed = manager.get_avatar(&me).unwrap().expect("avatar present");
  assert_eq!(deployed, home.join(AVATAR_FILENAME));

  let meta = std::fs::symlink_metadata(&deployed).unwrap();
  use std::os::unix::fs::MetadataExt;
  assert!(meta.is_file(), "must be a regular file");
  assert_eq!(meta.mode() & 0o777, 0o644);
  assert_eq!(
    meta.uid(),
    passwd::current_user().unwrap().uid,
    "avatar must be owned by the user"
  );

  let decoded = image::ImageReader::open(&deployed)
    .unwrap()
    .with_guessed_format()
    .unwrap()
    .decode()
    .unwrap();
  assert_eq!(
    (decoded.width(), decoded.height()),
    (OUTPUT_SIZE, OUTPUT_SIZE)
  );
  let center = (OUTPUT_SIZE / 2, OUTPUT_SIZE / 2);
  assert_eq!(
    decoded.to_rgb8().get_pixel(center.0, center.1),
    &image::Rgb([255, 0, 0]),
    "first avatar content"
  );

  // Replacement must succeed atomically and leave no temp files behind.
  let second = fixture_png(64, 64, [0, 255, 0]);
  manager.set_avatar(&me, &second).expect("replace avatar");
  std::fs::remove_file(&second).unwrap();
  let entries = std::fs::read_dir(&home).unwrap().count();
  assert_eq!(entries, 1, "only .face should remain");
  let replaced = image::ImageReader::open(&deployed)
    .unwrap()
    .with_guessed_format()
    .unwrap()
    .decode()
    .unwrap();
  assert_eq!(
    replaced.to_rgb8().get_pixel(center.0, center.1),
    &image::Rgb([0, 255, 0]),
    "replacement content"
  );

  manager.remove_avatar(&me).expect("remove avatar");
  assert!(manager.get_avatar(&me).unwrap().is_none());
  let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn avatar_rejects_bad_inputs() {
  let me = me();
  let home = temp_home("badinputs");
  let manager =
    AccountManager::with_components(Box::new(AllowAllProvider), Box::<RecordingOps>::default())
      .with_home_override(home.clone());

  // Garbage bytes with an image extension: sniffing must win.
  let garbage = home.join("garbage.png");
  std::fs::write(&garbage, vec![0xde_u8; 1024]).unwrap();
  assert!(matches!(
    manager.set_avatar(&me, &garbage),
    Err(Error::UnsupportedImageFormat(_))
  ));

  // Oversized file.
  let big = home.join("big.png");
  std::fs::write(&big, vec![0u8; MAX_SOURCE_FILE_BYTES as usize + 1]).unwrap();
  assert!(matches!(
    manager.set_avatar(&me, &big),
    Err(Error::ImageTooLarge { .. })
  ));
  std::fs::remove_file(&big).unwrap();

  // Undersized dimensions.
  let tiny = fixture_png(8, 8, [0, 0, 255]);
  assert!(matches!(
    manager.set_avatar(&me, &tiny),
    Err(Error::InvalidImage(_))
  ));
  std::fs::remove_file(&tiny).unwrap();

  // A FIFO reports length 0 and would block a plain read. It must be refused
  // without waiting for a writer to appear.
  {
    use std::os::unix::ffi::OsStrExt;
    let fifo = home.join("fifo.png");
    let fifo_path = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
    // SAFETY: `fifo_path` is a valid NUL-terminated string that outlives the call.
    assert_eq!(unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) }, 0);
    assert!(matches!(
      manager.set_avatar(&me, &fifo),
      Err(Error::InvalidImage(_))
    ));
    std::fs::remove_file(&fifo).unwrap();
  }

  // Missing source file.
  let missing = home.join("does-not-exist.png");
  assert!(matches!(
    manager.set_avatar(&me, &missing),
    Err(Error::InvalidImage(_))
  ));

  // JPEG source must be accepted and normalized like any other format.
  let jpeg_image = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(300, 300, |_, _| {
    [10, 20, 30].into()
  }));
  let jpeg_path = home.join("source.jpg");
  jpeg_image
    .write_to(
      &mut std::fs::File::create(&jpeg_path).unwrap(),
      image::ImageFormat::Jpeg,
    )
    .unwrap();
  manager.set_avatar(&me, &jpeg_path).expect("jpeg accepted");
  std::fs::remove_file(&jpeg_path).unwrap();

  let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn symlinked_and_directory_avatars_are_handled_safely() {
  let me = me();
  let home = temp_home("symlink");
  let manager =
    AccountManager::with_components(Box::new(AllowAllProvider), Box::<RecordingOps>::default())
      .with_home_override(home.clone());

  let dest = home.join(AVATAR_FILENAME);
  std::os::unix::fs::symlink("/etc/passwd", &dest).expect("symlink setup");
  assert!(
    avatar::find_avatar(&home).is_none(),
    "symlinks are not avatars"
  );
  // Deploying replaces the symlink itself instead of following it.
  let source = fixture_png(32, 32, [9, 9, 9]);
  manager
    .set_avatar(&me, &source)
    .expect("set replaces symlink");
  std::fs::remove_file(&source).unwrap();
  assert!(std::fs::symlink_metadata(&dest).unwrap().is_file());

  // A directory named `.face` is refused, never recursed into.
  let dir_home = temp_home("dirface");
  std::fs::create_dir_all(dir_home.join(AVATAR_FILENAME)).unwrap();
  let err = avatar::remove_avatar(&dir_home).unwrap_err();
  assert!(err.to_string().contains("directory"));

  let _ = std::fs::remove_dir_all(&home);
  let _ = std::fs::remove_dir_all(&dir_home);
}

#[test]
fn avatar_never_writes_through_a_symlinked_home() {
  let me = me();
  let real_home = temp_home("symlink-target");
  let link_root = temp_home("symlink-root");
  let home = link_root.join("home");
  std::os::unix::fs::symlink(&real_home, &home).expect("symlink setup");
  let manager =
    AccountManager::with_components(Box::new(AllowAllProvider), Box::<RecordingOps>::default())
      .with_home_override(home.clone());

  // The home path is a symlink: deploying must refuse before writing anything.
  let source = fixture_png(32, 32, [1, 2, 3]);
  assert!(manager.set_avatar(&me, &source).is_err());
  std::fs::remove_file(&source).unwrap();
  assert!(
    std::fs::read_dir(&real_home).unwrap().next().is_none(),
    "nothing may be written through the symlink"
  );

  // Removal refuses the same way and leaves the target's avatar untouched.
  std::fs::write(real_home.join(AVATAR_FILENAME), b"keep").unwrap();
  assert!(avatar::remove_avatar(&home).is_err());
  assert_eq!(
    std::fs::read(real_home.join(AVATAR_FILENAME)).unwrap(),
    b"keep"
  );

  let _ = std::fs::remove_dir_all(&real_home);
  let _ = std::fs::remove_dir_all(&link_root);
}

#[test]
fn authorization_matrix_end_to_end() {
  let me = me();
  let privileged = nix::unistd::Uid::effective().is_root();
  // Never the acting user: guarantees cross-account semantics are exercised.
  let target = passwd::list_users()
    .expect("user enumeration works")
    .into_iter()
    .find(|user| user.username != me)
    .map(|user| user.username)
    .expect("at least one other account exists");

  let unix_manager = AccountManager::with_components(
    Box::new(UnixAuthorizationProvider),
    Box::<RecordingOps>::default(),
  );

  // Public information is readable for everyone, including other accounts.
  assert!(unix_manager.get_user(&target).is_ok());

  let outcome = unix_manager.set_display_name(&target, "Someone Else");
  if privileged {
    assert!(outcome.is_ok(), "root is an administrator");
  } else {
    assert!(matches!(outcome, Err(Error::PermissionDenied(_))));
  }

  // Group administration always routes through the authorization layer; the
  // recording backend proves no real gpasswd ran when it should not have.
  let ops = RecordingOps::default();
  let unix_ops_manager =
    AccountManager::with_components(Box::new(UnixAuthorizationProvider), Box::new(ops.clone()));
  let result = unix_ops_manager.add_group(&target, "audio");
  if privileged {
    assert!(result.is_ok());
  } else {
    assert!(matches!(result, Err(Error::PermissionDenied(_))));
  }
  if !privileged {
    assert!(ops.snapshot().is_empty(), "no privileged call may execute");
  }
}

#[test]
fn self_service_display_name_selects_the_right_backend() {
  let me = me();
  let ops = RecordingOps::default();
  let manager = AccountManager::with_components(Box::new(AllowAllProvider), Box::new(ops.clone()));

  manager
    .set_display_name(&me, "Argvus Tester")
    .expect("own profile change is always authorized");

  let expected = if nix::unistd::Uid::effective().is_root() {
    format!("admin-name:{me}:Argvus Tester")
  } else {
    format!("own-name:{me}:Argvus Tester")
  };
  assert_eq!(ops.snapshot(), vec![expected]);

  // Validation failures never reach the backend.
  assert!(matches!(
    manager.set_display_name(&me, "has,comma"),
    Err(Error::InvalidDisplayName(_))
  ));
  assert_eq!(ops.snapshot().len(), 1);
}

#[test]
fn group_mutations_are_recorded_without_touching_the_system() {
  let me = me();
  let ops = RecordingOps::default();
  let manager = AccountManager::with_components(Box::new(AllowAllProvider), Box::new(ops.clone()));

  manager.add_group(&me, "audio").expect("recorded add");
  manager.remove_group(&me, "audio").expect("recorded remove");

  assert!(matches!(
    manager.add_group(&me, "../evil"),
    Err(Error::InvalidGroupName(_))
  ));
  assert!(matches!(
    manager.remove_group(&me, "not a group"),
    Err(Error::InvalidGroupName(_))
  ));

  let mut calls = ops.snapshot();
  assert_eq!(calls.remove(0), format!("add:{me}:audio"));
  assert_eq!(calls.remove(0), format!("remove:{me}:audio"));
}

#[test]
fn traversal_attempts_never_reach_nss_or_fs() {
  let manager = AccountManager::new();
  assert!(matches!(
    manager.get_user("../etc/passwd"),
    Err(Error::InvalidUsername(_))
  ));
  assert!(matches!(
    manager.get_user("."),
    Err(Error::InvalidUsername(_))
  ));
  assert!(matches!(
    manager.get_user(".."),
    Err(Error::InvalidUsername(_))
  ));
  assert!(matches!(
    manager.get_user("a/b"),
    Err(Error::InvalidUsername(_))
  ));
  assert!(matches!(
    manager.get_user("definitely_missing_user_xyz"),
    Err(Error::UserNotFound(_))
  ));
}

#[test]
fn changing_own_password_without_proof_is_refused() {
  let me = me();
  let manager = AccountManager::with_components(
    Box::new(UnixAuthorizationProvider),
    Box::<RecordingOps>::default(),
  );
  assert!(matches!(
    manager.change_password(&me, None, "longenough1"),
    Err(Error::InvalidOperation(_))
  ));
}

#[test]
fn wrong_current_password_blocks_the_change() {
  if argvus_accounts_core::password::chkpwd_helper_path().is_none() {
    return; // no shadow helper in this environment
  }
  let me = me();
  let manager = AccountManager::with_components(
    Box::new(UnixAuthorizationProvider),
    Box::<RecordingOps>::default(),
  );
  assert!(matches!(
    manager.change_password(&me, Some("surely-not-my-password"), "longenough1"),
    Err(Error::PermissionDenied(_))
  ));
}

#[test]
fn unprivileged_caller_cannot_reset_someone_elses_password() {
  if nix::unistd::Uid::effective().is_root() {
    return; // root is an administrator by definition
  }
  let ops = RecordingOps::default();
  let manager = AccountManager::with_components(Box::new(UnixAuthorizationProvider), Box::new(ops));
  assert!(matches!(
    manager.change_password("root", None, "longenough1"),
    Err(Error::PermissionDenied(_))
  ));
}

#[test]
fn administrator_reset_delegates_to_backend_with_validated_secret() {
  let ops = RecordingOps::default();
  let manager = AccountManager::with_components(Box::new(AllowAllProvider), Box::new(ops.clone()));

  // Only protocol-level rules apply: empty is refused, short is fine.
  assert!(matches!(
    manager.change_password("root", None, ""),
    Err(Error::InvalidPassword(_))
  ));

  manager
    .change_password("root", None, "a-very-new-password")
    .expect("allow-all provider authorizes the reset");
  manager
    .change_password("root", None, "x")
    .expect("single-character passwords carry no policy here");
  assert_eq!(
    ops.snapshot(),
    vec![
      format!("passwd:root:{}", "a-very-new-password".len()),
      format!("passwd:root:{}", "x".len()),
    ]
  );
}
