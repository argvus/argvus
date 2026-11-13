//! Import skeleton-format themes from Git without executing repository code.
use crate::{
  manifest::ThemeManifest,
  package,
  paths::{ThemePaths, relative_inside},
};
use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

fn git(args: &[&str]) -> Result<Vec<u8>, String> {
  let output = Command::new("git")
    .args(args)
    .output()
    .map_err(|error| format!("cannot run git: {error}"))?;
  if !output.status.success() {
    return Err(format!(
      "git failed: {}",
      String::from_utf8_lossy(&output.stderr)
    ));
  }
  Ok(output.stdout)
}

/// Installs exactly one skeleton theme from a remote branch or tag.
pub fn install(
  paths: &ThemePaths,
  url: &str,
  revision: Option<&str>,
  replace: bool,
) -> Result<PathBuf, String> {
  if !(url.starts_with("https://")
    || url.starts_with("ssh://")
    || (!url.contains("://")
      && url.split_once('@').is_some_and(|(user, host)| {
        !user.is_empty()
          && !user.starts_with('-')
          && host
            .split_once(':')
            .is_some_and(|(host, path)| !host.is_empty() && !path.is_empty())
      })))
  {
    return Err("expected an HTTPS or SSH Git URL".into());
  }
  fs::create_dir_all("/tmp/argvus").map_err(|error| error.to_string())?;
  let staging = tempfile::tempdir_in("/tmp/argvus").map_err(|error| error.to_string())?;
  let archive = acquire(url, revision, staging.path())?;
  prepare(paths, &archive, staging.path(), replace)
}

fn acquire(url: &str, revision: Option<&str>, staging: &Path) -> Result<Vec<u8>, String> {
  let repo = staging.join("repository.git");
  let repo_text = repo.to_str().ok_or("invalid temporary path")?;
  let mut args = vec!["clone", "--bare", "--depth", "1", "--single-branch"];
  if let Some(revision) = revision {
    args.extend(["--branch", revision]);
  }
  args.extend(["--", url, repo_text]);
  git(&args)?;
  git(&["--git-dir", repo_text, "archive", "--format=tar", "HEAD"])
}

