//! Terminal UI rendering and interaction of the Storage page.
use crate::{backend, model::*};
use argvus_control_center_core::format::human_bytes;
use argvus_control_center_core::navigation::HomeCardId;
use argvus_control_center_core::{
  capabilities::Capabilities,
  config::AppConfig,
  jobs::{JobHandle, JobManager, JobState},
};
use argvus_i18n::{Lang, tr};
use argvus_theme::Theme;
use argvus_tui::{
  components::{StatusKind, StatusMessage, draw_status},
  page::{Selection, list, readonly, shell},
};
use crossterm::event::KeyCode;
use ratatui::{Frame, layout::Rect, text::Line};

mod jobs;
mod render;
mod rows;

use rows::{bool_label, smart_health_label};

pub struct StorageApp {
  pub page: StoragePage,
  pub snapshot: StorageSnapshot,
  selection: Selection,
  job: Option<JobHandle<StorageSnapshot>>,
  jobs: JobManager,
  cap: Capabilities,
  cache: backend::StorageCache,
  pub lang: Lang,
  pub theme: Theme,
  pub status: Option<StatusMessage>,
}
impl StorageApp {
  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn new(lang: Lang, theme: Theme, cap: Capabilities) -> Self {
    Self::new_with_cache(lang, theme, cap, backend::StorageCache::default())
  }

