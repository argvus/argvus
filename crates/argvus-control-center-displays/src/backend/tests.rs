use super::*;

const SAMPLE: &str = r#"[
{
  "id": 0,
  "name": "eDP-1",
  "x": 0, "y": 0, "width": 1920, "height": 1080,
  "refreshRate": 60.001,
  "scale": 1.25, "transform": 0, "focused": true,
  "vrr": true, "dpmsStatus": true, "disabled": false,
  "description": "AUO 16.0\"", "make": "AUO", "model": "B160QAN02",
  "serial": "1234", "currentFormat": "XRGB2101010",
  "physicalWidth": 344, "physicalHeight": 194,
  "activeWorkspace": { "id": 1 },
  "availableModes": [
    { "width": 1920, "height": 1080, "refreshRate": 60.0, "bitDepth": 8, "id": 1 },
    { "width": 1920, "height": 1080, "refreshRate": 59.94, "bitDepth": 10, "id": 2 }
  ]
},
{
  "id": 1,
  "name": "DP-1", "x": 1920, "y": 0, "width": 2560, "height": 1440,
  "refreshRate": 165.0, "scale": 1.0, "transform": 0, "focused": false,
  "vrr": 0, "dpmsStatus": true, "disabled": true,
  "mirrorOf": "eDP-1", "currentFormat": "XRGB8888",
  "availableModes": []
}
]"#;

#[test]
fn parses_hyprctl_monitor_json() {
  let value: Value = serde_json::from_str(SAMPLE).unwrap();
  let monitor = monitor_from(&value[0]).unwrap();
  assert_eq!(monitor.name, "eDP-1");
  assert_eq!(monitor.refresh_rate, 60.001);
  assert_eq!(monitor.modes.len(), 2);
  assert!(monitor.focused);
  assert_eq!(monitor.info.make, "AUO");
  assert_eq!(monitor.info.mirror_of, None);
  assert_eq!(monitor.active_workspace, Some(1));
  let disabled = monitor_from(&value[1]).unwrap();
  assert!(disabled.disabled);
  assert_eq!(disabled.info.mirror_of.as_deref(), Some("eDP-1"));
}

#[test]
fn vrr_field_accepts_boolean_and_integer() {
  let value: Value = serde_json::from_str(SAMPLE).unwrap();
  assert_eq!(monitor_from(&value[0]).unwrap().vrr, 1);
  assert_eq!(monitor_from(&value[1]).unwrap().vrr, 0);
}

#[test]
fn versions_are_extracted_from_text() {
  assert_eq!(parse_version("Hyprland 0.42.0, commit..."), (0, 42));
  assert_eq!(parse_version("0.44.1"), (0, 44));
  assert_eq!(parse_version("garbage"), (0, 0));
}

#[test]
fn generated_lua_round_trips() {
  let config = PersistedConfig {
    primary_monitor: Some("eDP-1".into()),
    monitors: vec![
      (
        "eDP-1".into(),
        PersistedMonitor {
          mode: Some("1920x1080@60".into()),
          position: Some("0x0".into()),
          scale: Some(1.25),
          transform: Some(0),
          vrr: Some(1),
          hdr: Some(1),
          bitdepth: Some(10),
          ..Default::default()
        },
      ),
      (
        "DP-1".into(),
        PersistedMonitor {
          disabled: Some(true),
          ..Default::default()
        },
      ),
    ],
    workspaces: vec![("eDP-1".into(), vec![1, 2])],
  };
  let text = serialize_config(&config);
  assert!(text.contains("hl.monitor({ output = \"eDP-1\""), "{text}");
  assert!(text.contains("supports_hdr = 1"), "{text}");
  assert!(text.contains("disabled = true"), "{text}");
  assert!(
    text.contains("hl.workspace_rule({ workspace = \"1\", monitor = \"eDP-1\", default = true })"),
    "{text}"
  );
  let parsed = parse_generated(&text);
  assert_eq!(parsed.primary_monitor.as_deref(), Some("eDP-1"));
  assert_eq!(parsed.monitors.len(), 2);
  assert_eq!(parsed.monitors[0].0, "DP-1");
  assert!(parsed.monitors[0].1.disabled == Some(true));
  let (name, monitor) = &parsed.monitors[1];
  assert_eq!(name, "eDP-1");
  assert_eq!(monitor.scale, Some(1.25));
  assert_eq!(monitor.mode.as_deref(), Some("1920x1080@60"));
  assert_eq!(monitor.hdr, Some(1));
  assert_eq!(monitor.bitdepth, Some(10));
  assert_eq!(monitor.vrr, Some(1));
  assert_eq!(parsed.workspaces_of("eDP-1"), vec![1, 2]);
}

