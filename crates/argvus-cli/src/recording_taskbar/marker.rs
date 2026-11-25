use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::gio;
use gtk::gio::prelude::*;

use super::files::track_path;

struct Directory {
  monitor: gio::FileMonitor,
  identity: (u64, u64),
  targets: Rc<BTreeSet<PathBuf>>,
}

impl Drop for Directory {
  fn drop(&mut self) {
    self.monitor.cancel();
  }
}

#[derive(Default)]
pub(super) struct Watches(BTreeMap<PathBuf, Directory>);

impl Watches {
  pub(super) fn refresh(&mut self, marker: &Path, changed: &Rc<dyn Fn()>) -> bool {
    let mut targets = BTreeMap::new();
    track_path(&mut targets, marker);
    if let Some(parent) = marker.parent() {
      track_path(&mut targets, parent);
    }
    self.0.retain(|path, _| targets.contains_key(path));
    let mut complete = !targets.is_empty();
    for (parent, paths) in targets {
      let identity = fs::metadata(&parent)
        .ok()
        .map(|info| (info.dev(), info.ino()));
      if self.0.get(&parent).is_some_and(|watch| {
        Some(watch.identity) == identity && !watch.monitor.is_cancelled() && *watch.targets == paths
      }) {
        continue;
      }
      self.0.remove(&parent);
      let monitor = gio::Vfs::local().file_for_path(&parent).monitor_directory(
        gio::FileMonitorFlags::WATCH_MOVES | gio::FileMonitorFlags::WATCH_MOUNTS,
        gio::Cancellable::NONE,
      );
      let (Some(identity), Ok(monitor)) = (identity, monitor) else {
        complete = false;
        continue;
      };
      monitor.set_rate_limit(30);
      let targets = Rc::new(paths);
      monitor.connect_changed({
        let changed = changed.clone();
        let targets = targets.clone();
        let parent = parent.clone();
        move |_, file, other, event| {
          if matches!(
            event,
            gio::FileMonitorEvent::Unmounted | gio::FileMonitorEvent::PreUnmount
          ) || std::iter::once(file)
            .chain(other)
            .filter_map(gio::File::path)
            .any(|path| path == parent || targets.contains(&path))
          {
            changed();
          }
        }
      });
      self.0.insert(
        parent,
        Directory {
          monitor,
          identity,
          targets,
        },
      );
    }
    complete
  }
}

pub(super) fn read(path: &Path) -> std::io::Result<Vec<u8>> {
  const MAX_MARKER_BYTES: u64 = 4096;
  let file = match OpenOptions::new()
    .read(true)
    .custom_flags(libc::O_NONBLOCK)
    .open(path)
  {
    Ok(file) => file,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
    Err(error) => return Err(error),
  };
  if !file.metadata()?.is_file() {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      "marker is not a regular file",
    ));
  }
  let mut contents = Vec::new();
  file.take(MAX_MARKER_BYTES + 1).read_to_end(&mut contents)?;
  if contents.len() as u64 > MAX_MARKER_BYTES || contents.contains(&0) {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      "unsupported marker contents",
    ));
  }
  Ok(contents)
}
