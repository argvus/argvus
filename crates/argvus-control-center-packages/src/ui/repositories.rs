//! Official Repositories > Repositories: adding an unofficial pacman
//! repository (its configuration and signing key) and listing, enabling,
//! disabling and removing the ones configured.
//!
//! The Add page is a draft: nothing changes until `Review and add`, which
//! downloads and checks everything as the user and then asks for
//! confirmation with the key fingerprint. The changes themselves run as
//! root (`system-settings package repo-*`).

use super::{Action, Item, Loaded, PackagesApp};
use crate::{backend::PackageBackend, model::PackagesPage};
use argvus_control_center_core::{
  pacman_conf::{self, KeyInfo, Repository, RepositoryOrigin, RepositorySection},
  process::SystemProcessRunner,
};
use argvus_i18n::{Lang, tr};
use argvus_tui::{
  icons,
  menu::{Emphasis, Row},
  text_editor::TextEditor,
};

/// Values of `SigLevel` offered by the form, after the global default
/// (`None`, no `SigLevel` line), as in the Arch Wiki examples.
const SIG_LEVEL_PRESETS: &[Option<&str>] = &[
  None,
  Some("Required DatabaseOptional"),
  Some("Required TrustedOnly"),
  Some("Optional TrustAll"),
  Some("Never"),
];

/// Where the configuration of a repository being added comes from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ConfSource {
  /// A `.conf` downloaded from an https address.
  #[default]
  Url,
  /// A `.conf` typed or pasted.
  Text,
  /// Name, Server and SigLevel, turned into a `.conf`.
  Form,
}

/// Where the signing key of a repository being added comes from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum KeySource {
  /// No key: the repository is unsigned or its key is already trusted.
  #[default]
  None,
  Url,
  Text,
}

/// A field of the Add page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DraftField {
  Source,
  ConfUrl,
  ConfText,
  Name,
  Server,
  SigLevel,
  KeySource,
  KeyUrl,
  KeyText,
}

/// The repository being added; nothing is written until it is confirmed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RepositoryDraft {
  pub(super) source: ConfSource,
  pub(super) conf_url: String,
  pub(super) conf_text: String,
  pub(super) name: String,
  pub(super) server: String,
  /// Index into [`SIG_LEVEL_PRESETS`].
  pub(super) sig_level: usize,
  pub(super) key_source: KeySource,
  pub(super) key_url: String,
  pub(super) key_text: String,
}

impl RepositoryDraft {
  /// Whether the user changed anything, so leaving asks first.
  pub(super) fn is_touched(&self) -> bool {
    *self != Self::default()
  }

  /// Whether every field the chosen sources need is filled, so the review
  /// can run.
  pub(super) fn is_complete(&self) -> bool {
    let configuration = match self.source {
      ConfSource::Url => !self.conf_url.trim().is_empty(),
      ConfSource::Text => !self.conf_text.trim().is_empty(),
      ConfSource::Form => {
        pacman_conf::is_valid_name(self.name.trim()) && !self.server.trim().is_empty()
      }
    };
    let key = match self.key_source {
      KeySource::None => true,
      KeySource::Url => !self.key_url.trim().is_empty(),
      KeySource::Text => !self.key_text.trim().is_empty(),
    };
    configuration && key
  }

  fn value(&self, field: DraftField) -> &str {
    match field {
      DraftField::ConfUrl => &self.conf_url,
      DraftField::ConfText => &self.conf_text,
      DraftField::Name => &self.name,
      DraftField::Server => &self.server,
      DraftField::KeyUrl => &self.key_url,
      DraftField::KeyText => &self.key_text,
      DraftField::Source | DraftField::SigLevel | DraftField::KeySource => "",
    }
  }

