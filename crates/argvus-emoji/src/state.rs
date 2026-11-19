use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

#[derive(Default, Deserialize, Serialize)]
pub struct History {
  pub recent: Vec<String>,
  pub favorites: Vec<String>,
}

fn path() -> io::Result<PathBuf> {
  let base = env::var_os("XDG_STATE_HOME")
    .filter(|value| !value.is_empty())
    .map(PathBuf::from)
    .or_else(|| env::home_dir().map(|home| home.join(".local/state")))
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG state directory"))?;
  Ok(base.join("argvus/emoji.json"))
}

impl History {
  pub fn load() -> io::Result<Self> {
    match fs::read(path()?) {
      Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
      Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
      Err(error) => Err(error),
    }
  }

  pub fn save(&self) -> io::Result<()> {
    let path = path()?;
    fs::create_dir_all(path.with_file_name(""))?;
    let staging = path.with_extension("json.tmp");
    fs::write(&staging, serde_json::to_vec(self)?)?;
    fs::rename(staging, path)
  }

  pub fn record(&mut self, sequence: String) {
    self.recent.retain(|existing| *existing != sequence);
    self.recent.insert(0, sequence);
    self.recent.truncate(50);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn recent_history_preserves_variants_and_limits_entries() {
    let mut history = History::default();
    for index in 0..60 {
      history.record(index.to_string());
    }
    history.record("👍🏽".into());
    history.record("👍".into());
    history.record("👍🏽".into());
    assert_eq!(history.recent.len(), 50);
    assert_eq!(&history.recent[..3], ["👍🏽", "👍", "59"]);
    let restored: History = serde_json::from_slice(&serde_json::to_vec(&history).unwrap()).unwrap();
    assert_eq!(restored.recent, history.recent);
  }
}
