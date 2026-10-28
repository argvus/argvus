//! Network operations: manual DNS servers in `resolv.conf`, its backup and restore, and the network argument checks.

use super::*;

pub(super) fn run_network(action: &str, values: &[String]) -> Result<(), String> {
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
  let resolv = Path::new(RESOLV_CONF);
  if fs::symlink_metadata(resolv)
    .map(|metadata| metadata.file_type().is_symlink())
    .unwrap_or(false)
  {
    return Err(
      "/etc/resolv.conf is managed by a service (symlink); use automatic DNS mode".into(),
    );
  }
  backup_resolv_conf()?;
  write_resolv_conf(&resolv_conf_contents(&servers))?;
  run_command(
    "nmcli",
    &[
      "connection",
      "modify",
      connection,
      "ipv4.dns",
      dns_list,
      "ipv6.dns",
      dns_list,
      "ipv4.ignore-auto-dns",
      "yes",
      "ipv6.ignore-auto-dns",
      "yes",
    ],
  )
}

pub(super) fn restore_resolv_conf(connection: &str) -> Result<(), String> {
  let backup = Path::new(RESOLV_BACKUP);
  let restored = if backup.exists() {
    let contents = fs::read_to_string(backup).map_err(|error| error.to_string())?;
    write_resolv_conf(&contents)?;
    true
  } else {
    false
  };
  run_command(
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
  if restored {
    Ok(())
  } else {
    run_command("nmcli", &["connection", "up", connection])
  }
}

pub(super) fn backup_resolv_conf() -> Result<(), String> {
  let contents = fs::read_to_string(RESOLV_CONF).map_err(|error| error.to_string())?;
  if contents.contains(RESOLV_MARKER) || Path::new(RESOLV_BACKUP).exists() {
    return Ok(());
  }
  let mut file = fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(RESOLV_BACKUP)
    .map_err(|error| error.to_string())?;
  file
    .write_all(contents.as_bytes())
    .map_err(|error| error.to_string())?;
  file.sync_all().map_err(|error| error.to_string())
}

pub(super) fn write_resolv_conf(contents: &str) -> Result<(), String> {
  let path = Path::new(RESOLV_CONF);
  let tmp = path.with_extension("argvus-control-center.tmp");
  {
    let mut file = fs::File::create(&tmp).map_err(|error| error.to_string())?;
    file
      .set_permissions(fs::Permissions::from_mode(0o644))
      .map_err(|error| error.to_string())?;
    file
      .write_all(contents.as_bytes())
      .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
  }
  fs::rename(&tmp, path).map_err(|error| error.to_string())
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
      if values[0].is_empty() || values[0].contains('\n') || values[0].contains('\'') {
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