  fn set_value(&mut self, field: DraftField, text: String) {
    // Addresses and names are one line; surrounding spaces are typos.
    let line = || text.trim().to_owned();
    match field {
      DraftField::ConfUrl => self.conf_url = line(),
      DraftField::Name => self.name = line(),
      DraftField::Server => self.server = line(),
      DraftField::KeyUrl => self.key_url = line(),
      DraftField::ConfText => self.conf_text = text,
      DraftField::KeyText => self.key_text = text,
      DraftField::Source | DraftField::SigLevel | DraftField::KeySource => {}
    }
  }

  /// Moves a choice field by `step`, wrapping around.
  fn cycle(&mut self, field: DraftField, step: i32) {
    match field {
      DraftField::Source => {
        const SOURCES: [ConfSource; 3] = [ConfSource::Url, ConfSource::Text, ConfSource::Form];
        self.source = SOURCES[cycled(position(&SOURCES, self.source), step, SOURCES.len())];
      }
      DraftField::KeySource => {
        const KEYS: [KeySource; 3] = [KeySource::None, KeySource::Url, KeySource::Text];
        self.key_source = KEYS[cycled(position(&KEYS, self.key_source), step, KEYS.len())];
      }
      DraftField::SigLevel => {
        self.sig_level = cycled(self.sig_level, step, SIG_LEVEL_PRESETS.len());
      }
      _ => {}
    }
  }
}

fn position<T: PartialEq>(values: &[T], value: T) -> usize {
  values
    .iter()
    .position(|candidate| *candidate == value)
    .unwrap_or(0)
}

fn cycled(index: usize, step: i32, len: usize) -> usize {
  (index as i64 + i64::from(step)).rem_euclid(len.max(1) as i64) as usize
}

/// A checked signing key, ready for the privileged import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReviewedKey {
  pub(super) info: KeyInfo,
  /// The key bytes as hexadecimal text (see `pacman_conf::encode_key`).
  pub(super) hex: String,
}

/// Everything `Review and add` downloaded and checked, shown in the
/// confirmation and passed to the privileged `repo-add`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RepositoryReview {
  pub(super) conf: String,
  pub(super) sections: Vec<RepositorySection>,
  pub(super) key: Option<ReviewedKey>,
}

impl RepositoryReview {
  /// The repository name, which also names its file in `/etc/pacman.d/`.
  pub(super) fn name(&self) -> &str {
    self
      .sections
      .first()
      .map(|section| section.name.as_str())
      .unwrap_or_default()
  }
}

/// Downloads and checks the draft as the user. Runs in a job: it may wait
/// for the network.
fn review_draft(
  draft: &RepositoryDraft,
  lang: Lang,
  backend: &PackageBackend<SystemProcessRunner>,
) -> Result<RepositoryReview, String> {
  let fetch = |url: &str| {
    backend
      .fetch_https(url.trim())
      .map_err(|error| error.to_string())
  };
  let conf = match draft.source {
    ConfSource::Url => String::from_utf8(fetch(&draft.conf_url)?)
      .map_err(|_| tr(lang, "control_center.repo_not_text").to_owned())?,
    ConfSource::Text => draft.conf_text.clone(),
    ConfSource::Form => pacman_conf::form_conf(
      draft.name.trim(),
      draft.server.trim(),
      SIG_LEVEL_PRESETS[draft.sig_level],
    ),
  };
  let sections = pacman_conf::parse_repository_conf(&conf)
    .map_err(|error| format!("{}: {error}", tr(lang, "control_center.repo_invalid_conf")))?;
  let configured = backend.repositories().map_err(|error| error.to_string())?;
  if let Some(section) = sections.iter().find(|section| {
    configured
      .iter()
      .any(|repository| repository.name == section.name)
      || pacman_conf::repository_file(&section.name).exists()
  }) {
    return Err(tr(lang, "control_center.repo_exists").replace("{name}", &section.name));
  }
  let key_bytes = match draft.key_source {
    KeySource::None => None,
    KeySource::Url => Some(fetch(&draft.key_url)?),
    KeySource::Text => Some(draft.key_text.clone().into_bytes()),
  };
  let key = match key_bytes {
    None => None,
    Some(bytes) => {
      let keys = backend
        .key_info(&bytes)
        .map_err(|error| error.to_string())?;
      let [info] = <[KeyInfo; 1]>::try_from(keys).map_err(|keys| {
        tr(lang, "control_center.repo_key_count").replace("{count}", &keys.len().to_string())
      })?;
      Some(ReviewedKey {
        info,
        hex: pacman_conf::encode_key(&bytes),
      })
    }
  };
  Ok(RepositoryReview {
    conf,
    sections,
    key,
  })
}

