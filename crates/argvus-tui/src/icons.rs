//! Implements shared icon catalog in crate `argvus tui`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! Icons use the Material Design Icons section of Nerd Fonts (`nf-md-*`).
//! Keep constants to one glyph with no layout whitespace; spacing belongs to
//! [`icon_label`]. The recommended terminal font is Symbols Nerd Font Mono.

/// Fixed gap between the icon cell and its label.
pub const ICON_TEXT_GAP: &str = " ";

/// Render a consistently spaced icon/label pair for legacy string-based rows.
pub fn icon_label(icon: &str, label: impl AsRef<str>) -> String {
  if icon.is_empty() {
    label.as_ref().to_owned()
  } else {
    format!("{icon}{ICON_TEXT_GAP}{}", label.as_ref())
  }
}

/// Nerd Fonts: nf-md-network
pub const NETWORK: &str = "\u{f06f3}";
/// Nerd Fonts: nf-md-wifi-strength-4
pub const WIFI: &str = "\u{f0928}";
/// Nerd Fonts: nf-md-ethernet
pub const ETHERNET: &str = "\u{f0200}";
/// Nerd Fonts: nf-md-vpn
pub const VPN: &str = "\u{f0582}";
/// Nerd Fonts: nf-md-dns
pub const DNS: &str = "\u{f01d6}";
/// Nerd Fonts: nf-md-magnify
pub const SEARCH: &str = "\u{f0349}";
/// Nerd Fonts: nf-md-bluetooth
pub const BLUETOOTH: &str = "\u{f00af}";
/// Nerd Fonts: nf-md-volume-high
pub const AUDIO: &str = "\u{f057e}";
/// Nerd Fonts: nf-md-speaker
pub const SPEAKER: &str = "\u{f04c3}";
/// Nerd Fonts: nf-md-microphone
pub const MICROPHONE: &str = "\u{f036c}";
/// Nerd Fonts: nf-md-chip
pub const HARDWARE: &str = "\u{f061a}";
/// Nerd Fonts: nf-md-cpu-64-bit
pub const CPU: &str = "\u{f0ee0}";
/// Nerd Fonts: nf-md-expansion-card-variant
pub const GPU: &str = "\u{f0fb2}";
/// Nerd Fonts: nf-md-memory
pub const MEMORY: &str = "\u{f035b}";
/// Nerd Fonts: nf-md-power
pub const POWER: &str = "\u{f0425}";
/// Nerd Fonts: nf-md-cog
pub const SETTINGS: &str = "\u{f0493}";
/// Nerd Fonts: nf-md-server
pub const SERVICES: &str = "\u{f048b}";
/// Nerd Fonts: nf-md-account
pub const USER: &str = "\u{f0004}";
/// Nerd Fonts: nf-md-text-box
pub const LOGS: &str = "\u{f021a}";
/// Nerd Fonts: nf-md-alert
pub const WARNING: &str = "\u{f0026}";
/// Nerd Fonts: nf-md-power-cycle (Material Design has no `boot` glyph)
pub const BOOT: &str = "\u{f0901}";
/// Nerd Fonts: nf-md-package-variant-closed
pub const PACKAGES: &str = "\u{f03d7}";
/// Nerd Fonts: nf-md-package-variant-closed-plus (Material Design has no `package-check` glyph)
pub const INSTALLED: &str = "\u{f19d5}";
/// Nerd Fonts: nf-md-update
pub const UPDATE: &str = "\u{f06b0}";
/// Nerd Fonts: nf-md-history
pub const HISTORY: &str = "\u{f02da}";
/// Nerd Fonts: nf-md-harddisk
pub const STORAGE: &str = "\u{f02ca}";
/// Nerd Fonts: nf-md-chart-box
pub const DIAGNOSTICS: &str = "\u{f154d}";
/// Nerd Fonts: nf-md-apps
pub const APPS: &str = "\u{f003b}";
/// Nerd Fonts: nf-md-widgets
pub const WIDGET: &str = "\u{f072c}";
/// Nerd Fonts: nf-md-format-font
pub const FONTS: &str = "\u{f06d6}";
/// Nerd Fonts: nf-md-information
pub const INFO: &str = "\u{f02fc}";
/// Nerd Fonts: nf-md-check-circle
pub const SUCCESS: &str = "\u{f05e0}";
/// Nerd Fonts: nf-md-close-circle
pub const ERROR: &str = "\u{f0159}";
/// Nerd Fonts: nf-md-refresh
pub const REFRESH: &str = "\u{f0450}";
/// Nerd Fonts: nf-md-link
pub const LINK: &str = "\u{f0337}";
/// Nerd Fonts: nf-md-monitor
pub const MONITOR: &str = "\u{f0379}";
/// Nerd Fonts: nf-md-keyboard
pub const KEYBOARD: &str = "\u{f030c}";
/// Nerd Fonts: nf-md-image
pub const IMAGE: &str = "\u{f02e9}";
/// Nerd Fonts: nf-md-palette
pub const PALETTE: &str = "\u{f03d8}";
/// Nerd Fonts: nf-md-mouse
pub const MOUSE: &str = "\u{f037d}";
/// Nerd Fonts: nf-md-folder
pub const FOLDER: &str = "\u{f024b}";
/// Nerd Fonts: nf-md-lock
pub const LOCK: &str = "\u{f033e}";
/// Nerd Fonts: nf-md-battery
pub const BATTERY: &str = "\u{f0079}";
/// Nerd Fonts: nf-md-account-multiple
pub const USERS: &str = "\u{f000e}";
/// Nerd Fonts: nf-md-bell
pub const BELL: &str = "\u{f009a}";
/// Nerd Fonts: nf-md-bell-off
pub const BELL_OFF: &str = "\u{f009b}";
/// Nerd Fonts: nf-md-form-textbox
pub const TEXT_EDITOR: &str = "\u{f060e}";
/// Nerd Fonts: nf-md-file-pdf-box
pub const PDF: &str = "\u{f0226}";
/// Nerd Fonts: nf-md-video
pub const VIDEO: &str = "\u{f0567}";
/// Nerd Fonts: nf-md-music
pub const MUSIC: &str = "\u{f075a}";
/// Nerd Fonts: nf-md-clock-outline
pub const CLOCK: &str = "\u{f0150}";
/// Nerd Fonts: nf-md-satellite-variant
pub const SATELLITE: &str = "\u{f0471}";
/// Nerd Fonts: nf-md-ruler
pub const RULER: &str = "\u{f046d}";
/// Nerd Fonts: nf-md-auto-fix
pub const EFFECT: &str = "\u{f0068}";