#[test]
fn multiline_lua_calls_parse() {
  let text = "\
hl.monitor({
output = \"eDP-1\",
mode = \"1920x1080@144\",
scale = 1.25,
transform = 1,
})
-- trailing comment
hl.workspace_rule({
workspace = \"7\",
monitor = \"DP-2\",
})
";
  let parsed = parse_generated(text);
  assert_eq!(parsed.monitors.len(), 1);
  let (name, monitor) = &parsed.monitors[0];
  assert_eq!(name, "eDP-1");
  assert_eq!(monitor.mode.as_deref(), Some("1920x1080@144"));
  assert_eq!(monitor.scale, Some(1.25));
  assert_eq!(monitor.transform, Some(1));
  assert_eq!(parsed.workspaces_of("DP-2"), vec![7]);
}

#[test]
fn legacy_configs_still_parse() {
  let config = parse_legacy(
    "-- old\nreturn {\n  primary_monitor = \"eDP-1\"\n  monitors = { { name = \"eDP-1\", mode = \"1920x1080@60\", pos = \"0x0\", scale = 1.25, hdr = 1 } }\n}\n",
  );
  assert_eq!(config.primary_monitor.as_deref(), Some("eDP-1"));
  assert_eq!(config.monitors.len(), 1);
  assert_eq!(config.monitors[0].1.hdr, Some(1));
}

#[test]
fn foreign_configs_parse_to_defaults() {
  let config = parse_generated("-- not ours\nreturn { }\n");
  assert_eq!(config.primary_monitor, None);
  assert!(config.monitors.is_empty());
}

#[test]
fn workspace_rules_add_primary_default_rule_when_unbound() {
  let config = PersistedConfig {
    primary_monitor: Some("DP-1".into()),
    ..Default::default()
  };
  let mut text = String::new();
  append_workspace_rules(&mut text, &config);
  assert!(
    text.contains("hl.workspace_rule({ workspace = \"1\", monitor = \"DP-1\", default = true })"),
    "{text}"
  );
}

use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn state_round_trips_profiles_through_toml() {
  let state = DisplayState {
    primary_monitor: Some("eDP-1".into()),
    active_profile: Some("Trabalho".into()),
    profiles: vec![MonitorProfile {
      name: "Trabalho".into(),
      apply_wallpapers: true,
      config: PersistedConfig {
        primary_monitor: Some("eDP-1".into()),
        monitors: vec![(
          "eDP-1".into(),
          PersistedMonitor {
            scale: Some(1.5),
            vrr: Some(2),
            mirror: Some("DP-1".into()),
            ..Default::default()
          },
        )],
        workspaces: vec![("eDP-1".into(), vec![1, 2, 3])],
      },
    }],
  };
  let dir = std::env::temp_dir().join(format!("argvus-displays-{}", std::process::id()));
  let _guard = ENV_LOCK.lock().unwrap();
  let original = argvus_control_center_core::paths::argvus_config_home();
  // Safety: single-threaded test, no other thread reads ARGVUS_CONFIG_HOME.
  unsafe { std::env::set_var("ARGVUS_CONFIG_HOME", &dir) };
  save_state(&state).unwrap();
  let loaded = load_state();
  unsafe {
    std::env::set_var(
      "ARGVUS_CONFIG_HOME",
      original.to_str().unwrap_or("/tmp/nonexistent"),
    )
  };
  let _ = fs::remove_dir_all(&dir);
  assert_eq!(loaded.active_profile.as_deref(), Some("Trabalho"));
  let profile = loaded.active_profile().unwrap();
  assert!(profile.apply_wallpapers);
  assert_eq!(profile.config.workspaces_of("eDP-1"), vec![1, 2, 3]);
  assert_eq!(profile.config.persisted("eDP-1").vrr, Some(2));
  assert_eq!(
    profile.config.persisted("eDP-1").mirror.as_deref(),
    Some("DP-1")
  );
}

