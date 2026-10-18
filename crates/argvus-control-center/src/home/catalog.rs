//! Catalog of the Home screen: the six cards and the destinations they list.
//!
//! Each destination has a stable identifier and owns its icon, and each card
//! owns its title, description and icon. The feature gate of every destination
//! is the gate of the row that lists it; runtime availability (capabilities,
//! system locales) is passed in explicitly through [`HomeAvailability`].

use argvus_control_center_core::navigation::HomeCardId;
use argvus_i18n::{Lang, tr};
use argvus_tui::icons;

/// Stable identifier of one Home destination. A variant exists only when the
/// feature that provides its page is compiled in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeDestination {
  #[cfg(feature = "appearance")]
  DesktopStyle,
  #[cfg(feature = "appearance")]
  ApplicationStyle,
  #[cfg(feature = "appearance")]
  Taskbar,
  #[cfg(feature = "appearance")]
  ControlCenter,
  #[cfg(feature = "appearance")]
  WidgetTelemetry,
  Hyprland,
  #[cfg(feature = "displays")]
  Displays,
  KeyboardAndShortcuts,
  #[cfg(feature = "hardware")]
  MouseTouchpad,
  #[cfg(feature = "locale")]
  LanguageRegion,
  #[cfg(feature = "locale")]
  DateTime,
  #[cfg(feature = "apps")]
  DefaultApplications,
  #[cfg(feature = "power")]
  Power,
  #[cfg(feature = "session")]
  Session,
  #[cfg(feature = "hardware")]
  HardwareOverview,
  #[cfg(feature = "network")]
  Network,
  #[cfg(feature = "bluetooth")]
  Bluetooth,
  #[cfg(feature = "audio")]
  Audio,
  #[cfg(feature = "storage")]
  Storage,
  #[cfg(feature = "packages")]
  Packages,
  #[cfg(feature = "services")]
  Services,
  #[cfg(feature = "boot")]
  Boot,
  #[cfg(feature = "system")]
  UsersGroups,
  #[cfg(feature = "system")]
  Firewall,
  #[cfg(feature = "system")]
  Hostname,
  #[cfg(feature = "system")]
  Notifications,
  #[cfg(feature = "apps")]
  Projects,
  #[cfg(feature = "apps")]
  Snippets,
  #[cfg(feature = "apps")]
  SshKeys,
  #[cfg(feature = "apps")]
  GpgKeys,
  #[cfg(feature = "dev-services")]
  DevServices,
  #[cfg(feature = "diagnostics")]
  Diagnostics,
  #[cfg(feature = "about")]
  About,
  Settings,
}

impl HomeDestination {
  /// Icon of the destination. Every destination has one, and no two
  /// destinations of the same card share it.
  pub const fn glyph(self) -> &'static str {
    match self {
      #[cfg(feature = "appearance")]
      Self::DesktopStyle => icons::WALLPAPER,
      #[cfg(feature = "appearance")]
      Self::ApplicationStyle => icons::TERMINAL,
      #[cfg(feature = "appearance")]
      Self::Taskbar => icons::TASKBAR,
      #[cfg(feature = "appearance")]
      Self::ControlCenter => icons::CONTROL_PANEL,
      #[cfg(feature = "appearance")]
      Self::WidgetTelemetry => icons::TELEMETRY,
      Self::Hyprland => icons::LAYOUT,
      #[cfg(feature = "displays")]
      Self::Displays => icons::MONITOR,
      Self::KeyboardAndShortcuts => icons::KEYBOARD,
      #[cfg(feature = "hardware")]
      Self::MouseTouchpad => icons::MOUSE,
      #[cfg(feature = "locale")]
      Self::LanguageRegion => icons::EARTH,
      #[cfg(feature = "locale")]
      Self::DateTime => icons::CALENDAR,
      #[cfg(feature = "apps")]
      Self::DefaultApplications => icons::APPS,
      #[cfg(feature = "power")]
      Self::Power => icons::POWER,
      #[cfg(feature = "session")]
      Self::Session => icons::REFRESH,
      #[cfg(feature = "hardware")]
      Self::HardwareOverview => icons::CPU,
      #[cfg(feature = "network")]
      Self::Network => icons::NETWORK,
      #[cfg(feature = "bluetooth")]
      Self::Bluetooth => icons::LINK,
      #[cfg(feature = "audio")]
      Self::Audio => icons::AUDIO,
      #[cfg(feature = "storage")]
      Self::Storage => icons::STORAGE,
      #[cfg(feature = "packages")]
      Self::Packages => icons::PACKAGES,
      #[cfg(feature = "services")]
      Self::Services => icons::SERVICES,
      #[cfg(feature = "boot")]
      Self::Boot => icons::BOOT,
      #[cfg(feature = "system")]
      Self::UsersGroups => icons::USERS,
      #[cfg(feature = "system")]
      Self::Firewall => icons::SHIELD,
      #[cfg(feature = "system")]
      Self::Hostname => icons::MONITOR,
      #[cfg(feature = "system")]
      Self::Notifications => icons::BELL_OFF,
      #[cfg(feature = "apps")]
      Self::Projects => icons::PROJECTS,
      #[cfg(feature = "apps")]
      Self::Snippets => icons::TEXT_EDITOR,
      #[cfg(feature = "apps")]
      Self::SshKeys => icons::KEY,
      #[cfg(feature = "apps")]
      Self::GpgKeys => icons::LOCK,
      #[cfg(feature = "dev-services")]
      Self::DevServices => icons::SCRIPT,
      #[cfg(feature = "diagnostics")]
      Self::Diagnostics => icons::DIAGNOSTICS,
      #[cfg(feature = "about")]
      Self::About => icons::INFO,
      Self::Settings => icons::SETTINGS,
    }
  }
}

