//! Network operations: manual DNS servers in `resolv.conf`, its backup and restore, and the network argument checks.

use super::*;

struct DnsPaths<'a> {
  resolv: &'a Path,
  backup: &'a Path,
}

pub(super) fn run_network(action: &str, values: &[String]) -> Result<(), String> {
  validate_network(action, values)?;
  match action {
    "dns" => {
      if values.len() != 3 {
        return Err("network dns requires 3 arguments".into());
      }
      match values[1].as_str() {
        "automatic" => restore_resolv_conf(&values[0]),
        "manual" => set_resolv_conf_manual(&values[0], &values[2]),
        _ => Err("mode must be automatic or manual".into()),
      }
    }
    _ => Err("unsupported network operation".into()),
  }
}

pub(super) fn set_resolv_conf_manual(connection: &str, dns_list: &str) -> Result<(), String> {
  set_manual_at(
    connection,
    dns_list,
    &DnsPaths {
      resolv: Path::new(RESOLV_CONF),
      backup: Path::new(RESOLV_BACKUP),
    },
    run_command,
  )
}
fn set_manual_at(
  connection: &str,
  dns_list: &str,
  paths: &DnsPaths<'_>,
  mut run: impl FnMut(&str, &[&str]) -> Result<(), String>,
) -> Result<(), String> {
  super::super::validate_plain_arg(connection)?;
  let servers: Vec<&str> = dns_list
    .split(',')
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .collect();
  if servers.is_empty() {
    return Err("no DNS servers provided".into());
  }
  for server in &servers {
    if !valid_ipv4(server) && !valid_ipv6(server) {
      return Err(format!("invalid DNS server: {server}"));
    }
  }
  let resolv = paths.resolv;
  if fs::symlink_metadata(resolv)
    .map(|metadata| metadata.file_type().is_symlink())
    .unwrap_or(false)
  {
    return Err(
      "/etc/resolv.conf is managed by a service (symlink); use automatic DNS mode".into(),
    );
  }
  backup_resolv_conf_at(paths)?;
  let ipv4 = servers
    .iter()
    .copied()
    .filter(|server| valid_ipv4(server))
    .collect::<Vec<_>>()
    .join(",");
  let ipv6 = servers
    .iter()
    .copied()
    .filter(|server| valid_ipv6(server))
    .collect::<Vec<_>>()
    .join(",");
  run(
    "nmcli",
    &[
      "connection",
      "modify",
      connection,
      "ipv4.dns",
      &ipv4,
      "ipv6.dns",
      &ipv6,
      "ipv4.ignore-auto-dns",
      "yes",
      "ipv6.ignore-auto-dns",
      "yes",
    ],
  )?;
  write_resolv_conf_at(paths.resolv, &resolv_conf_contents(&servers))
}

pub(super) fn restore_resolv_conf(connection: &str) -> Result<(), String> {
  restore_at(
    connection,
    &DnsPaths {
      resolv: Path::new(RESOLV_CONF),
      backup: Path::new(RESOLV_BACKUP),
    },
    run_command,
  )
}
fn restore_at(
  connection: &str,
  paths: &DnsPaths<'_>,
  mut run: impl FnMut(&str, &[&str]) -> Result<(), String>,
) -> Result<(), String> {
  super::super::validate_plain_arg(connection)?;
  reject_symlink(paths.resolv)?;
  reject_symlink(paths.backup)?;
  let contents = match fs::read_to_string(paths.backup) {
    Ok(contents) => Some(contents),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
    Err(error) => return Err(error.to_string()),
  };
  run(
    "nmcli",
    &[
      "connection",
      "modify",
      connection,
      "ipv4.dns",
      "",
      "ipv6.dns",
      "",
      "ipv4.ignore-auto-dns",
      "no",
      "ipv6.ignore-auto-dns",
      "no",
    ],
  )?;
  if let Some(contents) = contents {
    write_resolv_conf_at(paths.resolv, &contents)?;
    fs::remove_file(paths.backup).map_err(|error| error.to_string())
  } else {
    run("nmcli", &["connection", "up", connection])
  }
}

fn reject_symlink(path: &Path) -> Result<(), String> {
  match fs::symlink_metadata(path) {
    Ok(metadata) if metadata.file_type().is_symlink() => {
      Err("refusing to replace a symlinked DNS file".into())
    }
    Ok(_) => Ok(()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error.to_string()),
  }
}

fn backup_resolv_conf_at(paths: &DnsPaths<'_>) -> Result<(), String> {
  reject_symlink(paths.resolv)?;
  reject_symlink(paths.backup)?;
  let contents = fs::read_to_string(paths.resolv).map_err(|error| error.to_string())?;
  if contents.contains(RESOLV_MARKER) {
    return if paths.backup.is_file() {
      Ok(())
    } else {
      Err("manual DNS backup is missing".into())
    };
  }
  // A new manual cycle backs up the current resolver, replacing an obsolete backup.
  write_resolv_conf_at(paths.backup, &contents)
}

