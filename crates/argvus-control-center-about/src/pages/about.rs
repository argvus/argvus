//! About ARGVUS page: the desktop modules grouped by area.
use crate::app::App;
use crate::i18n::tr;

use super::{ARGVUS_URL, DONATE_URL, Doc, Row, simple_doc};

pub struct Module {
  pub name: &'static str,
  pub key: &'static str,
}

pub struct Group {
  pub key: &'static str,
  pub modules: &'static [Module],
}

pub fn doc(app: &App, width: usize, selected: usize) -> Doc<'static> {
  let lang = app.lang;
  let mut rows = vec![
    Row::Lead(
      tr(
        lang,
        "control_center.argvus_is_a_modular_collection_of_packages_that_together_provide_a_com",
      )
      .to_string(),
    ),
    Row::Spacer,
    kv_row(
      lang,
      "control_center.about_version",
      app.argvus_version().to_string(),
    ),
    kv_row(
      lang,
      "control_center.about_license",
      "GPL-3.0-only".to_string(),
    ),
    Row::Spacer,
  ];

  for group in groups() {
    rows.push(Row::Divider {
      label: Some(tr(lang, group.key).to_string()),
    });
    for module in group.modules {
      rows.push(Row::Module {
        name: module.name.to_string(),
        description: tr(lang, module.key).to_string(),
      });
    }
  }

  rows.push(Row::Spacer);
  rows.push(Row::Divider {
    label: Some(tr(lang, "control_center.links").to_string()),
  });
  rows.push(Row::Link {
    label: ARGVUS_URL.to_string(),
    url: ARGVUS_URL.to_string(),
  });
  rows.push(Row::Link {
    label: tr(lang, "control_center.support_the_project").to_string(),
    url: DONATE_URL.to_string(),
  });

  simple_doc(&rows, &app.theme, width, selected)
}

fn kv_row(lang: crate::i18n::Lang, key: &str, value: String) -> Row {
  Row::KeyValue {
    key: format!("{:<8}", tr(lang, key)),
    value,
  }
}

pub fn groups() -> [Group; 4] {
  [
    Group {
      key: "control_center.about_core",
      modules: &[
        Module {
          name: "argvus-session",
          key: "control_center.about_session_lifecycle_targets_and_hyprland_integration",
        },
        Module {
          name: "argvus-hyprland",
          key: "control_center.about_hyprland_configuration_and_argvus_shell_scripts",
        },
        Module {
          name: "argvus-portal",
          key: "control_center.about_wayland_portals_dbus_and_integration_preferences",
        },
        Module {
          name: "argvus",
          key: "control_center.about_master_package_command_and_shared_runtime",
        },
      ],
    },
    Group {
      key: "control_center.about_shell_and_desktop",
      modules: &[
        Module {
          name: "argvus-launcher",
          key: "control_center.about_rofi_launcher_menus_and_themes",
        },
        Module {
          name: "argvus-control-panel",
          key: "control_center.about_quickshell_sidebar_control_panel_and_its_themes",
        },
        Module {
          name: "argvus-appearance",
          key: "control_center.about_themes_fonts_wallpapers_and_visual_integration",
        },
        Module {
          name: "argvus taskbar-calendar",
          key: "control_center.about_calendar_and_taskbar_popup_integration",
        },
        Module {
          name: "argvus removable-devices",
          key: "control_center.about_removable_device_and_storage_module",
        },
        Module {
          name: "argvus greeter",
          key: "control_center.about_argvus_graphical_login_screen",
        },
        Module {
          name: "argvus-lock",
          key: "control_center.about_screen_locking_and_lock_screen_themes",
        },
      ],
    },
    Group {
      key: "control_center.about_configuration",
      modules: &[
        Module {
          name: "argvus control-center",
          key: "control_center.about_argvus_control_center_including_fonts_and_default_applications",
        },
        Module {
          name: "argvus accounts",
          key: "control_center.about_account_and_user_settings",
        },
        Module {
          name: "argvus-display",
          key: "control_center.about_monitor_and_layout_management",
        },
      ],
    },
    Group {
      key: "control_center.about_services_and_power",
      modules: &[
        Module {
          name: "argvus network",
          key: "control_center.about_networkmanager_wi_fi_and_bluetooth",
        },
        Module {
          name: "argvus-power",
          key: "control_center.about_power_menu_idle_and_session_actions",
        },
      ],
    },
  ]
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::App;

  #[test]
  fn about_doc_mentions_all_modules() {
    use crate::app::Tab;
    let mut app = App::test();
    app.active_tab = Tab::About;
    let doc = doc(&app, 100, 0);
    assert!(doc.actions.iter().any(|a| a.url == ARGVUS_URL));
    let text = doc.lines.iter().map(|l| l.to_string()).collect::<String>();
    assert!(text.contains("argvus-session"));
    assert!(text.contains("argvus-portal"));
  }

  #[test]
  fn every_group_is_represented_in_both_languages() {
    use crate::i18n::Lang;
    for lang in [Lang::for_locale("pt-BR"), Lang::for_locale("en-US")] {
      let mut app = App::test();
      app.lang = lang;
      let doc = doc(&app, 80, 0);
      let text = doc.lines.iter().map(|l| l.to_string()).collect::<String>();
      for group in groups() {
        let label = tr(lang, group.key);
        assert!(
          text.contains(label),
          "missing group '{}' for {:?}",
          label,
          lang
        );
      }
    }
  }

  #[test]
  fn about_offers_site_and_support_links() {
    let doc = doc(&App::test(), 80, 0);
    assert!(doc.actions.iter().any(|a| a.url == ARGVUS_URL));
    assert!(doc.actions.iter().any(|a| a.url == DONATE_URL));
  }

  #[test]
  fn groups_cover_every_module_name() {
    let names = groups()
      .iter()
      .flat_map(|group| group.modules.iter())
      .map(|module| module.name)
      .collect::<Vec<_>>();
    assert_eq!(names.len(), 16);
    assert!(names.contains(&"argvus"));
    assert!(names.contains(&"argvus removable-devices"));
    assert!(
      !names
        .iter()
        .any(|name| name.contains("argvus-taskbar-calendar")
          || name.contains("argvus-removable-devices")
          || name.contains("argvus-about"))
    );
  }
}