/// Nerd Fonts: nf-md-dock-bottom
pub const TASKBAR: &str = "\u{f10a9}";
/// Nerd Fonts: nf-md-wallpaper
pub const WALLPAPER: &str = "\u{f0e09}";
/// Nerd Fonts: nf-md-console
pub const TERMINAL: &str = "\u{f018d}";
/// Nerd Fonts: nf-md-rocket-launch
pub const LAUNCHER: &str = "\u{f14de}";
/// Nerd Fonts: nf-md-gauge
pub const TELEMETRY: &str = "\u{f029a}";
/// Nerd Fonts: nf-md-tune-variant
pub const CONTROL_PANEL: &str = "\u{f1542}";
/// Nerd Fonts: nf-md-view-dashboard
pub const LAYOUT: &str = "\u{f056e}";
/// Nerd Fonts: nf-md-theme-light-dark
pub const THEME_MODE: &str = "\u{f050e}";
/// Nerd Fonts: nf-md-window-restore
pub const WINDOW_FLOAT: &str = "\u{f05b2}";
/// Nerd Fonts: nf-md-view-split-vertical
pub const WINDOW_STICKY: &str = "\u{f0bcc}";
/// Nerd Fonts: nf-md-window-open
pub const WINDOW_RULES: &str = "\u{f05b1}";
/// Nerd Fonts: nf-md-wrench
pub const WRENCH: &str = "\u{f05b7}";
/// Nerd Fonts: nf-md-folder-multiple
pub const PROJECTS: &str = "\u{f0253}";
/// Nerd Fonts: nf-md-format-color-fill
pub const ACCENT: &str = "\u{f0266}";
/// Nerd Fonts: nf-md-arrow-up-bold
pub const ARROW_UP: &str = "\u{f0737}";
/// Nerd Fonts: nf-md-arrow-down-bold
pub const ARROW_DOWN: &str = "\u{f072e}";
/// Nerd Fonts: nf-md-arrow-left-bold
pub const ARROW_LEFT: &str = "\u{f0731}";
/// Nerd Fonts: nf-md-arrow-right-bold
pub const ARROW_RIGHT: &str = "\u{f0734}";
/// Nerd Fonts: nf-md-arrow-expand-horizontal
pub const GAP: &str = "\u{f084e}";
/// Nerd Fonts: nf-md-border-style
pub const BORDER: &str = "\u{f00d0}";
/// Nerd Fonts: nf-md-rounded-corner
pub const ROUNDED: &str = "\u{f0607}";
/// Nerd Fonts: nf-md-format-line-weight
pub const THICKNESS: &str = "\u{f05c9}";
/// Nerd Fonts: nf-md-opacity
pub const OPACITY: &str = "\u{f05cc}";
/// Nerd Fonts: nf-md-blur
pub const BLUR: &str = "\u{f00b5}";
/// Nerd Fonts: nf-md-animation-play
pub const ANIMATION: &str = "\u{f093a}";
/// Nerd Fonts: nf-md-calendar
pub const CALENDAR: &str = "\u{f00ed}";
/// Nerd Fonts: nf-md-timer-outline
pub const TIMER: &str = "\u{f051b}";
/// Nerd Fonts: nf-md-translate
pub const TRANSLATE: &str = "\u{f05ca}";
/// Nerd Fonts: nf-md-earth
pub const EARTH: &str = "\u{f01e7}";
/// Nerd Fonts: nf-md-check-bold
pub const APPLY: &str = "\u{f0e1e}";
/// Nerd Fonts: nf-md-cancel
pub const CANCEL: &str = "\u{f073a}";
/// Nerd Fonts: nf-md-delete
pub const DELETE: &str = "\u{f01b4}";
/// Nerd Fonts: nf-md-content-copy
pub const COPY: &str = "\u{f018f}";
/// Nerd Fonts: nf-md-restore
pub const RESTORE: &str = "\u{f099b}";
/// Nerd Fonts: nf-md-import
pub const IMPORT: &str = "\u{f02fa}";
/// Nerd Fonts: nf-md-export
pub const EXPORT: &str = "\u{f0207}";
/// Nerd Fonts: nf-md-pencil
pub const EDIT: &str = "\u{f03eb}";
/// Nerd Fonts: nf-md-plus
pub const ADD: &str = "\u{f0415}";
/// Nerd Fonts: nf-md-play
pub const PLAY: &str = "\u{f040a}";
/// Nerd Fonts: nf-md-stop
pub const STOP: &str = "\u{f04db}";
/// Nerd Fonts: nf-md-restart
pub const RESTART: &str = "\u{f0709}";
/// Nerd Fonts: nf-md-filter-variant
pub const FILTER: &str = "\u{f0236}";
/// Nerd Fonts: nf-md-power-sleep
pub const SLEEP: &str = "\u{f0904}";
/// Nerd Fonts: nf-md-snowflake
pub const HIBERNATE: &str = "\u{f0717}";
/// Nerd Fonts: nf-md-lan-connect
pub const LINK_ON: &str = "\u{f0318}";
/// Nerd Fonts: nf-md-lan-disconnect
pub const LINK_OFF: &str = "\u{f0319}";
/// Nerd Fonts: nf-md-shield-outline
pub const SHIELD: &str = "\u{f0499}";
/// Nerd Fonts: nf-md-broom
pub const CLEAN: &str = "\u{f00e2}";
/// Nerd Fonts: nf-md-package-down
pub const DOWNGRADE: &str = "\u{f03d4}";
/// Nerd Fonts: nf-md-database-refresh
pub const DATABASE: &str = "\u{f05c2}";
/// Nerd Fonts: nf-md-devices
pub const DEVICES: &str = "\u{f0fb0}";
/// Nerd Fonts: nf-md-layers-outline
pub const PROFILE: &str = "\u{f09fe}";
/// Nerd Fonts: nf-md-launch
pub const AUTOSTART: &str = "\u{f0327}";
/// Nerd Fonts: nf-md-usb
pub const USB: &str = "\u{f0553}";
/// Nerd Fonts: nf-md-volume-plus
pub const VOLUME_UP: &str = "\u{f075d}";
/// Nerd Fonts: nf-md-volume-minus
pub const VOLUME_DOWN: &str = "\u{f075e}";
/// Nerd Fonts: nf-md-volume-off
pub const MUTE: &str = "\u{f0581}";
/// Nerd Fonts: nf-md-shield-check
pub const TRUST: &str = "\u{f0565}";
/// Nerd Fonts: nf-md-eye
pub const VISIBLE: &str = "\u{f0208}";
/// Nerd Fonts: nf-md-priority-high
pub const PRIORITY_HIGH: &str = "\u{f0603}";
/// Nerd Fonts: nf-md-power-plug
pub const POWER_PLUG: &str = "\u{f06a5}";
/// Nerd Fonts: nf-md-key
pub const KEY: &str = "\u{f0306}";
/// Nerd Fonts: nf-md-lock-open
pub const LOCK_OPEN: &str = "\u{f033f}";
/// Nerd Fonts: nf-md-lock-reset
pub const LOCK_RESET: &str = "\u{f0773}";
/// Nerd Fonts: nf-md-account-circle
pub const AVATAR: &str = "\u{f0009}";
/// Nerd Fonts: nf-md-image-remove
pub const IMAGE_REMOVE: &str = "\u{f1418}";
/// Nerd Fonts: nf-md-account-remove
pub const ACCOUNT_REMOVE: &str = "\u{f0015}";
/// Nerd Fonts: nf-md-delete-forever
pub const DELETE_FOREVER: &str = "\u{f05e8}";
/// Nerd Fonts: nf-md-account-cog
pub const ACCOUNT_COG: &str = "\u{f1370}";
/// Nerd Fonts: nf-md-account-star
pub const ACCOUNT_STAR: &str = "\u{f0017}";
/// Nerd Fonts: nf-md-account-group
pub const GROUP: &str = "\u{f0849}";
/// Nerd Fonts: nf-md-card-account-details
pub const ID_CARD: &str = "\u{f05d2}";
/// Nerd Fonts: nf-md-keyboard-variant
pub const KEYBOARD_VARIANT: &str = "\u{f0313}";
/// Nerd Fonts: nf-md-keyboard-off
pub const KEYBOARD_OFF: &str = "\u{f0310}";
/// Nerd Fonts: nf-md-format-size
pub const FONT_SIZE: &str = "\u{f027f}";
/// Nerd Fonts: nf-md-script-text
pub const SCRIPT: &str = "\u{f0bc2}";
/// Nerd Fonts: nf-md-shield-refresh
pub const SHIELD_REFRESH: &str = "\u{f00aa}";
/// Nerd Fonts: nf-md-aspect-ratio
pub const ASPECT_RATIO: &str = "\u{f0a24}";
/// Nerd Fonts: nf-md-sine-wave
pub const SINE_WAVE: &str = "\u{f095b}";
/// Nerd Fonts: nf-md-magnify-plus-outline
pub const ZOOM: &str = "\u{f06ed}";
/// Nerd Fonts: nf-md-arrow-all
pub const ARROW_ALL: &str = "\u{f0041}";
/// Nerd Fonts: nf-md-screen-rotation
pub const ROTATE: &str = "\u{f0475}";
/// Nerd Fonts: nf-md-monitor-multiple
pub const MONITOR_MULTIPLE: &str = "\u{f037a}";
/// Nerd Fonts: nf-md-sync
pub const SYNC: &str = "\u{f04e6}";
/// Nerd Fonts: nf-md-hdr
pub const HDR: &str = "\u{f0d7d}";
/// Nerd Fonts: nf-md-brightness-6
pub const BRIGHTNESS: &str = "\u{f00df}";
/// Nerd Fonts: nf-md-contrast-box
pub const CONTRAST: &str = "\u{f0196}";
/// Nerd Fonts: nf-md-view-grid
pub const GRID: &str = "\u{f0570}";
/// Nerd Fonts: nf-md-star
pub const STAR: &str = "\u{f04ce}";
/// Nerd Fonts: nf-md-sort
pub const SORT: &str = "\u{f04ba}";
/// Nerd Fonts: nf-md-counter
pub const COUNTER: &str = "\u{f0199}";
/// Nerd Fonts: nf-md-routes
pub const PROXY: &str = "\u{f046a}";
/// Nerd Fonts: nf-md-autorenew
pub const AUTORENEW: &str = "\u{f006a}";