fn write_resolv_conf_at(path: &Path, contents: &str) -> Result<(), String> {
  super::replace_atomic(path, contents, fs::Permissions::from_mode(0o644))
}

pub(super) fn resolv_conf_contents(servers: &[&str]) -> String {
  let bodies = servers
    .iter()
    .map(|server| format!("nameserver {server}"))
    .collect::<Vec<_>>()
    .join("\n");
  format!("{RESOLV_MARKER}\n{bodies}\n")
}

pub(super) fn valid_ipv4(value: &str) -> bool {
  !value.is_empty() && value.parse::<Ipv4Addr>().is_ok()
}

pub(super) fn valid_ipv6(value: &str) -> bool {
  !value.is_empty() && value.parse::<Ipv6Addr>().is_ok()
}

pub(super) fn validate_network(action: &str, values: &[String]) -> Result<(), String> {
  match action {
    "dns" => {
      if values.len() != 3 {
        return Err("network dns requires 3 arguments".into());
      }
      if super::super::validate_plain_arg(&values[0]).is_err()
        || values[0].contains('\n')
        || values[0].contains('\'')
      {
        return Err("invalid connection name/uuid".into());
      }
      if values[1] != "automatic" && values[1] != "manual" {
        return Err("mode must be automatic or manual".into());
      }
      if values[1] == "manual" {
        for server in values[2]
          .split(',')
          .map(str::trim)
          .filter(|value| !value.is_empty())
        {
          if !valid_ipv4(server) && !valid_ipv6(server) {
            return Err(format!("invalid DNS server: {server}"));
          }
        }
      }
      Ok(())
    }
    _ => Err("unsupported network operation".into()),
  }
}

#[cfg(test)]
mod audit_h3_tests {
  use super::*;
  #[test]
  fn nmcli_failure_does_not_change_resolver() {
    let directory = tempfile::tempdir().unwrap();
    let resolv = directory.path().join("resolv.conf");
    let backup = directory.path().join("backup");
    fs::write(&resolv, "original\n").unwrap();
    let paths = DnsPaths {
      resolv: &resolv,
      backup: &backup,
    };
    assert!(
      set_manual_at("connection", "1.1.1.1", &paths, |_, _| Err(
        "nmcli failed".into()
      ))
      .is_err()
    );
    assert_eq!(fs::read_to_string(resolv).unwrap(), "original\n");
  }
  #[test]
  fn dns_families_and_backup_follow_successful_cycles() {
    let directory = tempfile::tempdir().unwrap();
    let resolv = directory.path().join("resolv.conf");
    let backup = directory.path().join("backup");
    let paths = DnsPaths {
      resolv: &resolv,
      backup: &backup,
    };
    fs::write(&resolv, "current\n").unwrap();
    fs::write(&backup, "obsolete\n").unwrap();
    set_manual_at(
      "connection",
      "1.1.1.1, 2606:4700:4700::1111",
      &paths,
      |_, args| {
        assert_eq!(fs::read_to_string(&resolv).unwrap(), "current\n");
        assert_eq!(args[4], "1.1.1.1");
        assert_eq!(args[6], "2606:4700:4700::1111");
        Ok(())
      },
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&backup).unwrap(), "current\n");
    assert!(restore_at("connection", &paths, |_, _| Err("nmcli failed".into())).is_err());
    assert!(backup.exists());
    assert!(fs::read_to_string(&resolv).unwrap().contains(RESOLV_MARKER));
    restore_at("connection", &paths, |_, _| Ok(())).unwrap();
    assert_eq!(fs::read_to_string(&resolv).unwrap(), "current\n");
    assert!(!backup.exists());
    fs::write(&resolv, "next cycle\n").unwrap();
    set_manual_at("connection", "8.8.8.8", &paths, |_, args| {
      assert_eq!(args[6], "");
      Ok(())
    })
    .unwrap();
    assert_eq!(fs::read_to_string(&backup).unwrap(), "next cycle\n");
  }

  #[test]
  fn dns_restore_preserves_symlink_and_does_not_run_nmcli() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("managed");
    let resolv = directory.path().join("resolv.conf");
    let backup = directory.path().join("backup");
    fs::write(&target, "managed\n").unwrap();
    fs::write(&backup, "original\n").unwrap();
    std::os::unix::fs::symlink(&target, &resolv).unwrap();
    assert!(
      restore_at(
        "connection",
        &DnsPaths {
          resolv: &resolv,
          backup: &backup
        },
        |_, _| panic!("must reject before nmcli")
      )
      .is_err()
    );
    assert!(resolv.is_symlink());
    assert_eq!(fs::read_to_string(target).unwrap(), "managed\n");
    assert!(backup.exists());
  }
}
