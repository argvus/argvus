//! System settings handled by the app itself: Do Not Disturb, ratbag devices, hostname, locales, keyboard layout and time zone.

use super::*;

impl App {
  pub fn refresh_dnd(&mut self) {
    if self.dnd_job.is_some() {
      return;
    }
    self.dnd_job = Some(self.jobs.spawn(|_| {
      Self::read_dnd_command(
        &ProcessRequest::new("argvus")
          .arg("notifications")
          .arg("dnd")
          .arg("status"),
      )
      .map(|enabled| (enabled, false))
    }));
    self.dnd_last_refresh = Instant::now();
  }

  pub(super) fn toggle_dnd(&mut self) {
    if self.dnd_job.is_some() {
      return;
    }
    self.dnd_job = Some(self.jobs.spawn(|_| {
      Self::read_dnd_command(
        &ProcessRequest::new("argvus")
          .arg("notifications")
          .arg("dnd")
          .arg("toggle"),
      )
      .map(|state| (state, true))
    }));
  }

  pub(super) fn read_dnd_command(request: &ProcessRequest) -> Result<bool, String> {
    let output = SystemProcessRunner
      .run(&request.clone().timeout(Duration::from_secs(2)))
      .map_err(|error| error.to_string())?;
    if output.timed_out || output.status != Some(0) {
      let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
      return Err(if error.is_empty() {
        "notification backend failed".into()
      } else {
        error
      });
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
      "dnd=true" => Ok(true),
      "dnd=false" => Ok(false),
      value => Err(format!("invalid notification state: {value}")),
    }
  }

  pub(super) fn cycle_ratbag_device(&mut self, direction: i8) {
    let Some(next) = crate::system::ratbag::next_device_index(
      self.ratbag_device,
      self.input.ratbag.len(),
      direction,
    ) else {
      return;
    };
    self.ratbag_device = next;
  }

  pub(super) fn queue_ratbag(&mut self, change: crate::system::ratbag::Change) {
    if self.ratbag_job.is_some() {
      return;
    }
    let Some(device) = self
      .input
      .ratbag
      .get(self.ratbag_device)
      .or_else(|| self.input.ratbag.first())
      .cloned()
    else {
      return;
    };
    self.status = Some(Status {
      text: tr(self.lang, "control_center.input_hardware_applying").to_string(),
      kind: StatusKind::Success,
      created: Instant::now(),
    });
    let device_path = device.path.clone();
    self.ratbag_job = Some(
      self
        .jobs
        .spawn(move |_| crate::system::ratbag::apply(&device, change)),
    );
    self.ratbag_pending_path = Some(device_path);
  }

  pub fn hostname_input(&mut self, key: crossterm::event::KeyCode) {
    match key {
      crossterm::event::KeyCode::Esc => self.hostname_editing = false,
      crossterm::event::KeyCode::Enter => {
        let value = self.hostname_input.trim().to_string();
        match host::set(&value) {
          Ok(()) => {
            self.hostname_editing = false;
            self.hostname = host::current();
            self.success(format!(
              "{}: {value}",
              tr(self.lang, "control_center.hostname_changed")
            ));
          }
          Err(error) => self.fail(error),
        }
      }
      crossterm::event::KeyCode::Backspace => {
        self.hostname_input.pop();
      }
      crossterm::event::KeyCode::Char(character)
        if character.is_ascii_alphanumeric() || character == '-' =>
      {
        self.hostname_input.push(character);
      }
      _ => {}
    }
  }

  /// Whether the System Locales page can run on this distribution. Its
  /// backend (`locale-gen`) is implemented only for Arch Linux, so callers
  /// hide the entry instead of offering a page that always fails.
  pub fn system_locales_available(&self) -> bool {
    self.distro.id == "arch"
  }

  pub(super) fn apply_system_locales(&mut self) {
    let valid: BTreeSet<String> = self
      .locale_gen_entries
      .iter()
      .map(|entry| format!("{} {}", entry.locale, entry.encoding))
      .collect();
    if self
      .selected_locales
      .iter()
      .any(|value| !valid.contains(value))
    {
      self.fail("selected locale is not present in /etc/locale.gen");
      return;
    }
    let executable = match std::env::current_exe() {
      Ok(path) => path.to_string_lossy().into_owned(),
      Err(error) => {
        self.fail(error.to_string());
        return;
      }
    };
    let request = match PrivilegedRequest::new(
      "locale",
      "set-generated",
      self.selected_locales.iter().cloned().collect(),
    ) {
      Ok(request) => request,
      Err(error) => {
        self.fail(error.to_string());
        return;
      }
    };
    let live = LiveProcess::new();
    self.task_live = Some(live.clone());
    self.task_open = true;
    self.task_scroll = 0;
    self.task_follow = true;
    live.push_line("$ locale-gen");
    let applied = tr(self.lang, "control_center.applied").to_string();
    let operation = SystemSettingsOperation::new(SystemProcessRunner, executable);
    self.task_job = Some(self.jobs.spawn(move |_| {
      Ok(match operation.execute_live(&request, &live) {
        Ok(output) if output.status == Some(0) => {
          let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
          Ok(if stdout.is_empty() { applied } else { stdout })
        }
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        Err(error) => Err(error.to_string()),
      })
    }));
  }

  /// Whether a page that shows the layout in use is open. Only those pages poll
  /// Hyprland, so the rest of the Control Center never spawns `hyprctl`.
  pub(super) fn keyboard_page_is_open(&self) -> bool {
    matches!(
      self.page(),
      Page::LocaleRegion | Page::Keyboard | Page::KeyboardLayout | Page::KeyboardVariant
    )
  }

  /// Follows the layout in use when it changes outside the Control Center.
  /// Only the marker and the detail move; the draft and the variant list belong
  /// to the configured default, so the pending `Apply` state is not touched.
  pub(in super::super) fn apply_live_layout(&mut self, active_layout: &str) -> bool {
    if active_layout.is_empty() || self.keyboard_info.x11_layout == active_layout {
      return false;
    }
    self.keyboard_info.x11_layout = active_layout.to_string();
    true
  }

  /// Whether Time Zone has a pick that `Apply` has not written yet.
  pub(in super::super) fn timezone_changed(&self) -> bool {
    self.timezone_draft != self.datetime.time_zone
  }

  /// Puts the Time Zone draft back on the zone the system uses.
  pub(in super::super) fn load_timezone_draft(&mut self) {
    self.timezone_draft = self.datetime.time_zone.clone();
  }

  /// Whether Keyboard > Layout has edits that `Apply` has not written yet.
  pub(in super::super) fn keyboard_layout_changed(&self) -> bool {
    self
      .keyboard_draft
      .changed_from(&keyboard_draft::LayoutDraft::loaded(&self.keyboard_info))
  }

  pub(super) fn default_detail(&self, value: String, default: bool) -> String {
    if default {
      format!(
        "{value} [{}]",
        tr(self.lang, "control_center.argvus_default_5cdad7")
      )
    } else {
      value
    }
  }
}
