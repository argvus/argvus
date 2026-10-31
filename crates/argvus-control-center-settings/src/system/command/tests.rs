use super::dns::{resolv_conf_contents, valid_ipv4, valid_ipv6};
use super::power::{lid_key, update_logind_drop_in_at};
use super::*;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());
static CONFIG_TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn rejects_injection_like_arguments() {
  assert!(validate_args(&["timezone".into(), "set".into(), "UTC;reboot".into()]).is_err());
  assert!(validate_args(&["hostname".into(), "set".into(), "bad name".into()]).is_err());
  assert!(validate_args(&["keyboard".into(), "layout".into(), "br".into()]).is_ok());
}

#[test]
fn validates_boot_operations_without_accepting_paths_or_bad_timeouts() {
  assert!(validate_boot("systemd-default", &["arch.conf".into()]).is_ok());
  assert!(validate_boot("systemd-default", &["../evil".into()]).is_err());
  assert!(validate_boot("systemd-timeout", &["60".into()]).is_ok());
  assert!(validate_boot("systemd-timeout", &["61".into()]).is_err());
  assert!(validate_boot("grub-cmdline", &["quiet splash".into()]).is_ok());
  assert!(validate_boot("grub-cmdline", &["bad\nline".into()]).is_err());
  assert!(validate_boot("grub-cmdline", &["quiet $(id)".into()]).is_err());
  assert!(validate_boot("grub-cmdline", &["quiet `id`".into()]).is_err());
  assert!(validate_boot("plymouth-theme", &["argvus-boot-splash".into()]).is_ok());
  assert!(validate_boot("plymouth-theme", &["theme/name".into()]).is_err());
}

#[test]
fn validates_package_operations_without_partial_upgrade_or_arbitrary_paths() {
  assert!(validate_package("install", &["firefox".into()]).is_ok());
  assert!(validate_package("upgrade", &[]).is_ok());
  assert!(validate_package("install", &["foo;reboot".into()]).is_err());
  assert!(validate_package("install", &["--dbonly".into()]).is_err());
  assert!(validate_package("remove", &["-dd".into(), "glibc".into()]).is_err());
  assert!(validate_package("upgrade-package", &[".hidden".into()]).is_err());
  assert!(validate_package("downgrade", &["/tmp/package.pkg.tar.zst".into()]).is_err());
  assert!(validate_package("clean-cache", &["keep-three".into()]).is_ok());
  assert!(validate_package("clean-cache", &["all".into()]).is_err());
  assert!(
    validate_package(
      "mirror-apply",
      &["Server = https://mirror.example/$repo/os/$arch\n".into()]
    )
    .is_ok()
  );
}

#[test]
fn validates_individual_install_and_reinstall_operations() {
  assert!(validate_package("install", &["firefox".into()]).is_ok());
  assert!(validate_package("reinstall", &["firefox".into()]).is_ok());
}

#[test]
fn validates_dependent_aware_removal_names_without_options() {
  for action in ["remove-only", "remove-cascade"] {
    assert!(validate_package(action, &["7zip".into()]).is_ok());
    assert!(validate_package(action, &[]).is_err());
    assert!(validate_package(action, &["-dd".into(), "glibc".into()]).is_err());
    assert!(validate_package(action, &["foo;reboot".into()]).is_err());
  }
}

#[test]
fn validates_network_dns_modes_and_servers() {
  assert!(validate_network("dns", &["eth0".into(), "automatic".into(), "".into()]).is_ok());
  assert!(
    validate_network(
      "dns",
      &["eth0".into(), "manual".into(), "1.1.1.1,8.8.8.8".into()]
    )
    .is_ok()
  );
  assert!(
    validate_network(
      "dns",
      &[
        "eth0".into(),
        "manual".into(),
        "2001:4860:4860::8888".into()
      ]
    )
    .is_ok()
  );
  assert!(
    validate_network(
      "dns",
      &[
        "eth0".into(),
        "manual".into(),
        "1.1.1.1,not-a-server".into()
      ]
    )
    .is_err()
  );
  assert!(validate_network("dns", &["".into(), "manual".into(), "1.1.1.1".into()],).is_err());
  assert!(validate_network("dns", &["eth'0".into(), "manual".into(), "1.1.1.1".into()],).is_err());
}

#[test]
fn validates_icons_only_accepts_booleans() {
  assert!(validate_args(&["config".into(), "icons".into(), "true".into()]).is_ok());
  assert!(validate_args(&["config".into(), "icons".into(), "false".into()]).is_ok());
  assert!(validate_args(&["config".into(), "icons".into(), "yes".into()]).is_err());
  assert!(validate_args(&["config".into(), "other".into(), "true".into()]).is_err());
}

#[test]
fn validates_power_lid_and_button_values() {
  assert!(validate_power("set-lid", &["suspend".into()]).is_ok());
  assert!(validate_power("set-lid", &["battery".into(), "hibernate".into()]).is_ok());
  assert!(validate_power("set-lid", &["ac".into(), "poweroff".into()]).is_ok());
  assert!(validate_power("set-lid", &["docked".into(), "lock".into()]).is_ok());
  assert!(validate_power("set-lid", &["ac".into(), "explode".into()]).is_err());
  assert!(validate_power("set-lid", &["wrong-context".into(), "suspend".into()]).is_err());
  assert!(validate_power("set-power-button", &["poweroff".into()]).is_ok());
  assert!(validate_power("set-power-button", &["suspend".into()]).is_ok());
  assert!(validate_power("set-power-button", &["hibernate".into()]).is_err());
  assert!(validate_power("set-power-button", &[]).is_err());
  assert!(validate_power("unsupported", &[]).is_err());
}

