//! Rows and labels of the Audio page lists.

use super::*;

impl AudioApp {
  pub(super) fn row_count(&self) -> usize {
    match self.page {
      AudioPage::Home => 4,
      AudioPage::Summary => 1,
      AudioPage::Output => self.snapshot.outputs.len(),
      AudioPage::Input => self.snapshot.inputs.len(),
      AudioPage::Devices => self.snapshot.outputs.len() + self.snapshot.inputs.len(),
    }
  }

  pub(super) fn rows(&self) -> Vec<String> {
    match self.page {
      AudioPage::Home => self.home_rows(),
      AudioPage::Summary => self.summary_rows(),
      AudioPage::Output | AudioPage::Input => self.device_rows(),
      AudioPage::Devices => self.all_devices_rows(),
    }
  }

  pub(super) fn home_rows(&self) -> Vec<String> {
    let summary_label = tr(self.lang, "control_center.summary");
    let output_label = tr(self.lang, "control_center.output");
    let input_label = tr(self.lang, "control_center.input");
    let devices_label = tr(self.lang, "control_center.devices");

    let status_str = if self.snapshot.available {
      tr(self.lang, "control_center.active")
    } else {
      tr(self.lang, "control_center.unavailable")
    };

    let default_out_str = self
      .snapshot
      .default_output
      .and_then(|id| self.snapshot.outputs.iter().find(|d| d.id == id))
      .map(|d| d.description.as_str())
      .unwrap_or("—");

    let default_in_str = self
      .snapshot
      .default_input
      .and_then(|id| self.snapshot.inputs.iter().find(|d| d.id == id))
      .map(|d| d.description.as_str())
      .unwrap_or("—");

    let total_devs = self.snapshot.outputs.len() + self.snapshot.inputs.len();

    vec![
      format!(
        "{} {}  ·  {} · {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        summary_label,
        if self.snapshot.backend.is_empty() {
          "PipeWire/WirePlumber".into()
        } else {
          self.snapshot.backend.clone()
        },
        status_str
      ),
      format!(
        "{} {}  ·  {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::AUDIO),
        output_label,
        format!(
          "{} {}",
          self.snapshot.outputs.len(),
          tr(self.lang, "control_center.outputs")
        ),
        default_out_str
      ),
      format!(
        "{} {}  ·  {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::MICROPHONE),
        input_label,
        format!(
          "{} {}",
          self.snapshot.inputs.len(),
          tr(self.lang, "control_center.inputs")
        ),
        default_in_str
      ),
      format!(
        "{} {}  ·  {}",
        AppConfig::icon(argvus_tui::icons::SPEAKER),
        devices_label,
        format!(
          "{} {}",
          total_devs,
          tr(self.lang, "control_center.devices_a41e65")
        )
      ),
    ]
  }