/// Title, description and icon of one Home card, already translated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HomeCategory {
  pub title: &'static str,
  pub description: &'static str,
  pub glyph: &'static str,
}

/// One line of the Home catalog: a card heading or a destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeRow {
  Header(HomeCategory),
  Item {
    label: &'static str,
    destination: HomeDestination,
  },
  /// A destination listed outside the cards, below them (the Settings entry).
  Standalone {
    label: &'static str,
    destination: HomeDestination,
  },
}

/// Runtime facts that decide whether a destination is listed. Built once from
/// the detected capabilities so the catalog itself stays a pure function.
#[derive(Debug, Clone, Copy, Default)]
pub struct HomeAvailability {
  pub hyprctl: bool,
  pub loginctl: bool,
  pub sessionctl: bool,
  pub bluetooth: bool,
  pub systemd: bool,
  pub podman: bool,
  pub docker: bool,
}

/// Builds the Home catalog for a language and the detected availability. The
/// cards appear in the order of the approved layout; a card without any
/// available destination is not listed at all.
#[allow(clippy::vec_init_then_push)]
pub fn home_rows(
  lang: Lang,
  // Unread when no feature that depends on availability is compiled in.
  #[allow(unused_variables)] availability: HomeAvailability,
) -> Vec<HomeRow> {
  let mut rows = Vec::new();

  #[allow(unused_mut)]
  let mut items = Vec::new();
  #[cfg(feature = "appearance")]
  items.push(item(
    lang,
    "control_center.desktop_style",
    HomeDestination::DesktopStyle,
  ));
  #[cfg(feature = "appearance")]
  items.push(item(
    lang,
    "control_center.application_style",
    HomeDestination::ApplicationStyle,
  ));
  #[cfg(feature = "appearance")]
  items.push(item(
    lang,
    "control_center.taskbar",
    HomeDestination::Taskbar,
  ));
  #[cfg(feature = "appearance")]
  items.push(item(
    lang,
    "control_center.control_panel",
    HomeDestination::ControlCenter,
  ));
  #[cfg(feature = "appearance")]
  items.push(item(
    lang,
    "control_center.widget_telemetry",
    HomeDestination::WidgetTelemetry,
  ));
  push_card(
    &mut rows,
    card(
      lang,
      HomeCardId::Appearance.title_key(),
      "control_center.card_appearance_description",
      icons::PALETTE,
    ),
    items,
  );

  #[allow(unused_mut)]
  let mut items = Vec::new();
  items.push(item(
    lang,
    "control_center.hyprland",
    HomeDestination::Hyprland,
  ));
  #[cfg(feature = "displays")]
  if availability.hyprctl {
    items.push(item(
      lang,
      "control_center.displays",
      HomeDestination::Displays,
    ));
  }
  #[cfg(feature = "hardware")]
  items.push(item(
    lang,
    "control_center.keyboard_and_shortcuts",
    HomeDestination::KeyboardAndShortcuts,
  ));
  #[cfg(feature = "hardware")]
  items.push(item(
    lang,
    "control_center.mouse_touchpad",
    HomeDestination::MouseTouchpad,
  ));
  push_card(
    &mut rows,
    card(
      lang,
      HomeCardId::DesktopInput.title_key(),
      "control_center.card_desktop_input_description",
      icons::KEYBOARD,
    ),
    items,
  );

  #[allow(unused_mut)]
  let mut items = Vec::new();
  #[cfg(feature = "locale")]
  items.push(item(
    lang,
    "control_center.language_region",
    HomeDestination::LanguageRegion,
  ));
  #[cfg(feature = "locale")]
  items.push(item(
    lang,
    "control_center.date_time",
    HomeDestination::DateTime,
  ));
  #[cfg(feature = "apps")]
  items.push(item(
    lang,
    "control_center.default_apps",
    HomeDestination::DefaultApplications,
  ));
  #[cfg(feature = "power")]
  if availability.loginctl {
    items.push(item(lang, "control_center.power", HomeDestination::Power));
  }
  #[cfg(feature = "session")]
  if availability.sessionctl {
    items.push(item(
      lang,
      "control_center.session",
      HomeDestination::Session,
    ));
  }
  push_card(
    &mut rows,
    card(
      lang,
      HomeCardId::Preferences.title_key(),
      "control_center.card_preferences_description",
      icons::ACCOUNT_COG,
    ),
    items,
  );

  #[allow(unused_mut)]
  let mut items = Vec::new();
  #[cfg(feature = "hardware")]
  items.push(item(
    lang,
    "control_center.hardware_overview",
    HomeDestination::HardwareOverview,
  ));
  #[cfg(feature = "network")]
  items.push(item(
    lang,
    "control_center.network_wifi",
    HomeDestination::Network,
  ));
  #[cfg(feature = "bluetooth")]
  if availability.bluetooth {
    items.push(item(
      lang,
      "control_center.bluetooth",
      HomeDestination::Bluetooth,
    ));
  }
  #[cfg(feature = "audio")]
  items.push(item(lang, "control_center.audio", HomeDestination::Audio));
  #[cfg(feature = "storage")]
  items.push(item(
    lang,
    "control_center.storage",
    HomeDestination::Storage,
  ));
  push_card(
    &mut rows,
    card(
      lang,
      HomeCardId::Hardware.title_key(),
      "control_center.card_hardware_description",
      icons::HARDWARE,
    ),
    items,
  );

  #[allow(unused_mut)]
  let mut items = Vec::new();
  #[cfg(feature = "packages")]
  items.push(item(
    lang,
    "control_center.packages",
    HomeDestination::Packages,
  ));
  #[cfg(feature = "services")]
  items.push(item(
    lang,
    "control_center.services",
    HomeDestination::Services,
  ));
  #[cfg(feature = "boot")]
  items.push(item(lang, "control_center.boot", HomeDestination::Boot));
  #[cfg(feature = "system")]
  items.push(item(
    lang,
    "control_center.hostname",
    HomeDestination::Hostname,
  ));
  #[cfg(feature = "system")]
  items.push(item(
    lang,
    "control_center.users_groups",
    HomeDestination::UsersGroups,
  ));
  #[cfg(feature = "system")]
  items.push(item(
    lang,
    "control_center.firewall",
    HomeDestination::Firewall,
  ));
  #[cfg(feature = "system")]
  items.push(item(
    lang,
    "control_center.notifications",
    HomeDestination::Notifications,
  ));
  push_card(
    &mut rows,
    card(
      lang,
      HomeCardId::System.title_key(),
      "control_center.card_system_description",
      icons::SETTINGS,
    ),
    items,
  );

  #[allow(unused_mut)]
  let mut items = Vec::new();
  #[cfg(feature = "apps")]
  items.push(item(
    lang,
    "control_center.projects",
    HomeDestination::Projects,
  ));
  #[cfg(feature = "apps")]
  items.push(item(
    lang,
    "control_center.snippets",
    HomeDestination::Snippets,
  ));
  #[cfg(feature = "apps")]
  items.push(item(
    lang,
    "control_center.ssh_keys",
    HomeDestination::SshKeys,
  ));
  #[cfg(feature = "apps")]
  items.push(item(
    lang,
    "control_center.gpg_keys",
    HomeDestination::GpgKeys,
  ));
  #[cfg(feature = "dev-services")]
  if availability.systemd || availability.podman || availability.docker {
    items.push(item(
      lang,
      "control_center.developer_services",
      HomeDestination::DevServices,
    ));
  }
  #[cfg(feature = "diagnostics")]
  items.push(item(
    lang,
    "control_center.diagnostics",
    HomeDestination::Diagnostics,
  ));
  #[cfg(feature = "about")]
  items.push(item(
    lang,
    "control_center.about_argvus",
    HomeDestination::About,
  ));
  push_card(
    &mut rows,
    card(
      lang,
      HomeCardId::Tools.title_key(),
      "control_center.card_tools_description",
      icons::WRENCH,
    ),
    items,
  );

  rows.push(HomeRow::Standalone {
    label: tr(lang, "control_center.settings"),
    destination: HomeDestination::Settings,
  });
  rows
}