/// The confirmation text of `repo-add`.
pub(super) fn add_message(lang: Lang, review: &RepositoryReview) -> String {
  let names = review
    .sections
    .iter()
    .map(|section| section.name.as_str())
    .collect::<Vec<_>>()
    .join(", ");
  let servers = review
    .sections
    .iter()
    .flat_map(|section| section.servers.iter().map(String::as_str))
    .collect::<Vec<_>>()
    .join(", ");
  let sig_level = review
    .sections
    .first()
    .and_then(|section| section.sig_level.clone())
    .unwrap_or_else(|| tr(lang, "control_center.repo_sig_default").to_owned());
  let file = pacman_conf::repository_file(review.name());
  let mut message = tr(lang, "control_center.repo_add_message")
    .replace("{names}", &names)
    .replace("{servers}", &servers)
    .replace("{sig_level}", &sig_level)
    .replace("{file}", &file.display().to_string());
  message.push('\n');
  match &review.key {
    Some(key) => message.push_str(
      &tr(lang, "control_center.repo_add_key")
        .replace("{user_id}", key.info.user_id.as_deref().unwrap_or("—"))
        .replace("{fingerprint}", &spaced_fingerprint(&key.info.fingerprint)),
    ),
    None => message.push_str(tr(lang, "control_center.repo_add_no_key")),
  }
  message
}

/// A fingerprint in groups of four, as gpg prints it, so it can be compared
/// by eye with the one the repository publishes.
fn spaced_fingerprint(fingerprint: &str) -> String {
  fingerprint
    .as_bytes()
    .chunks(4)
    .map(|group| String::from_utf8_lossy(group).into_owned())
    .collect::<Vec<_>>()
    .join(" ")
}

impl PackagesApp {
  /// The Repositories page: Add and List.
  pub(super) fn repositories_rows(&self) -> Vec<Row<Item>> {
    vec![
      Row::submenu(
        Item::Open(PackagesPage::RepositoryAdd),
        tr(self.lang, "control_center.repository_add"),
      )
      .icon(icons::ADD),
      Row::submenu(
        Item::Open(PackagesPage::RepositoryList),
        tr(self.lang, "control_center.repository_list"),
      )
      .icon(icons::LOGS)
      .detail(self.repository_count_text(self.repositories.len())),
    ]
  }

  pub(super) fn repository_count_text(&self, count: usize) -> String {
    tr(self.lang, "control_center.repo_count").replace("{count}", &count.to_string())
  }

