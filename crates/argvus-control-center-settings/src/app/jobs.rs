//! Background work of the settings pages: polling finished jobs, status messages and the refreshes they trigger.

use super::*;

impl App {
  pub fn refresh_system(&mut self) {
    self.hostname = host::current();
    self.admin.load(false);
    self.success(tr(self.lang, "control_center.data_updated").to_string());
  }

  /// Lists the key pairs on a worker thread. `argvus-keys list` runs one
  /// ssh-keygen per key, so doing it in the draw loop would stall the screen.
  pub(super) fn refresh_ssh_keys_in_background(&mut self) {
    if self.ssh_keys_job.is_some() {
      return;
    }
    self.ssh_keys_job = Some(self.jobs.spawn(|_| Ok(ssh_keys::load())));
    self.ssh_keys_last_refresh = Instant::now();
  }

  pub(crate) fn input_success(&mut self) {
    self.success(tr(self.lang, "control_center.input_applied").to_string());
  }

  pub(crate) fn input_failure(&mut self, error: String) {
    eprintln!("argvus-control-center: input setting failed: {error}");
    self.fail(tr(self.lang, "control_center.input_apply_failed"));
  }

  pub fn expire_status(&mut self) -> bool {
    if let Some(result) = self.admin.poll() {
      match result {
        Ok(()) => {
          match self.admin.completed.take().as_deref() {
            Some("create") if self.page() == Page::CreateUser => {
              self.navigation.current_mut().page = Page::User;
              self.normalize_selection();
            }
            // A deleted user leaves every user subpage; the account is gone.
            Some("delete") => self.leave_user_pages(),
            Some("delete-group") => {
              self.navigation.back();
            }
            _ => {}
          }
          self.success(tr(self.lang, "control_center.data_updated").into());
        }
        Err(error) => self.fail(error),
      }
      return true;
    }
    if self
      .status
      .as_ref()
      .is_some_and(|status| status.created.elapsed() >= Duration::from_secs(4))
    {
      self.status = None;
      true
    } else {
      false
    }
  }

