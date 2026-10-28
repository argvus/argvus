//! Rows and labels of the Storage page lists.

use super::*;

impl StorageApp {
  pub(super) fn rows(&self) -> Vec<String> {
    match self.page {
      StoragePage::Home => self.home_rows(),
      StoragePage::Summary => self.summary_rows(),
      StoragePage::Disks => self
        .snapshot
        .devices
        .iter()
        .map(|d| disk_row(self.lang, d))
        .collect(),
      StoragePage::Partitions => self.partitions().iter().map(|p| partition_row(p)).collect(),
      StoragePage::Filesystems => self
        .snapshot
        .filesystems
        .iter()
        .map(filesystem_row)
        .collect(),
      StoragePage::Mounts => self
        .snapshot
        .mounts
        .iter()
        .map(|m| {
          format!(
            "{:<18} {:<9}  {}{}",
            m.target,
            m.fstype,
            m.source,
            if m.readonly {
              format!("  [{}]", tr(self.lang, "control_center.ro"))
            } else {
              String::new()
            }
          )
        })
        .collect(),
      StoragePage::Smart => self.smart_rows(),
      StoragePage::Usage => self.usage_rows(),
      StoragePage::DiskDetails(index) => self.disk_detail(index),
      StoragePage::PartitionDetails(index) => self.partition_detail(index),
      StoragePage::FilesystemDetails(index) => self.filesystem_detail(index),
      StoragePage::MountDetails(index) => self.mount_detail(index),
      StoragePage::SmartDetails(index) => self.smart_detail(index),
    }
  }

  pub(super) fn home_rows(&self) -> Vec<String> {
    let disks = physical_devices(&self.snapshot.devices);
    let total: u64 = disks.iter().map(|d| d.size_bytes.unwrap_or(0)).sum();
    let root_pct = self
      .snapshot
      .filesystems
      .iter()
      .find(|f| f.mountpoint.as_deref() == Some("/"))
      .and_then(|f| f.total_bytes.zip(f.available_bytes))
      .map(|(total, available)| percentage(total, available))
      .unwrap_or(0);
    let smart = self.smart_rows().first().cloned().unwrap_or_default();
    let summary = tr(self.lang, "control_center.summary");
    let disks_label = tr(self.lang, "control_center.disks");
    let partitions_label = tr(self.lang, "control_center.partitions");
    let filesystems_label = tr(self.lang, "control_center.filesystems");
    let mounts_label = tr(self.lang, "control_center.mount_points");
    let usage_label = tr(self.lang, "control_center.disk_usage");
    vec![
      format!(
        "{} {}  ·  {} {} · {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        summary,
        disks.len(),
        tr(self.lang, "control_center.disks_5d5d98"),
        human_bytes(total)
      ),
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        disks_label,
        disks.len(),
        human_bytes(total)
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        partitions_label,
        self.partitions().len()
      ),
      format!(
        "{} {}  ·  {} · / {}%",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        filesystems_label,
        self.snapshot.filesystems.len(),
        root_pct
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        mounts_label,
        self.snapshot.mounts.len()
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::LOCK),
        "SMART",
        smart.trim_start()
      ),
      format!(
        "{} {}  ·  / {}%",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        usage_label,
        root_pct
      ),
    ]
  }

  pub(super) fn summary_rows(&self) -> Vec<String> {
    let disks = physical_devices(&self.snapshot.devices);
    let total: u64 = disks.iter().map(|d| d.size_bytes.unwrap_or(0)).sum();
    let smart_available = if self.snapshot.smart.is_empty() {
      if self.snapshot.smart_available {
        tr(self.lang, "control_center.no_data_collected").to_string()
      } else {
        tr(self.lang, "control_center.unavailable").to_string()
      }
    } else {
      tr(self.lang, "control_center.available").to_string()
    };
    let swap = self
      .snapshot
      .swap
      .first()
      .map(|s| {
        format!(
          "{} · {} ({} {})",
          s.source,
          human_bytes(s.total_bytes),
          human_bytes(s.used_bytes),
          tr(self.lang, "control_center.used")
        )
      })
      .unwrap_or_else(|| "—".into());
    let mut rows = vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        tr(self.lang, "control_center.storage_summary")
      ),
      format!(
        "   {:<16} {}",
        tr(self.lang, "control_center.disks_0a36a0"),
        disks.len()
      ),
      format!(
        "   {:<16} {}",
        tr(self.lang, "control_center.total_size"),
        human_bytes(total)
      ),
      format!(
        "   {:<16} {}",
        tr(self.lang, "control_center.partitions_2d5f30"),
        self.partitions().len()
      ),
      format!(
        "   {:<16} {}",
        tr(self.lang, "control_center.filesystems_602f7b"),
        self.snapshot.filesystems.len()
      ),
      format!(
        "   {:<16} {}",
        tr(self.lang, "control_center.mount_points_78b940"),
        self.snapshot.mounts.len()
      ),
      format!("   {:<16} {}", tr(self.lang, "control_center.swap"), swap),
      format!(
        "   {:<16} {}",
        tr(self.lang, "control_center.smart"),
        smart_available
      ),
      "".into(),
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::STORAGE),
        tr(self.lang, "control_center.usage_by_filesystem")
      ),
    ];
    if self.snapshot.filesystems.is_empty() {
      rows.push(format!(
        "   {}",
        tr(self.lang, "control_center.no_filesystems_found")
      ));
    }
    for f in &self.snapshot.filesystems {
      rows.push(filesystem_row(f));
    }
    rows
  }

  pub(super) fn smart_rows(&self) -> Vec<String> {
    if self.snapshot.smart.is_empty() {
      return vec![if self.snapshot.smart_available {
        tr(self.lang, "control_center.no_smart_data_available").into()
      } else {
        tr(self.lang, "control_center.smartctl_is_not_installed").into()
      }];
    }
    self
      .snapshot
      .smart
      .iter()
      .map(|s| {
        let temp = s
          .temperature_c
          .map(|t| format!(" · {}°C", t))
          .unwrap_or_default();
        let level = match s.health {
          SmartHealth::Failed => {
            format!("  [{}]", tr(self.lang, "control_center.failed"))
          }
          _ => String::new(),
        };
        format!(
          "{}  ·  {}{}{}",
          s.device,
          smart_health_label(self.lang, s.health),
          level,
          temp
        )
      })
      .collect()
  }

  pub(super) fn usage_rows(&self) -> Vec<String> {
    let mut rows = self
      .snapshot
      .filesystems
      .iter()
      .map(|f| usage_row(self.lang, f))
      .collect::<Vec<_>>();
    for swap in &self.snapshot.swap {
      rows.push(format!(
        "{} {} {}  ·  {} / {}",
        AppConfig::icon(argvus_tui::icons::REFRESH),
        tr(self.lang, "control_center.swap_0b62a3"),
        swap.source,
        human_bytes(swap.used_bytes),
        human_bytes(swap.total_bytes)
      ));
    }
    rows
  }
}

