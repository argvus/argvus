//! Application state and main-flow coordination of the About page.

use argvus_control_center_core::clipboard;
use argvus_control_center_core::jobs::{JobHandle, JobManager, JobState};

use crate::AboutError;
use crate::i18n::{Lang, na, tr};
use crate::image::Logo;
use crate::pages::{self, Doc};
use crate::system::SystemInfo;
use crate::theme::Theme;

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tab {
  System,
  About,
  Donate,
  Credits,
  Copyright,
}

impl Tab {
  pub const ALL: [Tab; 5] = [
    Tab::System,
    Tab::About,
    Tab::Donate,
    Tab::Credits,
    Tab::Copyright,
  ];

  pub const fn index(self) -> usize {
    match self {
      Tab::System => 0,
      Tab::About => 1,
      Tab::Donate => 2,
      Tab::Credits => 3,
      Tab::Copyright => 4,
    }
  }

  pub const fn from_index(index: usize) -> Self {
    match index % Self::ALL.len() {
      0 => Tab::System,
      1 => Tab::About,
      2 => Tab::Donate,
      3 => Tab::Credits,
      _ => Tab::Copyright,
    }
  }

  pub fn label(self, lang: Lang) -> &'static str {
    match self {
      Tab::System => tr(lang, "control_center.about_system_hardware"),
      Tab::About => tr(lang, "control_center.about"),
      Tab::Donate => tr(lang, "control_center.donate"),
      Tab::Credits => tr(lang, "control_center.credits"),
      Tab::Copyright => tr(lang, "control_center.copyright"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
  Info,
  Success,
  Error,
}

#[derive(Debug, Clone)]
pub struct Status {
  pub text: String,
  pub kind: StatusKind,
}

struct DocCache {
  tab: Tab,
  width: usize,
  selected: usize,
  doc: Doc<'static>,
}

/// Progress of the background read of [`SystemInfo`].
enum SystemLoad {
  /// Not requested yet: the page has not been drawn.
  Pending,
  /// The probes run on a worker thread; see [`App::poll`].
  Loading(JobHandle<SystemInfo>),
  Loaded,
}

pub struct App {
  pub lang: Lang,
  pub theme: Theme,
  pub system: SystemInfo,
  /// Whether `system` has been gathered; see [`App::ensure_loaded`].
  system_load: SystemLoad,
  jobs: JobManager,
  pub logo: Option<Logo>,
  pub active_tab: Tab,
  pub scroll: usize,
  pub selected: usize,
  pub status: Option<Status>,
  pub viewport: usize,
  pub width: u16,
  pub height: u16,
  /// Columns available to the text, set by the renderer; `0` until the first draw.
  text_width: u16,
  cache: Option<DocCache>,
  tab_scroll: [usize; 5],
  tab_selected: [usize; 5],
}

impl App {
  pub fn new(initial_tab: Tab) -> Self {
    let lang = Lang::detect();
    Self::with_context(initial_tab, lang, Theme::load())
  }

  /// Replaces the semantic theme used by this page.
  pub fn set_theme(&mut self, theme: &Theme) {
    self.theme = theme.clone();
  }

  pub fn with_context(initial_tab: Tab, lang: Lang, theme: Theme) -> Self {
    let status = Status {
      text: tr(lang, "control_center.welcome_to_argvus").to_string(),
      kind: StatusKind::Info,
    };
    Self {
      lang,
      theme,
      // Gathered on first draw: the Control Center builds this state at startup even
      // when the About page is never opened.
      system: SystemInfo::default(),
      system_load: SystemLoad::Pending,
      jobs: JobManager::default(),
      logo: Some(Logo::new()),
      active_tab: initial_tab,
      scroll: 0,
      selected: 0,
      status: Some(status),
      viewport: 0,
      width: 80,
      height: 24,
      text_width: 0,
      cache: None,
      tab_scroll: [0; 5],
      tab_selected: [0; 5],
    }
  }

  #[cfg(test)]
  pub fn test() -> Self {
    use crate::system::battery::{Battery, BatteryStatus};
    use crate::system::boot::BootMode;
    use crate::system::displays::Display;
    use crate::system::memory::MemoryUsage;
    use crate::system::packages::PackageCounts;
    use crate::system::storage::{DiskKind, Storage};

    const GIB: u64 = 1024 * 1024 * 1024;

    let lang = Lang::for_locale("pt-BR");
    let theme = Theme::load();
    Self {
      lang,
      theme,
      system: SystemInfo {
        firmware: Some("AMI 1.40 (03/15/2024)".into()),
        boot_mode: Some(BootMode::Uefi),
        secure_boot: Some(false),
        virtualization: Some("none".into()),
        bootloader: Some("systemd-boot 258.1".into()),
        argvus: Some("0.4.0".into()),
        hostname: Some("host".into()),
        os_name: Some("Arch Linux".into()),
        distributor: Some("Arch Linux".into()),
        kernel: Some("6.14.4-arch1-1".into()),
        architecture: Some("x86_64".into()),
        hyprland: Some("0.56.2".into()),
        wayland: Some("1.26.0".into()),
        gpu_drivers: Some("i915 (Mesa 25.2.3)".into()),
        gtk: Some("4.22.4".into()),
        displays: Some(vec![
          Display {
            name: "eDP-1".into(),
            width: 1920,
            height: 1200,
            refresh_hz: 60.0,
            scale: 1.0,
          },
          Display {
            name: "HDMI-A-1".into(),
            width: 2560,
            height: 1440,
            refresh_hz: 143.912,
            scale: 1.25,
          },
        ]),
        pipewire: Some("1.4.8".into()),
        quickshell: Some("0.3.1".into()),
        greetd: Some("0.10.3".into()),
        systemd: Some("258.1".into()),
        packages: Some(PackageCounts {
          total: 1024,
          explicit: 300,
          foreign: 12,
        }),
        uptime_seconds: Some(2 * 86_400 + 3 * 3_600 + 4 * 60),
        locale: Some("en_US.UTF-8".into()),
        batteries: vec![Battery {
          name: "BAT0".into(),
          percent: Some(85),
          status: BatteryStatus::Discharging,
          health: Some(91),
          cycles: Some(312),
        }],
        cpu: Some("Intel Core i5".into()),
        cpu_max_mhz: Some(4200),
        threads: Some(8),
        cores: Some(4),
        gpus: Some("Intel Graphics".into()),
        memory: Some(MemoryUsage {
          total_kib: 16_000_000,
          available_kib: Some(10_000_000),
          swap_total_kib: Some(8 * 1024 * 1024),
          swap_free_kib: Some(7 * 1024 * 1024),
        }),
        storage: Some(Storage {
          model: Some("Example SSD 1TB".into()),
          kind: DiskKind::Nvme,
          filesystem: "btrfs".into(),
          total_bytes: Some(465 * GIB),
          used_bytes: Some(120 * GIB),
        }),
        model: Some("LENOVO 21AH".into()),
        motherboard: Some("LENOVO 21AHCTO1WW".into()),
      },
      system_load: SystemLoad::Loaded,
      jobs: JobManager::default(),
      logo: None,
      active_tab: Tab::System,
      scroll: 0,
      selected: 0,
      status: None,
      viewport: 0,
      width: 80,
      height: 24,
      text_width: 0,
      cache: None,
      tab_scroll: [0; 5],
      tab_selected: [0; 5],
    }
  }

  /// Starts reading the system facts the first time the page is drawn, from any entry
  /// point (Home, search or CLI route).
  ///
  /// The probes run commands such as `pacman`, which take a few hundred milliseconds
  /// together, so they run on a worker thread and the page shows a loading line until
  /// [`App::poll`] receives the result.
  pub fn ensure_loaded(&mut self) {
    if matches!(self.system_load, SystemLoad::Pending) {
      let job = self.jobs.spawn(|_| Ok(SystemInfo::gather()));
      self.system_load = SystemLoad::Loading(job);
    }
  }

  /// Takes the background result when it is ready; returns `true` when the page must be redrawn.
  pub fn poll(&mut self) -> bool {
    let SystemLoad::Loading(job) = &self.system_load else {
      return false;
    };
    match job.try_state() {
      JobState::Running => false,
      JobState::Finished(result) => {
        // A failed worker leaves every value unknown ("N/A") instead of retrying forever.
        self.system = result.unwrap_or_default();
        self.system_load = SystemLoad::Loaded;
        self.cache = None;
        true
      }
    }
  }

  /// Whether the system facts are available.
  pub fn is_loaded(&self) -> bool {
    matches!(self.system_load, SystemLoad::Loaded)
  }

  /// Returns the installed ARGVUS version, or "N/A" in the interface language.
  pub fn argvus_version(&self) -> &str {
    self.system.argvus.as_deref().unwrap_or(na(self.lang))
  }

  pub fn current_doc(&mut self) -> Doc<'static> {
    let width = self.text_columns();
    if let Some(cache) = &self.cache
      && cache.tab == self.active_tab
      && cache.width == width
      && cache.selected == self.selected
    {
      return cache.doc.clone();
    }
    let doc = pages::doc_for(self, width, self.selected);
    self.cache = Some(DocCache {
      tab: self.active_tab,
      width,
      selected: self.selected,
      doc: doc.clone(),
    });
    doc
  }

  /// Records the area that shows the text: its height is the scroll viewport, and its
  /// width, minus the scrollbar column, is the wrapping width of the document.
  pub fn set_text_area(&mut self, width: u16, height: u16) {
    self.text_width = width;
    self.set_viewport(height as usize);
  }

  /// Wrapping width of the document. One column is kept free for the scrollbar, so the
  /// text does not change its wrapping when the scrollbar appears.
  fn text_columns(&self) -> usize {
    if self.text_width > 0 {
      usize::from(self.text_width.saturating_sub(1))
    } else {
      usize::from(self.width.max(MIN_WIDTH))
    }
  }

  pub fn set_viewport(&mut self, viewport: usize) {
    self.viewport = viewport;
    self.clamp_scroll();
  }

  pub fn next_tab(&mut self) {
    self.goto_tab(Tab::from_index(self.active_tab.index() + 1));
  }

  pub fn prev_tab(&mut self) {
    let count = Tab::ALL.len();
    self.goto_tab(Tab::from_index(self.active_tab.index() + count - 1));
  }

  pub fn goto_tab(&mut self, tab: Tab) {
    if tab == self.active_tab {
      return;
    }
    let current = self.active_tab.index();
    self.tab_scroll[current] = self.scroll;
    self.tab_selected[current] = self.selected;
    self.active_tab = tab;
    self.scroll = self.tab_scroll[tab.index()];
    self.selected = self.tab_selected[tab.index()];
    self.cache = None;
    self.clamp_scroll();
  }

  pub fn move_cursor(&mut self, delta: isize) {
    let doc = self.current_doc();
    if doc.actions.is_empty() {
      self.scroll_lines(delta);
      return;
    }
    let last = doc.actions.len() as isize - 1;
    let next = (self.selected as isize + delta).clamp(0, last) as usize;
    self.selected = next;
    self.ensure_selected_visible();
  }

  pub fn scroll_lines(&mut self, delta: isize) {
    let doc = self.current_doc();
    let max = doc.height().saturating_sub(self.viewport);
    let next = self.scroll as isize + delta;
    self.scroll = next.clamp(0, max as isize) as usize;
  }

  pub fn page(&mut self, delta: isize) {
    let step = self.viewport.max(1) as isize;
    self.scroll_lines(delta * step);
  }

  pub fn home(&mut self) {
    let doc = self.current_doc();
    if doc.actions.is_empty() {
      self.scroll = 0;
      return;
    }
    self.selected = 0;
    self.scroll = 0;
  }

  pub fn end(&mut self) {
    let doc = self.current_doc();
    if doc.actions.is_empty() {
      let max = doc.height().saturating_sub(self.viewport);
      self.scroll = max;
      return;
    }
    self.selected = doc.actions.len() - 1;
    self.ensure_selected_visible();
  }

  pub fn enter(&mut self) {
    let doc = self.current_doc();
    let Some(action) = doc.actions.get(self.selected) else {
      self.status = Some(Status {
        text: tr(self.lang, "control_center.no_link_here").to_string(),
        kind: StatusKind::Info,
      });
      return;
    };
    match open_url(&action.url) {
      Ok(()) => {
        self.status = Some(Status {
          text: tr(self.lang, "control_center.opening_link").to_string(),
          kind: StatusKind::Success,
        });
      }
      Err(_) => {
        self.status = Some(Status {
          text: tr(self.lang, "control_center.failed_to_open_the_link").to_string(),
          kind: StatusKind::Error,
        });
      }
    }
  }

  /// Copies the System & Hardware tab as plain text to the Wayland clipboard,
  /// for bug reports and support requests.
  pub fn copy_report(&mut self) {
    if self.active_tab != Tab::System || !self.is_loaded() {
      return;
    }
    let report = pages::system::report_text(self);
    let (key, kind) = match clipboard::copy(&report) {
      Ok(()) => ("control_center.about_report_copied", StatusKind::Success),
      Err(_) => ("control_center.about_report_copy_failed", StatusKind::Error),
    };
    self.status = Some(Status {
      text: tr(self.lang, key).to_string(),
      kind,
    });
  }

  pub fn ensure_selected_visible(&mut self) {
    let line = self
      .current_doc()
      .actions
      .get(self.selected)
      .map(|action| action.line)
      .unwrap_or(0);
    if self.scroll > line {
      self.scroll = line;
    }
    if self.viewport > 0 {
      let bottom = self.scroll + self.viewport;
      if line + 1 > bottom {
        self.scroll = line + 1 - self.viewport;
      }
    }
  }

  pub fn clamp_scroll(&mut self) {
    let height = self.current_doc().height();
    let max = height.saturating_sub(self.viewport);
    self.scroll = self.scroll.min(max);
  }

  pub fn resize(&mut self, width: u16, height: u16) {
    self.width = width;
    self.height = height;
  }

  pub fn too_small(&self) -> bool {
    self.width < MIN_WIDTH || self.height < MIN_HEIGHT
  }
}

fn open_url(url: &str) -> Result<(), AboutError> {
  argvus_control_center_core::process::command("xdg-open")
    .arg(url)
    .spawn()
    .map(|_| ())
    .map_err(AboutError::OpenLink)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tab_switching_wraps_around() {
    let mut app = App::test();
    assert_eq!(app.active_tab, Tab::System);
    app.prev_tab();
    assert_eq!(app.active_tab, Tab::Copyright);
    app.next_tab();
    assert_eq!(app.active_tab, Tab::System);
  }

  #[test]
  fn cursor_moves_within_links() {
    let mut app = App::test();
    app.active_tab = Tab::About;
    app.cache = None;
    app.move_cursor(1);
    assert!(app.selected <= 1);
  }

  #[test]
  fn scrolls_plain_content_without_links() {
    let mut app = App::test();
    app.active_tab = Tab::Copyright;
    app.cache = None;
    app.viewport = 20;
    app.scroll_lines(1);
    assert_eq!(app.scroll, 1);
    app.end();
    let expected = app.current_doc().height().saturating_sub(app.viewport);
    assert_eq!(app.scroll, expected);
  }

  #[test]
  fn shows_loading_line_until_the_background_read_finishes() {
    use std::time::{Duration, Instant};

    let mut app = App::with_context(Tab::System, Lang::for_locale("en-US"), Theme::load());
    assert!(!app.is_loaded());
    app.ensure_loaded();
    let loading = app
      .current_doc()
      .lines
      .iter()
      .map(ToString::to_string)
      .collect::<String>();
    assert!(loading.contains("Loading system information"));

    let deadline = Instant::now() + Duration::from_secs(30);
    while !app.poll() {
      assert!(Instant::now() < deadline, "system read did not finish");
      std::thread::sleep(Duration::from_millis(5));
    }
    assert!(app.is_loaded());
    assert!(!app.poll(), "a finished read is delivered once");
  }

  #[test]
  fn reports_clipboard_failure_in_the_status_line() {
    // Test builds never start programs, so `wl-copy` always fails here.
    let mut app = App::test();
    app.lang = Lang::for_locale("en-US");
    app.copy_report();
    let status = app.status.expect("status");
    assert_eq!(status.kind, StatusKind::Error);
    assert!(status.text.contains("wl-copy"));
  }

  #[test]
  fn copies_only_from_the_system_tab() {
    let mut app = App::test();
    app.goto_tab(Tab::Credits);
    app.copy_report();
    assert!(app.status.is_none());
  }

  #[test]
  fn resize_updates_dimensions() {
    let mut app = App::test();
    app.resize(120, 40);
    assert_eq!((app.width, app.height), (120, 40));
  }

  #[test]
  fn each_tab_restores_its_scroll_and_selection() {
    let mut app = App::test();
    app.active_tab = Tab::About;
    app.scroll = 4;
    app.selected = 1;
    app.goto_tab(Tab::Credits);
    app.scroll = 2;
    app.goto_tab(Tab::About);
    assert_eq!(app.scroll, 4);
    assert_eq!(app.selected, 1);
    app.goto_tab(Tab::Credits);
    assert_eq!(app.scroll, 2);
  }
}