#[test]
fn power_lid_contexts_map_to_logind_keys() {
  assert_eq!(lid_key("default").unwrap(), "HandleLidSwitch");
  assert_eq!(lid_key("battery").unwrap(), "HandleLidSwitch");
  assert_eq!(lid_key("ac").unwrap(), "HandleLidSwitchExternalPower");
  assert_eq!(lid_key("docked").unwrap(), "HandleLidSwitchDocked");
  assert!(lid_key("sleepy").is_err());
}

#[test]
fn logind_drop_in_preserves_existing_keys() {
  let dir = temp_dir().join(format!("argvus-logind-drop-in-{}", std::process::id()));
  std::fs::create_dir_all(&dir).unwrap();
  let path = dir.join("argvus.conf");
  std::fs::write(
    &path,
    "HandleLidSwitch=suspend\n# a comment\nHandlePowerKey=poweroff\n",
  )
  .unwrap();
  let entries: [(&str, String); 1] = [("HandleLidSwitch", "lock".into())];
  assert_eq!(
    update_logind_drop_in_at(&path, &entries, true),
    Ok(()),
    "drop-in is written once"
  );
  let contents = std::fs::read_to_string(&path).unwrap();
  assert!(contents.contains("HandleLidSwitch=lock"), "{contents}");
  assert!(contents.contains("HandlePowerKey=poweroff"), "{contents}");
  assert!(contents.contains("# a comment"), "{contents}");
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn logind_drop_in_creates_missing_file() {
  let dir = temp_dir().join(format!("argvus-logind-drop-in-new-{}", std::process::id()));
  std::fs::create_dir_all(&dir).unwrap();
  let path = dir.join("argvus.conf");
  let entries: [(&str, String); 2] = [
    ("HandleLidSwitch", "suspend-then-hibernate".into()),
    ("HandlePowerKey", "lock".into()),
  ];
  assert_eq!(update_logind_drop_in_at(&path, &entries, true), Ok(()));
  let contents = std::fs::read_to_string(&path).unwrap();
  assert!(
    contents.contains("HandleLidSwitch=suspend-then-hibernate"),
    "{contents}"
  );
  assert!(contents.contains("HandlePowerKey=lock"), "{contents}");
  let _ = std::fs::remove_dir_all(&dir);
}

fn temp_dir() -> PathBuf {
  std::env::var_os("TMPDIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("/tmp"))
}

#[test]
fn validates_and_applies_appearance_icons() {
  let _guard = CONFIG_TEST_LOCK.lock().unwrap();
  let sequence = CONFIG_TEST_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  let dir = std::env::temp_dir().join(format!(
    "argvus-cc-command-config-{}-{sequence}",
    std::process::id()
  ));
  std::fs::create_dir_all(&dir).unwrap();
  let path = dir.join("config.toml");
  std::fs::write(&path, "[Appearance]\nicons = true\n").unwrap();
  unsafe {
    std::env::set_var("ARGVUS_CONFIG_PATH", &path);
  }
  assert!(set_appearance_icons("maybe").is_err());
  assert!(set_appearance_icons("nope").is_err());
  set_appearance_icons("false").unwrap();
  assert!(!AppConfig::icons_enabled());
  let contents = std::fs::read_to_string(&path).unwrap();
  assert!(contents.contains("icons = false"), "{contents}");
  set_appearance_icons("true").unwrap();
  assert!(AppConfig::icons_enabled());
  unsafe {
    std::env::remove_var("ARGVUS_CONFIG_PATH");
  }
  let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolv_conf_content_marks_manual_servers() {
  assert_eq!(
    resolv_conf_contents(&["1.1.1.1", "8.8.8.8"]),
    format!("{RESOLV_MARKER}\nnameserver 1.1.1.1\nnameserver 8.8.8.8\n")
  );
  assert!(valid_ipv4("192.168.0.1"));
  assert!(!valid_ipv4("999.1.1.1"));
  assert!(valid_ipv6("2001:db8::1"));
  assert!(!valid_ipv6("not-an-ip"));
}

#[cfg(feature = "snapshot")]
#[test]
fn snapshot_dispatch_accepts_only_the_closed_root_management_grammar() {
  for arguments in [
    vec!["snapshot", "inspect"],
    vec!["snapshot", "inspect", "system"],
    vec!["snapshot", "create", "system", "spaces $HOME; `id`"],
    vec!["snapshot", "delete", "system", "12"],
  ] {
    assert!(validate_args(&arguments.into_iter().map(String::from).collect::<Vec<_>>()).is_ok());
  }
  for arguments in [
    vec!["snapshot", "rollback", "system", "12"],
    vec!["snapshot", "delete", "system", "0"],
    vec!["snapshot", "delete", "system", "1-12"],
    vec!["snapshot", "inspect", "../etc"],
    vec!["snapshot", "create-config", "/"],
  ] {
    assert!(validate_args(&arguments.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
  }
}

#[test]
fn global_language_command_is_rejected_before_elevation() {
  let args = ["locale".into(), "set-lang".into(), "en_US.UTF-8".into()];
  let validation = validate_args(&args).unwrap_err();
  assert_eq!(run(&args).unwrap_err(), validation);
  assert!(
    validate_args(&[
      "locale".into(),
      "set-generated".into(),
      "en_US.UTF-8 UTF-8".into()
    ])
    .is_ok()
  );
}
