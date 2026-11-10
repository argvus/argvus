//! Event driven Waybar output. Monitor text is a wakeup, never a state protocol.
use argvus_network_core::status::Status;
use argvus_runtime::messages::text_in;
use std::io::{self, BufRead, Write};
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn render(status: &Status) -> String {
  let (icon, class, key) = match status.status {
    "connected" => ("󰱓 ", "connected", "waybar.connected"),
    "limited" => ("󰍸 ", "limited", "waybar.limited"),
    _ => ("󰅛 ", "disconnected", "waybar.disconnected"),
  };
  let mut tooltip = text_in("network", key, &[]);
  let detail = if status.ssid.is_empty() {
    &status.iface
  } else {
    &status.ssid
  };
  if !detail.is_empty() {
    tooltip.push_str(&format!(" — {detail}"));
  }
  if class == "limited" {
    tooltip.push_str(&format!(
      " — {}",
      text_in("network", "waybar.no_internet", &[])
    ));
  }
  serde_json::json!({"text": icon, "class": class, "tooltip": tooltip}).to_string()
}

fn emit(output: &str) -> io::Result<()> {
  let mut stdout = io::stdout().lock();
  writeln!(stdout, "{output}")?;
  stdout.flush()
}

pub(super) fn run(follow: bool) -> ExitCode {
  if !follow {
    return if emit(&render(&super::networkctl::gather_status())).is_ok() {
      ExitCode::SUCCESS
    } else {
      ExitCode::FAILURE
    };
  }
  loop {
    // Spawn before the snapshot so changes during collection remain queued.
    let monitor = Command::new("nmcli")
      .arg("monitor")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .spawn();
    let mut monitor = match monitor {
      Ok(child) => child,
      Err(error) => {
        eprintln!("network monitor: {error}");
        if emit(&render(&Status::unavailable())).is_err() {
          return ExitCode::SUCCESS;
        }
        std::thread::sleep(Duration::from_secs(2));
        continue;
      }
    };
    let (sender, receiver) = mpsc::sync_channel(1);
    let reader = monitor.stdout.take().map(|stdout| {
      std::thread::spawn(move || {
        for line in io::BufReader::new(stdout).lines() {
          if line.is_err() || sender.send(()).is_err() {
            break;
          }
        }
      })
    });
    let mut output = render(&super::networkctl::gather_status());
    let mut closed = false;
    loop {
      // Cached heartbeat detects Waybar closing its pipe even on an idle network.
      if emit(&output).is_err() {
        closed = true;
        break;
      }
      match receiver.recv_timeout(Duration::from_secs(1)) {
        Ok(()) => {
          while receiver.try_recv().is_ok() {}
          output = render(&super::networkctl::gather_status());
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {}
        Err(mpsc::RecvTimeoutError::Disconnected) => break,
      }
    }
    drop(receiver);
    let _ = monitor.kill();
    let _ = monitor.wait();
    if let Some(reader) = reader {
      let _ = reader.join();
    }
    if closed {
      return ExitCode::SUCCESS;
    }
    eprintln!("network monitor ended; reconnecting");
    if emit(&render(&Status::unavailable())).is_err() {
      return ExitCode::SUCCESS;
    }
    std::thread::sleep(Duration::from_secs(2));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn renders_transitions_and_escapes_connection_details() {
    let mut status = Status::unavailable();
    for (state, class) in [
      ("connected", "connected"),
      ("limited", "limited"),
      ("disconnected", "disconnected"),
      ("disabled", "disconnected"),
    ] {
      status.status = state;
      status.ssid = "Wi-Fi \"office\"\n".into();
      let json: serde_json::Value = serde_json::from_str(&render(&status)).unwrap();
      assert_eq!(json["class"], class);
      assert!(json["tooltip"].as_str().unwrap().contains(&status.ssid));
    }
  }
}