  pub fn new_with_cache(
    lang: Lang,
    theme: Theme,
    cap: Capabilities,
    cache: backend::StorageCache,
  ) -> Self {
    let mut s = Self {
      page: StoragePage::Home,
      snapshot: Default::default(),
      selection: Selection::default(),
      job: None,
      jobs: JobManager::default(),
      cap,
      cache,
      lang,
      theme,
      status: None,
    };
    s.refresh();
    s
  }
  fn len(&self) -> usize {
    match self.page {
      StoragePage::Home => 7,
      StoragePage::Disks => self.snapshot.devices.len(),
      StoragePage::Partitions => self.partitions().len(),
      StoragePage::Filesystems => self.snapshot.filesystems.len(),
      StoragePage::Mounts => self.snapshot.mounts.len(),
      StoragePage::Smart => {
        if self.snapshot.smart.is_empty() {
          1
        } else {
          self.snapshot.smart.len()
        }
      }
      StoragePage::Usage => {
        self.snapshot.filesystems.len() + usize::from(!self.snapshot.swap.is_empty())
      }
      _ => 1,
    }
  }
  fn partitions(&self) -> Vec<&StorageDevice> {
    self
      .snapshot
      .devices
      .iter()
      .flat_map(|d| d.children.iter())
      .collect()
  }
  pub fn handle(&mut self, key: KeyCode) -> bool {
    if matches!(key, KeyCode::Esc | KeyCode::Left) {
      if self.page == StoragePage::Home {
        return true;
      }
      self.page = match self.page {
        StoragePage::DiskDetails(_) => StoragePage::Disks,
        StoragePage::PartitionDetails(_) => StoragePage::Partitions,
        StoragePage::FilesystemDetails(_) => StoragePage::Filesystems,
        StoragePage::MountDetails(_) => StoragePage::Mounts,
        StoragePage::SmartDetails(_) => StoragePage::Smart,
        _ => StoragePage::Home,
      };
      self.selection.index = 0;
      return false;
    }
    if self.busy() {
      return false;
    }
    if key == KeyCode::Char('r') {
      self.refresh_forced();
      return false;
    }
    self.selection.handle(key, self.len(), 8);
    if matches!(key, KeyCode::Enter | KeyCode::Right) {
      self.page = match self.page {
        StoragePage::Home => [
          StoragePage::Summary,
          StoragePage::Disks,
          StoragePage::Partitions,
          StoragePage::Filesystems,
          StoragePage::Mounts,
          StoragePage::Smart,
          StoragePage::Usage,
        ][self.selection.index.min(6)],
        StoragePage::Disks => StoragePage::DiskDetails(self.selection.index),
        StoragePage::Partitions => StoragePage::PartitionDetails(self.selection.index),
        StoragePage::Filesystems => StoragePage::FilesystemDetails(self.selection.index),
        StoragePage::Mounts => StoragePage::MountDetails(self.selection.index),
        StoragePage::Smart => StoragePage::SmartDetails(self.selection.index),
        _ => self.page,
      };
      self.selection.index = 0;
    }
    false
  }
  fn label(&self) -> &'static str {
    match self.page {
      StoragePage::Summary => tr(self.lang, "control_center.summary"),
      StoragePage::Disks | StoragePage::DiskDetails(_) => tr(self.lang, "control_center.disks"),
      StoragePage::Partitions | StoragePage::PartitionDetails(_) => {
        tr(self.lang, "control_center.partitions")
      }
      StoragePage::Filesystems | StoragePage::FilesystemDetails(_) => {
        tr(self.lang, "control_center.filesystems")
      }
      StoragePage::Mounts | StoragePage::MountDetails(_) => {
        tr(self.lang, "control_center.mount_points")
      }
      StoragePage::Smart | StoragePage::SmartDetails(_) => "SMART",
      StoragePage::Usage => tr(self.lang, "control_center.disk_usage"),
      _ => tr(self.lang, "control_center.summary"),
    }
  }
  fn disk_detail(&self, index: usize) -> Vec<String> {
    let Some(d) = self.snapshot.devices.get(index) else {
      return vec![tr(self.lang, "control_center.disk_not_found").into()];
    };
    vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        tr(self.lang, "control_center.disk")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.device_945796"),
        d.name
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.model"),
        d.model.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.vendor"),
        d.vendor.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.serial"),
        d.serial.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.size"),
        human_bytes(d.size_bytes.unwrap_or(0))
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.transport"),
        d.transport.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.rotational"),
        bool_label(self.lang, d.rotational)
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.removable"),
        bool_label(self.lang, d.removable)
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.uuid"),
        d.uuid.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.partitions_2d5f30"),
        d.children.len()
      ),
    ]
  }
  fn partition_detail(&self, index: usize) -> Vec<String> {
    let Some(p) = self.partitions().get(index).copied() else {
      return vec![tr(self.lang, "control_center.partition_not_found").into()];
    };
    let parent = p.parent.as_deref().unwrap_or("—");
    vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        tr(self.lang, "control_center.partition")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.device_945796"),
        p.name
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.disk_ea4ba6"),
        parent
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.filesystem"),
        p.fstype.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.size"),
        human_bytes(p.size_bytes.unwrap_or(0))
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.mounted_at"),
        p.mountpoint.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.label"),
        p.label.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.uuid"),
        p.uuid.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.read_only"),
        bool_label(self.lang, p.readonly)
      ),
    ]
  }
  fn filesystem_detail(&self, index: usize) -> Vec<String> {
    let Some(f) = self.snapshot.filesystems.get(index) else {
      return vec![tr(self.lang, "control_center.filesystem_not_found").into()];
    };
    let (used, total, available, pct) = match (f.total_bytes, f.used_bytes) {
      (Some(total), Some(used)) => {
        let available = f.available_bytes.unwrap_or(total.saturating_sub(used));
        (
          Some(used),
          Some(total),
          Some(available),
          percentage(total, available),
        )
      }
      _ => (None, None, None, 0),
    };
    let level = usage_level(used.unwrap_or(0), available.unwrap_or(0));
    let level_label = match level {
      UsageLevel::Ok => tr(self.lang, "control_center.ok"),
      UsageLevel::Warning => tr(self.lang, "control_center.warning_34c524"),
      UsageLevel::Critical => tr(self.lang, "control_center.critical"),
    };
    let mut rows = vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        tr(self.lang, "control_center.filesystem_6effc4")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.source"),
        f.source
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.type"),
        f.fstype
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.mounted_at"),
        f.mountpoint.as_deref().unwrap_or("—")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.read_only"),
        yes_no(self.lang, f.readonly)
      ),
      "".into(),
    ];
    if let (Some(total), Some(used), Some(available)) = (total, used, available) {
      rows.push(format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.size"),
        human_bytes(total)
      ));
      rows.push(format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.used_b284db"),
        human_bytes(used)
      ));
      rows.push(format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.available_93698a"),
        human_bytes(available)
      ));
      rows.push(format!(
        "   {:<14} {} · {}  {}",
        tr(self.lang, "control_center.usage"),
        pct,
        "%",
        level_label
      ));
      rows.push(format!("   {}", usage_bar(pct, 20)));
    }
    rows
  }
  fn mount_detail(&self, index: usize) -> Vec<String> {
    let Some(m) = self.snapshot.mounts.get(index) else {
      return vec![tr(self.lang, "control_center.mount_point_not_found").into()];
    };
    vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        tr(self.lang, "control_center.mount_point")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.target"),
        m.target
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.source"),
        m.source
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.type"),
        m.fstype
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.options_4dbf67"),
        m.options.join(" ")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.read_only"),
        yes_no(self.lang, m.readonly)
      ),
    ]
  }
  fn smart_detail(&self, index: usize) -> Vec<String> {
    let Some(s) = self.snapshot.smart.get(index) else {
      return vec![tr(self.lang, "control_center.smart_unavailable").into()];
    };
    let health = smart_health_label(self.lang, s.health);
    let temp = s
      .temperature_c
      .map(|t| format!("{t}°C"))
      .unwrap_or_else(|| "—".into());
    let mut rows = vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::LOCK),
        tr(self.lang, "control_center.smart_eb1ea9")
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.device_945796"),
        s.device
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.health"),
        health
      ),
      format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.temperature"),
        temp
      ),
    ];
    if let Some(hours) = s.power_on_hours {
      rows.push(format!(
        "   {:<14} {} h",
        tr(self.lang, "control_center.power_on"),
        hours
      ));
    }
    if let Some(cycles) = s.power_cycles {
      rows.push(format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.cycles"),
        cycles
      ));
    }
    if let Some(percent) = s.percentage_used {
      rows.push(format!(
        "   {:<14} {}%",
        tr(self.lang, "control_center.used_5b1256"),
        percent
      ));
    }
    if let Some(warning) = s.critical_warning {
      rows.push(format!(
        "   {:<14} {}",
        tr(self.lang, "control_center.critical_warning"),
        warning
      ));
    }
    rows
  }
}