/// Builds the heading of a card from its i18n keys and icon.
fn card(lang: Lang, title_key: &str, description_key: &str, glyph: &'static str) -> HomeCategory {
  HomeCategory {
    title: tr(lang, title_key),
    description: tr(lang, description_key),
    glyph,
  }
}

/// Appends a card heading and its items, unless the card has nothing to show.
fn push_card(rows: &mut Vec<HomeRow>, category: HomeCategory, items: Vec<HomeRow>) {
  if items.is_empty() {
    return;
  }
  rows.push(HomeRow::Header(category));
  rows.extend(items);
}

/// Shorthand for an item row whose label comes from the i18n catalog.
fn item(lang: Lang, key: &str, destination: HomeDestination) -> HomeRow {
  HomeRow::Item {
    label: tr(lang, key),
    destination,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn english() -> Lang {
    Lang::for_locale("en-US")
  }

  fn everything() -> HomeAvailability {
    HomeAvailability {
      hyprctl: true,
      loginctl: true,
      sessionctl: true,
      bluetooth: true,
      systemd: true,
      podman: true,
      docker: true,
    }
  }

  fn destinations(rows: &[HomeRow]) -> Vec<HomeDestination> {
    rows
      .iter()
      .filter_map(|row| match row {
        HomeRow::Item { destination, .. } | HomeRow::Standalone { destination, .. } => {
          Some(*destination)
        }
        HomeRow::Header(_) => None,
      })
      .collect()
  }

  #[test]
  /// A destination appears at most once, so the same page is not listed twice.
  fn each_destination_is_listed_once() {
    let listed = destinations(&home_rows(english(), everything()));
    for (index, destination) in listed.iter().enumerate() {
      assert!(
        !listed[index + 1..].contains(destination),
        "{destination:?} is listed twice"
      );
    }
  }

  #[test]
  /// Every listed destination has an icon.
  fn every_destination_has_a_glyph() {
    for destination in destinations(&home_rows(english(), everything())) {
      assert!(!destination.glyph().is_empty(), "{destination:?}");
    }
  }

  #[test]
  /// Two destinations of the same card never share an icon.
  fn siblings_have_distinct_icons() {
    let rows = home_rows(english(), everything());
    let mut card: Vec<HomeDestination> = Vec::new();
    for row in &rows {
      match row {
        HomeRow::Header(_) => card.clear(),
        HomeRow::Standalone { .. } => card.clear(),
        HomeRow::Item { destination, .. } => {
          assert!(
            card
              .iter()
              .all(|other| other.glyph() != destination.glyph()),
            "{destination:?} shares its icon with a sibling"
          );
          card.push(*destination);
        }
      }
    }
  }

  #[test]
  /// Every card heading is followed by at least one destination.
  fn no_heading_is_left_without_items() {
    let rows = home_rows(english(), everything());
    for (index, row) in rows.iter().enumerate() {
      if matches!(row, HomeRow::Header(_)) {
        assert!(
          matches!(rows.get(index + 1), Some(HomeRow::Item { .. })),
          "heading at row {index} has no items"
        );
      }
    }
  }

  #[test]
  /// The cards follow the approved order of the layout.
  fn cards_follow_the_approved_order() {
    let titles: Vec<&str> = home_rows(english(), everything())
      .iter()
      .filter_map(|row| match row {
        HomeRow::Header(category) => Some(category.title),
        HomeRow::Item { .. } | HomeRow::Standalone { .. } => None,
      })
      .collect();
    assert_eq!(
      titles,
      [
        "Appearance",
        "Desktop & Input",
        "Preferences",
        "Hardware & Connectivity",
        "System Administration",
        "Tools & Diagnostics",
      ]
    );
  }

  #[test]
  /// Capability-dependent destinations disappear when their capability is missing.
  fn capability_gated_destinations_follow_availability() {
    let listed = destinations(&home_rows(english(), HomeAvailability::default()));
    #[cfg(feature = "displays")]
    assert!(!listed.contains(&HomeDestination::Displays));
    #[cfg(feature = "power")]
    assert!(!listed.contains(&HomeDestination::Power));
    #[cfg(feature = "session")]
    assert!(!listed.contains(&HomeDestination::Session));
    #[cfg(feature = "bluetooth")]
    assert!(!listed.contains(&HomeDestination::Bluetooth));
    #[cfg(feature = "dev-services")]
    assert!(!listed.contains(&HomeDestination::DevServices));
  }
}