#[test]
/// Only the fields that differ from the canonical layout are written, and a
/// field that was removed is written as an empty value so it is cleared.
fn monitor_fields_report_only_changes() {
  let before = PersistedMonitor {
    scale: Some(1.5),
    mode: Some("1920x1080@60".into()),
    ..Default::default()
  };
  let now = PersistedMonitor {
    scale: Some(2.0),
    vrr: Some(1),
    ..Default::default()
  };
  let changed: Vec<_> = monitor_fields(&now, &before)
    .into_iter()
    .filter(|(_, now, then)| now != then)
    .map(|(key, now, _)| (key, now))
    .collect();
  assert_eq!(
    changed,
    vec![
      ("mode", String::new()),
      ("scale", "2".to_string()),
      ("vrr", "1".to_string()),
    ]
  );
}

#[test]
/// The primary monitor belongs to the canonical layout, so the state file
/// never stores it.
fn state_file_does_not_store_the_primary_monitor() {
  let dir = std::env::temp_dir().join(format!("argvus-displays-primary-{}", std::process::id()));
  let _guard = ENV_LOCK.lock().unwrap();
  let original = argvus_control_center_core::paths::argvus_config_home();
  // Safety: single-threaded test, no other thread reads ARGVUS_CONFIG_HOME.
  unsafe { std::env::set_var("ARGVUS_CONFIG_HOME", &dir) };
  let state = DisplayState {
    primary_monitor: Some("DP-3".into()),
    ..Default::default()
  };
  save_state(&state).unwrap();
  let stored = fs::read_to_string(state_path()).unwrap();
  assert!(!stored.contains("primary_monitor"), "{stored}");
  unsafe {
    std::env::set_var(
      "ARGVUS_CONFIG_HOME",
      original.to_str().unwrap_or("/tmp/nonexistent"),
    )
  };
  let _ = fs::remove_dir_all(&dir);
}

fn serialize_config(config: &PersistedConfig) -> String {
  let mut text = String::from("-- Generated by ARGVUS Control Center. Do not edit.\n");
  for (name, monitor) in &config.monitors {
    text.push_str(&format!("hl.monitor({{ output = {name:?}"));
    if monitor.disabled == Some(true) {
      text.push_str(", disabled = true");
    } else {
      if let Some(mode) = &monitor.mode {
        text.push_str(&format!(", mode = {mode:?}"));
      }
      if let Some(position) = &monitor.position {
        text.push_str(&format!(", position = {position:?}"));
      }
      if let Some(scale) = monitor.scale {
        text.push_str(&format!(", scale = {scale}"));
      }
      if let Some(transform) = monitor.transform {
        text.push_str(&format!(", transform = {transform}"));
      }
      if let Some(bitdepth) = monitor.bitdepth {
        text.push_str(&format!(", bitdepth = {bitdepth}"));
      }
      if let Some(vrr) = monitor.vrr {
        text.push_str(&format!(", vrr = {vrr}"));
      }
      if let Some(hdr) = monitor.hdr {
        text.push_str(&format!(", supports_hdr = {hdr}"));
      }
    }
    text.push_str(" })\n");
  }
  append_workspace_rules(&mut text, config);
  text
}
