//! Builds the System & Hardware tab: Boot, System and Hardware sections with aligned labels.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use argvus_control_center_core::format::human_bytes;
use argvus_tui::text::display_width;

use crate::app::App;
use crate::i18n::{Lang, na, tr};
use crate::system::SystemInfo;
use crate::system::battery::{Battery, BatteryStatus};
use crate::system::boot::BootMode;
use crate::system::memory::MemoryUsage;
use crate::system::packages::PackageCounts;
use crate::system::storage::Storage;

use super::{Doc, Row, simple_doc};

/// Indentation of the labels under each section title.
const LABEL_INDENT: &str = "  ";
/// Separator between the parts of one value (`model · up to 4.30 GHz`).
const PART_SEPARATOR: &str = " · ";
/// Width used for the copied report, wide enough that no value is wrapped.
const REPORT_WIDTH: usize = 1000;

/// One labelled value of a section, before the labels are aligned.
type Field = (&'static str, String);

/// Builds the tab from the gathered [`SystemInfo`].
pub fn doc(app: &App, width: usize, selected: usize) -> Doc<'static> {
  let lang = app.lang;
  if !app.is_loaded() {
    let loading = Row::Para(tr(lang, "control_center.about_loading_system").to_string());
    return simple_doc(&[loading], &app.theme, width, selected);
  }
  let sections = [
    (
      tr(lang, "control_center.boot"),
      boot_fields(&app.system, lang),
    ),
    (tr(lang, "control_center.system"), system_fields(app)),
    (
      tr(lang, "control_center.hardware"),
      hardware_fields(&app.system, lang),
    ),
  ];

  // Every label is padded to the widest one of the page, so values form one column.
  let label_width = sections
    .iter()
    .flat_map(|(_, fields)| fields.iter())
    .map(|(label, _)| display_width(label))
    .max()
    .unwrap_or(0);

  let mut rows = Vec::new();
  for (index, (title, fields)) in sections.into_iter().enumerate() {
    if index > 0 {
      rows.push(Row::Spacer);
    }
    rows.push(Row::Section(title.to_string()));
    rows.extend(fields.into_iter().map(|(label, value)| Row::KeyValue {
      key: aligned_label(label, label_width),
      value,
    }));
  }

  simple_doc(&rows, &app.theme, width, selected)
}

/// Renders the tab as plain text for the clipboard, one unwrapped line per value.
pub fn report_text(app: &App) -> String {
  doc(app, REPORT_WIDTH, usize::MAX)
    .lines
    .iter()
    .map(|line| line.to_string().trim_end().to_string())
    .collect::<Vec<_>>()
    .join("\n")
}

/// Firmware, boot mode, Secure Boot (UEFI only), virtualization and bootloader.
fn boot_fields(info: &SystemInfo, lang: Lang) -> Vec<Field> {
  let missing = na(lang);
  let boot_mode = match info.boot_mode {
    Some(BootMode::Uefi) => "UEFI".to_string(),
    Some(BootMode::Legacy) => tr(lang, "control_center.about_legacy_bios").to_string(),
    None => missing.to_string(),
  };
  let mut fields = vec![
    (
      tr(lang, "control_center.hardware_firmware"),
      or_missing(info.firmware.as_deref(), missing),
    ),
    (tr(lang, "control_center.hardware_type"), boot_mode),
  ];
  if info.boot_mode == Some(BootMode::Uefi) {
    let secure_boot = match info.secure_boot {
      Some(true) => tr(lang, "control_center.enabled"),
      Some(false) => tr(lang, "control_center.disabled"),
      None => missing,
    };
    fields.push((
      tr(lang, "control_center.secure_boot"),
      secure_boot.to_string(),
    ));
  }
  let virtualization = match info.virtualization.as_deref() {
    Some("none") => tr(lang, "control_center.about_no_virtualization").to_string(),
    other => or_missing(other, missing),
  };
  fields.push((
    tr(lang, "control_center.hardware_virtualization"),
    virtualization,
  ));
  fields.push((
    tr(lang, "control_center.about_bootloader"),
    or_missing(info.bootloader.as_deref(), missing),
  ));
  fields
}