  pub fn poll(&mut self) -> bool {
    let mut changed = false;
    if self.keyboard_page_is_open()
      && self.keyboard_last_refresh.elapsed() >= Duration::from_secs(1)
    {
      self.keyboard_last_refresh = Instant::now();
      changed |= self.apply_live_layout(&keyboard::current_layout());
    }
    if self.dnd_job.is_none()
      && self.page() == Page::Notifications
      && self.dnd_last_refresh.elapsed() >= Duration::from_secs(2)
    {
      self.refresh_dnd();
    }
    if self.ssh_keys_job.is_none()
      && self.page() == Page::SshKeys
      && self.ssh_keys_last_refresh.elapsed() >= Duration::from_secs(3)
    {
      self.refresh_ssh_keys_in_background();
    }
    if let Some(job) = self.ssh_keys_job.take() {
      match job.try_state() {
        JobState::Running => self.ssh_keys_job = Some(job),
        JobState::Finished(Ok(keys)) => {
          self.ssh_keys = keys;
          changed = true;
        }
        // The listing itself never fails; the next tick simply tries again.
        JobState::Finished(Err(_)) => changed = true,
      }
    }
    if let Some((job, kind)) = self.gpg_pick.take() {
      match job.try_state() {
        JobState::Running => self.gpg_pick = Some((job, kind)),
        // A chosen file is imported at once; a GUI file manager reports no
        // choice, so the user types the path instead.
        JobState::Finished(Ok(Ok(Some(path)))) => {
          self.import_gpg_file(kind, &path.to_string_lossy());
          changed = true;
        }
        JobState::Finished(Ok(Ok(None))) => {
          self.open_gpg_import_path(kind);
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.fail(error);
          changed = true;
        }
      }
    }
    if let Some((job, success_key)) = self.gpg_action.take() {
      match job.try_state() {
        JobState::Running => self.gpg_action = Some((job, success_key)),
        JobState::Finished(Ok(Ok(()))) => {
          self.gpg_keys = gpg_keys::load();
          self.success(tr(self.lang, success_key).to_string());
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.fail(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.gpg_job.take() {
      match job.try_state() {
        JobState::Running => self.gpg_job = Some(job),
        JobState::Finished(Ok(Ok(()))) => {
          let order = self.gpg_date_order();
          self.gpg_form.clear(order);
          self.gpg_keys = gpg_keys::load();
          self.success(tr(self.lang, "control_center.gpg_keys_generated").to_string());
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => {
          self.fail(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.dnd_job.take() {
      match job.try_state() {
        JobState::Running => self.dnd_job = Some(job),
        JobState::Finished(Ok((enabled, action))) => {
          self.dnd_enabled = enabled;
          if action {
            self.success(if enabled {
              tr(self.lang, "control_center.dnd_enabled").to_string()
            } else {
              tr(self.lang, "control_center.dnd_disabled").to_string()
            });
          }
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.fail(format!(
            "{}: {error}",
            tr(self.lang, "control_center.notification_state_error")
          ));
          changed = true;
        }
      }
    }
    if let Some(job) = self.apps_job.take() {
      match job.try_state() {
        JobState::Running => self.apps_job = Some(job),
        JobState::Finished(Ok(apps)) => {
          self.apps = apps;
          self.apps_loading = false;
          self.select_current();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.apps_loading = false;
          self.error_modal = Some(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.fonts_job.take() {
      match job.try_state() {
        JobState::Running => self.fonts_job = Some(job),
        JobState::Finished(Ok(fonts)) => {
          self.system_fonts = fonts;
          self.select_current();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.error_modal = Some(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.input_load_job.take() {
      match job.try_state() {
        JobState::Running => self.input_load_job = Some(job),
        JobState::Finished(Ok(settings)) => {
          self.input = settings;
          self.input_loading = false;
          self.input_last_device_refresh = Instant::now();
          self.normalize_selection();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.input_loading = false;
          self.input_failure(error);
          changed = true;
        }
      }
    }
    if !self.input_loading
      && self.page() == Page::MouseTouchpad
      && self.input_device_job.is_none()
      && self.input_last_device_refresh.elapsed() >= Duration::from_secs(2)
    {
      self.input_last_device_refresh = Instant::now();
      self.input_device_job = Some(
        self
          .jobs
          .spawn(|_| Ok((input::detect_devices(), crate::system::ratbag::devices()))),
      );
      changed = true;
    }
    if let Some(job) = self.input_device_job.take() {
      match job.try_state() {
        JobState::Running => self.input_device_job = Some(job),
        JobState::Finished(Ok((devices, ratbag))) => {
          self.input.devices = devices;
          self.input.ratbag = ratbag;
          self.ratbag_device = self
            .ratbag_device
            .min(self.input.ratbag.len().saturating_sub(1));
          self.normalize_selection();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          eprintln!("argvus-control-center: input device refresh failed: {error}");
        }
      }
    }
    if self.input_job.is_none()
      && let Some(selected) = self.input_toggle_pending.take()
    {
      let mut settings = self.input.clone();
      self.input_job = Some(self.jobs.spawn(move |_| {
        settings.toggle(selected)?;
        Ok(settings)
      }));
      changed = true;
    }
    if self.input_job.is_none()
      && let Some((selected, direction, deadline)) = self.input_pending
      && deadline <= Instant::now()
    {
      self.input_pending = None;
      let mut settings = self.input.clone();
      self.input_job = Some(self.jobs.spawn(move |_| {
        settings.cycle_by(selected, direction)?;
        Ok(settings)
      }));
      changed = true;
    }
    if let Some(job) = self.input_job.take() {
      match job.try_state() {
        JobState::Running => self.input_job = Some(job),
        JobState::Finished(Ok(settings)) => {
          self.input = settings;
          self.input_success();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.input_failure(error);
          changed = true;
        }
      }
    }
    if let Some(job) = self.ratbag_job.take() {
      match job.try_state() {
        JobState::Running => self.ratbag_job = Some(job),
        JobState::Finished(Ok(devices)) => {
          let requested_path = self.ratbag_pending_path.take();
          self.input.ratbag = devices;
          if let Some(path) = requested_path
            && let Some(index) = self
              .input
              .ratbag
              .iter()
              .position(|device| device.path == path)
          {
            self.ratbag_device = index;
          }
          self.ratbag_device = self
            .ratbag_device
            .min(self.input.ratbag.len().saturating_sub(1));
          self.success(tr(self.lang, "control_center.input_applied").to_string());
          self.normalize_selection();
          changed = true;
        }
        JobState::Finished(Err(error)) => {
          self.ratbag_pending_path = None;
          eprintln!("argvus-control-center: ratbag setting failed: {error}");
          self.fail(tr(self.lang, "control_center.input_hardware_apply_failed"));
          changed = true;
        }
      }
    }
    if let Some(job) = self.task_job.take() {
      match job.try_state() {
        JobState::Running => {
          self.task_job = Some(job);
          if self.task_open {
            if self.task_follow
              && let Some(live) = &self.task_live
            {
              self.task_scroll = task_bottom_offset(&live.output());
            }
            changed = true;
          }
        }
        JobState::Finished(Ok(Ok(_))) => {
          self.locale_gen_entries =
            locale::locale_gen_entries(std::path::Path::new("/etc/locale.gen")).unwrap_or_default();
          self.generated_locales = locale::generated_locales();
          self.success(tr(self.lang, "control_center.locales_generated_successfully").to_string());
          changed = true;
        }
        JobState::Finished(Ok(Err(error))) | JobState::Finished(Err(error)) => self.fail(error),
      }
    }
    changed
  }

  pub(crate) fn refresh_time(&mut self) {
    self.datetime = time::datetime_info();
  }

  pub(super) fn refresh_keyboard(&mut self) {
    // A draft with pending edits survives the refresh; an untouched draft
    // follows the system, so it never shows a stale layout list.
    let dirty = self.keyboard_layout_changed();
    self.keyboard_info = keyboard::info();
    self.keyboard_variants = keyboard::variants(&self.keyboard_info.default_layout);
    if !dirty {
      self.keyboard_draft = keyboard_draft::LayoutDraft::loaded(&self.keyboard_info);
    }
  }

  pub(crate) fn success(&mut self, text: String) {
    self.status = Some(Status {
      text,
      kind: StatusKind::Success,
      created: Instant::now(),
    });
  }

  pub(crate) fn fail(&mut self, error: impl ToString) {
    let error = error.to_string();
    self.status = Some(Status {
      text: error.clone(),
      kind: StatusKind::Error,
      created: Instant::now(),
    });
    self.error_modal = Some(error);
  }
}