#[cfg(test)]
mod tests {
  use super::*;

  /// Defines the constant `ALL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  const ALL: &[&str] = &[
    NETWORK,
    WIFI,
    ETHERNET,
    VPN,
    DNS,
    SEARCH,
    BLUETOOTH,
    AUDIO,
    SPEAKER,
    MICROPHONE,
    HARDWARE,
    CPU,
    GPU,
    MEMORY,
    POWER,
    SETTINGS,
    SERVICES,
    USER,
    LOGS,
    WARNING,
    BOOT,
    PACKAGES,
    INSTALLED,
    UPDATE,
    HISTORY,
    STORAGE,
    DIAGNOSTICS,
    APPS,
    WIDGET,
    FONTS,
    INFO,
    SUCCESS,
    ERROR,
    REFRESH,
    LINK,
    MONITOR,
    KEYBOARD,
    IMAGE,
    PALETTE,
    MOUSE,
    FOLDER,
    LOCK,
    BATTERY,
    USERS,
    TEXT_EDITOR,
    PDF,
    VIDEO,
    MUSIC,
    CLOCK,
    SATELLITE,
    RULER,
    EFFECT,
    BELL,
    BELL_OFF,
    TASKBAR,
    WALLPAPER,
    TERMINAL,
    LAUNCHER,
    TELEMETRY,
    CONTROL_PANEL,
    LAYOUT,
    THEME_MODE,
    WINDOW_FLOAT,
    WINDOW_STICKY,
    WINDOW_RULES,
    WRENCH,
    PROJECTS,
    ACCENT,
    ARROW_UP,
    ARROW_DOWN,
    ARROW_LEFT,
    ARROW_RIGHT,
    GAP,
    BORDER,
    ROUNDED,
    THICKNESS,
    OPACITY,
    BLUR,
    ANIMATION,
    CALENDAR,
    TIMER,
    TRANSLATE,
    EARTH,
    APPLY,
    CANCEL,
    DELETE,
    RESTORE,
    IMPORT,
    EXPORT,
    EDIT,
    ADD,
    PLAY,
    STOP,
    RESTART,
    FILTER,
    SLEEP,
    HIBERNATE,
    LINK_ON,
    LINK_OFF,
    SHIELD,
    CLEAN,
    DOWNGRADE,
    DATABASE,
    DEVICES,
    PROFILE,
    AUTOSTART,
    USB,
    VOLUME_UP,
    VOLUME_DOWN,
    MUTE,
    TRUST,
    VISIBLE,
    PRIORITY_HIGH,
    POWER_PLUG,
    KEY,
    LOCK_OPEN,
    LOCK_RESET,
    AVATAR,
    IMAGE_REMOVE,
    ACCOUNT_REMOVE,
    DELETE_FOREVER,
    ACCOUNT_COG,
    ACCOUNT_STAR,
    GROUP,
    ID_CARD,
    KEYBOARD_VARIANT,
    KEYBOARD_OFF,
    FONT_SIZE,
    SCRIPT,
    SHIELD_REFRESH,
    ASPECT_RATIO,
    SINE_WAVE,
    ZOOM,
    ARROW_ALL,
    ROTATE,
    MONITOR_MULTIPLE,
    SYNC,
    HDR,
    BRIGHTNESS,
    CONTRAST,
    GRID,
    STAR,
    SORT,
    COUNTER,
    PROXY,
    AUTORENEW,
  ];

  #[test]
  /// Executes the `catalog_entries_are_single_non_whitespace_glyphs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn catalog_entries_are_single_non_whitespace_glyphs() {
    for icon in ALL {
      assert!(!icon.is_empty());
      assert_eq!(icon.trim(), *icon);
      assert!(!icon.chars().any(char::is_whitespace));
      assert_eq!(crate::text::display_width(icon), 1, "{icon:?}");
    }
  }

  #[test]
  /// Two constants sharing a codepoint would make sibling items look identical
  /// while claiming different meanings, so every catalog entry must be unique.
  fn catalog_entries_have_unique_codepoints() {
    let mut seen = std::collections::HashSet::new();
    for icon in ALL {
      assert!(seen.insert(*icon), "duplicated icon codepoint {icon:?}");
    }
  }

  #[test]
  /// Executes the `icon_label_has_stable_geometry_and_safe_narrow_widths` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn icon_label_has_stable_geometry_and_safe_narrow_widths() {
    assert_eq!(icon_label(WIFI, "Network"), format!("{WIFI} Network"));
    assert_eq!(crate::text::display_width(&icon_label(WIFI, "Network")), 9);
    assert_eq!(icon_label("", "Network"), "Network");
  }
}
