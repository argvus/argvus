use super::*;

#[test]
fn config_path_matches_the_session_runtime_consumer() {
  assert!(config_path().ends_with("argvus/data/hypr/hypridle.conf"));
}

const SAMPLE: &str = "\
# ARGVUS idle
timeout = 300
onetime = false
label = dpms
on-timeout = hyprctl dispatch dpms off
on-resume = hyprctl dispatch dpms on

timeout = 1800
onetime = false
label = suspend
on-timeout = systemctl suspend
";

#[test]
fn finds_the_dpms_listener_timeout() {
  assert_eq!(listener_minutes_from(SAMPLE, calls_dpms_off), Some(5));
}

#[test]
fn finds_the_lock_listener_timeout() {
  let config = "\
general {
lock_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}

listener {
timeout = 900
on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}

listener {
timeout = 300
on-timeout = hyprctl dispatch dpms off
}
";
  assert_eq!(lock_minutes_from(config), Some(15));
  let updated = set_lock_minutes(config, 10).unwrap();
  assert_eq!(lock_minutes_from(&updated), Some(10));
  assert!(updated.contains("timeout = 300"), "{updated}");
  assert_eq!(listener_minutes_from(&updated, calls_dpms_off), Some(5));
}

#[test]
fn rewrites_only_the_dpms_timeout() {
  let updated = set_screen_off_minutes(SAMPLE, 30).unwrap();
  assert!(updated.contains("# ARGVUS idle"), "{updated}");
  assert!(updated.contains("timeout = 1800\n"), "{updated}");
  assert!(updated.contains("systemctl suspend"), "{updated}");
  assert!(!updated.contains("timeout = 300\n"), "{updated}");
  assert_eq!(screen_off_minutes_from(&updated), Some(30));
}

#[test]
fn recognizes_an_off_idiom_without_an_installer() {
  assert!(calls_dpms_off("hyprctl dispatch dpms off"));
  assert!(calls_dpms_off("hyprctl dispatch dpms_state 0"));
  assert!(!calls_dpms_off("systemctl suspend"));
  assert!(!calls_dpms_off("hyprctl dispatch dpms on"));
  assert!(calls_dpms_off(
    "hyprctl dispatch 'hl.dsp.dpms({ action = \"off\" })'"
  ));
  assert!(!calls_dpms_off(
    "hyprctl dispatch 'hl.dsp.dpms({ action = \"on\" })'"
  ));
}

#[test]
fn refuses_configs_without_a_dpms_listener() {
  let idle_only = "\
timeout = 1800
onetime = false
on-timeout = systemctl suspend
";
  assert!(set_screen_off_minutes(idle_only, 10).is_err());
  assert!(screen_off_minutes_from(idle_only).is_none());
}

#[test]
fn adds_a_dpms_listener_when_missing() {
  let stock = "\
general {
lock_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}

listener {
timeout = 900
on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}
";
  assert_eq!(screen_off_minutes_from(stock), None);
  let updated = with_screen_off_listener(stock, 1);
  assert_eq!(screen_off_minutes_from(&updated), Some(1));
  assert!(updated.contains("timeout = 900"), "{updated}");
  assert!(
    updated.contains("on-timeout = hyprctl dispatch 'hl.dsp.dpms({ action = \"off\" })'"),
    "{updated}"
  );
  assert!(!updated.contains("Screen-off timer"), "{updated}");
}

#[test]
fn removing_the_dpms_listener_disables_screen_off() {
  let config = "\
general {
lock_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}

listener {
timeout = 1
on-timeout = hyprctl dispatch dpms off
on-resume = hyprctl dispatch dpms on
}

listener {
timeout = 1800
on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}
";
  assert_eq!(screen_off_minutes_from(config), Some(1));
  let updated = remove_listener(config, calls_dpms_off);
  assert_eq!(screen_off_minutes_from(&updated), None);
  assert!(updated.contains("general {"), "{updated}");
  assert!(updated.contains("timeout = 1800"), "{updated}");
  assert!(!updated.contains("dpms off"), "{updated}");
}

#[test]
fn a_zero_timeout_reads_as_disabled() {
  let content = "timeout = 0\non-timeout = hyprctl dispatch dpms off\n";
  assert_eq!(screen_off_minutes_from(content), None);
}

#[test]
fn adds_a_lock_listener_when_missing() {
  let bare = "\
listener {
timeout = 300
on-timeout = hyprctl dispatch dpms off
}
";
  assert_eq!(lock_minutes_from(bare), None);
  let updated = with_lock_listener(bare, 1);
  assert_eq!(lock_minutes_from(&updated), Some(1));
  assert!(updated.contains("timeout = 300"), "{updated}");
  assert!(!updated.contains("Screen-lock timer"), "{updated}");
  assert_eq!(listener_minutes_from(&updated, calls_dpms_off), Some(5));
}

#[test]
fn lock_listener_reuses_the_configured_lock_command() {
  let config = "\
general {
lock_cmd = custom-locker --damp
}

listener {
timeout = 900
on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}
";
  assert_eq!(lock_command(config), "custom-locker --damp");
  let updated = with_lock_listener(config, 5);
  assert!(
    updated.contains("on-timeout = custom-locker --damp"),
    "{updated}"
  );
  assert_eq!(lock_minutes_from(&updated), Some(5));
}

#[test]
fn removing_the_lock_listener_disables_lock() {
  let config = "\
general {
lock_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}

listener {
timeout = 300
on-timeout = hyprctl dispatch dpms off
}

listener {
timeout = 900
on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock
}
";
  assert_eq!(lock_minutes_from(config), Some(15));
  let updated = remove_listener(config, calls_lock);
  assert_eq!(lock_minutes_from(&updated), None);
  assert_eq!(listener_minutes_from(&updated, calls_dpms_off), Some(5));
  assert!(
    updated.contains("lock_cmd = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock"),
    "{updated}"
  );
  assert!(
    !updated.contains("on-timeout = sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock"),
    "{updated}"
  );
}

#[test]
fn calls_lock_matches_lock_idioms_but_not_sleep_or_dpms() {
  assert!(calls_lock(
    "sh /usr/share/argvus/power/sh/hypr-power-menu.sh --lock"
  ));
  assert!(calls_lock("swaylock"));
  assert!(!calls_lock("systemctl suspend"));
  assert!(!calls_lock("hyprctl dispatch dpms off"));
}

#[test]
fn only_removes_blocks_named_listener() {
  let config = "\
listen {
timeout = 900
on-timeout = sh loginctl lock-session
}
";
  assert_eq!(lock_minutes_from(config), Some(15));
  let updated = remove_listener(config, calls_lock);
  assert_eq!(updated, config, "{updated}");
}