  /// The Add page: the configuration, the key, then `Review and add`.
  pub(super) fn repository_add_rows(&self) -> Vec<Row<Item>> {
    let lang = self.lang;
    let draft = &self.repo_draft;
    let label = |key: &'static str| tr(lang, key);
    let field = |field: DraftField, key: &'static str, icon: &'static str| {
      Row::value(
        Item::RepoField(field),
        label(key),
        self.field_display(field),
        None,
      )
      .icon(icon)
    };
    let choice = |field: DraftField, key: &'static str, icon: &'static str| {
      Row::value(
        Item::RepoField(field),
        label(key),
        self.field_display(field),
        Some(1),
      )
      .icon(icon)
    };
    let mut rows = vec![
      Row::section(label("control_center.repo_section_repository")),
      choice(
        DraftField::Source,
        "control_center.repo_source",
        icons::IMPORT,
      ),
    ];
    match draft.source {
      ConfSource::Url => rows.push(field(
        DraftField::ConfUrl,
        "control_center.repo_conf_url",
        icons::LINK,
      )),
      ConfSource::Text => rows.push(field(
        DraftField::ConfText,
        "control_center.repo_conf_text",
        icons::TEXT_EDITOR,
      )),
      ConfSource::Form => rows.extend([
        field(DraftField::Name, "control_center.repo_name", icons::ID_CARD),
        field(
          DraftField::Server,
          "control_center.repo_server",
          icons::NETWORK,
        ),
        choice(
          DraftField::SigLevel,
          "control_center.repo_sig_level",
          icons::SHIELD,
        ),
      ]),
    }
    rows.push(Row::section(label("control_center.repo_section_key")));
    rows.push(choice(
      DraftField::KeySource,
      "control_center.repo_key",
      icons::KEY,
    ));
    match draft.key_source {
      KeySource::None => {}
      KeySource::Url => rows.push(field(
        DraftField::KeyUrl,
        "control_center.repo_key_url",
        icons::LINK,
      )),
      KeySource::Text => rows.push(field(
        DraftField::KeyText,
        "control_center.repo_key_text",
        icons::TEXT_EDITOR,
      )),
    }
    rows.push(Row::separator());
    rows.push(
      Row::action(Item::RepoReview, label("control_center.repo_review"))
        .icon(icons::APPLY)
        .emphasis(Emphasis::Primary)
        .enabled(draft.is_complete()),
    );
    rows
  }

  /// The value shown at the right of a field of the Add page.
  fn field_display(&self, field: DraftField) -> String {
    let lang = self.lang;
    let draft = &self.repo_draft;
    let text = |key| tr(lang, key).to_owned();
    match field {
      DraftField::Source => text(match draft.source {
        ConfSource::Url => "control_center.repo_source_url",
        ConfSource::Text => "control_center.repo_source_text",
        ConfSource::Form => "control_center.repo_source_form",
      }),
      DraftField::KeySource => text(match draft.key_source {
        KeySource::None => "control_center.repo_key_none",
        KeySource::Url => "control_center.repo_source_url",
        KeySource::Text => "control_center.repo_source_text",
      }),
      DraftField::SigLevel => SIG_LEVEL_PRESETS[draft.sig_level]
        .map(str::to_owned)
        .unwrap_or_else(|| text("control_center.repo_sig_default")),
      DraftField::ConfText | DraftField::KeyText => {
        let lines = draft.value(field).lines().count();
        if lines == 0 {
          "—".into()
        } else {
          tr(lang, "control_center.repo_lines").replace("{count}", &lines.to_string())
        }
      }
      _ => {
        let value = draft.value(field);
        if value.is_empty() {
          "—".into()
        } else {
          value.to_owned()
        }
      }
    }
  }

  /// The List page: one row per repository, which opens its page.
  pub(super) fn repository_list_rows(&self) -> Vec<Row<Item>> {
    if self.repositories.is_empty() && self.job.is_none() {
      return vec![Row::info(tr(self.lang, "control_center.repo_none"), "")];
    }
    self
      .repositories
      .iter()
      .enumerate()
      .map(|(index, repository)| {
        Row::submenu(Item::Repository(index), repository.name.clone()).detail(format!(
          "{} · {}",
          self.repository_state(repository),
          origin_name(&repository.origin)
        ))
      })
      .collect()
  }

