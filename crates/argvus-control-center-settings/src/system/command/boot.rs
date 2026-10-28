//! Boot operations: validated bootloader keys written atomically with a backup.

use super::streaming::{locate_command, stream_command, stream_command_path};
use super::*;

pub(super) fn validate_boot(action: &str, values: &[String]) -> Result<(), String> {
  let token = |value: &str| {
    !value.is_empty()
      && value.len() <= 255
      && value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
  };
  match action {
    "systemd-default" if values.len() == 1 => token(&values[0])
      .then_some(())
      .ok_or_else(|| "invalid systemd-boot entry".into()),
    "systemd-timeout" | "grub-timeout" if values.len() == 1 => values[0]
      .parse::<u32>()
      .ok()
      .filter(|value| *value <= 60)
      .map(|_| ())
      .ok_or_else(|| "timeout must be between 0 and 60".into()),
    // grub-mkconfig sources /etc/default/grub as a shell script, so characters
    // that the shell expands inside double quotes are refused.
    "grub-cmdline" if values.len() == 1 => (!values[0].is_empty()
      && values[0].len() <= 2048
      && values[0]
        .chars()
        .all(|character| !character.is_control() && !matches!(character, '"' | '\\' | '$' | '`')))
    .then_some(())
    .ok_or_else(|| "invalid GRUB kernel command line".into()),
    "grub-regenerate" | "initramfs-regenerate" if values.is_empty() => Ok(()),
    "plymouth-theme" if values.len() == 1 => token(&values[0])
      .then_some(())
      .ok_or_else(|| "invalid Plymouth theme".into()),
    _ => Err("unsupported boot operation".into()),
  }
}

pub(super) fn run_boot(action: &str, values: &[String]) -> Result<(), String> {
  match action {
    "systemd-default" => {
      let root = boot_root().ok_or("systemd-boot ESP was not found")?;
      let direct = root.join("loader/entries").join(&values[0]);
      let named = root
        .join("loader/entries")
        .join(format!("{}.conf", values[0]));
      if (!direct.is_file() || direct.is_symlink()) && (!named.is_file() || named.is_symlink()) {
        return Err("systemd-boot entry was not found".into());
      }
      stream_command(
        "bootctl",
        &[
          &format!("--esp-path={}", root.display()),
          "set-default",
          &values[0],
        ],
      )
    }
    "systemd-timeout" => {
      let root = boot_root().ok_or("systemd-boot ESP was not found")?;
      update_boot_key(&root.join("loader/loader.conf"), "timeout", &values[0])?;
      println!("ok");
      Ok(())
    }
    "grub-timeout" => {
      update_boot_key(Path::new("/etc/default/grub"), "GRUB_TIMEOUT", &values[0])?;
      println!("ok");
      Ok(())
    }
    "grub-cmdline" => {
      update_boot_key(
        Path::new("/etc/default/grub"),
        "GRUB_CMDLINE_LINUX_DEFAULT",
        &values[0],
      )?;
      println!("ok");
      Ok(())
    }
    "grub-regenerate" => {
      let config = ["/boot/grub/grub.cfg", "/boot/grub2/grub.cfg"]
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
        .ok_or("GRUB configuration path was not found")?;
      let tool = locate_command(&["grub-mkconfig", "grub2-mkconfig"])
        .ok_or("grub-mkconfig was not found")?;
      let config = config.to_string_lossy().into_owned();
      stream_command_path(&tool, &["-o", &config])
    }
    "initramfs-regenerate" => stream_command("mkinitcpio", &["-P"]),
    "plymouth-theme" => {
      let path = Path::new("/usr/share/plymouth/themes").join(&values[0]);
      if !path.is_dir() || path.is_symlink() {
        return Err("Plymouth theme was not found".into());
      }
      stream_command("plymouth-set-default-theme", &["-R", values[0].as_str()])
    }
    _ => Err("unsupported boot operation".into()),
  }
}

pub(super) fn boot_root() -> Option<PathBuf> {
  ["/boot", "/efi", "/boot/efi"]
    .iter()
    .map(PathBuf::from)
    .find(|root| root.join("loader/loader.conf").is_file() || root.join("loader/entries").is_dir())
}

pub(super) fn update_boot_key(path: &Path, key: &str, value: &str) -> Result<(), String> {
  if path.is_symlink() {
    return Err("refusing to update a symlinked boot configuration".into());
  }
  let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
  let mut found = false;
  let mut updated = String::new();
  for line in contents.lines() {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') && trimmed.split_whitespace().next() == Some(key) {
      updated.push_str(key);
      updated.push(' ');
      updated.push_str(value);
      found = true;
    } else if key.starts_with("GRUB_")
      && !trimmed.starts_with('#')
      && trimmed.starts_with(&format!("{key}="))
    {
      updated.push_str(key);
      updated.push_str("=\"");
      updated.push_str(value);
      updated.push('"');
      found = true;
    } else {
      updated.push_str(line);
    }
    updated.push('\n');
  }
  if !found {
    if key.starts_with("GRUB_") {
      updated.push_str(&format!("{key}=\"{value}\"\n"));
    } else {
      updated.push_str(&format!("{key} {value}\n"));
    }
  }
  if updated != contents {
    backup_file(path)?;
    boot_write_atomic(path, &updated)?;
  }
  Ok(())
}

pub(in super::super) fn backup_file(path: &Path) -> Result<PathBuf, String> {
  let stamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();
  for suffix in 0..1000 {
    let backup = PathBuf::from(format!("{}.argvus-backup-{stamp}-{suffix}", path.display()));
    match fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(&backup)
    {
      Ok(mut file) => {
        file
          .write_all(&fs::read(path).map_err(|error| error.to_string())?)
          .map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        return Ok(backup);
      }
      Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(error.to_string()),
    }
  }
  Err("could not create a unique boot backup".into())
}

pub(in super::super) fn boot_write_atomic(path: &Path, contents: &str) -> Result<(), String> {
  let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
  let tmp = path.with_extension(format!("argvus-control-center-{}.tmp", std::process::id()));
  let mut file = fs::File::create(&tmp).map_err(|error| error.to_string())?;
  file
    .set_permissions(metadata.permissions())
    .map_err(|error| error.to_string())?;
  file
    .write_all(contents.as_bytes())
    .map_err(|error| error.to_string())?;
  file.sync_all().map_err(|error| error.to_string())?;
  fs::rename(&tmp, path).map_err(|error| error.to_string())
}