pub(super) fn disk_row(lang: Lang, device: &StorageDevice) -> String {
  let label = if device.name.starts_with("zram") || device.name.starts_with("loop") {
    device.kind.clone()
  } else {
    device
      .model
      .clone()
      .or_else(|| device.transport.clone())
      .unwrap_or(device.kind.clone())
  };
  let parts = if device.children.is_empty() {
    String::new()
  } else {
    format!(
      "  ·  {} {}",
      device.children.len(),
      tr(lang, "control_center.partitions_10c74c")
    )
  };
  format!(
    "{} {:<8} {:>9}  ·  {}{}",
    disk_icon(device),
    device.name,
    human_bytes(device.size_bytes.unwrap_or(0)),
    label,
    parts
  )
}

pub(super) fn partition_row(partition: &StorageDevice) -> String {
  format!(
    "{} {:<12} {:>9}  {:<11}  {}",
    partition_icon(partition),
    partition.name,
    human_bytes(partition.size_bytes.unwrap_or(0)),
    partition.fstype.as_deref().unwrap_or("—"),
    partition.mountpoint.as_deref().unwrap_or("—")
  )
}

pub(super) fn filesystem_row(filesystem: &Filesystem) -> String {
  let mountpoint = filesystem
    .mountpoint
    .as_deref()
    .unwrap_or(filesystem.source.as_str());
  if filesystem.used_bytes.is_none() || filesystem.total_bytes.is_none() {
    return format!("{:<18} {:<9}", mountpoint, filesystem.fstype);
  }
  let total = filesystem.total_bytes.unwrap_or(0);
  let used = filesystem.used_bytes.unwrap_or(0);
  let available = filesystem
    .available_bytes
    .unwrap_or(total.saturating_sub(used));
  let pct = percentage(total, available);
  format!(
    "{:<18} {:<9} {:>9} / {:<9} {:>3}%",
    mountpoint,
    filesystem.fstype,
    human_bytes(used),
    human_bytes(total),
    pct
  )
}

pub(super) fn usage_row(lang: Lang, filesystem: &Filesystem) -> String {
  let mountpoint = filesystem
    .mountpoint
    .as_deref()
    .unwrap_or(filesystem.source.as_str());
  if let (Some(total), Some(available)) = (filesystem.total_bytes, filesystem.available_bytes)
    && total > 0
  {
    let pct = percentage(total, available);
    return format!(
      "{:<18} {:<9} {:>3}%  {}{}",
      mountpoint,
      filesystem.fstype,
      pct,
      usage_bar(pct, 10),
      if pct >= 80 {
        format!("  [{}]", usage_label(lang, usage_level(total, available)))
      } else {
        String::new()
      }
    );
  }
  format!("{:<18} {:<9}", mountpoint, filesystem.fstype)
}

pub(super) fn usage_label(lang: Lang, level: UsageLevel) -> String {
  match level {
    UsageLevel::Ok => tr(lang, "control_center.ok").into(),
    UsageLevel::Warning => tr(lang, "control_center.warning_34c524").into(),
    UsageLevel::Critical => tr(lang, "control_center.critical").into(),
  }
}

pub(super) fn smart_health_label(lang: Lang, health: SmartHealth) -> String {
  match health {
    SmartHealth::Passed => tr(lang, "control_center.passed").into(),
    SmartHealth::Warning => tr(lang, "control_center.warning_34c524").into(),
    SmartHealth::Failed => tr(lang, "control_center.failed").into(),
    SmartHealth::Unsupported => tr(lang, "control_center.unavailable").into(),
    SmartHealth::Unknown => tr(lang, "control_center.unknown_b75709").into(),
  }
}

pub(super) fn bool_label(lang: Lang, value: Option<bool>) -> String {
  match value {
    Some(value) => yes_no(lang, value),
    None => "—".into(),
  }
}