  pub(super) fn summary_rows(&self) -> Vec<String> {
    if !self.snapshot.available {
      return vec![
        format!(
          " {} {}",
          AppConfig::icon(argvus_tui::icons::MONITOR),
          tr(self.lang, "control_center.audio_system")
        ),
        format!("   Status: {}", tr(self.lang, "control_center.unavailable")),
      ];
    }

    let default_out = self
      .snapshot
      .default_output
      .and_then(|id| self.snapshot.outputs.iter().find(|d| d.id == id))
      .map(|d| {
        format!(
          "{} ({}%)",
          d.description,
          d.volume
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".into())
        )
      })
      .unwrap_or_else(|| "—".into());

    let default_in = self
      .snapshot
      .default_input
      .and_then(|id| self.snapshot.inputs.iter().find(|d| d.id == id))
      .map(|d| {
        format!(
          "{} ({}%)",
          d.description,
          d.volume
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".into())
        )
      })
      .unwrap_or_else(|| "—".into());

    vec![
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::MONITOR),
        tr(self.lang, "control_center.audio_system")
      ),
      format!(
        "   Servidor:    {}",
        if self.snapshot.backend.is_empty() {
          "PipeWire / WirePlumber".into()
        } else {
          self.snapshot.backend.clone()
        }
      ),
      format!("   Status:      {}", tr(self.lang, "control_center.active")),
      "".into(),
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::AUDIO),
        tr(self.lang, "control_center.default_devices")
      ),
      format!("   Saída:       {}", default_out),
      format!("   Entrada:     {}", default_in),
      "".into(),
      format!(
        " {} {}",
        AppConfig::icon(argvus_tui::icons::SPEAKER),
        tr(self.lang, "control_center.available_devices")
      ),
      format!("   Saídas:      {}", self.snapshot.outputs.len()),
      format!("   Entradas:    {}", self.snapshot.inputs.len()),
    ]
  }

  pub(super) fn device_rows(&self) -> Vec<String> {
    if !self.snapshot.available {
      return vec![
        tr(
          self.lang,
          "control_center.pipewire_wireplumber_is_unavailable",
        )
        .into(),
      ];
    }
    let devices = self.devices();
    if devices.is_empty() {
      return vec![tr(self.lang, "control_center.no_devices_found").into()];
    }
    devices
      .iter()
      .map(|d| {
        let mut badges = Vec::new();
        if self.is_default_of(d) {
          badges.push(format!("● {}", tr(self.lang, "control_center.default")));
        }
        if self.muted_of(d) {
          badges.push(format!("✖ {}", tr(self.lang, "control_center.muted")));
        }
        let vol = self
          .volume_of(d)
          .map(|v| format!("{}%", v))
          .unwrap_or_else(|| "—".into());
        let badge_str = if badges.is_empty() {
          String::new()
        } else {
          format!("   {}", badges.join(" · "))
        };
        format!("{} · {}{}", d.description, vol, badge_str)
      })
      .collect()
  }

  pub(super) fn all_devices_rows(&self) -> Vec<String> {
    if !self.snapshot.available {
      return vec![
        tr(
          self.lang,
          "control_center.pipewire_wireplumber_is_unavailable",
        )
        .into(),
      ];
    }
    let mut rows = Vec::new();

    rows.push(format!(
      " {} {}",
      AppConfig::icon(argvus_tui::icons::AUDIO),
      tr(self.lang, "control_center.output_devices")
    ));
    if self.snapshot.outputs.is_empty() {
      rows.push(format!(
        "   {}",
        tr(self.lang, "control_center.no_output_found")
      ));
    } else {
      for d in &self.snapshot.outputs {
        let is_def = Some(d.id) == self.snapshot.default_output;
        let mut flags = Vec::new();
        if is_def {
          flags.push(tr(self.lang, "control_center.default"));
        }
        if d.muted {
          flags.push(tr(self.lang, "control_center.muted"));
        }
        let flag_str = if flags.is_empty() {
          String::new()
        } else {
          format!(" [{}]", flags.join(", "))
        };
        let vol = d
          .volume
          .map(|v| format!("{}%", v))
          .unwrap_or_else(|| "—".into());
        rows.push(format!("   {} ({}){}", d.description, vol, flag_str));
      }
    }

    rows.push("".into());
    rows.push(format!(
      " {} {}",
      AppConfig::icon(argvus_tui::icons::MICROPHONE),
      tr(self.lang, "control_center.input_devices")
    ));
    if self.snapshot.inputs.is_empty() {
      rows.push(format!(
        "   {}",
        tr(self.lang, "control_center.no_input_found")
      ));
    } else {
      for d in &self.snapshot.inputs {
        let is_def = Some(d.id) == self.snapshot.default_input;
        let mut flags = Vec::new();
        if is_def {
          flags.push(tr(self.lang, "control_center.default"));
        }
        if d.muted {
          flags.push(tr(self.lang, "control_center.muted"));
        }
        let flag_str = if flags.is_empty() {
          String::new()
        } else {
          format!(" [{}]", flags.join(", "))
        };
        let vol = d
          .volume
          .map(|v| format!("{}%", v))
          .unwrap_or_else(|| "—".into());
        rows.push(format!("   {} ({}){}", d.description, vol, flag_str));
      }
    }

    rows
  }

  pub(super) fn page_label(&self) -> &'static str {
    match self.page {
      AudioPage::Home => tr(self.lang, "control_center.audio"),
      AudioPage::Summary => tr(self.lang, "control_center.summary"),
      AudioPage::Output => tr(self.lang, "control_center.output"),
      AudioPage::Input => tr(self.lang, "control_center.input"),
      AudioPage::Devices => tr(self.lang, "control_center.devices"),
    }
  }
}
