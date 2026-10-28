//! Login flow against greetd: submitting credentials, the replies it sends back and the status line they set.

use super::*;

impl LoginApp {
  /// Starts authentication or posts the response to the active PAM prompt.
  pub(super) fn submit(&mut self) {
    if self.waiting_for_prompt {
      // `take` clears the secret immediately after ownership moves to the
      // worker, minimizing how long credentials remain in UI state.
      let response = std::mem::take(&mut self.password);
      self.waiting_for_prompt = false;
      if self
        .commands
        .send(Command::AuthResponse(Some(response)))
        .is_err()
      {
        self.authentication_in_progress = false;
        self.set_error("internal_channel_unavailable");
      }
      return;
    }
    // A key repeat or an impatient second Enter must not create another greetd
    // session while the first request is still waiting for PAM. The worker
    // remains the owner of the active socket until it emits a final event.
    if self.authentication_in_progress {
      return;
    }
    let Some(user) = self.users.get(self.selected_user) else {
      self.set_error("no_login_user");
      return;
    };
    let Some(session) = self.sessions.get(self.selected_session) else {
      self.set_error("no_wayland_session");
      return;
    };
    // Some PAM stacks request a password only after CreateSession; retain an
    // initial response so it can be posted automatically when that prompt
    // arrives.
    let initial = std::mem::take(&mut self.password);
    self.pending_secret_response = (!initial.is_empty()).then_some(initial);
    self.authentication_in_progress = true;
    self.status = Some((self.i18n.tr("starting_authentication"), StatusKind::Info));
    self.focus = Focus::Password;
    if self
      .commands
      .send(Command::Begin {
        username: user.username.clone(),
        session: session.clone(),
        theme: self.theme_name.clone(),
        accent: self.theme_accent.clone(),
      })
      .is_err()
    {
      self.authentication_in_progress = false;
      self.set_error("internal_channel_unavailable");
    }
  }

  /// Drains all currently available worker events without blocking rendering.
  pub fn poll_events(&mut self) {
    while let Ok(event) = self.events.try_recv() {
      self.handle_event(event);
    }
  }

  /// Applies one worker event to the visible login state.
  pub(super) fn handle_event(&mut self, event: Event) {
    match event {
      Event::Ready => {}
      Event::AuthMessage(prompt) => {
        // A secret prompt may arrive after the user already entered a
        // password; post that pending response without displaying it again.
        if prompt.secret
          && let Some(response) = self.pending_secret_response.take()
        {
          let _ = self.commands.send(Command::AuthResponse(Some(response)));
          return;
        }
        self.prompt = prompt;
        self.waiting_for_prompt = true;
        self.focus = Focus::Password;
        self.status = None;
      }
      // Authentication messages are normalized at the UI boundary so backend
      // wording does not leak into the rest of the state machine.
      Event::Info(message) => {
        self.status = Some((i18n::auth_message(&self.i18n, &message), StatusKind::Warn))
      }
      Event::Error(message) => {
        self.password.clear();
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        self.set_status(i18n::auth_message(&self.i18n, &message), StatusKind::Error);
      }
      Event::AuthFailed(message) => {
        self.password.clear();
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        let text = if message.is_empty() {
          self.i18n.tr("authentication_failed")
        } else {
          i18n::auth_message(&self.i18n, &message)
        };
        self.set_status(text, StatusKind::Error);
      }
      Event::AuthPromptUnavailable => {
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        self.set_error("no_active_auth_prompt");
      }
      Event::GreeterUnavailable => {
        self.waiting_for_prompt = false;
        self.pending_secret_response = None;
        self.authentication_in_progress = false;
        self.set_error("login_service_unavailable");
      }
      Event::SessionStarting => {
        self.set_status(self.i18n.tr("starting_session"), StatusKind::Info);
        self.waiting_for_prompt = false;
      }
      Event::SessionStarted => {
        self.set_status(self.i18n.tr("session_started"), StatusKind::Success);
        self.authentication_in_progress = false;
        // Keep Kitty and the greeter compositor alive under the mapped overlay
        // until greetd sends SIGTERM. Returning here would restore the terminal
        // and create the visible gap this handoff prevents.
        self.handoff_active = true;
      }
    }
  }

  /// Displays an initial state error when discovery found no usable choices.
  pub(super) fn set_empty_state(&mut self) {
    if self.users.is_empty() {
      self.set_error("no_local_login_users");
    } else if self.sessions.is_empty() {
      self.set_error("no_wayland_sessions");
    }
  }

  /// Translates a catalog key into an error status.
  pub(super) fn set_error(&mut self, key: &str) {
    self.set_status(self.i18n.tr(key), StatusKind::Error);
  }

  /// Stores a status message together with its semantic severity.
  pub(super) fn set_status(&mut self, text: String, kind: StatusKind) {
    self.status = Some((text, kind));
  }
}