/// ARGVUS, operating system, graphics and desktop stack, packages and session.
fn system_fields(app: &App) -> Vec<Field> {
  let (info, lang) = (&app.system, app.lang);
  let missing = na(lang);
  let theme_category = if app.theme.is_light {
    "control_center.theme_category_light"
  } else {
    "control_center.theme_category_dark"
  };
  let displays = info.displays.as_ref().map_or_else(
    || missing.to_string(),
    |displays| {
      displays
        .iter()
        .map(|display| display.label())
        .collect::<Vec<_>>()
        .join("\n")
    },
  );
  vec![
    ("ARGVUS", or_missing(info.argvus.as_deref(), missing)),
    (
      tr(lang, "control_center.hostname"),
      or_missing(info.hostname.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.about_operating_system"),
      or_missing(info.os_name.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.distributor"),
      or_missing(info.distributor.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.kernel"),
      or_missing(info.kernel.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.hardware_architecture"),
      or_missing(info.architecture.as_deref(), missing),
    ),
    // ARGVUS always runs on Hyprland and Wayland; only the version can be missing.
    (
      tr(lang, "control_center.about_window_manager"),
      named_version("Hyprland", info.hyprland.as_deref()),
    ),
    (
      tr(lang, "control_center.about_compositor"),
      named_version("Wayland", info.wayland.as_deref()),
    ),
    (
      tr(lang, "control_center.about_gpu_driver"),
      or_missing(info.gpu_drivers.as_deref(), missing),
    ),
    ("GTK", or_missing(info.gtk.as_deref(), missing)),
    (
      tr(lang, "control_center.theme"),
      format!("{} ({})", app.theme.name, tr(lang, theme_category)),
    ),
    (tr(lang, "control_center.displays"), displays),
    (
      tr(lang, "control_center.audio"),
      installed_version("PipeWire", info.pipewire.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.shell"),
      installed_version("Quickshell", info.quickshell.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.about_login"),
      installed_version("greetd", info.greetd.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.about_init"),
      installed_version("systemd", info.systemd.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.packages"),
      info.packages.map_or_else(
        || missing.to_string(),
        |counts| packages_label(counts, lang),
      ),
    ),
    (
      tr(lang, "control_center.about_uptime"),
      info.uptime_seconds.map_or_else(
        || missing.to_string(),
        |seconds| uptime_label(seconds, lang),
      ),
    ),
    (
      tr(lang, "control_center.about_locale"),
      or_missing(info.locale.as_deref(), missing),
    ),
  ]
}

/// Batteries, processor, graphics, memory, storage and board.
fn hardware_fields(info: &SystemInfo, lang: Lang) -> Vec<Field> {
  let missing = na(lang);
  let cpu = match (info.cpu.as_deref(), info.cpu_max_mhz) {
    (Some(model), Some(mhz)) => format!(
      "{model}{PART_SEPARATOR}{}",
      lang.tr_args(
        "control_center.about_cpu_up_to",
        [("frequency", format!("{:.2} GHz", mhz as f64 / 1000.0))]
      )
    ),
    (Some(model), None) => model.to_string(),
    (None, _) => missing.to_string(),
  };
  let mut fields = vec![
    (
      tr(lang, "control_center.hardware_battery"),
      batteries_label(&info.batteries, lang),
    ),
    (tr(lang, "control_center.hardware_cpu"), cpu),
    (
      tr(lang, "control_center.hardware_threads"),
      count_or_missing(info.threads, missing),
    ),
    (
      tr(lang, "control_center.about_cores"),
      count_or_missing(info.cores, missing),
    ),
    (
      tr(lang, "control_center.hardware_gpu"),
      or_missing(info.gpus.as_deref(), missing),
    ),
    (
      tr(lang, "control_center.memory"),
      info
        .memory
        .map_or_else(|| missing.to_string(), |memory| memory_label(&memory)),
    ),
    (
      tr(lang, "control_center.about_swap"),
      info
        .memory
        .map_or_else(|| missing.to_string(), |memory| swap_label(&memory, lang)),
    ),
    (
      tr(lang, "control_center.storage"),
      info
        .storage
        .as_ref()
        .map_or_else(|| missing.to_string(), storage_label),
    ),
  ];
  // Desktop firmware usually reports the board as the product, so the model is shown
  // only when it adds information (laptops and branded desktops).
  if let Some(model) = info.model.as_deref()
    && info.motherboard.as_deref() != Some(model)
  {
    fields.push((tr(lang, "control_center.hardware_model"), model.to_string()));
  }
  fields.push((
    tr(lang, "control_center.about_motherboard"),
    or_missing(info.motherboard.as_deref(), missing),
  ));
  fields
}

/// One line per battery; names are shown only when there is more than one.
fn batteries_label(batteries: &[Battery], lang: Lang) -> String {
  if batteries.is_empty() {
    return tr(lang, "control_center.about_battery_not_detected").to_string();
  }
  batteries
    .iter()
    .map(|battery| {
      let label = battery_label(battery, lang);
      if batteries.len() > 1 {
        format!("{}: {label}", battery.name)
      } else {
        label
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// Formats `85% (Discharging) · health 91% · 312 cycles`, leaving out unknown parts.
fn battery_label(battery: &Battery, lang: Lang) -> String {
  let status = tr(
    lang,
    match battery.status {
      BatteryStatus::Charging => "control_center.about_battery_charging",
      BatteryStatus::Discharging => "control_center.about_battery_discharging",
      BatteryStatus::Full => "control_center.about_battery_full",
      BatteryStatus::NotCharging => "control_center.about_battery_not_charging",
      BatteryStatus::Unknown => "control_center.unknown",
    },
  );
  let mut parts = vec![match battery.percent {
    Some(percent) => format!("{percent}% ({status})"),
    None => status.to_string(),
  }];
  if let Some(health) = battery.health {
    parts.push(lang.tr_args(
      "control_center.about_battery_health",
      [("percent", health.to_string())],
    ));
  }
  if let Some(cycles) = battery.cycles {
    parts.push(lang.tr_args(
      "control_center.about_battery_cycles",
      [("count", cycles.to_string())],
    ));
  }
  parts.join(PART_SEPARATOR)
}

/// Formats `5.7 GiB / 15.5 GiB` (used / total), or the total alone.
fn memory_label(memory: &MemoryUsage) -> String {
  let total = human_bytes(memory.total_kib.saturating_mul(1024));
  match memory.used_kib() {
    Some(used) => format!("{} / {total}", human_bytes(used.saturating_mul(1024))),
    None => total,
  }
}

/// Formats swap usage; "Disabled" when the system has no swap.
fn swap_label(memory: &MemoryUsage, lang: Lang) -> String {
  match (memory.swap_total_kib, memory.swap_used_kib()) {
    (Some(0), _) => tr(lang, "control_center.disabled").to_string(),
    (Some(total), Some(used)) => format!(
      "{} / {}",
      human_bytes(used.saturating_mul(1024)),
      human_bytes(total.saturating_mul(1024))
    ),
    _ => na(lang).to_string(),
  }
}

/// Formats `Model · NVMe · btrfs · 120.4 GiB / 465.6 GiB`, leaving out unknown parts.
fn storage_label(storage: &Storage) -> String {
  let mut parts = Vec::new();
  parts.extend(storage.model.clone());
  parts.extend(storage.kind.label().map(str::to_string));
  parts.push(storage.filesystem.clone());
  if let (Some(used), Some(total)) = (storage.used_bytes, storage.total_bytes) {
    parts.push(format!("{} / {}", human_bytes(used), human_bytes(total)));
  }
  parts.join(PART_SEPARATOR)
}

/// Formats `1234 (312 explicit, 18 AUR/foreign)`.
fn packages_label(counts: PackageCounts, lang: Lang) -> String {
  lang.tr_args(
    "control_center.about_packages_summary",
    [
      ("total", counts.total.to_string()),
      ("explicit", counts.explicit.to_string()),
      ("foreign", counts.foreign.to_string()),
    ],
  )
}

/// Formats an uptime as `3 d 4 h 12 min`, skipping leading zero units.
fn uptime_label(seconds: u64, lang: Lang) -> String {
  let (days, hours, minutes) = (
    seconds / 86_400,
    seconds % 86_400 / 3_600,
    seconds % 3_600 / 60,
  );
  let mut parts = Vec::new();
  if days > 0 {
    parts.push(unit(lang, "control_center.about_uptime_days", days));
  }
  if days > 0 || hours > 0 {
    parts.push(unit(lang, "control_center.about_uptime_hours", hours));
  }
  parts.push(unit(lang, "control_center.about_uptime_minutes", minutes));
  parts.join(" ")
}

fn unit(lang: Lang, key: &str, count: u64) -> String {
  lang.tr_args(key, [("count", count.to_string())])
}

/// Indents a label and pads it to `width` columns.
fn aligned_label(label: &str, width: usize) -> String {
  let padding = width.saturating_sub(display_width(label));
  format!("{LABEL_INDENT}{label}{}", " ".repeat(padding))
}

/// Joins a component name and its version, such as `Hyprland 0.56.2`.
fn named_version(name: &str, version: Option<&str>) -> String {
  version.map_or_else(|| name.to_string(), |version| format!("{name} {version}"))
}

/// Like [`named_version`], but for optional components: "N/A" when not installed.
fn installed_version(name: &str, version: Option<&str>, missing: &str) -> String {
  version.map_or_else(
    || missing.to_string(),
    |version| format!("{name} {version}"),
  )
}

fn or_missing(value: Option<&str>, missing: &str) -> String {
  value.unwrap_or(missing).to_string()
}

fn count_or_missing(value: Option<usize>, missing: &str) -> String {
  value.map_or_else(|| missing.to_string(), |count| count.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::App;
  use crate::system::storage::DiskKind;

  /// The test fixture rendered in English.
  fn english_app() -> App {
    let mut app = App::test();
    app.lang = Lang::for_locale("en-US");
    app
  }

  /// Renders the page as plain text lines.
  fn rendered_lines(app: &App) -> Vec<String> {
    doc(app, 100, 0)
      .lines
      .iter()
      .map(ToString::to_string)
      .collect()
  }

  fn row_starts_with(lines: &[String], label: &str) -> bool {
    lines
      .iter()
      .any(|line| line.trim_start().starts_with(label))
  }

  #[test]
  fn groups_values_under_boot_system_and_hardware() {
    let lines = rendered_lines(&english_app());
    for title in ["Boot", "System", "Hardware"] {
      assert!(lines.iter().any(|line| line == title), "missing {title}");
    }
    let text = lines.join("\n");
    for value in [
      "AMI 1.40 (03/15/2024)",
      "UEFI",
      "systemd-boot 258.1",
      "Hyprland 0.56.2",
      "Wayland 1.26.0",
      "i915 (Mesa 25.2.3)",
      "PipeWire 1.4.8",
      "Quickshell 0.3.1",
      "greetd 0.10.3",
      "systemd 258.1",
      "1024 (300 explicit, 12 AUR/foreign)",
      "2 d 3 h 4 min",
      "en_US.UTF-8",
      "eDP-1  1920x1200 @ 60 Hz",
      "HDMI-A-1  2560x1440 @ 143.91 Hz  ×1.25",
      "85% (Discharging) · health 91% · 312 cycles",
      "Intel Core i5 · up to 4.20 GHz",
      "5.7 GiB / 15.3 GiB",
      "1.0 GiB / 8.0 GiB",
      "Example SSD 1TB · NVMe · btrfs · 120.0 GiB / 465.0 GiB",
      "LENOVO 21AHCTO1WW",
    ] {
      assert!(text.contains(value), "missing {value}");
    }
  }

  #[test]
  fn aligns_every_value_in_one_column() {
    let lines = rendered_lines(&english_app());
    let firmware = lines
      .iter()
      .find(|line| line.trim_start().starts_with("Firmware"))
      .expect("firmware row");
    let kernel = lines
      .iter()
      .find(|line| line.trim_start().starts_with("Kernel"))
      .expect("kernel row");
    assert!(firmware.starts_with(LABEL_INDENT));
    assert_eq!(firmware.find("AMI"), kernel.find("6.14.4"));
  }

  #[test]
  fn hides_secure_boot_on_legacy_boot() {
    let mut app = english_app();
    app.system.boot_mode = Some(BootMode::Legacy);
    let text = rendered_lines(&app).join("\n");
    assert!(text.contains("Legacy (BIOS)"));
    assert!(!text.contains("Secure Boot"));
  }

  #[test]
  fn reports_missing_battery_and_values() {
    let mut app = english_app();
    app.system.batteries.clear();
    app.system.motherboard = None;
    app.system.displays = None;
    app.system.bootloader = None;
    let text = rendered_lines(&app).join("\n");
    assert!(text.contains("Not detected"));
    assert!(text.contains(na(app.lang)));
  }

  #[test]
  fn names_each_battery_when_there_are_several() {
    let lang = Lang::for_locale("en-US");
    let battery = |name: &str, percent| Battery {
      name: name.into(),
      percent: Some(percent),
      status: BatteryStatus::Full,
      health: None,
      cycles: None,
    };
    let label = batteries_label(&[battery("BAT0", 100), battery("BAT1", 97)], lang);
    assert_eq!(label, "BAT0: 100% (Full)\nBAT1: 97% (Full)");
  }

  #[test]
  fn hides_model_that_repeats_the_motherboard() {
    let mut app = english_app();
    app.system.model = app.system.motherboard.clone();
    let lines = rendered_lines(&app);
    assert!(!row_starts_with(&lines, "Model"));
    assert!(row_starts_with(&lines, "Motherboard"));
  }

  #[test]
  fn shows_disabled_swap_and_cpu_without_frequency() {
    let mut app = english_app();
    app.system.cpu_max_mhz = None;
    if let Some(memory) = app.system.memory.as_mut() {
      memory.swap_total_kib = Some(0);
    }
    let text = rendered_lines(&app).join("\n");
    assert!(text.contains("Disabled"));
    assert!(!text.contains("up to"));
  }

  #[test]
  fn leaves_unknown_disk_kind_out_of_storage() {
    let storage = Storage {
      model: Some("vda".into()),
      kind: DiskKind::Unknown,
      filesystem: "ext4".into(),
      total_bytes: None,
      used_bytes: None,
    };
    assert_eq!(storage_label(&storage), "vda · ext4");
  }

  #[test]
  fn formats_uptime_without_leading_zero_units() {
    let lang = Lang::for_locale("en-US");
    assert_eq!(uptime_label(59, lang), "0 min");
    assert_eq!(uptime_label(3_660, lang), "1 h 1 min");
    assert_eq!(uptime_label(86_400, lang), "1 d 0 h 0 min");
  }

  #[test]
  fn report_is_plain_unwrapped_text() {
    let app = english_app();
    let report = report_text(&app);
    assert!(report.starts_with("Boot\n  Firmware"));
    assert!(report.contains("Example SSD 1TB · NVMe · btrfs · 120.0 GiB / 465.0 GiB"));
    assert!(report.lines().all(|line| line == line.trim_end()));
  }
}