fn physical_devices(devices: &[StorageDevice]) -> Vec<&StorageDevice> {
  devices
    .iter()
    .filter(|d| d.kind == "disk" && !d.name.starts_with("zram") && !d.name.starts_with("loop"))
    .collect()
}
fn percentage(total: u64, available: u64) -> u64 {
  total
    .saturating_sub(available)
    .checked_mul(100)
    .and_then(|used| used.checked_div(total))
    .unwrap_or(0)
}
fn disk_icon(device: &StorageDevice) -> &'static str {
  if device.name.starts_with("zram") || device.name.starts_with("loop") {
    AppConfig::icon(argvus_tui::icons::MEMORY)
  } else {
    AppConfig::icon(argvus_tui::icons::STORAGE)
  }
}
fn partition_icon(partition: &StorageDevice) -> &'static str {
  match partition.fstype.as_deref() {
    Some("crypto_LUKS") | Some("crypt") => AppConfig::icon(argvus_tui::icons::LOCK),
    Some("swap") => AppConfig::icon(argvus_tui::icons::REFRESH),
    _ => AppConfig::icon(argvus_tui::icons::STORAGE),
  }
}
fn usage_bar(percent: u64, width: usize) -> String {
  let filled = (percent as usize * width / 100).min(width);
  let mut bar = String::with_capacity(width);
  for index in 0..width {
    bar.push(if index < filled { '█' } else { '░' });
  }
  bar
}
fn yes_no(lang: Lang, value: bool) -> String {
  tr(
    lang,
    if value {
      "control_center.yes"
    } else {
      "control_center.no"
    },
  )
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use argvus_i18n::Lang;
  use argvus_theme::Theme;

  fn test_app(snapshot: StorageSnapshot) -> StorageApp {
    let mut app = StorageApp::new(
      Lang::for_locale("en-US"),
      Theme::load(),
      Capabilities::default(),
    );
    app.snapshot = snapshot;
    app.job = None;
    app
  }

  fn sample_snapshot() -> StorageSnapshot {
    StorageSnapshot {
      devices: vec![
        StorageDevice {
          name: "sda".into(),
          kind: "disk".into(),
          model: Some("KINGSTON SA400S37240G".into()),
          size_bytes: Some(240_057_409_536),
          children: vec![StorageDevice {
            name: "sda1".into(),
            kind: "part".into(),
            fstype: Some("vfat".into()),
            mountpoint: Some("/boot".into()),
            parent: Some("sda".into()),
            size_bytes: Some(536_870_912),
            ..Default::default()
          }],
          ..Default::default()
        },
        StorageDevice {
          name: "zram0".into(),
          kind: "disk".into(),
          size_bytes: Some(4_294_967_296),
          ..Default::default()
        },
      ],
      filesystems: vec![Filesystem {
        source: "/dev/sda2".into(),
        fstype: "ext4".into(),
        mountpoint: Some("/".into()),
        total_bytes: Some(100_000_000_000),
        used_bytes: Some(60_000_000_000),
        available_bytes: Some(40_000_000_000),
        readonly: false,
        ..Default::default()
      }],
      mounts: vec![
        Mount {
          target: "/".into(),
          source: "/dev/sda2".into(),
          fstype: "ext4".into(),
          options: vec!["rw".into()],
          readonly: false,
        },
        Mount {
          target: "/proc".into(),
          source: "proc".into(),
          fstype: "proc".into(),
          options: vec!["rw".into()],
          readonly: false,
        },
      ],
      swap: vec![SwapDevice {
        source: "/dev/zram0".into(),
        kind: "zram".into(),
        total_bytes: 4_294_967_296,
        used_bytes: 0,
      }],
      smart_available: true,
      smart: vec![],
    }
  }

  #[test]
  fn storage_home_rows_act_as_a_status_dashboard() {
    let app = test_app(sample_snapshot());
    let rows = app.home_rows();
    assert_eq!(rows.len(), 7);
    assert!(rows[0].contains("1 disks"), "{}", rows[0]);
    assert!(rows[0].contains("223.6 GiB"), "{}", rows[0]);
    assert!(rows[2].contains("1"));
    assert!(rows[3].contains("%"));
    assert!(rows[4].contains("2"));
    assert!(rows[5].contains("No SMART data"));
  }

  #[test]
  fn storage_summary_page_shows_counts_and_usage() {
    let mut app = test_app(sample_snapshot());
    app.page = StoragePage::Summary;
    let rows = app.rows();
    assert!(rows[0].contains("SUMMARY"), "{}", rows[0]);
    assert!(rows.iter().any(|r| r.contains("Disks:")));
    assert!(rows.iter().any(|r| r.contains("/")));
  }

  #[test]
  fn storage_smart_page_falls_back_when_no_data() {
    let mut app = test_app(sample_snapshot());
    app.page = StoragePage::Smart;
    assert_eq!(app.len(), 1);
    let rows = app.rows();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].contains("No SMART data"));
    assert!(app.len() >= 1);
  }

  #[test]
  fn storage_usage_rows_append_swap() {
    let mut app = test_app(sample_snapshot());
    app.page = StoragePage::Usage;
    assert_eq!(app.len(), 2);
    let rows = app.rows();
    assert_eq!(rows.len(), 2);
    assert!(rows[1].contains("Swap"));
    assert!(rows[0].contains("%"));
  }

  #[test]
  fn storage_detail_pages_render_section_headers_and_aligned_rows() {
    let mut app = test_app(sample_snapshot());
    app.page = StoragePage::DiskDetails(0);
    let rows = app.rows();
    assert!(rows[0].contains("DISK"));
    assert!(rows.iter().any(|r| r.contains("KINGSTON")));
    assert!(
      rows
        .iter()
        .any(|r| r.contains("223.6 GiB") || r.contains("GiB"))
    );
    assert!(rows.len() > 5);

    app.page = StoragePage::FilesystemDetails(0);
    let rows = app.rows();
    assert!(rows[0].contains("FILESYSTEM"));
    assert!(rows.iter().any(|r| r.contains("60 ") && r.contains('%')));
    assert!(rows.iter().any(|r| r.contains('█')));
  }

  #[test]
  fn storage_human_bytes_and_percentage_helpers() {
    assert_eq!(human_bytes(0), "0 B");
    assert_eq!(human_bytes(1024), "1 KiB");
    assert_eq!(human_bytes(240_057_409_536), "223.6 GiB");
    assert_eq!(percentage(100, 40), 60);
    assert_eq!(percentage(0, 0), 0);
    assert_eq!(usage_bar(50, 10), "█████░░░░░");
  }
}