fn prepare(
  paths: &ThemePaths,
  archive: &[u8],
  staging: &Path,
  replace: bool,
) -> Result<PathBuf, String> {
  let tree = staging.join("tree");
  fs::create_dir(&tree).map_err(|error| error.to_string())?;
  let mut archive = tar::Archive::new(archive);
  let mut total = 0u64;
  for entry in archive.entries().map_err(|error| error.to_string())? {
    let mut entry = entry.map_err(|error| error.to_string())?;
    total = total.checked_add(entry.size()).ok_or("archive too large")?;
    if total > 520 * 1024 * 1024 {
      return Err("archive too large".into());
    }
    let member = entry
      .path()
      .map_err(|error| error.to_string())?
      .into_owned();
    relative_inside(member.to_str().ok_or("invalid archive path")?)?;
    if !(entry.header().entry_type().is_file() || entry.header().entry_type().is_dir()) {
      return Err("repository must contain only regular files and directories".into());
    }
    if !entry.unpack_in(&tree).map_err(|error| error.to_string())? {
      return Err("unsafe archive path".into());
    }
  }
  let themes = tree.join("src/usr/share/argvus/themes");
  let entries = fs::read_dir(&themes)
    .map_err(|error| format!("missing skeleton theme directory: {error}"))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| error.to_string())?;
  if entries.len() != 1 {
    return Err("expected exactly one skeleton theme".into());
  }
  let source = entries[0].path();
  let manifest = ThemeManifest::load(&source).map_err(|error| error.to_string())?;
  let text = fs::read_to_string(source.join("theme.toml")).map_err(|error| error.to_string())?;
  let mut document: toml::Table = toml::from_str(&text).map_err(|error| error.to_string())?;
  for key in ["wallpaper", "preview"] {
    if let Some(value) = document.get(key).and_then(toml::Value::as_str) {
      let asset = if Path::new(value).is_absolute() {
        if !value.starts_with("/usr/share/") {
          return Err("asset must be shipped in src/usr/share".into());
        }
        tree
          .join("src")
          .join(relative_inside(value.trim_start_matches('/'))?)
      } else {
        source.join(relative_inside(value)?)
      };
      if !asset.is_file() {
        return Err(format!("missing {key}"));
      }
      use std::io::Read;
      let mut prefix = [0u8; 128];
      let count = fs::File::open(&asset)
        .and_then(|mut file| file.read(&mut prefix))
        .map_err(|error| error.to_string())?;
      if prefix[..count].starts_with(b"version https://git-lfs.github.com/spec/") {
        return Err("Git LFS assets are unsupported; commit the image directly".into());
      }
      let extension = asset
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("img");
      let name = format!("installed-{key}.{extension}");
      if asset != source.join(&name) {
        fs::copy(&asset, source.join(&name)).map_err(|error| error.to_string())?;
      }
      document.insert(key.into(), toml::Value::String(name));
    }
  }
  fs::write(
    source.join("theme.toml"),
    toml::to_string(&document).map_err(|error| error.to_string())?,
  )
  .map_err(|error| error.to_string())?;
  if source.file_name().and_then(|name| name.to_str()) != Some(&manifest.id) {
    return Err("theme id must match its directory".into());
  }
  package::install_user_theme(paths, &source, replace)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn git_acquires_default_branch_and_explicit_ref_without_network() {
    use std::io::Write;
    use std::process::Stdio;
    let temporary = tempfile::tempdir().unwrap();
    let repo = temporary.path().join("source.git");
    git(&[
      "init",
      "--bare",
      "--initial-branch=main",
      repo.to_str().unwrap(),
    ])
    .unwrap();
    let mut child = Command::new("git")
      .arg("--git-dir")
      .arg(&repo)
      .arg("fast-import")
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .unwrap();
    child.stdin.take().unwrap().write_all(b"commit refs/heads/main\ncommitter Test <test@example.invalid> 0 +0000\ndata 4\ntest\nM 100644 inline README\ndata 5\nhello\n\ntag v1\nfrom refs/heads/main\ntagger Test <test@example.invalid> 0 +0000\ndata 4\ntest\n").unwrap();
    assert!(child.wait().unwrap().success());
    for revision in [None, Some("main"), Some("v1")] {
      let stage = tempfile::tempdir().unwrap();
      let bytes = acquire(repo.to_str().unwrap(), revision, stage.path()).unwrap();
      let mut archive = tar::Archive::new(bytes.as_slice());
      assert!(
        archive
          .entries()
          .unwrap()
          .any(|entry| entry.unwrap().path().unwrap() == Path::new("README"))
      );
    }
    let stage = tempfile::tempdir().unwrap();
    assert!(acquire(repo.to_str().unwrap(), Some("missing"), stage.path()).is_err());
  }

  fn archive(files: &[(&str, &str)]) -> Vec<u8> {
    let mut archive = tar::Builder::new(Vec::new());
    for (name, contents) in files {
      let mut header = tar::Header::new_gnu();
      header.set_size(contents.len() as u64);
      header.set_mode(0o644);
      header.set_cksum();
      archive
        .append_data(&mut header, name, contents.as_bytes())
        .unwrap();
    }
    archive.into_inner().unwrap()
  }
  #[test]
  fn invalid_repositories_do_not_publish_themes() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = ThemePaths {
      system_root: temporary.path().join("system"),
      user_themes: temporary.path().join("user"),
      generated: temporary.path().join("generated"),
    };
    let manifest = "schema = 2\nid = \"ocean\"\nname = \"Ocean\"\ncategory = \"dark\"\nwallpaper = \"/usr/share/backgrounds/missing.png\"\n[palette]\nbg = \"#000000\"\nfg = \"#ffffff\"\naccent = \"#123456\"\n";
    for files in [
      vec![("README", "empty")],
      vec![("src/usr/share/argvus/themes/ocean/theme.toml", "schema = 1")],
      vec![("src/usr/share/argvus/themes/ocean/theme.toml", manifest)],
      vec![
        ("src/usr/share/argvus/themes/ocean/theme.toml", manifest),
        ("src/usr/share/argvus/themes/other/theme.toml", manifest),
      ],
    ] {
      let stage = tempfile::tempdir().unwrap();
      assert!(prepare(&paths, &archive(&files), stage.path(), false).is_err());
      assert!(!paths.user_themes.exists());
    }
    let stage = tempfile::tempdir().unwrap();
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    header.set_mode(0o777);
    builder
      .append_link(&mut header, "link", "/etc/passwd")
      .unwrap();
    assert!(prepare(&paths, &builder.into_inner().unwrap(), stage.path(), false).is_err());
  }

  #[test]
  fn installs_skeleton_assets_and_requires_explicit_replacement() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = ThemePaths {
      system_root: temporary.path().join("system"),
      user_themes: temporary.path().join("user"),
      generated: temporary.path().join("generated"),
    };
    let contents = archive(&[
      (
        "src/usr/share/argvus/themes/ocean/theme.toml",
        "schema = 2\nid = \"ocean\"\nname = \"Ocean\"\ncategory = \"dark\"\nwallpaper = \"/usr/share/backgrounds/ocean.png\"\n[palette]\nbg = \"#000000\"\nfg = \"#ffffff\"\naccent = \"#123456\"\n",
      ),
      ("src/usr/share/backgrounds/ocean.png", "wallpaper"),
    ]);
    let stage = temporary.path().join("stage");
    fs::create_dir(&stage).unwrap();
    let installed = prepare(&paths, &contents, &stage, false).unwrap();
    let manifest = ThemeManifest::load(&installed).unwrap();
    assert_eq!(
      fs::read_to_string(manifest.wallpaper.unwrap()).unwrap(),
      "wallpaper"
    );
    fs::remove_dir_all(stage.join("tree")).unwrap();
    assert!(prepare(&paths, &contents, &stage, false).is_err());
    fs::remove_dir_all(stage.join("tree")).unwrap();
    assert!(prepare(&paths, &contents, &stage, true).is_ok());
  }
}
