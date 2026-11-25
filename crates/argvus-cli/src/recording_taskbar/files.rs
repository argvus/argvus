use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use argvus_i18n::{FALLBACK_LOCALE, I18n, SYSTEM_CATALOG_DIR, config_path, detect_locale_from};
use gtk::gio::prelude::*;
use gtk::{gio, glib};

#[derive(PartialEq, Eq)]
pub(super) struct Labels {
  pub(super) stopped: String,
  pub(super) recording: String,
  pub(super) paused: String,
}

struct Directory {
  monitor: gio::FileMonitor,
  identity: (u64, u64),
  targets: BTreeSet<PathBuf>,
}

impl Drop for Directory {
  fn drop(&mut self) {
    self.monitor.cancel();
  }
}

pub(super) struct Watch {
  root: PathBuf,
  locale_file: Option<PathBuf>,
  locale: RefCell<String>,
  canonical_root: RefCell<Option<PathBuf>>,
  directories: RefCell<BTreeMap<PathBuf, Directory>>,
  pending: Cell<Option<glib::SourceId>>,
  publish: Box<dyn Fn(Option<Labels>)>,
}

pub(super) fn watch(publish: impl Fn(Option<Labels>) + 'static) -> Rc<Watch> {
  let root = std::env::var_os("ARGVUS_I18N_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|| SYSTEM_CATALOG_DIR.into());
  let watch = Rc::new(Watch {
    root: std::path::absolute(&root).unwrap_or(root),
    locale_file: config_path(),
    locale: RefCell::new(String::new()),
    canonical_root: RefCell::new(None),
    directories: RefCell::new(BTreeMap::new()),
    pending: Cell::new(None),
    publish: Box::new(publish),
  });
  watch.reload();
  watch
}

impl Watch {
  fn reload(self: &Rc<Self>) {
    let locale = detect_locale_from(&self.root);
    *self.locale.borrow_mut() = locale.clone();
    if !self.update_watches(&locale) {
      (self.publish)(None);
      self.schedule(Duration::from_secs(5));
      return;
    }
    // A changed locale can select a different catalog while watches are being
    // installed. Reload with the newly resolved locale before publishing.
    let labels = I18n::from_root(&self.root, "hyprland")
      .ok()
      .and_then(|i18n| {
        if i18n.locale() != locale {
          self.schedule(Duration::ZERO);
          return None;
        }
        let translated = |key| {
          i18n
            .tr_args(key, std::iter::empty::<(&str, &str)>())
            .trim_end_matches('\n')
            .to_owned()
        };
        Some(Labels {
          stopped: translated("screenshot.recording.none"),
          recording: translated("screenshot.recording.started"),
          paused: translated("screenshot.recording.paused"),
        })
      });
    (self.publish)(labels);
  }

  fn update_watches(self: &Rc<Self>, locale: &str) -> bool {
    let mut targets = BTreeMap::new();
    if let Some(path) = &self.locale_file {
      track_path(&mut targets, path);
    }
    track_path(&mut targets, &self.root);
    // The root also reports when an exact or regional-fallback locale appears.
    let canonical_root = self.root.canonicalize().ok().filter(|root| root.is_dir());
    if let Some(root) = &canonical_root {
      targets.entry(root.clone()).or_default();
    }
    *self.canonical_root.borrow_mut() = canonical_root;
    for locale in [FALLBACK_LOCALE, locale] {
      let catalog = self.root.join(locale);
      track_path(&mut targets, &catalog);
      track_path(&mut targets, &catalog.join("hyprland.json"));
    }
    let mut directories = self.directories.borrow_mut();
    directories.retain(|path, _| targets.contains_key(path));
    let mut complete = true;
    for (path, targets) in targets {
      let identity = fs::metadata(&path)
        .ok()
        .map(|metadata| (metadata.dev(), metadata.ino()));
      if let Some(directory) = directories.get_mut(&path)
        && Some(directory.identity) == identity
        && !directory.monitor.is_cancelled()
      {
        directory.targets = targets;
        continue;
      }
      directories.remove(&path);
      let monitor = gio::Vfs::local()
        .file_for_path(&path)
        .monitor_directory(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE);
      if let (Some(identity), Ok(monitor)) = (identity, monitor) {
        monitor.connect_changed({
          let watch = Rc::downgrade(self);
          let path = path.clone();
          move |_, file, other, event| {
            if let Some(watch) = watch.upgrade() {
              watch.changed(&path, file, other, event);
            }
          }
        });
        directories.insert(
          path,
          Directory {
            monitor,
            identity,
            targets,
          },
        );
      } else {
        complete = false;
      }
    }
    complete
  }

  fn changed(
    self: &Rc<Self>,
    directory: &Path,
    file: &gio::File,
    other: Option<&gio::File>,
    event: gio::FileMonitorEvent,
  ) {
    let directories = self.directories.borrow();
    let Some(watched) = directories.get(directory) else {
      return;
    };
    let mut paths = std::iter::once(file)
      .chain(other)
      .filter_map(gio::File::path);
    let matches_target = paths
      .clone()
      .any(|path| path == directory || watched.targets.contains(&path));
    // Removing the selected locale is already covered by its watched entry.
    // Only new directories can otherwise change locale resolution.
    let locale_structure_changed = self.canonical_root.borrow().as_deref() == Some(directory)
      && matches!(
        event,
        gio::FileMonitorEvent::Created
          | gio::FileMonitorEvent::Renamed
          | gio::FileMonitorEvent::MovedIn
      )
      && paths.any(|path| path.is_dir())
      && *self.locale.borrow() != detect_locale_from(&self.root);
    if matches_target || locale_structure_changed {
      self.schedule(Duration::from_millis(40));
    }
  }

  fn schedule(self: &Rc<Self>, delay: Duration) {
    if let Some(timer) = self.pending.take() {
      timer.remove();
    }
    let watch = Rc::downgrade(self);
    self
      .pending
      .set(Some(glib::timeout_add_local_once(delay, move || {
        if let Some(watch) = watch.upgrade() {
          watch.pending.take();
          watch.reload();
        }
      })));
  }
}

impl Drop for Watch {
  fn drop(&mut self) {
    if let Some(timer) = self.pending.take() {
      timer.remove();
    }
  }
}

pub(super) fn track_path(targets: &mut BTreeMap<PathBuf, BTreeSet<PathBuf>>, path: &Path) {
  track_entry(targets, path);
  for ancestor in path.ancestors().skip(1) {
    if ancestor.is_symlink() {
      track_entry(targets, ancestor);
    }
  }
  // Watch both the symlink entry and its resolved target, including the
  // parent of a dangling symlink so restoration invalidates the catalog.
  let resolved = fs::canonicalize(path).ok().or_else(|| {
    fs::read_link(path)
      .ok()
      .map(|link| path.with_file_name(link))
  });
  if let Some(target) = resolved.filter(|resolved| resolved != path) {
    track_entry(targets, &target);
  }
}

fn track_entry(targets: &mut BTreeMap<PathBuf, BTreeSet<PathBuf>>, path: &Path) {
  let mut entry = path;
  while let Some(parent) = entry.parent() {
    if let Some(directory) = parent.canonicalize().ok().filter(|path| path.is_dir())
      && let Some(name) = entry.file_name()
    {
      let target = directory.join(name);
      targets.entry(directory).or_default().insert(target);
      break;
    }
    entry = parent;
  }
}