  fn repository_state(&self, repository: &Repository) -> &'static str {
    tr(
      self.lang,
      if repository.enabled {
        "control_center.enabled"
      } else {
        "control_center.disabled"
      },
    )
  }

  /// One repository: its details, the Enabled toggle and, in the Danger
  /// zone, Remove.
  pub(super) fn repository_details_rows(&self, index: usize) -> Vec<Row<Item>> {
    let lang = self.lang;
    let Some(repository) = self.repositories.get(index) else {
      return Vec::new();
    };
    let mut rows = vec![
      Row::section(tr(lang, "control_center.repo_section_repository")),
      Row::info(
        tr(lang, "control_center.repo_name"),
        repository.name.clone(),
      ),
      Row::info(
        self.field("control_center.status"),
        self.repository_state(repository),
      ),
      Row::info(
        tr(lang, "control_center.repo_file"),
        match &repository.origin {
          RepositoryOrigin::Inline => pacman_conf::PACMAN_CONF.to_owned(),
          RepositoryOrigin::Include(path) => path.display().to_string(),
        },
      ),
      Row::info(
        tr(lang, "control_center.repo_sig_level"),
        repository
          .sig_level
          .clone()
          .unwrap_or_else(|| tr(lang, "control_center.repo_sig_default").to_owned()),
      ),
    ];
    rows.extend(
      repository
        .servers
        .iter()
        .map(|server| Row::info(tr(lang, "control_center.repo_server"), server.clone())),
    );
    rows.push(Row::info(
      tr(lang, "control_center.repo_key"),
      repository
        .key
        .as_deref()
        .map(spaced_fingerprint)
        .unwrap_or_else(|| tr(lang, "control_center.repo_key_unknown").to_owned()),
    ));
    rows.push(Row::section(tr(lang, "control_center.section_actions")));
    rows.push(Row::toggle(
      Item::RepoToggle,
      tr(lang, "control_center.enabled"),
      repository.enabled,
    ));
    rows.push(Row::section(tr(lang, "control_center.danger_zone")));
    rows.push(
      Row::destructive(Item::RepoRemove, tr(lang, "control_center.remove")).icon(icons::DELETE),
    );
    rows
  }

  /// The repository of the open details page.
  fn open_repository(&self) -> Option<&Repository> {
    match self.page {
      PackagesPage::RepositoryDetails(index) => self.repositories.get(index),
      _ => None,
    }
  }

  /// Enter on a field of the Add page: a choice moves forward, a text field
  /// opens the editor.
  pub(super) fn activate_repository_field(&mut self, field: DraftField) {
    match field {
      DraftField::Source | DraftField::SigLevel | DraftField::KeySource => {
        self.repo_draft.cycle(field, 1)
      }
      DraftField::ConfText | DraftField::KeyText => {
        self.editor = Some((
          field,
          TextEditor::multi_line(self.field_label(field), self.repo_draft.value(field)),
        ))
      }
      _ => {
        self.editor = Some((
          field,
          TextEditor::single_line(self.field_label(field), self.repo_draft.value(field)),
        ))
      }
    }
  }

  fn field_label(&self, field: DraftField) -> &'static str {
    tr(
      self.lang,
      match field {
        DraftField::ConfUrl => "control_center.repo_conf_url",
        DraftField::ConfText => "control_center.repo_conf_text",
        DraftField::Name => "control_center.repo_name",
        DraftField::Server => "control_center.repo_server",
        DraftField::KeyUrl => "control_center.repo_key_url",
        DraftField::KeyText => "control_center.repo_key_text",
        DraftField::Source => "control_center.repo_source",
        DraftField::SigLevel => "control_center.repo_sig_level",
        DraftField::KeySource => "control_center.repo_key",
      },
    )
  }

  /// `←/→` on a choice field of the Add page.
  pub(super) fn adjust_repository_field(&mut self, field: DraftField, step: i32) {
    self.repo_draft.cycle(field, step);
  }

  /// Stores the text saved in the editor into its field.
  pub(super) fn save_repository_field(&mut self, field: DraftField, text: String) {
    self.repo_draft.set_value(field, text);
  }

  /// `Review and add`: downloads and checks the draft in a job; the
  /// confirmation opens when it finishes (see `Loaded::RepositoryReview`).
  pub(super) fn start_repository_review(&mut self) {
    if self.busy() || !self.repo_draft.is_complete() {
      return;
    }
    let draft = self.repo_draft.clone();
    let lang = self.lang;
    let capabilities = self.capabilities.clone();
    self.status = Some(argvus_tui::components::StatusMessage {
      kind: argvus_tui::components::StatusKind::Info,
      text: tr(lang, "control_center.repo_reviewing").into(),
    });
    self.job = Some(self.jobs.spawn(move |_| {
      let backend = PackageBackend::new(SystemProcessRunner, capabilities);
      Ok(
        review_draft(&draft, lang, &backend)
          .map(|review| Loaded::RepositoryReview(Box::new(review))),
      )
    }));
  }

  /// The Enabled toggle of the open repository: asks before switching it.
  pub(super) fn toggle_open_repository(&mut self) {
    if let Some(repository) = self.open_repository() {
      let name = repository.name.clone();
      self.request(if repository.enabled {
        Action::RepoDisable(name)
      } else {
        Action::RepoEnable(name)
      });
    }
  }

  /// Remove on the open repository. With a recorded key the confirmation
  /// is a choice: remove only the repository, or its key too.
  pub(super) fn remove_open_repository(&mut self) {
    if let Some(repository) = self.open_repository() {
      self.request(Action::RepoRemove {
        name: repository.name.clone(),
        key: repository.key.clone(),
        delete_key: false,
      });
    }
  }
}

