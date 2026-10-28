use super::actions::boot_request;
use super::input::{accepts_input_char, input_tail};
use super::render::transaction_bottom_offset;
use super::*;
use crate::model::{BootEntry, KernelInfo};
use argvus_tui::menu::RowKind;
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;

mod kernel_command_line;
mod pages;
mod transactions;

fn app() -> BootApp {
  BootApp::new(
    Lang::for_locale("en-US"),
    Theme::load(),
    Capabilities::default(),
  )
}

fn screen(app: &mut BootApp, width: u16, height: u16) -> String {
  let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
  terminal.draw(|frame| app.draw(frame)).unwrap();
  terminal
    .backend()
    .buffer()
    .content
    .iter()
    .map(|cell| cell.symbol())
    .collect()
}

/// Moves the cursor to `item` and presses Enter.
fn press(app: &mut BootApp, item: Item) {
  let rows = app.rows();
  assert!(app.menu.select(&rows, &item), "{item:?} is not selectable");
  app.handle(KeyCode::Enter);
}

fn row(rows: &[Row<Item>], item: Item) -> &Row<Item> {
  rows
    .iter()
    .find(|row| row.id() == Some(&item))
    .unwrap_or_else(|| panic!("{item:?} is missing"))
}

fn ids(rows: &[Row<Item>]) -> Vec<Item> {
  rows.iter().filter_map(|row| row.id().copied()).collect()
}

fn systemd_app() -> BootApp {
  let mut app = app();
  app.snapshot.bootloader = BootloaderKind::SystemdBoot;
  app.snapshot.bootloader_info.entries = vec![BootEntry {
    id: "arch.conf".into(),
    title: "Arch".into(),
    linux: Some("/vmlinuz-linux".into()),
    is_default: true,
    ..Default::default()
  }];
  app.snapshot.kernels = vec![KernelInfo {
    package: "linux".into(),
    version: "6.1".into(),
    current: true,
    uki: Some("/boot/EFI/Linux/linux.efi".into()),
    ..Default::default()
  }];
  app
}

/// A snapshot job that stays running long enough for the test.
fn running_job(app: &mut BootApp) {
  app.job = Some(app.jobs.spawn(|_| {
    std::thread::sleep(Duration::from_secs(2));
    Ok(Ok(BootSnapshot::default()))
  }));
}
