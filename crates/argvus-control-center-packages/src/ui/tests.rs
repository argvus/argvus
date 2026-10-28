use super::actions::{
  ActionCommand, action_command, action_message, cycle_country, preview_message,
};
use super::render::{
  TRANSACTION_CONTENT_HEIGHT, transaction_bottom_offset, transaction_wrapped_lines,
};
use super::*;
use ratatui::{Terminal, backend::TestBackend};

mod home_and_navigation;
mod mirror_pages;
mod package_lists;
mod repository_pages;
mod transactions;

fn app() -> PackagesApp {
  PackagesApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  )
}

fn screen(app: &mut PackagesApp, width: u16, height: u16) -> String {
  let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  let buffer = terminal.backend().buffer();
  (0..height)
    .map(|y| {
      (0..width)
        .map(|x| buffer[(x, y)].symbol())
        .collect::<String>()
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn row(rows: &[Row<Item>], item: Item) -> &Row<Item> {
  rows
    .iter()
    .find(|row| row.id() == Some(&item))
    .unwrap_or_else(|| panic!("no row {item:?}"))
}

fn package(name: &str, installed: bool) -> Package {
  Package {
    name: name.into(),
    version: "1.0".into(),
    installed,
    ..Default::default()
  }
}

/// Moves the cursor to `item` and presses Enter.
fn press(app: &mut PackagesApp, item: Item) {
  let rows = app.rows();
  assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
  app.handle(KeyCode::Enter);
}

const OFFICIAL_INSTALL: PackagesPage =
  PackagesPage::Task(PackageSource::Official, PackageTask::Install);
const OFFICIAL_REMOVE: PackagesPage =
  PackagesPage::Task(PackageSource::Official, PackageTask::Remove);
const OFFICIAL_UPDATE: PackagesPage =
  PackagesPage::Task(PackageSource::Official, PackageTask::Update);
const AUR_INSTALL: PackagesPage = PackagesPage::Task(PackageSource::Aur, PackageTask::Install);
const AUR_REMOVE: PackagesPage = PackagesPage::Task(PackageSource::Aur, PackageTask::Remove);
const AUR_UPDATE: PackagesPage = PackagesPage::Task(PackageSource::Aur, PackageTask::Update);

fn aur_app() -> PackagesApp {
  PackagesApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities {
      has_paru: true,
      ..Default::default()
    },
  )
}

/// Moves the cursor to the package `item` of a task list and presses `→`.
fn open_details_of(app: &mut PackagesApp, item: Item) {
  let rows = app.rows();
  assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
  app.handle(KeyCode::Right);
}

fn orphans_app() -> PackagesApp {
  let mut app = app();
  app.page = PackagesPage::Orphans;
  app.packages = vec![
    package("libfoo", true),
    package("python-bar", true),
    package("libbaz", true),
  ];
  app
}

/// Moves the focus to `item` on the Updates action bar and presses Enter.
fn press_action(app: &mut PackagesApp, item: Item) {
  let rows = app.update_action_rows();
  let index = rows
    .iter()
    .position(|row| row.id() == Some(&item))
    .unwrap_or_else(|| panic!("no action {item:?}"));
  assert!(rows[index].is_selectable(), "{item:?} is disabled");
  if app.actions.cursor().is_none() {
    app.handle(KeyCode::Tab);
  }
  while app.actions.cursor() != Some(index) {
    app.handle(KeyCode::Right);
  }
  app.handle(KeyCode::Enter);
}

fn update(name: &str) -> Update {
  Update {
    name: name.into(),
    current: "1".into(),
    available: "2".into(),
    ..Default::default()
  }
}

fn cascade(dependents: &[&str]) -> Action {
  Action::RemoveCascade {
    packages: vec!["7zip".into()],
    dependents: dependents.iter().map(|name| (*name).to_owned()).collect(),
  }
}

/// A Remove whose plan found dependents: the choice is open, on Cancel.
fn removal_choice_app() -> PackagesApp {
  let mut app = app();
  app.request(cascade(&["julia"]));
  app.preview = Some(TransactionPlan {
    remove: vec![package("julia", true), package("7zip", true)],
    ..Default::default()
  });
  app
}

/// The process a privileged operation would start, built without running
/// it; the executable stands for the installed Control Center.
fn privileged_process(action: Action) -> (String, ProcessRequest) {
  let Ok(ActionCommand::Privileged { echo, request }) =
    action_command(action, &Capabilities::default())
  else {
    panic!("not a privileged operation");
  };
  let process = SystemSettingsOperation::new(
    SystemProcessRunner,
    "/usr/lib/argvus/commands/control-center",
  )
  .process_for(&request)
  .unwrap();
  (echo, process)
}

fn repository(name: &str, enabled: bool, key: Option<&str>) -> Repository {
  use argvus_control_center_core::pacman_conf::RepositoryOrigin;
  Repository {
    name: name.into(),
    servers: vec![format!("https://{name}.example.org/$arch")],
    sig_level: None,
    origin: RepositoryOrigin::Include(format!("/etc/pacman.d/{name}.conf").into()),
    enabled,
    key: key.map(str::to_owned),
  }
}

fn ctrl(character: char) -> KeyEvent {
  KeyEvent::new(
    KeyCode::Char(character),
    crossterm::event::KeyModifiers::CONTROL,
  )
}

fn review(with_key: bool) -> RepositoryReview {
  use argvus_control_center_core::pacman_conf::{KeyInfo, RepositorySection};
  RepositoryReview {
    conf: "[argvus]\nServer = https://argvus.github.io/packages/arch/$arch\n".into(),
    sections: vec![RepositorySection {
      name: "argvus".into(),
      servers: vec!["https://argvus.github.io/packages/arch/$arch".into()],
      sig_level: None,
    }],
    key: with_key.then(|| repositories::ReviewedKey {
      info: KeyInfo {
        fingerprint: "0123456789ABCDEF0123456789ABCDEF01234567".into(),
        user_id: Some("ARGVUS <packages@argvus.org>".into()),
      },
      hex: "99010d".into(),
    }),
  }
}