/// How the List names where a repository is written.
fn origin_name(origin: &RepositoryOrigin) -> String {
  match origin {
    RepositoryOrigin::Inline => "pacman.conf".into(),
    RepositoryOrigin::Include(path) => path
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
      .unwrap_or_default(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_draft_is_complete_only_with_what_its_sources_need() {
    let mut draft = RepositoryDraft::default();
    assert!(!draft.is_touched());
    assert!(!draft.is_complete());
    draft.conf_url = "https://argvus.github.io/packages/arch/argvus.conf".into();
    assert!(draft.is_touched() && draft.is_complete());
    draft.key_source = KeySource::Url;
    assert!(!draft.is_complete(), "the key URL is missing");
    draft.key_url = "https://argvus.github.io/packages/arch/argvus.gpg".into();
    assert!(draft.is_complete());
    draft.source = ConfSource::Form;
    draft.name = "bad name".into();
    draft.server = "https://a".into();
    assert!(!draft.is_complete(), "the form needs a valid name");
    draft.name = "mine".into();
    assert!(draft.is_complete());
  }

  #[test]
  fn choices_cycle_both_ways_and_text_fields_trim_single_lines() {
    let mut draft = RepositoryDraft::default();
    draft.cycle(DraftField::Source, -1);
    assert_eq!(draft.source, ConfSource::Form);
    draft.cycle(DraftField::KeySource, 1);
    assert_eq!(draft.key_source, KeySource::Url);
    draft.cycle(DraftField::SigLevel, -1);
    assert_eq!(SIG_LEVEL_PRESETS[draft.sig_level], Some("Never"));
    draft.set_value(DraftField::ConfUrl, " https://a/x.conf \n".into());
    assert_eq!(draft.conf_url, "https://a/x.conf");
    draft.set_value(DraftField::KeyText, "-----BEGIN\nkey\n".into());
    assert_eq!(draft.key_text, "-----BEGIN\nkey\n");
  }

  #[test]
  fn the_add_confirmation_shows_the_fingerprint_in_groups() {
    let review = RepositoryReview {
      conf: "[argvus]\nServer = https://a/$arch\n".into(),
      sections: vec![RepositorySection {
        name: "argvus".into(),
        servers: vec!["https://a/$arch".into()],
        sig_level: None,
      }],
      key: Some(ReviewedKey {
        info: KeyInfo {
          fingerprint: "0123456789ABCDEF0123456789ABCDEF01234567".into(),
          user_id: Some("ARGVUS <packages@argvus.org>".into()),
        },
        hex: "99".into(),
      }),
    };
    let message = add_message(Lang::for_locale("en-US"), &review);
    assert!(message.contains("argvus"), "{message}");
    assert!(message.contains("/etc/pacman.d/argvus.conf"), "{message}");
    assert!(message.contains("0123 4567 89AB CDEF"), "{message}");
    assert!(
      message.contains("ARGVUS <packages@argvus.org>"),
      "{message}"
    );
    assert!(!message.contains('{'), "placeholders are filled: {message}");
  }
}
